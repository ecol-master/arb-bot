mod config;
mod strategy;
mod types;
mod uniswap;

use crate::config::Config;

use envconfig::Envconfig;
use tracing::{info, Level};
use tracing_appender::rolling;
use tracing_subscriber::FmtSubscriber;

use alloy::{
    consensus::{transaction, Transaction},
    primitives::{address, Address, TxNumber, U256},
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        //.with_writer(rolling::daily("logs", "router_02.log"))
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

    strategy::run_strategy(provider.clone()).await?;

    //let router03_adr = address!("0xE592427A0AEce92De3Edee1F18E0157C05861564");

    return Ok(());
}
