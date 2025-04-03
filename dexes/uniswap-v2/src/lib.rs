use alloy::{
    primitives::{Address, Uint},
    providers::{Provider, RootProvider},
    rpc::types::Filter,
    sol_types::SolEvent,
};
use anyhow::Result;
use bot_db::{
    tables::{Dex, Pair},
    DB,
};
use bot_math::find_triangular_arbitrage;
use dex_common::{Reserves, DEX};
use ethereum_abi::IUniswapV2Pair;
use futures_util::StreamExt;
use std::{collections::HashSet, hash::Hash, sync::Arc};

const DEX_NAME: &str = "uniswap_v2";

type P = Arc<RootProvider>;

#[derive(Clone)]
pub struct UniswapV2 {
    dex_id: i32,

    db: DB,
    provider: P,
}

impl UniswapV2 {
    pub async fn new(db: DB, provider: P) -> Result<Self> {
        let dex_id = db.postgres().get_dex_id(DEX_NAME).await?;
        Ok(Self {
            dex_id,
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
}

// Run function for uniswap-v2, method drop structure, because after running there is no need to use
#[async_trait::async_trait]
impl DEX for UniswapV2 {
    async fn adjacent(&self, token: &Address) -> Result<HashSet<Address>> {
        self.db.adjacent(self.dex_id, &token).await
    }

    async fn fetch_reserves(&self, pair_adr: &Address) -> Result<Reserves> {
        let instance = IUniswapV2Pair::new(*pair_adr, self.provider.clone());
        let reserves = instance.getReserves().call().await?;
        Ok((reserves.reserve0, reserves.reserve1))
    }

    async fn token_reserves(&self, token0: &Address, token1: &Address) -> Result<Reserves> {
        match self.db.reserves(self.dex_id, token0, token1).await {
            // reserves are cached in redis
            Ok(reserves) => Ok(reserves),
            Err(_) => {
                let pair_adr = self.db.pair_adr(self.dex_id, token0, token1).await?;
                let reserves = self.fetch_reserves(&pair_adr).await?;

                // corrent order of tokens
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

            let mut updated_tokens = Vec::new();

            for log in self.provider.get_logs(&f).await? {
                let sync = IUniswapV2Pair::Sync::decode_log(&log.inner, false)?;

                tracing::info!("update pair: {:?}", sync.address);
                let (token0, token1) = match self.db.pair_tokens(self.dex_id, &sync.address).await {
                    Ok(r) => (r.0, r.1),
                    Err(_) => {
                        let pair = self.fetch_pair(sync.address).await?;
                        self.db.add_pair(pair.clone()).await?;
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
            let _paths = find_triangular_arbitrage(&updated_tokens, dex).await?;
        }
        Ok(())
    }
}
