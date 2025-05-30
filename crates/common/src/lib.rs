use alloy::primitives::Uint;

#[derive(Debug, Clone)]
pub struct Reserves(pub Uint<112, 2>, pub Uint<112, 2>);

#[derive(Debug, thiserror::Error)]
pub enum DexError {
    #[error("Max rpc request per block")]
    BlockRpcLimitExceed,
}
