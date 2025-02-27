use crate::{
    storage::Storage,
    types::IUniswapV2Pair,
};
use alloy::{
    primitives::Address,
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    sol_types::SolEvent,
    transports::http::{Client, Http},
};
use std::sync::Arc;
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
