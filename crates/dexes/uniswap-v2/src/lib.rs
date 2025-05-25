use alloy::{
    primitives::{address, Address, Uint},
    providers::{Provider, RootProvider},
    rpc::types::{Filter, Header},
    sol_types::SolEvent,
};
use anyhow::Result;
use bot_db::{
    tables::{Pair, Ticker},
    PricesStorage, TokensGraphStorage, UpdateReservesData, DB,
};
use bot_math::cpmm::{find_profit, find_triangular_arbitrage, ArbitrageData};
use crossbeam::channel::Sender;
use dex_common::{AddressBook, Arbitrage, DexError, Reserves, DEX};
use ethereum_abi::{IUniswapV2Pair, IERC20};
use hashbrown::{hash_map::Entry, HashMap};
use std::{
    collections::HashSet,
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

const DEX_NAME: &str = "uniswap_v2";
const MAX_REQUESTS_PER_BLOCK: usize = 50;
static REQUESTS_PER_BLOCK: AtomicUsize = AtomicUsize::new(0);

pub fn request_wrapper(inc: usize) -> Result<()> {
    if REQUESTS_PER_BLOCK.load(Ordering::Relaxed) >= MAX_REQUESTS_PER_BLOCK {
        tracing::info!("reach requests limit per block");
        return Err(DexError::BlockRpcLimitExceed.into());
    }

    REQUESTS_PER_BLOCK.fetch_add(inc, Ordering::Relaxed);
    Ok(())
}

type P = Arc<RootProvider>;

#[derive(Clone)]
pub struct UniswapV2 {
    dex_id: i32,
    address_book: AddressBook,

    s: Sender<Arbitrage>,
    db: DB,
    provider: Arc<RootProvider>,
}

impl UniswapV2 {
    pub async fn new(s: Sender<Arbitrage>, db: DB, provider: P) -> Result<Self> {
        let dex_id = db.postgres().get_dex_id(DEX_NAME).await?;

        let address_book = AddressBook {
            factory: address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            router: address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"),
        };

        tracing::info!("Uniswap-V2 successfully created");
        Ok(Self {
            dex_id,
            address_book,
            s,
            db,
            provider,
        })
    }

    pub async fn fetch_pair(&self, pair_adr: Address) -> Result<Pair> {
        request_wrapper(1usize)?;

        let instance = IUniswapV2Pair::new(pair_adr.clone(), self.provider.clone());
        let token0 = instance.token0().call().await?._0;
        let token1 = instance.token1().call().await?._0;

        Ok(Pair {
            address: pair_adr,
            // TODO: replace here with better checking
            // Now it is ok, because of method 'owns_pairs'
            dex_id: self.dex_id,
            token0,
            token1,
        })
    }

    async fn add_pair(&self, pair: Pair) -> Result<()> {
        self.db.add_pair(pair.clone()).await?;
        self.db.postgres().insert_pair(pair.clone()).await?;

        for token in vec![pair.token0, pair.token1] {
            if self.db.postgres().get_token_ticker(&token).await.is_err() {
                let instance = IERC20::new(token, self.provider.clone());
                let ticker = Ticker {
                    token,
                    ticker: instance.symbol().call().await?._0,
                };
                self.db.postgres().insert_ticker(ticker).await?;
            }
        }

        Ok(())
    }

    async fn best_arbitrages(
        &self,
        paths: Vec<Vec<(Address, Address)>>,
    ) -> Result<HashMap<Address, Arbitrage>> {
        let mut best_arbitrages: HashMap<Address, Arbitrage> = HashMap::new();

        for path in paths.into_iter() {
            let mut data = Vec::new();
            for tokens in &path {
                data.push(ArbitrageData {
                    reserves: self.token_reserves(&tokens.0, &tokens.1).await?,
                    fee: Uint::from(3),
                });
            }

            if let Some(profit) = find_profit(&data) {
                match best_arbitrages.entry(path[0].0) {
                    Entry::Occupied(mut entry) => {
                        let arbitrage = entry.get_mut();

                        if arbitrage.revenue < profit.1 - profit.0 {
                            *arbitrage = Arbitrage {
                                dex_id: self.dex_id,
                                amount_in: profit.0,
                                revenue: profit.1 - profit.0,
                                path: path.clone(),
                            }
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(Arbitrage {
                            dex_id: self.dex_id,
                            amount_in: profit.0,
                            revenue: profit.1 - profit.0,
                            path: path.clone(),
                        });
                    }
                }
            }
        }

        Ok(best_arbitrages)
    }
}

#[async_trait::async_trait]
impl DEX for UniswapV2 {
    async fn adjacent(&self, token: &Address) -> Result<HashSet<Address>> {
        self.db.adjacent_tokens(self.dex_id, &token).await
    }

    async fn fetch_reserves(&self, pair_adr: &Address) -> Result<Reserves> {
        request_wrapper(1usize)?;
        let instance = IUniswapV2Pair::new(*pair_adr, self.provider.clone());
        let reserves = instance.getReserves().call().await?;
        Ok(Reserves(reserves.reserve0, reserves.reserve1))
    }

    // TODO: think about correctness this
    async fn owns_pair(&self, pair_adr: &Address) -> Result<bool> {
        match self.db.postgres().get_pair_dex_id(pair_adr).await {
            Ok(pair_dex_id) => Ok(pair_dex_id == self.dex_id),
            Err(err) => {
                if let Some(sqlx_err) = err.downcast_ref::<sqlx::Error>() {
                    if matches!(sqlx_err, sqlx::Error::RowNotFound) {
                        let instance = IUniswapV2Pair::new(*pair_adr, self.provider.clone());
                        return Ok(instance.factory().call().await?._0 == self.address_book.factory);
                    }
                }
                return Err(err);
            }
        }
    }

    async fn token_reserves(&self, token0: &Address, token1: &Address) -> Result<Reserves> {
        match self.db.reserves(self.dex_id, token0, token1).await {
            // reserves are cached in redis
            Ok(reserves) => Ok(reserves),
            Err(_) => {
                let pair_adr = self.db.pair_adr(self.dex_id, token0, token1).await?;
                let reserves = self.fetch_reserves(&pair_adr).await?;

                // correct order of tokens
                let (r0, r1) = match *token0 < *token1 {
                    true => (reserves.0, reserves.1),
                    false => (reserves.1, reserves.0),
                };

                let data = UpdateReservesData {
                    token0: *token0,
                    token1: *token1,
                    reserves: Reserves(r0, r1),
                };
                self.db.update_reserves(self.dex_id, data).await?;

                Ok(Reserves(r0, r1))
            }
        }
    }

    async fn on_block(&self, header: Header) -> Result<()> {
        tracing::info!("handle new block: {}", header.number);
        // TODO: maybe chhose better approach for ordering
        REQUESTS_PER_BLOCK.store(0usize, Ordering::Relaxed);

        let filter = Filter::new()
            .event_signature(IUniswapV2Pair::Sync::SIGNATURE_HASH)
            .from_block(header.number);

        let mut updated_tokens = vec![];

        for log in self.provider.get_logs(&filter).await? {
            let sync = IUniswapV2Pair::Sync::decode_log(&log.inner, false)?;

            if !self.owns_pair(&sync.address).await? {
                continue;
            }

            let (token0, token1) = match self.db.pair_by_tokens(self.dex_id, &sync.address).await {
                Ok(r) => (r.0, r.1),
                Err(_) => {
                    let pair = self.fetch_pair(sync.address).await?;
                    self.add_pair(pair.clone()).await?;
                    (pair.token0, pair.token1)
                }
            };

            let data = UpdateReservesData {
                token0,
                token1,
                reserves: Reserves(sync.reserve0, sync.reserve1),
            };
            self.db.update_reserves(self.dex_id, data).await?;

            updated_tokens.push(token0);
            updated_tokens.push(token0);
        }

        let paths = find_triangular_arbitrage(&updated_tokens, Box::new(self.clone())).await?;
        tracing::info!("(uniswap-v2): found arbitrage paths: {}", paths.len());

        let best_arbitrages = self.best_arbitrages(paths).await?;
        tracing::info!("best arbitrages: {best_arbitrages:?}");

        for arbitrage in best_arbitrages.into_values() {
            self.s.send(arbitrage);
        }

        Ok(())
    }
}
