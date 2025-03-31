use crate::{postgres::PostgresDB, tables::PairV2};
use alloy::{
    dyn_abi::abi::token,
    primitives::{Address, Uint},
};
use anyhow::{anyhow, Result};
use arbbot_config::Config;
use ethereum_abi::IUniswapV2Pair;
use hashbrown::{
    hash_map::{Entry, HashMap},
    HashSet,
};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum PairState {
    NotInitialize,
    Updated,
}

#[derive(Clone, Debug)]
pub struct PairV2Data {
    pub reserve0: Uint<112, 2>,
    pub reserve1: Uint<112, 2>,
    pub k: Uint<256, 4>,
}

pub struct MemDB {
    pairs_v2: HashMap<Address, (Address, Address)>,

    // token0 -> HashSet{token1}
    adjacent_tokens: HashMap<Address, HashSet<Address>>,

    //pairs_v2_state: HashMap<Address, PairState>,

    // The state of actual information about reserves
    reserves: HashMap<Address, HashMap<Address, Uint<112, 2>>>,

    // (token0, token1) -> pair_address
    // NOTE: tokens must be sort
    tokens_to_pair: HashMap<(Address, Address), Address>,

    //pairs_v2_data: HashMap<Address, PairV2Data>,
    postgres: PostgresDB,
}

impl MemDB {
    /// Initialize `Storage` using data from `PostgresDB`
    pub async fn new(cfg: Config) -> Result<Self> {
        let mut storage = Self {
            pairs_v2: HashMap::new(),
            adjacent_tokens: HashMap::new(),
            reserves: HashMap::new(),
            tokens_to_pair: HashMap::new(),
            postgres: PostgresDB::connect(&cfg.postgres).await?,
        };
        tracing::info!("📦 Storage initialized");

        let pairs_v2 = storage.postgres.select_pairs_v2().await?;
        tracing::info!("📦 Load {} pairs_v2 from PostgreSQL", pairs_v2.len());

        for pair_v2 in pairs_v2 {
            storage.insert_pair(pair_v2);
        }

        Ok(storage)
    }

    pub fn pair_exists(&self, pair_adr: &Address) -> bool {
        self.pairs_v2.get(pair_adr).is_some()
    }

    pub async fn add_pair_v2(&mut self, pair: PairV2) -> Result<()> {
        if self.pair_exists(&pair.address) {
            return Err(anyhow!("Pair {:?} already exists", pair.address));
        }

        self.insert_pair(pair.clone());

        self.postgres.insert_pair_v2(&pair).await
    }

    pub fn update_reserves(
        &mut self,
        pair_adr: &Address,
        reserve0: Uint<112, 2>,
        reserve1: Uint<112, 2>,
    ) -> Result<()> {
        let (token0, token1) = match self.pairs_v2.get(pair_adr) {
            Some((token0, token1)) => (token0, token1),
            None => return Err(anyhow!("Pair: {pair_adr:?} not found")),
        };

        // update reserve0 in pair with token1
        match self.reserves.entry(*token0) {
            Entry::Occupied(mut entry) => {
                let mut_map = entry.get_mut();
                match mut_map.get_mut(token1) {
                    Some(amount) => *amount = reserve0,
                    None => {
                        mut_map.insert(*token1, reserve0);
                    }
                }
            }
            Entry::Vacant(entry) => {
                let mut new_map = HashMap::new();
                new_map.insert(*token1, reserve0);
                entry.insert(new_map);
            }
        }

        // update reserve1 in pair with token0
        match self.reserves.entry(*token1) {
            Entry::Occupied(mut entry) => {
                let mut_map = entry.get_mut();
                match mut_map.get_mut(token0) {
                    Some(amount) => *amount = reserve1,
                    None => {
                        mut_map.insert(*token0, reserve1);
                    }
                }
            }
            Entry::Vacant(entry) => {
                let mut new_map = HashMap::new();
                new_map.insert(*token0, reserve1);
                entry.insert(new_map);
            }
        }

        Ok(())
    }

    pub fn reserves_for(
        &self,
        token0: &Address,
        token1: &Address,
    ) -> Option<(Uint<112, 2>, Uint<112, 2>)> {
        let reserve0 = self.reserves.get(token0).and_then(|map| map.get(token1))?;
        let reserve1 = self.reserves.get(token1).and_then(|map| map.get(token0))?;

        Some((*reserve0, *reserve1))
    }

    /// Returns: HashSet<(token1, pair_adr)>
    pub fn adjacent_for(&self, token0: &Address) -> Result<HashSet<Address>> {
        match self.adjacent_tokens.get(token0) {
            Some(set) => Ok(set.clone()),
            None => Err(anyhow!("No pairs for token: {token0:?}")),
        }
    }

    pub fn key_from_tokens(token0: &Address, token1: &Address) -> (Address, Address) {
        if token0 < token1 {
            (*token0, *token1)
        } else {
            (*token1, *token0)
        }
    }

    // pub fn get_pair_v2_data(&self, pair_adr: &Address) -> Result<PairV2Data> {
    //     match self.pairs_v2_data.get(pair_adr) {
    //         Some(data) => Ok(data.clone()),
    //         None => Err(anyhow!("Not found data for pair: {pair_adr:?}")),
    //     }
    // }

    pub fn lookup_pair_adr(&self, token0: &Address, token1: &Address) -> Result<Address> {
        let key = Self::key_from_tokens(token0, token1);
        match self.tokens_to_pair.get(&key) {
            Some(adr) => Ok(*adr),
            None => Err(anyhow!(
                "Not found pair for token0: {token0:?} and token1: {token1:?}"
            )),
        }
    }

    /// Private function. Insert tokens into `tokens_graph`
    fn insert_pair(&mut self, pair: PairV2) -> Result<()> {
        match self.adjacent_tokens.entry(pair.token0) {
            Entry::Vacant(entry) => {
                let mut new_set = HashSet::new();
                new_set.insert(pair.token1.clone());
                entry.insert(new_set);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(pair.token1.clone());
            }
        }

        // add token0 in token1's hash-set
        match self.adjacent_tokens.entry(pair.token1) {
            Entry::Vacant(entry) => {
                let mut new_set = HashSet::new();
                new_set.insert(pair.token0.clone());
                entry.insert(new_set);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().insert(pair.token0.clone());
            }
        }

        let pair_key = Self::key_from_tokens(&pair.token0, &pair.token1);
        self.tokens_to_pair.insert(pair_key, pair.address.clone());

        self.pairs_v2
            .insert(pair.address, (pair.token0, pair.token1));
        Ok(())
    }
}
