use ethers::types::{Address, Filter, H160, H256};

pub fn get_uniswap_v2_filter() -> Filter {
    let sync_event = "Sync(uint112,uint112)";
    let _uniswap_v2_factory: Address = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D"
        .parse()
        .expect("failed to parse uniswap v2 router address");

    /*
    Filter::default()
        .address(uniswap_v2_factory)
        .event(sync_event)
        */

    Filter::default().event(sync_event)
}

pub fn get_uniswap_v3_filter() -> Filter {
    const V3FACTORY_ADDRESS: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
    const DAI_ADDRESS: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    const USDC_ADDRESS: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    const USDT_ADDRESS: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";

    let token_topics = [
        H256::from(USDC_ADDRESS.parse::<H160>().unwrap()),
        H256::from(USDT_ADDRESS.parse::<H160>().unwrap()),
        H256::from(DAI_ADDRESS.parse::<H160>().unwrap()),
    ];
    Filter::new()
        .address(V3FACTORY_ADDRESS.parse::<Address>().unwrap())
        .event("PoolCreated(address,address,uint24,int24,address)")
        .topic1(token_topics.to_vec())
        .topic2(token_topics.to_vec())
        .from_block(0)
}
