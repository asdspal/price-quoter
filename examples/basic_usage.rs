// examples/basic_usage.rs

use tycho_price_quoter::{PriceQuoter, Token, Pool, Address, U256};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample data
    let (tokens, pools) = create_sample_data();
    
    // Create a PriceQuoter with initial data
    let mut quoter = PriceQuoter::with_initial_data(pools, tokens, Some(2))
        .with_indexer(
            "https://api.tycho.xyz".to_string(),
            Some("your-api-key".to_string()),
        );
    
    // Define token addresses
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    
    // Amount to swap: 1000 USDC
    let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
    
    // Find quote using local data
    println!("Finding quote using local data...");
    match quoter.find_quote(&usdc_addr, &dai_addr, amount_in) {
        Ok(Some((path, amount_out))) => {
            println!("Found path with {} hops:", path.hops.len());
            for (i, hop) in path.hops.iter().enumerate() {
                println!("  Hop {}: {} -> {}", i+1, hop.token_in, hop.token_out);
            }
            println!("Expected output: {} DAI", amount_out as f64 / 1e18);
        },
        Ok(None) => println!("No path found"),
        Err(e) => println!("Error: {}", e),
    }
    
    // Uncomment to fetch latest data from Indexer
    // Note: This requires a valid API key and internet connection
    /*
    println!("\nFetching latest data from Indexer...");
    match quoter.find_quote_with_latest_data(
        &usdc_addr,
        &dai_addr,
        amount_in,
        vec!["uniswap_v2".to_string(), "sushiswap".to_string()],
    ).await {
        Ok(Some((path, amount_out))) => {
            println!("Found path with {} hops:", path.hops.len());
            for (i, hop) in path.hops.iter().enumerate() {
                println!("  Hop {}: {} -> {}", i+1, hop.token_in, hop.token_out);
            }
            println!("Expected output: {} DAI", amount_out as f64 / 1e18);
        },
        Ok(None) => println!("No path found"),
        Err(e) => println!("Error: {}", e),
    }
    */
    
    Ok(())
}

fn create_sample_data() -> (HashMap<Address, Token>, Vec<Pool>) {
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();

    let usdc = Token { address: usdc_addr.clone(), decimals: 6, symbol: "USDC".to_string() };
    let weth = Token { address: weth_addr.clone(), decimals: 18, symbol: "WETH".to_string() };
    let dai = Token { address: dai_addr.clone(), decimals: 18, symbol: "DAI".to_string() };

    let mut tokens = HashMap::new();
    tokens.insert(usdc_addr.clone(), usdc);
    tokens.insert(weth_addr.clone(), weth);
    tokens.insert(dai_addr.clone(), dai);

    let pool1 = Pool {
        id: "0xPoolUsdcWeth".to_string(),
        token0: usdc_addr.clone(),
        token1: weth_addr.clone(),
        reserve0: 5_000_000 * 10u128.pow(6), // 5M USDC
        reserve1: 2_000 * 10u128.pow(18),    // 2k WETH
        fee: 30,
    };

    let pool2 = Pool {
        id: "0xPoolWethDai".to_string(),
        token0: weth_addr.clone(),
        token1: dai_addr.clone(),
        reserve0: 1_500 * 10u128.pow(18),     // 1.5k WETH
        reserve1: 4_500_000 * 10u128.pow(18), // 4.5M DAI
        fee: 30,
    };

    (tokens, vec![pool1, pool2])
}
