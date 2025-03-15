use alloy::{
    consensus::Transaction,
    network::TransactionBuilder,
    node_bindings::Anvil,
    primitives::FixedBytes,
    providers::{Provider, ProviderBuilder, RootProvider},
    pubsub::PubSubFrontend,
    rpc::types::TransactionRequest,
};
use crossbeam::channel::{Receiver, Sender};
use ethereum_abi::IUniswapV2Pair::Swap;
use std::ops::Range;
use std::sync::Arc;
use tokio::task::JoinHandle;

type R = Receiver<FixedBytes<32>>;
type S = Sender<Swap>;
type P = Arc<RootProvider>;

const PORTS: Range<u16> = 8000..8002;
pub async fn run_mempool_searches(r: R, s: S, p: P) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Start running mempool runners");

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    let alchemy_rpc_url = String::from("");

    for port in PORTS {
        let provider = p.clone();
        let r_clone = r.clone();
        let s_clone = s.clone();
        let rpc_url = alchemy_rpc_url.clone();

        let handle = tokio::spawn(async move {
            if let Err(e) = run_anvil(port, rpc_url, provider, r_clone, s_clone).await {
                tracing::info!("Failed to run_anvil: {e:?}")
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        let _ = handle.await;
    }
    Ok(())
}

async fn run_anvil(
    port: u16,
    rpc_url: String,
    provider: P,
    r: R,
    s: S,
) -> Result<(), Box<dyn std::error::Error>> {
    let anvil = Anvil::new().fork(rpc_url).port(port).try_spawn()?;
    let anvil_provider = ProviderBuilder::new().on_http(anvil.endpoint_url());
    tracing::info!("Run anvil runner on {port:?}");

    let processed_tx = 0u64;
    loop {
        match r.recv() {
            Ok(tx_hash) => {
                tracing::info!("received {tx_hash:?}");
                let tx = match provider.get_transaction_by_hash(tx_hash).await {
                    Ok(Some(tx)) => tx,
                    _ => {
                        tracing::info!("TX not found in mempool: {tx_hash:?}");
                        continue;
                    }
                };

                let input = tx.inner.input().clone();
                //let input = String::from("");
                let tx = TransactionRequest::default().with_input(input);

                let pending_tx = anvil_provider.send_transaction(tx).await?;
                let receipt = pending_tx.get_receipt().await?;

                let logs = receipt.inner.logs();
                if !logs.is_empty() {
                    tracing::info!("FOUND LOGS: {logs:?}")
                } else {
                    tracing::info!("NO LOGS");
                }
                //if processed_tx % 20 == 0 {
                //}
            }
            //Err(TryRecvError::Empty) => info!("no messages, i am still waiting"),
            Err(_) => break,
        }
    }

    // Build a transaction to send 100 wei from Alice to Bob.
    // The `from` field is automatically filled to the first signer's address (Alice).

    //// Send the transaction and wait for the broadcast.

    ////println!("Pending transaction... {}", pending_tx.tx_hash());

    //// Wait for the transaction to be included and get the receipt.
    ////info!(
    ////    "Transaction included in block {}",
    ////    receipt.block_number.expect("Failed to get block number")
    ////);

    //tx_processed += 1;
    //if tx_processed % 100 == 0 {
    //info!("tx processed: {tx_processed:?}");
    //}

    //info!("Logs: {:?}", receipt.inner.logs());
    //for log in receipt.inner.logs() {
    //if let Some(event_signature) = log.topics().get(0) {
    //if *event_signature == IUniswapV2Pair::Swap::SIGNATURE_HASH {
    //println!("Swap event detected!");

    //// Извлекаем indexed параметры (sender и to)
    //let sender = Address::from_slice(&log.topics()[1].as_slice());
    //let to = Address::from_slice(&log.topics()[2].as_slice());
    //info!("sender: {sender:?}, to: {to:?}");

    //// Декодируем non-indexed параметры (amounts) из `log.data`
    ///*
    //let data: &[u8] = &log.data().data.as_ref();
    //let amount0_in = U256::from_be_bytes(data[0..32].try_into().unwrap());
    //let amount1_in = U256::from_be_bytes(data[32..64].try_into().unwrap());
    //let amount0_out = U256::from_be_bytes(data[64..96].try_into().unwrap());
    //let amount1_out = U256::from_be_bytes(data[96..128].try_into().unwrap());

    //println!("Sender: {:?}", sender);
    //println!("To: {:?}", to);
    //println!("Amount0In: {}", amount0_in);
    //println!("Amount1In: {}", amount1_in);
    //println!("Amount0Out: {}", amount0_out);
    //println!("Amount1Out: {}", amount1_out);
    //*/
    //}
    //}
    //}
    Ok(())
}
