use crate::uniswap::{
    processor::{
        process_swap_exact_eth_for_tokens, process_swap_exact_tokens_for_tokens, SwapContext,
    },
    router::{swapExactETHForTokensCall, swapExactTokensForTokensCall, swapTokensForExactETHCall},
    types::{IUniswaV2Factory, IUniswapV2Pair},
};
use alloy::{
    consensus::Transaction,
    primitives::{address, Address},
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
    sol_types::SolCall,
};
use anyhow::anyhow;
use futures_util::{stream, StreamExt};
use std::{io::Read, ops::Add, sync::Arc};
use tracing::{error, info};

const ROUTER02_ADR: Address = address!("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D");
const FACTORY_V2_ADR: Address = address!("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");

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
