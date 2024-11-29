use envconfig::Envconfig;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "WS_ADDRESS")]
    pub ws_address: String,
}
