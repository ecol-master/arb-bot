use crate::tables::{
    Dex, Pair, PairRaw, Ticker, TickerRaw, DEXES_TABLE, PAIRS_TABLE, TICKERS_TABLE,
};
use alloy::primitives::Address;
use anyhow::Result;
use kronos_config::PostgresConfig;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct PostgresDB {
    pool: Pool<Postgres>,
}
impl PostgresDB {
    pub async fn connect(config: &PostgresConfig) -> Result<Self> {
        let conn_data = config.sqlx_connection();

        let pool = sqlx::PgPool::connect(&conn_data).await?;

        tracing::info!("🐘 Successfully connect to postgres on: {conn_data:?}");
        Ok(Self { pool })
    }

    pub async fn select_pairs(&self) -> Result<Vec<Pair>> {
        let query = format!("SELECT * FROM {PAIRS_TABLE}");
        let pairs_v2: Vec<PairRaw> = sqlx::query_as(&query).fetch_all(&self.pool).await?;
        Ok(pairs_v2
            .iter()
            .map(|pair_raw| Pair {
                address: Address::from_slice(&pair_raw.address),
                token0: Address::from_slice(&pair_raw.token0),
                token1: Address::from_slice(&pair_raw.token1),
                dex_id: pair_raw.dex_id,
            })
            .collect())
    }

    pub async fn insert_pair(&self, pair: Pair) -> Result<()> {
        let query = format!(
            "INSERT INTO {PAIRS_TABLE} (address, dex_id, token0, token1) VALUES ($1, $2, $3, $4)"
        );

        let rows_affected = sqlx::query(&query)
            .bind(pair.address.as_slice())
            .bind(pair.dex_id)
            .bind(pair.token0.as_slice())
            .bind(pair.token1.as_slice())
            .execute(&self.pool)
            .await?
            .rows_affected();

        debug_assert!(rows_affected == 1, "PostgresDB don't insert PairV2");

        tracing::trace!(
            "inserted new pair: {:?} on dex: {}",
            pair.address,
            pair.dex_id
        );
        Ok(())
    }

    pub async fn get_pair_dex_id(&self, pair_adr: &Address) -> Result<i32> {
        let query = format!("SELECT * FROM {PAIRS_TABLE} WHERE address = $1");

        let pair: PairRaw = sqlx::query_as(&query)
            .bind(pair_adr.as_slice())
            .fetch_one(&self.pool)
            .await?;

        Ok(pair.dex_id)
    }

    pub async fn get_dex_id(&self, dex_name: &str) -> Result<i32> {
        let query = format!("SELECT * FROM {DEXES_TABLE} WHERE name = $1");

        let dex: Dex = sqlx::query_as(&query)
            .bind(dex_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(dex.id)
    }

    pub async fn insert_ticker(&self, ticker: Ticker) -> Result<()> {
        let query = format!("INSERT INTO {TICKERS_TABLE} (token, ticker) VALUES ($1, $2)");
        let rows_affected = sqlx::query(&query)
            .bind(ticker.token.as_slice())
            .bind(ticker.ticker)
            .execute(&self.pool)
            .await?
            .rows_affected();

        debug_assert!(rows_affected == 1, "Insert token rows affected not equal 1");
        Ok(())
    }

    pub async fn get_token_ticker(&self, token: &Address) -> Result<Ticker> {
        let query = format!("SELECT * FROM {TICKERS_TABLE} WHERE token = $1");

        let ticker: TickerRaw = sqlx::query_as(&query)
            .bind(token.as_slice())
            .fetch_one(&self.pool)
            .await?;

        Ok(Ticker {
            token: Address::from_slice(&ticker.token),
            ticker: ticker.ticker,
        })
    }
}
