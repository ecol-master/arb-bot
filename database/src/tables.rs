use alloy::primitives::Address;

pub const PAIRS_TABLE: &str = "trading_pairs";
pub const DEXES_TABLE: &str = "dexes";

#[derive(Debug, Clone)]
pub struct Pair {
    pub address: Address,
    pub dex_id: i32,
    pub token0: Address,
    pub token1: Address,
}

/// This struct is needed for sqlx::query_as
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PairRaw {
    pub address: [u8; 20],
    pub dex_id: i32,
    pub token0: [u8; 20],
    pub token1: [u8; 20],
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct DEX {
    id: i32,
    name: String,
}
