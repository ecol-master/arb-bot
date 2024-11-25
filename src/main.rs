mod tx_filter;

use ethers::{
    providers::{Middleware, Provider, Ws},
    types::Transaction,
};
use std::sync::Arc;
use tokio_stream::StreamExt;

use tx_filter::filter_transaction;

async fn get_transactions() {
    let ws_node_public = "wss://ethereum-rpc.publicnode.com";
    let ws = Ws::connect(ws_node_public).await;

    if ws.is_err() {
        println!("Error connecting to the node: {:?}", ws.err());
        return;
    }

    let provider = Arc::new(Provider::new(ws.unwrap()));

    let mut pending_txs = provider.subscribe_full_pending_txs().await.unwrap();
    println!("Listening for pending transactions...");

    let mut tx_counter: u128 = 0;
    while let Some(tx) = pending_txs.next().await {
        if let Some(tx) = filter_transaction(tx).await {
            println!("To: {}, Tx: {:?}", tx.to, tx.tx);
        }
        println!("recieved tx {}", tx_counter);
        tx_counter += 1;
    }

    //println!("Received 100 pending transactions...");

    //let output_file = "pending_txs.json";
    //let file = std::fs::File::create(output_file).unwrap();
    //serde_json::to_writer(file, &txs).expect("Failed to write to file");
    //println!("Saved pending transactions to {}", output_file);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    get_transactions().await;
    Ok(())
}
