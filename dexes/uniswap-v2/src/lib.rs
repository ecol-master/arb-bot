use alloy::{
    primitives::{address, Address, Uint},
    providers::{Provider, RootProvider},
    rpc::types::Filter,
    sol_types::SolEvent,
};
use anyhow::Result;
use bot_db::{tables::Pair, DB};
use bot_math::cpmm::{find_profit, find_triangular_arbitrage, ArbitrageData};
use dex_common::{AddressBook, Reserves, DEX};
use ethereum_abi::IUniswapV2Pair;
use futures_util::StreamExt;
use std::{collections::HashSet, sync::Arc};

const DEX_NAME: &str = "uniswap_v2";

type P = Arc<RootProvider>;

#[derive(Clone)]
pub struct UniswapV2 {
    dex_id: i32,
    address_book: AddressBook,

    db: DB,
    provider: P,
}

const USDT: Address = address!("0xdAC17F958D2ee523a2206206994597C13D831ec7");
const USDC: Address = address!("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
const DAI: Address = address!("0x6B175474E89094C44Da98b954EedeAC495271d0F");

const STABLE_COINS: [Address; 3] = [DAI, USDC, USDT];

impl UniswapV2 {
    pub async fn new(db: DB, provider: P) -> Result<Self> {
        let dex_id = db.postgres().get_dex_id(DEX_NAME).await?;

        let address_book = AddressBook {
            factory: address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f"),
            router: address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"),
        };

        Ok(Self {
            dex_id,
            address_book,
            db,
            provider,
        })
    }

    pub async fn fetch_pair(&self, pair_adr: Address) -> Result<Pair> {
        let instance = IUniswapV2Pair::new(pair_adr.clone(), self.provider.clone());
        let token0 = instance.token0().call().await?._0;
        let token1 = instance.token1().call().await?._0;

        Ok(Pair {
            address: pair_adr,
            dex_id: self.dex_id,
            token0,
            token1,
        })
    }

    pub async fn price_to_usd(
        &self,
        token: &Address,
        amount_in: Uint<256, 4>,
        amount_out: Uint<256, 4>,
    ) -> Option<(f64, f64)> {
        let adjacent = self.adjacent(token).await.ok()?;
        for stable in STABLE_COINS.iter() {
            if !adjacent.contains(stable) {
                continue;
            }
            let reserves = self.token_reserves(token, stable).await.ok()?;
            let (r_token, r_stable) = if *token < *stable {
                (reserves.0, reserves.1)
            } else {
                (reserves.1, reserves.0)
            };
            let amount_in_usd =
                amount_in * Uint::<256, 4>::from(r_stable) / Uint::<256, 4>::from(r_token);

            let amount_out_usd =
                amount_out * Uint::<256, 4>::from(r_stable) / Uint::<256, 4>::from(r_token);

            let amount_in_usd_str = amount_in_usd.to_string();
            let amount_in_usd: f64 = amount_in_usd_str.parse().ok()?;

            let amount_out_usd_str = amount_out_usd.to_string();
            let amount_out_usd: f64 = amount_out_usd_str.parse().ok()?;

            if *stable == USDT || *stable == USDC {
                return Some((amount_in_usd / 1_000_000.0, amount_out_usd / 1_000_000.0));
            }
            if *stable == DAI {
                return Some((
                    amount_in_usd / 1_000_000_000_000_000_000.0,
                    amount_out_usd / 1_000_000_000_000_000_000.0,
                ));
            }
            return None;
        }

        None
    }
}

// Run function for uniswap-v2, method drop structure, because after running there is no need to use
#[async_trait::async_trait]
impl DEX for UniswapV2 {
    async fn adjacent(&self, token: &Address) -> Result<HashSet<Address>> {
        let instance = ethereum_abi::IERC20::new(token.clone(), self.provider.clone());
        let ticket = instance.symbol().call().await?._0;
        self.db.adjacent(self.dex_id, &token).await
    }

    async fn fetch_reserves(&self, pair_adr: &Address) -> Result<Reserves> {
        let instance = IUniswapV2Pair::new(*pair_adr, self.provider.clone());
        let reserves = instance.getReserves().call().await?;
        Ok((reserves.reserve0, reserves.reserve1))
    }

    // TODO: think about correctness this
    async fn owns_pair(&self, pair_adr: &Address) -> Result<bool> {
        let instance = IUniswapV2Pair::new(*pair_adr, self.provider.clone());
        Ok(instance.factory().call().await?._0 == self.address_book.factory)
    }

    async fn token_reserves(&self, token0: &Address, token1: &Address) -> Result<Reserves> {
        match self.db.reserves(self.dex_id, token0, token1).await {
            // reserves are cached in redis
            Ok(reserves) => Ok(reserves),
            Err(_) => {
                let pair_adr = self.db.pair_adr(self.dex_id, token0, token1).await?;
                let reserves = self.fetch_reserves(&pair_adr).await?;

                // correct order of tokens
                let (r0, r1) = if *token0 < *token1 {
                    (reserves.0, reserves.1)
                } else {
                    (reserves.1, reserves.0)
                };

                self.db
                    .update_reserves(self.dex_id, token0, token1, r0, r1)
                    .await?;

                Ok((r0, r1))
            }
        }
    }

    async fn run(&self) -> Result<()> {
        let filter = Filter::new().event_signature(IUniswapV2Pair::Sync::SIGNATURE_HASH);
        let mut stream = self.provider.subscribe_blocks().await?.into_stream();

        while let Some(header) = stream.next().await {
            tracing::info!("⚡ block {:?}", header.number);
            let f = filter.clone().from_block(header.number);

            let mut updated_tokens = vec![];
            for log in self.provider.get_logs(&f).await? {
                let sync = IUniswapV2Pair::Sync::decode_log(&log.inner, false)?;

                if !self.owns_pair(&sync.address).await? {
                    tracing::info!("NOT from UniswapV2");
                    continue;
                }

                let (token0, token1) = match self.db.pair_tokens(self.dex_id, &sync.address).await {
                    Ok(r) => (r.0, r.1),
                    Err(_) => {
                        let pair = self.fetch_pair(sync.address).await?;

                        self.db.add_pair(pair.clone()).await?;
                        self.db.postgres().insert_pair(pair.clone()).await?;

                        (pair.token0, pair.token1)
                    }
                };

                self.db
                    .update_reserves(self.dex_id, &token0, &token1, sync.reserve0, sync.reserve1)
                    .await?;

                updated_tokens.push(token0);
                updated_tokens.push(token1);
            }

            let dex = Box::new(self.clone());

            // The vector of Vec<(token0, token1)>
            let paths = find_triangular_arbitrage(&updated_tokens, dex).await?;
            tracing::info!("(uniswap-v2): found {} arbitrage paths", paths.len());

            let mut unique_first_tokens = HashSet::new();

            for path in paths {
                let mut arbitrages_data = Vec::new();

                for tokens in &path {
                    arbitrages_data.push(ArbitrageData {
                        reserves: self.token_reserves(&tokens.0, &tokens.1).await?,
                        fee: Uint::from(3),
                    });
                }

                if let Some(profit) = find_profit(&arbitrages_data) {
                    let first_token = path.first().unwrap().0;

                    unique_first_tokens.insert(first_token.clone());

                    if first_token == USDT {
                        let profit_str = profit.1.to_string();
                        let usd: f64 = profit_str.parse().unwrap();
                        tracing::info!(
                            "(uniswap-v2): profit best in: {:?}, best out: {:?}$",
                            profit.0,
                            usd / 1_000_000.0
                        );
                    } else {
                        let prices = self.price_to_usd(&first_token, profit.0, profit.1).await;

                        match prices {
                            Some((amount_in_usd, amount_out_usd)) => {
                                tracing::info!(
                                    "(uniswap-v2): profit: best in: {amount_in_usd}$, best out: {amount_out_usd}$",
                                );
                            }
                            None => {
                                tracing::info!("(uniswap-v2): failed to calculate price in usd")
                            }
                        }
                    }
                }
            }

            tracing::info!("unique first tokens: {unique_first_tokens:?}")
        }
        Ok(())
    }
}
