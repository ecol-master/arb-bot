mod config;

use crate::config::Config;
use envconfig::Envconfig;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use alloy::providers::{Provider, ProviderBuilder, WsConnect};
use futures_util::{stream, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config = Config::init_from_env()?;
    info!("Started listen rpc: {}", config.rpc_url);

    let ws = WsConnect::new(config.rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    let subscription = provider.subscribe_blocks().await?;
    let mut stream = subscription.into_stream();

    while let Some(block) = stream.next().await {
        info!("{}", block.number);
    }

    Ok(())
}
