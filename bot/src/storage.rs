use alloy::{
    primitives::{Address, Uint},
    providers::RootProvider,
    pubsub::PubSubFrontend,
};
use anyhow::anyhow;
use ethereum_abi::IUniswapV2Pair;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, RwLock},
};
use tracing::info;

type PairV2Instance =
    IUniswapV2Pair::IUniswapV2PairInstance<PubSubFrontend, Arc<RootProvider<PubSubFrontend>>>;

pub type StorageReserves = HashMap<Address, HashMap<Address, Uint<112, 2>>>;
pub struct Storage {
    reserves: Arc<RwLock<StorageReserves>>,
}

impl Storage {
    pub fn new() -> Self {
        let storage = Self {
            reserves: Arc::new(RwLock::new(HashMap::new())),
        };
        /*
        let mut reserves: HashMap<Address, HashMap<Address, Uint<112, 2>>> = HashMap::new();

        for pair in pairs {
            let token0_adr = pair.token0().call().await?._0;
            let token1_adr = pair.token1().call().await?._0;
            let pair_reserves = pair.getReserves().call().await?;
            reserves
                .entry(token0_adr.clone())
                .or_insert(HashMap::new())
                .insert(token1_adr.clone(), pair_reserves.reserve0);

            reserves
                .entry(token1_adr)
                .or_insert(HashMap::new())
                .insert(token0_adr.clone(), pair_reserves.reserve1);
        }

        Ok(Self {
            reserves: Arc::new(RwLock::new(reserves)),
        })
        */
        storage
    }

    pub async fn add_pair(
        &self,
        token0: &Address,
        token1: &Address,
        reserve0: Uint<112, 2>,
        reserve1: Uint<112, 2>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        /*
        self.reserves
            .entry(*token0)
            .or_insert(HashMap::new())
            .insert(token1_adr.clone(), pair_reserves.reserve0);

        reserves
            .entry(token1_adr)
            .or_insert(HashMap::new())
            .insert(token0_adr.clone(), pair_reserves.reserve1);
        */
        // add for reserve of token0 in pair with token1
        match self.reserves.write().unwrap().entry(*token0) {
            Entry::Vacant(entry) => {
                let mut new_map = HashMap::new();
                new_map.insert(*token1, reserve0);
                entry.insert(new_map);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(*token1, reserve0);
            }
        };

        // add for reserve of token1 in pair with token0
        match self.reserves.write().unwrap().entry(*token1) {
            Entry::Vacant(entry) => {
                let mut new_map = HashMap::new();
                new_map.insert(*token0, reserve1);
                entry.insert(new_map);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(*token0, reserve1);
            }
        };
        Ok(())
    }

    pub async fn update_reserves(
        &self,
        token0: Address,
        token1: Address,
        new_reserves: IUniswapV2Pair::getReservesReturn,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Update reserves for pair {token0:?}/{token1:?} -> {:?} {:?}",
            new_reserves.reserve0, new_reserves.reserve1
        );
        *self
            .reserves
            .write()
            .map_err(|_| anyhow!("failed write"))?
            .get_mut(&token0)
            .ok_or(anyhow!("Not found token0 {token0:?} in reserves"))?
            .get_mut(&token1)
            .ok_or(anyhow!("Failed to updated suply for {token1:?}"))? = new_reserves.reserve0;

        *self
            .reserves
            .write()
            .map_err(|_| anyhow!("failed write"))?
            .get_mut(&token1)
            .ok_or(anyhow!("Not found token0 {token0:?} in reserves"))?
            .get_mut(&token0)
            .ok_or(anyhow!("Failed update supply for {token0:?}"))? = new_reserves.reserve1;
        Ok(())
    }

    // return cloned reserves
    pub fn get_reserves(&self) -> StorageReserves {
        self.reserves.read().unwrap().clone()
    }
}
