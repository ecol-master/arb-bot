use alloy::{
    primitives::{Address, Uint},
    providers::RootProvider,
    pubsub::PubSubFrontend,
};
use anyhow::Result;
use ethereum_abi::IUniswapV2Pair;
use hashbrown::{hash_map::Entry, HashMap, HashSet};
use std::sync::Arc;

pub mod postgres;
use postgres::*;

pub struct Storage {
    reserves: HashMap<Address, HashMap<Address, Uint<112, 2>>>,
    pairs_v2: HashMap<Address, (Address, Address)>,
    postgres: PostgresDB,
}

type P = Arc<RootProvider<PubSubFrontend>>;

impl Storage {
    /// Initialize `Storage` using data from `PostgresDB`
    pub async fn new(provider: P) -> Result<Self> {
        let postgres = PostgresDB::connect().await?;

        let mut reserves = HashMap::new();
        let mut pairs_v2 = HashMap::new();

        for pair_v2 in postgres.select_pairs_v2().await? {
            pairs_v2.insert(
                pair_v2.address.clone(),
                (pair_v2.token0.clone(), pair_v2.token1.clone()),
            );

            let pair_reserves = IUniswapV2Pair::new(pair_v2.address, provider.clone())
                .getReserves()
                .call()
                .await?;

            // add for reserve of token0 in pair with token1
            match reserves.entry(pair_v2.token0.clone()) {
                Entry::Vacant(entry) => {
                    let mut new_map = HashMap::new();
                    new_map.insert(pair_v2.token1.clone(), pair_reserves.reserve0);
                    entry.insert(new_map);
                }
                Entry::Occupied(mut entry) => {
                    entry
                        .get_mut()
                        .insert(pair_v2.token1.clone(), pair_reserves.reserve0);
                }
            };

            // add for reserve of token1 in pair with token0
            match reserves.entry(pair_v2.token1.clone()) {
                Entry::Vacant(entry) => {
                    let mut new_map = HashMap::new();
                    new_map.insert(pair_v2.token0.clone(), pair_reserves.reserve1);
                    entry.insert(new_map);
                }
                Entry::Occupied(mut entry) => {
                    entry
                        .get_mut()
                        .insert(pair_v2.token0.clone(), pair_reserves.reserve1);
                }
            };
        }

        tracing::info!("📦 Storage initialized");
        Ok(Self {
            reserves,
            pairs_v2,
            postgres,
        })
    }

    pub fn reserves(&self) -> HashMap<Address, HashMap<Address, Uint<112, 2>>> {
        self.reserves.clone()
    }
}
