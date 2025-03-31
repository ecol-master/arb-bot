use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub user: String,
    pub password: String,
    pub db_name: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub postgres: PostgresConfig,
}

impl Config {
    pub fn load(path: PathBuf) -> Result<Self> {
        let data = std::fs::read(path)?;
        Ok(serde_json::from_slice(&data)?)
    }
}

/// Converts config into connection string
impl PostgresConfig {
    pub fn into_connection(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={} sslmode=disable",
            self.host, self.port, self.user, self.password, self.db_name
        )
    }
}
