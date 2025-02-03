use crate::{
    searcher::{dfs_search, floyd_warshall_search},
    storage::Storage,
    types::{IUniswapV2Pair, IUniswapV3Pool::IUniswapV3PoolCalls},
};
use alloy::{
    primitives::{keccak256, Address},
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    rpc::{
        client::RpcClient,
        types::{BlockNumberOrTag, Filter, Transaction},
    },
    sol_types::SolEvent,
};
use futures_util::{stream, FutureExt, StreamExt};
use std::sync::Arc;
use tracing::info;

/*
    let filter = Filter::new()
        .address(pair_adr)
        .event_signature(IUniswapV2Pair::Swap::SIGNATURE_HASH)
        .from_block(BlockNumberOrTag::Latest);

    let sub = provider.subscribe_logs(&filter).await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        let tx_hash = match log.block_hash {
            Some(hash) => hash,
            None => continue,
        };
        let tx = match provider.get_transaction_by_hash(tx_hash).await? {
            Some(tx) => tx,
            None => continue,
        };

        info!("{:?}", tx);

        info!("UniswapV2 token transfer: {log:?}")
    }
*/

type PairType =
    IUniswapV2Pair::IUniswapV2PairInstance<PubSubFrontend, Arc<RootProvider<PubSubFrontend>>>;

pub async fn subscribe_pool(
    pair: PairType,
    storage: Arc<Storage>,
    provider: Arc<RootProvider<PubSubFrontend>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pair_adr = pair.address();
    let token0 = pair.token0().call().await?._0;
    let token1 = pair.token1().call().await?._0;

    info!("Start listening: {pair_adr:?}");

    let filter = Filter::new()
        .address(*pair_adr)
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

        let data = storage.get_reserves();
        match dfs_search(data).await? {
            Some(path) => info!("Path found: {:?}", path),
            _ => info!("Path not found"),
        };
    }

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
