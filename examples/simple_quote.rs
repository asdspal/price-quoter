// examples/simple_quote.rs

use tycho_price_quoter::{Token, Pool, PriceQuoter};
use std::collections::HashMap;

fn main() {
    // Create sample data
    let (tokens, pools) = create_sample_data();
    
    // Create a PriceQuoter with initial data
    let quoter = PriceQuoter::with_initial_data(pools, tokens, Some(2));
    
    // Define token addresses
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    
    // Amount to swap: 1000 USDC
    let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
    
    // Find all possible paths
    let paths = quoter.find_paths(&usdc_addr, &dai_addr);
    println!("Found {} paths from USDC to DAI:", paths.len());
    
    for (i, path) in paths.iter().enumerate() {
        println!("Path {}:", i + 1);
        for (j, hop) in path.hops.iter().enumerate() {
            println!("  Hop {}: {} -> {}", j + 1, hop.token_in, hop.token_out);
        }
    }
    
    // Find the best quote
    match quoter.find_quote(&usdc_addr, &dai_addr, amount_in) {
        Ok(Some((path, amount_out))) => {
            println!("\nBest quote for 1000 USDC to DAI:");
            println!("Path with {} hops:", path.hops.len());
            for (i, hop) in path.hops.iter().enumerate() {
                println!("  Hop {}: {} -> {}", i+1, hop.token_in, hop.token_out);
            }
            println!("Expected output: {} DAI", amount_out as f64 / 1e18);
        },
        Ok(None) => println!("\nNo path found"),
        Err(e) => println!("\nError: {}", e),
    }
}

fn create_sample_data() -> (HashMap<String, Token>, Vec<Pool>) {
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
