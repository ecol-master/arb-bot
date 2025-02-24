use crate::{
    searcher::{calc_out, get_reserves, triangular_swap},
    storage::Storage,
    types::{IUniswapV2Pair, IUniswapV3Pool::IUniswapV3PoolCalls},
};
use alloy::{
    network::TransactionBuilder,
    primitives::{keccak256, Address, Uint, U256},
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    rpc::{
        client::RpcClient,
        types::{BlockNumberOrTag, Filter, Transaction, TransactionRequest},
    },
    signers::k256::sha2::digest::block_buffer::EagerBuffer,
    sol_types::SolEvent,
    transports::http::{Client, Http},
};
use futures_util::{stream, FutureExt, StreamExt};
use std::{io::Read, sync::Arc};
use tracing::info;

pub struct SubcribePoolContext {
    pub pair_adr: Address,
    pub storage: Arc<Storage>,
    pub provider: Arc<RootProvider<PubSubFrontend>>,
    pub anvil_provider: Arc<RootProvider<Http<Client>>>,
}

//type PairType =IUniswapV2Pair::IUniswapV2PairInstance<PubSubFrontend, Arc<RootProvider<PubSubFrontend>>>;

pub async fn subscribe_pool(ctx: SubcribePoolContext) -> Result<(), Box<dyn std::error::Error>> {
    let SubcribePoolContext {
        pair_adr,
        storage,
        provider,
        anvil_provider,
    } = ctx;

    let pair = IUniswapV2Pair::new(pair_adr, provider.clone());

    let token0 = pair.token0().call().await?._0;
    let token1 = pair.token1().call().await?._0;
    let reserves = pair.getReserves().call().await?;

    storage
        .add_pair(&token0, &token1, reserves.reserve0, reserves.reserve1)
        .await?;

    let accounts = anvil_provider.get_accounts().await?;
    info!("anvil accounts: {accounts:?}",);

    let alice = accounts[0];
    let bob = accounts[1];
    info!("Start listening: {pair_adr:?}");

    let mut stream = provider
        .subscribe_pending_transactions()
        .await?
        .into_stream();

    let mut tx_processed = 0u64;

    while let Some(tx_hash) = stream.next().await {
        let tx = match provider.get_transaction_by_hash(tx_hash).await {
            Ok(Some(tx)) => tx,
            _ => continue,
        };

        // Build a transaction to send 100 wei from Alice to Bob.
        // The `from` field is automatically filled to the first signer's address (Alice).
        let tx = TransactionRequest::default()
            .with_to(bob)
            .with_chain_id(provider.get_chain_id().await?)
            .with_value(U256::from(100))
            .with_gas_limit(21_000)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);

        // Send the transaction and wait for the broadcast.
        let pending_tx = anvil_provider.send_transaction(tx).await?;

        //println!("Pending transaction... {}", pending_tx.tx_hash());

        // Wait for the transaction to be included and get the receipt.
        let receipt = pending_tx.get_receipt().await?;
        //info!(
        //    "Transaction included in block {}",
        //    receipt.block_number.expect("Failed to get block number")
        //);

        tx_processed += 1;
        if tx_processed % 100 == 0 {
            info!("tx processed: {tx_processed:?}");
        }

        let logs = receipt.inner.logs();
        if logs.is_empty() {
            continue;
        }

        info!("Logs: {:?}", receipt.inner.logs());
        for log in receipt.inner.logs() {
            if let Some(event_signature) = log.topics().get(0) {
                if *event_signature == IUniswapV2Pair::Swap::SIGNATURE_HASH {
                    println!("Swap event detected!");

                    // Извлекаем indexed параметры (sender и to)
                    let sender = Address::from_slice(&log.topics()[1].as_slice());
                    let to = Address::from_slice(&log.topics()[2].as_slice());
                    info!("sender: {sender:?}, to: {to:?}");

                    // Декодируем non-indexed параметры (amounts) из `log.data`
                    /*
                    let data: &[u8] = &log.data().data.as_ref();
                    let amount0_in = U256::from_be_bytes(data[0..32].try_into().unwrap());
                    let amount1_in = U256::from_be_bytes(data[32..64].try_into().unwrap());
                    let amount0_out = U256::from_be_bytes(data[64..96].try_into().unwrap());
                    let amount1_out = U256::from_be_bytes(data[96..128].try_into().unwrap());

                    println!("Sender: {:?}", sender);
                    println!("To: {:?}", to);
                    println!("Amount0In: {}", amount0_in);
                    println!("Amount1In: {}", amount1_in);
                    println!("Amount0Out: {}", amount0_out);
                    println!("Amount1Out: {}", amount1_out);
                    */
                }
            }
        }
    }

    /*
        let filter = Filter::new()
            .address(pair_adr)
            .event_signature(IUniswapV2Pair::Swap::SIGNATURE_HASH)
            .from_block(BlockNumberOrTag::Latest);

        let mut stream = provider.subscribe_logs(&filter).await?.into_stream();

        while let Some(log) = stream.next().await {
            let cloned_storage = storage.clone();
            let tx_hash = match log.transaction_hash {
                Some(tx_hash) => tx_hash,
                None => continue,
            };

            let tx = match provider.get_transaction_by_hash(tx_hash).await {
                Ok(Some(tx)) => tx,
                _ => continue,
            };

            let reserves = pair.getReserves().call().await?;
            storage
                .update_reserves(token0.clone(), token1.clone(), reserves)
                .await?;

            let reserves = storage.get_reserves();
            for path in triangular_swap(reserves.clone()).await? {
                info!("Path found: {:?}", path);

                let start_amount: Uint<256, 4> = Uint::from(100_000);

                let (reserve0, reserve1) = get_reserves(&reserves, &path.0, &path.1);
                let out1 = calc_out(reserve0, reserve1, start_amount);

                let (reserve1, reserve2) = get_reserves(&reserves, &path.1, &path.2);
                let out2 = calc_out(reserve1, reserve2, out1);

                let (reserve2, reserve0) = get_reserves(&reserves, &path.2, &path.0);
                let out3 = calc_out(reserve2, reserve0, out2);
                info!("Start: {start_amount:?}");
                info!("After first swap: {out1:?}");
                info!("After second swap: {out2:?}");
                info!("Result out: {out3:?}");
                info!("==============================");
            }
        }
    */
    Ok(())
}

