use alloy::primitives::Address;
use anyhow::Result;
use tokio_postgres::NoTls;

const PAIR_V2_TABLE: &str = "pairs_v2";

#[derive(Debug, Clone)]
pub struct PairV2 {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
}

pub struct PostgresDB {
    client: tokio_postgres::Client,
}

const USER: &str = "postgres";
const PASSWORD: &str = "postgres";
const DB_NAME: &str = "arb_bot_db";
const HOST: &str = "127.0.0.1";
const PORT: u16 = 5432;

impl PostgresDB {
    pub async fn connect() -> Result<Self> {
        let conn_data =
            format!("host={HOST} port={PORT} user={USER} password={PASSWORD} dbname={DB_NAME} sslmode=disable");
        let (client, connection) = tokio_postgres::connect(&conn_data, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                panic!("Connect to Postgres: {e:?}");
            }
        });

        Ok(Self { client })
    }

    pub async fn select_pairs_v2(&self) -> Result<Vec<PairV2>> {
        let query = format!("SELECT * FROM {PAIR_V2_TABLE}");
        let rows = self.client.query(&query, &[]).await?;

        Ok(rows
            .iter()
            .map(|row| PairV2 {
                address: Address::from_slice(row.get::<&str, &[u8]>("address")),
                token0: Address::from_slice(row.get::<&str, &[u8]>("token0")),
                token1: Address::from_slice(row.get::<&str, &[u8]>("token1")),
            })
            .collect())
    }

    pub async fn insert_pair_v2(&self, pair: &PairV2) -> Result<()> {
        let query =
            format!("INSERT INTO {PAIR_V2_TABLE} (address, token0, token1) VALUES ($1, $2, $3)");
        let rows_modified = self
            .client
            .execute(
                &query,
                &[
                    &pair.address.to_string().as_str(),
                    &pair.token0.to_string().as_str(),
                    &pair.token1.to_string().as_str(),
                ],
            )
            .await?;

        debug_assert!(rows_modified == 1, "PostgresDB don't insert PairV2");
        Ok(())
    }
}
