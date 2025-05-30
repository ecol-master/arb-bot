use alloy::{
    primitives::{Address, Uint},
    rpc::types::Header,
};
use anyhow::Result;
use kronos_common::Reserves;
use std::collections::HashSet;

#[async_trait::async_trait]
pub trait DEX: Send + Sync {
    // DEX logic for new block
    // async fn on_block(&self, header: Header) -> Result<()>;

    async fn process_block(&self, block: Header) -> Result<()>;

    async fn fetch_reserves(&self, pair_adr: &Address) -> Result<Reserves>;

    // check that current pair is from this dex
    async fn owns_pair(&self, pair_adr: &Address) -> Result<bool>;

    // required db methods
    async fn adjacent(&self, token: &Address) -> Result<HashSet<Address>>;

    // returns (r0, r1) where r0 - reserve token0 in pair with token1. same for token1
    // coming (token0, token) may be in any order
    async fn token_reserves(&self, token0: &Address, token1: &Address) -> Result<Reserves>;
}

#[derive(Clone, Debug)]
pub struct AddressBook {
    pub factory: Address,
    pub router: Address,
}

#[derive(Debug)]
pub struct Arbitrage {
    pub dex_id: i32,
    pub amount_in: Uint<256, 4>,
    pub revenue: Uint<256, 4>,
    pub path: Vec<(Address, Address)>,
}
