// examples/tycho_integration.rs
use price_quoter::PriceQuoter;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment
    let api_key = env::var("TYCHO_API_KEY").ok();
    
    println!("Connecting to Tycho...");
    
    // Create price quoter from Tycho
    let quoter = PriceQuoter::from_tycho(
        "https://tycho-beta.propellerheads.xyz",
        api_key
    ).await?;
    
    // Print all tokens
    let tokens = quoter.get_storage().get_all_tokens();
    println!("Loaded {} tokens", tokens.len());
    
    println!("\nSample tokens:");
    for token in tokens.iter().take(5) {
        println!("  {} ({})", token.symbol, token.address);
    }
    
    // Get a quote for a token pair
    // Using WETH and USDC as examples
    let weth = tokens.iter().find(|t| t.symbol == "WETH");
    let usdc = tokens.iter().find(|t| t.symbol == "USDC");
    
    if let (Some(weth), Some(usdc)) = (weth, usdc) {
        println!("\nGetting quote for WETH -> USDC...");
        
        // 1 WETH (18 decimals)
        let amount_in = 1_000_000_000_000_000_000u128;
        
        match quoter.get_quote(&weth.address, &usdc.address, amount_in) {
            Ok((path, amount_out)) => {
                println!("Quote found:");
                println!("  Input: {} WETH", amount_in);
                println!("  Output: {} USDC", amount_out);
                println!("  Path length: {} hops", path.hops.len());
                
                for (i, hop) in path.hops.iter().enumerate() {
                    println!("  Hop {}: {} -> {}", i+1, hop.token_in, hop.token_out);
                }
            },
            Err(e) => {
                println!("Failed to get quote: {:?}", e);
            }
        }
    } else {
        println!("Could not find WETH or USDC in the token list");
    }
    
    Ok(())
}
