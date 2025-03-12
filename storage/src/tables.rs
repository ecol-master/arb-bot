use alloy::primitives::Address;

pub const PAIR_V2_TABLE: &str = "pairs_v2";
#[derive(Debug, Clone)]
pub struct PairV2 {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
}
