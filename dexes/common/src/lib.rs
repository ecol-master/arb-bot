use alloy::primitives::{Address, Uint};
use anyhow::Result;
use std::collections::HashSet;

pub type Reserves = (Uint<112, 2>, Uint<112, 2>);

#[async_trait::async_trait]
pub trait DEX: Send + Sync {
    // start the process of
    async fn run(&self) -> Result<()>;

    async fn fetch_reserves(&self, pair_adr: &Address) -> Result<Reserves>;

    // required db methods
    async fn adjacent(&self, token: &Address) -> Result<HashSet<Address>>;

    // returns (r0, r1) where r0 - reserve token0 in pair with token1. same for token1
    // coming (token0, token) may be in any order
    async fn token_reserves(&self, token0: &Address, token1: &Address) -> Result<Reserves>;
}