/*
pub async fn run_strategy(
    provider: Arc<RootProvider<PubSubFrontend>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = provider
        .subscribe_pending_transactions()
        .await?
        .into_stream(); // Wait and take the next 3 transactions.

    let factory_v2 = Arc::new(IUniswaV2Factory::new(FACTORY_V2_ADR, provider.clone()));

    info!("Awaiting pending transactions...");
    while let Some(tx_hash) = stream.next().await {
        let tx = match provider.get_transaction_by_hash(tx_hash).await {
            Ok(Some(tx)) => tx,
            _ => continue,
        };

        if tx.to() != Some(ROUTER02_ADR) {
            continue;
        }

        // "0x38ed17"
        if let Ok(swap_data) = swapExactTokensForTokensCall::abi_decode(tx.input(), false) {
            let pair_adr = factory_v2
                .getPair(swap_data.path[0], swap_data.path[1])
                .call()
                .await?
                .pair;
            let ctx = SwapContext {
                token0_adr: swap_data.path[0],
                token1_adr: swap_data.path[1],
                pair: IUniswapV2Pair::new(pair_adr, provider.clone()),
            };
            process_swap_exact_tokens_for_tokens(swap_data, ctx, provider.clone()).await?;
        // 0x7ff36ab5
        } else if let Ok(swap_data) = swapExactETHForTokensCall::abi_decode(tx.input(), false) {
            let pair_adr = factory_v2
                .getPair(swap_data.path[0], swap_data.path[1])
                .call()
                .await?
                .pair;
            let ctx = SwapContext {
                token0_adr: swap_data.path[0],
                token1_adr: swap_data.path[1],
                pair: IUniswapV2Pair::new(pair_adr, provider.clone()),
            };
            process_swap_exact_eth_for_tokens(swap_data, ctx, provider.clone());
        } else if let Ok(swap_data) = swapTokensForExactETHCall::abi_decode(tx.input(), false) {
            info!(
                "SwapTokensForExactETH Path: {:?}, AmountIn: {:?}",
                swap_data.path, swap_data.amountOut
            );
        }
    }
    Ok(())
}
*/
