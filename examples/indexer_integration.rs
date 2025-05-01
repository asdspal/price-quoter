// examples/indexer_integration.rs

use tycho_price_quoter::{Token, Pool, PriceQuoter, IndexerClient};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a basic PriceQuoter with empty data
    let mut quoter = PriceQuoter::new()
        .with_indexer(
            "https://tycho-beta.propellerheads.xyz".to_string(),
            Some("sampletoken".to_string()),
        );
    
    println!("Fetching data from Tycho Indexer...");
    
    // Fetch data for Uniswap V2 and Sushiswap
    let protocols = vec!["uniswap_v2".to_string(), "sushiswap".to_string()];
    
    match quoter.fetch_latest_data(protocols).await {
        Ok(_) => {
            println!("Successfully fetched data from Tycho Indexer");
            println!("Loaded {} pools and {} tokens", quoter.pool_count(), quoter.token_count());
            
            // Define some common token addresses
            let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
            let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
            let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
            
            // Try to find paths between USDC and DAI
            let paths = quoter.find_paths(&usdc_addr, &dai_addr);
            println!("Found {} paths from USDC to DAI", paths.len());
            
            for (i, path) in paths.iter().enumerate().take(5) { // Show at most 5 paths
                println!("Path {}:", i + 1);
                for (j, hop) in path.hops.iter().enumerate() {
                    println!("  Hop {}: {} -> {}", j + 1, hop.token_in, hop.token_out);
                }
            }
            
            if paths.len() > 5 {
                println!("... and {} more paths", paths.len() - 5);
            }
            
            // Try to get a quote for 1000 USDC to DAI
            let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
            
            match quoter.find_quote(&usdc_addr, &dai_addr, amount_in) {
                Ok(Some((path, amount_out))) => {
                    println!("\nBest quote for 1000 USDC to DAI:");
                    println!("Path with {} hops:", path.hops.len());
                    for (i, hop) in path.hops.iter().enumerate() {
                        println!("  Hop {}: {} -> {}", i+1, hop.token_in, hop.token_out);
                    }
                    println!("Expected output: {} DAI", amount_out as f64 / 1e18);
                },
                Ok(None) => println!("\nNo path found for USDC to DAI"),
                Err(e) => println!("\nError finding quote: {}", e),
            }
        },
        Err(e) => {
            println!("Failed to fetch data from Tycho Indexer: {}", e);
        }
    }
    
    Ok(())
}
