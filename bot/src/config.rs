use std::env;

pub struct Config {
    pub infura_rpc_url: String,
    pub alchemy_rpc_url: String,
}

impl Config {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            infura_rpc_url: env::var("INFURA_RPC_URL")?,
            alchemy_rpc_url: env::var("ALCHEMY_RPC_URL")?,
        })
    }
}
