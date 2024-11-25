use ethers::{core::k256::elliptic_curve::consts::U160, types::Transaction};
use std::collections::HashMap;

pub struct WrappedTx {
    pub tx: Transaction,
    pub to: String,
}

pub async fn filter_transaction(tx: Transaction) -> Option<WrappedTx> {
    let white_list = HashMap::from([
        (
            "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D",
            "Uniswap V2 Router",
        ),
        (
            "0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f",
            "Uniswap V2 Factory Mainnet",
        ),
        (
            "0xF62c03E08ada871A0bEb309762E260a7a6a880E6",
            "Uniswap V2 Factor Sepolia",
        ),
        (
            "0xf1D7CC64Fb4452F05c498126312eBE29f30Fbcf9",
            "UV@ Artbitrum",
        ),
        (
            "0x9e5A52f57b3038F1B8EeE45F28b3C1967e22799C",
            "Avalanche UV2",
        ),
        (
            "0x68b3465833fb72a70ecdf485e0e4c7bd8665fc45",
            "Uniswap V3 Router",
        ),
    ]);
    if let Some(to) = tx.to {
        let to = to.to_string();
        if white_list.contains_key(to.as_str()) {
            return Some(WrappedTx { tx, to });
        }
    }

    return None;
}
