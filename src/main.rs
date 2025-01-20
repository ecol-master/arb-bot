mod config;
mod contracts;

use crate::{
    config::Config,
    contracts::{IUniswapV3Pool, IERC20},
};

use envconfig::Envconfig;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use alloy::{
    consensus::Transaction,
    dyn_abi::abi::token,
    primitives::{address, TxNumber, U256},
    providers::{self, Provider, ProviderBuilder, WsConnect},
    sol,
};
use futures_util::{stream, StreamExt};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
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
    println!("Awaiting pending transactions...");

    let router02_adr = address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
    let router03_adr = address!("0xE592427A0AEce92De3Edee1F18E0157C05861564");
    // Take the stream and print the pending transaction.
    let handle = tokio::spawn(async move {
        while let Some(tx_hash) = stream.next().await {
            // Get the transaction details.
            let tx = match provider.get_transaction_by_hash(tx_hash).await {
                Ok(tx) => match tx {
                    Some(tx) => tx,
                    None => continue,
                },
                _ => continue,
            };

            if let Some(to) = tx.to() {
                if to == router02_adr {
                    info!("Router02: {:?}", tx.to());
                }
                if to == router03_adr {
                    info!("Router03: {:?}", tx.to());
                }
            }
        }
    });

    handle.await?;

    //while let Some(tx) = provider.sub

    Ok(())
}
