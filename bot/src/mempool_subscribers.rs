use alloy::{
    consensus::Transaction,
    primitives::{address, Address, FixedBytes, Log},
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    rpc::types::Filter,
    sol_types::SolEvent,
};
use anyhow::Result;
use crossbeam::channel::Sender;
use ethereum_abi::IUniswapV2Pair;
use futures_util::StreamExt;
use std::sync::Arc;

type S = Sender<FixedBytes<32>>;
type P = Arc<RootProvider<PubSubFrontend>>;

//pub async fn run_mempool_subscribers(sender: S, provider: P) -> Result<()> {
//let mut handles = Vec::new();

//let p = provider.clone();
//let s = sender.clone();
//handles.push(tokio::spawn(async move {
//if let Err(e) = subscribe_swaps_in_block(s, p).await {
//tracing::error!("Subscribe swaps in block: {e:?}");
//}
//}));

//handles.push(tokio::spawn(async move {
//if let Err(e) = subscribe_popular_swap_makers(sender, provider).await {
//tracing::error!("Subscribe popular swap makers: {e:?}");
//}
//}));

//for handle in handles {
//handle.await;
//}
//Ok(())
//}

pub async fn subscribe_sync_in_block(s: Sender<Log>, provider: P) -> Result<()> {
    let mut filter = Filter::new().event_signature(IUniswapV2Pair::Sync::SIGNATURE_HASH);

    let mut stream = provider.subscribe_blocks().await?.into_stream();
    while let Some(header) = stream.next().await {
        let f = filter.clone().from_block(header.number);
        let logs = provider.get_logs(&f).await?;

        for log in logs {
            s.send(log.inner)?;
        }
    }

    Ok(())
}

const ROUTER_V2: Address = address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");

/// Subscribe for transactions on Router02
async fn subscribe_router_v2(s: S, provider: P) -> Result<()> {
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
async fn subscribe_popular_swap_makers(_s: S, _provider: P) -> Result<()> {
    tracing::info!("Run subscription for most popular swap makers");
    Ok(())
}
