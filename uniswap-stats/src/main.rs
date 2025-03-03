use alloy::{
    primitives::address,
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    pubsub::PubSubFrontend,
    rpc::{self, types::Filter},
};
use anyhow::Result;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        //.with_writer(rolling::daily("logs", "processed_tx.log"))
        //.with_ansi(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let rpc_url = "wss://mainnet.infura.io/ws/v3/1bef1023164047a28c6e71599ea9d046";
    let ws = WsConnect::new(rpc_url);
    let provider = ProviderBuilder::new().on_ws(ws).await?;

    let latest_block = provider.get_block_number().await?;
    tracing::info!("latest block: {latest_block:?}");

    let usdc_eth_adr = address!("0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc");
    let filter = Filter::new().address(usdc_eth_adr).event(event_name);

    Ok(())
}
