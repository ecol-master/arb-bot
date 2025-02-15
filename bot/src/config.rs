use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "RPC_URL")]
    pub rpc_url: String,
}