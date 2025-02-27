use alloy::{
    consensus::Transaction,
    primitives::{address, Address, FixedBytes},
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
};
use crossbeam::channel::Sender;
use futures_util::StreamExt;
use std::sync::Arc;

type S = Sender<FixedBytes<32>>;
type P = Arc<RootProvider<PubSubFrontend>>;

pub async fn run_mempool_subscribers(s: S, provider: P) -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = Vec::new();

    let p = provider.clone();
    let sender = s.clone(); 
    handles.push(tokio::spawn(async move {
        if let Err(e) = subscribe_router_v2(sender, p).await {
            tracing::error!("Subscribe Router V2 error: {e:?}");
        }
    }));

    handles.push(tokio::spawn(async move {
        if let Err(e) = subscribe_popular_swap_makers(s.clone(), provider).await {
            tracing::error!("Subscribe popular swap makers: {e:?}");
        }
    }));

    for handle in handles {
        handle.await;
    }
    Ok(())
}

const ROUTER_V2: Address = address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");

/// Subscribe for transactions on Router02
async fn subscribe_router_v2(s: S, provider: P) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Run subscription for RouterV2");
    let mut stream = provider
        .subscribe_pending_transactions()
        .await?
        .into_stream();

    while let Some(tx_hash) = stream.next().await {
        let tx = match provider.get_transaction_by_hash(tx_hash).await {
            Ok(Some(tx)) => tx,
            _ => continue,
        };

        if let Some(to) = tx.to() {
            if to == ROUTER_V2 {
                s.send(tx.inner.signature_hash())?;
            }
        }
    }
    Ok(())
}

/// Subscribe most popular swap makers
async fn subscribe_popular_swap_makers(
    _s: S,
    _provider: P,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Run subscription for most popular swap makers");
    Ok(())
}
