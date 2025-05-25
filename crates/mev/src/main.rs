use anyhow::Result;
use futures_util::StreamExt;
use mev_share::sse::EventClient;

#[tokio::main]
async fn main() -> Result<()> {
    bot_logger::init_logger(tracing::Level::INFO);

    let mev_share_rpc = "https://mev-share.flashbots.net";
    let client = EventClient::default();
    let mut stream = client.events(mev_share_rpc).await?;
    tracing::info!("Subscribed to events from {mev_share_rpc}");

    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                tracing::info!("Received event: {:?}", event);
            }
            Err(e) => {
                tracing::error!("Error receiving event: {:?}", e);
            }
        }
    }

    Ok(())
}
