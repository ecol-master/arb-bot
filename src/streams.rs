use crate::event_filters::uniswap::get_uniswap_v2_filter;

use ethers::{
    abi::{ParamType, Token},
    providers::{Middleware, Provider, Ws},
    types::{BlockNumber, Bytes, Filter, Log, Transaction, H160, U256, U64},
};

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast::Sender;
use tokio_stream::StreamExt;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct NewBlock {
    pub block_number: U64,
    pub base_fee: U256,
    pub next_base_fee: U256,
}

#[derive(Debug, Clone, Serialize)]
pub enum Event {
    Block(NewBlock),
    PendingTx(Transaction),
    Log(Log),
}

#[derive(Clone, Debug, Serialize)]
pub struct UniswapVeMethod {
    signature: &'static str,
    tokens: Vec<Token>,
}

#[derive(Clone, Debug, Serialize)]
pub struct UniswapV2Tx {
    from: H160,
    to: Option<H160>,
    value: U256,
    gas_price: Option<U256>,
    priority_fee: Option<U256>,
    tx_type: Option<U64>,
    method: Option<UniswapVeMethod>,
}

//  signature: "swapExactTokensForTokens(uint256,uint256,address[],address,uint256)",
fn get_uniswap_v2_methods() -> HashMap<&'static str, (&'static str, Vec<ParamType>)> {
    HashMap::from([(
        "38ed1739",
        (
            "swapExactTokensForTokens(uint256,uint256,address[],address,uint256);",
            vec![
                ParamType::Uint(256),                           // amountIn
                ParamType::Uint(256),                           // amountOutMin
                ParamType::Array(Box::new(ParamType::Address)), // path
                ParamType::Address,                             // to
                ParamType::Uint(256),                           // deadline
            ],
        ),
    )])
}

/// Decodes the input selector and get the method signature with params for Uniswap V2
pub fn get_method_by_input(
    methods: &HashMap<&'static str, (&'static str, Vec<ParamType>)>,
    input: Bytes,
) -> Option<UniswapVeMethod> {
    let selector = hex::encode(&input.0[..4]);
    println!("selector: {}", selector);

    if let Some(result) = methods.get(selector.as_str()) {
        println!("found method: {}", result.0);
        if let Ok(decoded) = ethers::abi::decode(&result.1, &input.0[4..]) {
            return Some(UniswapVeMethod {
                signature: result.0,
                tokens: decoded,
            });
        }
    }

    return None;
}

pub async fn uniswap_v2_stream(provider: Arc<Provider<Ws>>, _event_sender: Sender<Event>) {
    let filter = get_uniswap_v2_filter();

    let mut stream = provider
        .subscribe_logs(&filter)
        .await
        .expect("Failed to create Uniswap V2 stream");

    println!("Intialized log subscription");

    let uniswap_v2_methods = get_uniswap_v2_methods();

    while let Some(log) = stream.next().await {
        // processing the log
        if let Some(tx_hash) = log.transaction_hash {
            let tx = provider.get_transaction(tx_hash).await.unwrap().unwrap();
            //let _ = decode_uniswap_tx_input(tx.clone().input);

            let uniswap_tx = UniswapV2Tx {
                from: tx.clone().from,
                to: tx.clone().to,
                value: tx.value,
                gas_price: tx.gas_price,
                priority_fee: tx.clone().max_fee_per_gas,
                tx_type: tx.clone().transaction_type,
                method: get_method_by_input(&uniswap_v2_methods, tx.clone().input),
            };

            let _min_value = 10u64.pow(10);
            let uniswap_v2_router = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";

            if uniswap_tx.to.is_some()
                && uniswap_tx.to.unwrap() == uniswap_v2_router.parse().unwrap()
            {
                dbg!(uniswap_tx);
                dbg!(tx.input);
            }
        }
    }
}

//const HTTP_URL: &str = "https://rpc.flashbots.net";

pub async fn erc20_transfer(provider: Arc<Provider<Ws>>, event_sender: Sender<Event>) {
    let last_block = provider
        .get_block(BlockNumber::Latest)
        .await
        .expect("failed to get last block")
        .unwrap()
        .number
        .unwrap();

    println!("last_block: {last_block}");

    let erc20_transfer_filter = Filter::new()
        .from_block(last_block - 25)
        .event("Transfer(address,address,uint256)");

    let mut stream = provider
        .subscribe_logs(&erc20_transfer_filter)
        .await
        .expect("failed to create erc20 stream");

    while let Some(result) = stream.next().await {
        dbg!("ERC20 EVENT: {}", result);
    }
}
