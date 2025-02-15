use crate::types::IUniswapV2Pair;
use alloy::{
    primitives::{Address, Uint, U256},
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
};
use anyhow::anyhow;
use enum_iterator::Sequence;
use std::{
    collections::{BTreeSet, HashMap},
    hash::Hash,
    sync::{Arc, RwLock},
};
use tracing::info;
use tracing_appender::rolling::Rotation;

type PairV2Instance =
    IUniswapV2Pair::IUniswapV2PairInstance<PubSubFrontend, Arc<RootProvider<PubSubFrontend>>>;

pub type StorageReserves = HashMap<Address, HashMap<Address, Uint<112, 2>>>;
pub struct Storage {
    reserves: Arc<RwLock<StorageReserves>>,
}

impl Storage {
    pub async fn new(pairs: &Vec<PairV2Instance>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut reserves: HashMap<Address, HashMap<Address, Uint<112, 2>>> = HashMap::new();

        for pair in pairs {
            let token0_adr = pair.token0().call().await?._0;
            let token1_adr = pair.token1().call().await?._0;
            reserves
                .entry(token0_adr.clone())
                .or_insert(HashMap::new())
                .insert(token1_adr.clone(), Uint::from(0));

            reserves
                .entry(token1_adr)
                .or_insert(HashMap::new())
                .insert(token0_adr.clone(), Uint::from(0));
        }

        Ok(Self {
            reserves: Arc::new(RwLock::new(reserves)),
        })
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
