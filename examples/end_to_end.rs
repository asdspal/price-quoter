// examples/end_to_end.rs

use tycho_price_quoter::{Token, Pool, PriceQuoter};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Tycho Price Quoter - End-to-End Test");
    println!("====================================\n");
    
    // Step 1: Initialize with sample data
    println!("Step 1: Initializing with sample data...");
    let (tokens, pools) = create_sample_data();
    let mut quoter = PriceQuoter::with_initial_data(pools, tokens, Some(2));
    println!("Initialized with {} tokens and {} pools\n", quoter.token_count(), quoter.pool_count());
    
    // Step 2: Find paths between tokens
    println!("Step 2: Finding paths between USDC and DAI...");
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    
    let paths = quoter.find_paths(&usdc_addr, &dai_addr);
    println!("Found {} paths", paths.len());
    
    for (i, path) in paths.iter().enumerate() {
        println!("Path {}:", i + 1);
        for (j, hop) in path.hops.iter().enumerate() {
            println!("  Hop {}: {} -> {}", j + 1, hop.token_in, hop.token_out);
        }
    }
    println!();
    
    // Step 3: Get a quote
    println!("Step 3: Getting a quote for 1000 USDC to DAI...");
    let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
    
    match quoter.find_quote(&usdc_addr, &dai_addr, amount_in) {
        Ok(Some((path, amount_out))) => {
            println!("Best quote:");
            println!("  Path with {} hops", path.hops.len());
            for (i, hop) in path.hops.iter().enumerate() {
                println!("    Hop {}: {} -> {}", i+1, hop.token_in, hop.token_out);
            }
            println!("  Expected output: {} DAI", amount_out as f64 / 1e18);
        },
        Ok(None) => println!("No path found"),
        Err(e) => println!("Error: {}", e),
    }
    println!();
    
    // Step 4: Update a pool and get a new quote
    println!("Step 4: Updating pool reserves and getting a new quote...");
    
    // Update the USDC-WETH pool with new reserves
    let new_reserve0 = 6_000_000 * 10u128.pow(6); // 6M USDC
    let new_reserve1 = 2_500 * 10u128.pow(18);    // 2.5k WETH
    
    quoter.update_pool("0xPoolUsdcWeth", new_reserve0, new_reserve1)?;
    println!("Updated pool reserves");
    
    // Get a new quote
    match quoter.find_quote(&usdc_addr, &dai_addr, amount_in) {
        Ok(Some((_, amount_out))) => {
            println!("New expected output: {} DAI", amount_out as f64 / 1e18);
        },
        Ok(None) => println!("No path found after update"),
        Err(e) => println!("Error after update: {}", e),
    }
    println!();
    
    // Step 5: Connect to Tycho Indexer (optional)
    println!("Step 5: Connecting to Tycho Indexer...");
    
    // Add Indexer client with the correct URL
    quoter = quoter.with_indexer(
        "https://tycho-beta.propellerheads.xyz".to_string(),
        Some("sampletoken".to_string()),
    );
    
    // Try to fetch data (this may fail if the API is not available)
    println!("Attempting to fetch data from Tycho Indexer...");
    match quoter.fetch_latest_data(vec!["uniswap_v2".to_string()]).await {
        Ok(_) => {
            println!("Successfully fetched data from Tycho Indexer");
            println!("Now have {} pools and {} tokens", quoter.pool_count(), quoter.token_count());
            
            // Try to get a quote with the new data
            match quoter.find_quote(&usdc_addr, &dai_addr, amount_in) {
                Ok(Some((_, amount_out))) => {
                    println!("Quote with Indexer data: {} DAI", amount_out as f64 / 1e18);
                },
                Ok(None) => println!("No path found with Indexer data"),
                Err(e) => println!("Error with Indexer data: {}", e),
            }
        },
        Err(e) => {
            println!("Failed to fetch data from Tycho Indexer: {}", e);
            println!("This is expected if the API is not available or the key is invalid");
        }
    }
    
    println!("\nEnd-to-End Test Complete");
    
    Ok(())
}

fn create_sample_data() -> (HashMap<String, Token>, Vec<Pool>) {
    // Same sample data as before
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
