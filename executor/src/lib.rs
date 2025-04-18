use alloy::{
    primitives::{Address, Uint},
    providers::{Provider, RootProvider},
};
use anyhow::Result;
use bot_db::DB;
use bot_math::price_to_usd;
use crossbeam::channel::Receiver;
use dex_common::Arbitrage;
use ethereum_abi::IUniswapV2Pair;
use std::sync::Arc;

pub mod max_price;
pub mod triangular_swap;

#[derive(Clone)]
pub struct ArbitrageExecutor {
    r: Receiver<Arbitrage>,
    db: DB,
    provider: Arc<RootProvider>,
}

impl ArbitrageExecutor {
    pub fn new(r: Receiver<Arbitrage>, db: DB, provider: Arc<RootProvider>) -> Self {
        Self { db, provider, r }
    }

    pub async fn run(self) -> Result<()> {
        while let Ok(arbitrage) = self.r.recv() {
            self.process_arbitrage(arbitrage).await?;
        }
        Ok(())
    }

    pub async fn process_arbitrage(&self, arbitrage: Arbitrage) -> Result<()> {
        let first_token = arbitrage.path[0].0;
        let revenue_usd = price_to_usd(
            self.db.clone(),
            arbitrage.dex_id,
            &first_token,
            arbitrage.revenue,
        )
        .await?;

        self.print_path(&arbitrage.path).await?;

        for tokens in arbitrage.path.iter() {
            let pair_adr = self
                .db
                .pair_adr(arbitrage.dex_id, &tokens.0, &tokens.1)
                .await?;
            // let instance = IUniswapV2Pair::new(pair_adr.clone(), self.provider.clone());
            let slot = self
                .provider
                .get_storage_at(pair_adr, Uint::<256, 4>::from(4))
                .await?;
            tracing::info!("pair: {pair_adr:?} slot: {slot:?}");
        }

        tracing::info!("revenue_usd: {revenue_usd}");

        Ok(())
    }

    async fn print_path(&self, path: &[(Address, Address)]) -> Result<()> {
        let mut path_str = String::new();
        for (index, tokens) in path.iter().enumerate() {
            path_str.push_str(&self.db.postgres().get_token_ticker(&tokens.0).await?.ticker);
            path_str.push_str(" -> ");
            path_str.push_str(&self.db.postgres().get_token_ticker(&tokens.1).await?.ticker);

            if index != path.len() - 1 {
                path_str.push_str(" -> ");
            }
        }
        tracing::info!("path: {path_str}");
        Ok(())
    }
}
