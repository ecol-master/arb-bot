mod config;
mod contracts;
mod uniswapv2;

use crate::{
    config::Config,
    contracts::{
        swapExactETHForTokensCall, swapExactTokensForTokensCall, swapTokensForExactETHCall,
        IUniswapV3Pool, IERC20,
    },
    uniswapv2::{UniswapV2Pair, ERC20},
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
    primitives::{address, TxNumber, U256},
    providers::{self, Provider, ProviderBuilder, WsConnect},
    sol,
    sol_types::SolCall,
};
use futures_util::{stream, StreamExt};
use hex;
use std::collections::BTreeSet;
use std::{mem::swap, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let rolling_appender = rolling::daily("logs", "router_02.log");
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(rolling_appender)
        .with_ansi(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::init_from_env()?;
    info!("Started listen rpc: {}", config.rpc_url);

    let provider = Arc::new(
        ProviderBuilder::new()
            .on_ws(WsConnect::new(config.rpc_url))
            .await?,
    );

    let pairs: BTreeSet<String> = BTreeSet::from([
        String::from("0xcC6f7439147338E0401A76dB978d7d0ca6E5bfeE"),
        String::from("0x859f7092f56c43BB48bb46dE7119d9c799716CDF"),
        String::from("0xA43fe16908251ee70EF74718545e4FE6C5cCEc9f"),
        String::from("0x21b8065d10f73EE2e260e5B47D3344d3Ced7596E"),
        String::from("0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852"),
        String::from("0x52c77b0CB827aFbAD022E6d6CAF2C44452eDbc39"),
        String::from("0x308C6fbD6a14881Af333649f17f2FdE9cd75e2a6"),
    ]);

    let mut stream = provider
        .subscribe_pending_transactions()
        .await?
        .into_stream(); // Wait and take the next 3 transactions.
    info!("Awaiting pending transactions...");

    let router02_adr = address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
    let router03_adr = address!("0xE592427A0AEce92De3Edee1F18E0157C05861564");
    // Take the stream and print the pending transaction.
    while let Some(tx_hash) = stream.next().await {
        let tx = match provider.get_transaction_by_hash(tx_hash).await {
            Ok(Some(tx)) => tx,
            _ => continue,
        };

        //if pairs.contains(&tx.to().unwrap().to_string()) {
        //    info!("{:?}", tx.to());
        //}

        let input = match tx.to() {
            Some(to) if to == router02_adr => {
                info!("Router02 tx, input: {:?}", tx.input());
                tx.input()
            }
            _ => continue,
        };

        //let input = match hex::decode(tx.input()) {
        //    Ok(input) => input,
        //    Err(e) => {
        //        info!("Deconding error: {e:?}");
        //        continue;
        //    }
        //};

        // Decode the input using the generated `swapExactTokensForTokens` bindings.
        if let Ok(swap_data) = swapExactTokensForTokensCall::abi_decode(&input, false) {
            info!(
                "SwapExactTokensForTokens Path: {:?}, AmountIn: {:?}",
                swap_data.path, swap_data.amountIn
            );
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
        };
    }

    //while let Some(tx) = provider.sub

    Ok(())
}
