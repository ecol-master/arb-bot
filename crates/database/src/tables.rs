use alloy::primitives::Address;

pub const PAIRS_TABLE: &str = "trading_pairs";
pub const DEXES_TABLE: &str = "dexes";
pub const TICKERS_TABLE: &str = "token_tickers";

/// `Pair` represents the trading pair in DEX
#[derive(Debug, Clone)]
pub struct Pair {
    pub address: Address,
    pub dex_id: i32,
    pub token0: Address,
    pub token1: Address,
}


#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Dex {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Ticker {
    pub token: Address,
    pub ticker: String,
}

// These structs are needed for sqlx::query_as
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PairRaw {
    pub address: [u8; 20],
    pub dex_id: i32,
    pub token0: [u8; 20],
    pub token1: [u8; 20],
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct TickerRaw {
    pub token: [u8; 20],
    pub ticker: String,
}
