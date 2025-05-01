// examples/test_specific_pair.rs

use tycho_price_quoter::{PriceQuoter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing specific token pair with Tycho Indexer data");
    println!("==================================================\n");
    
    // Create a PriceQuoter with the Indexer client
    let mut quoter = PriceQuoter::new()
        .with_indexer(
            "https://tycho-beta.propellerheads.xyz".to_string(),
            Some("sampletoken".to_string()),
        );
    
    // Fetch data for Uniswap V2
    println!("Fetching data from Tycho Indexer...");
    match quoter.fetch_latest_data(vec!["uniswap_v2".to_string()]).await {
        Ok(_) => {
            println!("Successfully fetched data");
            println!("Loaded {} pools and {} tokens", quoter.pool_count(), quoter.token_count());
            
            // Define token addresses for USDC and WBTC
            let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
            let wbtc_addr = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string();
            
            // Find paths between USDC and WBTC
            println!("\nFinding paths between USDC and WBTC...");
            let paths = quoter.find_paths(&usdc_addr, &wbtc_addr);
            
            println!("Found {} paths", paths.len());
            
            for (i, path) in paths.iter().enumerate().take(3) {
                println!("Path {}:", i + 1);
                for (j, hop) in path.hops.iter().enumerate() {
                    println!("  Hop {}: {} -> {}", j + 1, hop.token_in, hop.token_out);
                }
            }
            
            if paths.len() > 3 {
                println!("... and {} more paths", paths.len() - 3);
            }
            
            // Get quotes for different amounts
            println!("\nGetting quotes for different amounts:");
            
            let amounts = vec![
                (1000 * 10u128.pow(6), "1,000 USDC"),
                (10000 * 10u128.pow(6), "10,000 USDC"),
                (100000 * 10u128.pow(6), "100,000 USDC"),
            ];
            
            for (amount_in, amount_str) in amounts {
                match quoter.find_quote(&usdc_addr, &wbtc_addr, amount_in) {
                    Ok(Some((path, amount_out))) => {
                        println!("Quote for {}:", amount_str);
                        println!("  Path with {} hops", path.hops.len());
                        println!("  Expected output: {} WBTC", amount_out as f64 / 1e8); // WBTC has 8 decimals
                    },
                    Ok(None) => println!("No path found for {}", amount_str),
                    Err(e) => println!("Error for {}: {}", amount_str, e),
                }
            }
            
            // Try another pair: WETH to DAI
            println!("\nTesting another pair: WETH to DAI");
            
            let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
            let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
            
            let paths = quoter.find_paths(&weth_addr, &dai_addr);
            println!("Found {} paths", paths.len());
            
            // Get a quote for 1 WETH to DAI
            let amount_in = 1 * 10u128.pow(18); // 1 WETH
            
            match quoter.find_quote(&weth_addr, &dai_addr, amount_in) {
                Ok(Some((path, amount_out))) => {
                    println!("Quote for 1 WETH:");
                    println!("  Path with {} hops", path.hops.len());
                    println!("  Expected output: {} DAI", amount_out as f64 / 1e18);
                },
                Ok(None) => println!("No path found for 1 WETH"),
                Err(e) => println!("Error for 1 WETH: {}", e),
            }
        },
        Err(e) => {
            println!("Failed to fetch data from Tycho Indexer: {}", e);
        }
    }
    
    Ok(())
}
