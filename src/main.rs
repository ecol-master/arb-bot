mod config;
mod types;
mod uniswap_v2;

use crate::{
    config::Config,
    types::IUniswaV2Factory,
    uniswap_v2::{
        process_swap_exact_tokens_for_tokens, swapExactETHForTokensCall,
        swapExactTokensForTokensCall, swapTokensForExactETHCall,
    },
};

use envconfig::Envconfig;
use serde_json;
use tokio::fs;
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;

use alloy::{
    consensus::{transaction, Transaction},
    dyn_abi::abi::token,
    primitives::{address, Address, TxNumber, U256},
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    sol,
    sol_types::SolCall,
};
use futures_util::{stream, StreamExt};
use std::{mem::swap, sync::Arc};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        //.with_writer(rollig::daily("logs", "router_02.log"))
        //.with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::init_from_env()?;
    info!("Started listen rpc: {}", config.rpc_url);

    let provider = Arc::new(
        ProviderBuilder::new()
            .on_ws(WsConnect::new(config.rpc_url))
            .await?,
    );

    let mut stream = provider
        .subscribe_pending_transactions()
        .await?
        .into_stream(); // Wait and take the next 3 transactions.
    info!("Awaiting pending transactions...");

    let router02_adr = address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
    let router03_adr = address!("0xE592427A0AEce92De3Edee1F18E0157C05861564");

    let uniswap_v2_factory_adr: Address = address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
    let uniswap_v2_factory = Arc::new(IUniswaV2Factory::new(
        uniswap_v2_factory_adr,
        provider.clone(),
    ));

    // Take the stream and print the pending transaction.

    while let Some(tx_hash) = stream.next().await {
        let provider = provider.clone();
        let tx = match provider.get_transaction_by_hash(tx_hash).await {
            Ok(Some(tx)) => tx,
            _ => continue,
        };

        let input = match tx.to() {
            Some(to) if to == router02_adr => {
                info!("Router02 tx, input: {:?}", tx.input());
                tx.input()
            }
            _ => continue,
        };

        if let Ok(swap_data) = swapExactTokensForTokensCall::abi_decode(&input, false) {
            info!("Processing swapExactTokensForTokens");
            info!(
                "SwapExactTokensForTokens Path: {:?}, AmountIn: {:?}",
                swap_data.path, swap_data.amountIn
            );

            let address = match swap_data.path.get(0) {
                Some(adr) => adr,
                None => continue,
            };

            let pair_adr = uniswap_v2_factory
                .getPair(swap_data.path[0], swap_data.path[1])
                .call()
                .await
                .unwrap()
                .pair;

            info!("Pair address: {:?}", pair_adr);
            process_swap_exact_tokens_for_tokens(swap_data, pair_adr, provider.clone()).await?;
        } else if let Ok(swap_data) = swapExactETHForTokensCall::abi_decode(&input, false) {
            info!(
                "SwapExactETHForTokens Path: {:?}, AmountIn: {:?}",
                swap_data.path, swap_data.amountOutMin
            );
        } else if let Ok(swap_data) = swapTokensForExactETHCall::abi_decode(&input, false) {
            info!(
                "SwapTokensForExactETH Path: {:?}, AmountIn: {:?}",
                swap_data.path, swap_data.amountOut
            );
        }
    }

    return Ok(());
}
