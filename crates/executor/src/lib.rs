use alloy::{
    primitives::{Address, Uint},
    providers::{Provider, RootProvider},
};
use anyhow::Result;
use kronos_db::{TokensGraphStorage, DB};
use kronos_dexes::common::Arbitrage;
use kronos_math::price_to_usd;
use std::sync::Arc;

pub mod max_price;
pub mod triangular_swap;

pub enum ExecutorEvent {
    ArbitrageExecuted,
}

pub struct Executor {
    db: DB,
    provider: Arc<RootProvider>,

    rx: tokio::sync::mpsc::UnboundedReceiver<Arbitrage>,
}

impl Executor {
    pub fn new(
        db: DB,
        provider: Arc<RootProvider>,
        rx: tokio::sync::mpsc::UnboundedReceiver<Arbitrage>,
    ) -> Self {
        Self { db, provider, rx }
    }

    pub async fn start(mut self) -> Result<()> {
        while let Some(arbitrage) = self.rx.recv().await {
            self.process_arbitrage(arbitrage).await?;
        }
        Ok(())
    }

    pub async fn process_arbitrage(&self, arbitrage: Arbitrage) -> Result<()> {
        let first_token = arbitrage.path[0].0;
        let amount_in_usd = price_to_usd(
            self.db.clone(),
            arbitrage.dex_id,
            &first_token,
            arbitrage.amount_in,
        )
        .await?;

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
            let slot = self
                .provider
                .get_storage_at(pair_adr, Uint::<256, 4>::from(4))
                .await?;
            tracing::info!("pair: {pair_adr:?} slot: {slot:?}");
        }

        tracing::info!("revenue_usd: {revenue_usd}, amount in: {amount_in_usd}");

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
