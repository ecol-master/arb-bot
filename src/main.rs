mod config;
mod event_filters;
mod streams;

use config::Config;
use envconfig::Envconfig;

use ethers::providers::{Provider, Ws};
use log::info;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use streams::{uniswap_v2_stream, Event};

use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct CustomLog {
    address: String,
    topics: Vec<String>,
    data: String,
}

async fn event_handler(event_sender: Sender<Event>) {
    let mut event_receiver = event_sender.subscribe();
    let mut events: Vec<Event> = Vec::new();

    loop {
        match event_receiver.recv().await {
            Ok(event) => events.push(event),
            Err(_) => break,
        }
    }

    let _ = std::fs::write("events_log.json", serde_json::to_string(&events).unwrap());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::init_from_env()?;

    let ws = Ws::connect(config.ws_address).await?;
    let provider: Arc<Provider<Ws>> = Arc::new(Provider::new(ws));
    let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let mut set: JoinSet<()> = JoinSet::new();

    set.spawn(uniswap_v2_stream(provider.clone(), event_sender.clone()));

    while let Some(res) = set.join_next().await {
        info!("{:?}", res);
    }

    Ok(())
}
