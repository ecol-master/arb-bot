use crate::{postgres::PostgresDB, tables::PairV2};
use alloy::{
    primitives::{Address, Uint},
    providers::RootProvider,
    pubsub::PubSubFrontend,
};
use anyhow::Result;
use arbbot_config::Config;
use ethereum_abi::IUniswapV2Pair;
use hashbrown::hash_map::{Entry, HashMap};
use std::sync::Arc;

type P = Arc<RootProvider<PubSubFrontend>>;
pub type StorageReserves = HashMap<Address, HashMap<Address, Uint<112, 2>>>;

pub struct Storage {
    reserves: StorageReserves,
    pairs_v2: HashMap<Address, (Address, Address)>,
    postgres: PostgresDB,
    provider: P,
}

impl Storage {
    /// Initialize `Storage` using data from `PostgresDB`
    pub async fn new(cfg: Config, provider: P) -> Result<Self> {
        let mut storage = Self {
            reserves: HashMap::new(),
            pairs_v2: HashMap::new(),
            postgres: PostgresDB::connect(&cfg.postgres).await?,
            provider: provider.clone(),
        };

        for pair_v2 in storage.postgres.select_pairs_v2().await? {
            storage.pairs_v2.insert(
                pair_v2.address.clone(),
                (pair_v2.token0.clone(), pair_v2.token1.clone()),
            );

            let pair_reserves = IUniswapV2Pair::new(pair_v2.address, provider.clone())
                .getReserves()
                .call()
                .await?;

            storage.add_pair_to_reserves(
                &pair_v2.token0,
                &pair_v2.token1,
                pair_reserves.reserve0,
                pair_reserves.reserve1,
            );
        }

        tracing::info!("📦 Storage initialized");
        tracing::info!(
            "📦 Load {} pairs_v2 from PostgreSQL",
            storage.pairs_v2.len()
        );
        Ok(storage)
    }

    fn add_pair_to_reserves(
        &mut self,
        token0: &Address,
        token1: &Address,
        reserve0: Uint<112, 2>,
        reserve1: Uint<112, 2>,
    ) {
        // add for reserve of token0 in pair with token1
        match self.reserves.entry(token0.clone()) {
            Entry::Vacant(entry) => {
                let mut new_map = HashMap::new();
                new_map.insert(token1.clone(), reserve0);
                entry.insert(new_map);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(token1.clone(), reserve0);
            }
        };

        // add for reserve of token1 in pair with token0
        match self.reserves.entry(token1.clone()) {
            Entry::Vacant(entry) => {
                let mut new_map = HashMap::new();
                new_map.insert(token0.clone(), reserve1);
                entry.insert(new_map);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(token0.clone(), reserve1);
            }
        }
    }

    pub async fn update_reserves(
        &mut self,
        pair_adr: &Address,
        reserve0: Uint<112, 2>,
        reserve1: Uint<112, 2>,
    ) -> Result<()> {
        match self.pairs_v2.get(pair_adr) {
            Some((token0, token1)) => {
                *self
                    .reserves
                    .get_mut(token0)
                    .unwrap()
                    .get_mut(token1)
                    .unwrap() = reserve0;
                *self
                    .reserves
                    .get_mut(token1)
                    .unwrap()
                    .get_mut(token0)
                    .unwrap() = reserve1;
            }

            // Insert new pair if its not created
            None => {
                let instance = IUniswapV2Pair::new(*pair_adr, self.provider.clone());
                let token0 = instance.token0().call().await?._0;
                let token1 = instance.token1().call().await?._0;
                self.pairs_v2
                    .insert(*pair_adr, (token0.clone(), token1.clone()));

                self.postgres
                    .insert_pair_v2(&PairV2 {
                        address: *pair_adr,
                        token0: token0.clone(),
                        token1: token1.clone(),
                    })
                    .await?;

                let reserves = instance.getReserves().call().await?;
                self.add_pair_to_reserves(&token0, &token1, reserves.reserve0, reserves.reserve1);
            }
        };

        tracing::info!("☑️ Reserves for pair {:?} updated!", pair_adr);

        Ok(())
    }

    pub fn reserves(&self) -> HashMap<Address, HashMap<Address, Uint<112, 2>>> {
        self.reserves.clone()
    }
}
