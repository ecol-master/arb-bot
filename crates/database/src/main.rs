use alloy::providers::{Provider, ProviderBuilder, RootProvider};
use anyhow::Result;
use bot_config::Config;
use bot_db::{tables::Ticker, DB};
use std::sync::Arc;

// This script is needed to load the tickers for already exist tokens in pairs

#[tokio::main]
async fn main() -> Result<()> {
    bot_logger::init_logger(tracing::Level::INFO);

    let config = Config::load("../config.yml".into())?;
    let provider: Arc<RootProvider> =
        Arc::new(ProviderBuilder::default().connect(&config.rpc_url).await?);

    let db = DB::new(config).await?;
    let pairs = db.postgres().select_pairs().await?;

    for pair in pairs.iter() {
        for token in vec![pair.token0.clone(), pair.token1.clone()] {
            if db.postgres().get_token_ticker(&token).await.is_err() {
                let instance = ethereum_abi::IERC20::new(token.clone(), provider.clone());
                let ticker = instance.symbol().call().await?._0;

                db.postgres()
                    .insert_ticker(Ticker {
                        token,
                        ticker: ticker.clone(),
                    })
                    .await?;
                tracing::info!("insert ticker: {ticker}");
            }
        }
    }

    tracing::info!("All tickers check!");
    Ok(())
}
