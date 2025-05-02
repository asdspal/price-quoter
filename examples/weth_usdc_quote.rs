use std::env;
use tycho_price_quoter::{
    PriceQuoter,
    types::{Token, Pool, Address},
    graph::TokenGraph,
    pathfinder::find_paths,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("TYCHO_API_KEY").ok();
    let tycho_url = env::var("TYCHO_URL").unwrap_or_else(|_| "https://tycho-beta.propellerheads.xyz".to_string());
    
    println!("🔄 Connecting to Tycho at {}", tycho_url);
    
    // Initialize the price quoter with data from Tycho
    let quoter = PriceQuoter::from_tycho(&tycho_url, api_key).await?;
    
    // Define token addresses for WETH and USDC
	let weth_addr = "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
	let usdc_addr = "a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
    
    // Print all tokens in storage
    println!("Total tokens in storage: {}", quoter.get_storage().token_count());
    
    
    
    // Get token information
    let usdc = quoter.get_storage().get_token(usdc_addr)
        .expect("USDC token not found");
    let weth = quoter.get_storage().get_token(weth_addr)
        .expect("WETH token not found");

    
    println!("📊 Token Information:");
    println!("  WETH: {} ({} decimals)", weth.symbol, weth.decimals);
    println!("  USDC: {} ({} decimals)", usdc.symbol, usdc.decimals);
    
    // Get pools that contain these tokens
    let weth_pools = quoter.get_storage().get_pools_for_token(weth_addr);
    let usdc_pools = quoter.get_storage().get_pools_for_token(usdc_addr);
    
    println!("\n🏊 Pool Information:");
    println!("  WETH is in {} pools", weth_pools.len());
    println!("  USDC is in {} pools", usdc_pools.len());
    
    // Check if there's a direct pool between WETH and USDC
    let has_direct_pool = quoter.get_storage().has_direct_pool(weth_addr, usdc_addr);
    println!("\n🔍 Direct Pool Check:");
    println!("  Direct WETH-USDC pool exists: {}", has_direct_pool);
    
    // Find all paths between WETH and USDC with max 2 hops
    let max_hops = 2;
    let paths = find_paths(
        quoter.get_storage(),
        &weth_addr.to_string(),
        &usdc_addr.to_string(),
        max_hops
    );
    
    println!("\n🛣️ Path Finding Results:");
    println!("  Found {} paths with max {} hops", paths.len(), max_hops);
    
    // Print details of each path
    for (i, path) in paths.iter().enumerate() {
        println!("\n  Path #{} ({} hops):", i + 1, path.hops.len());
        
        for (j, hop) in path.hops.iter().enumerate() {
            let pool = quoter.get_storage().get_pool(&hop.pool_id).unwrap();
            let token_in = quoter.get_storage().get_token(&hop.token_in).unwrap();
            let token_out = quoter.get_storage().get_token(&hop.token_out).unwrap();
            
            println!("    Hop #{}: {} -> {} (Pool: {})", 
                j + 1,
                token_in.symbol,
                token_out.symbol,
                &hop.pool_id[0..10] // Show first 10 chars of pool ID
            );
            
            // Show pool reserves
            if pool.token0 == hop.token_in {
                println!("      Reserves: {} {} -> {} {}", 
                    format_amount(pool.reserve0, token_in.decimals),
                    token_in.symbol,
                    format_amount(pool.reserve1, token_out.decimals),
                    token_out.symbol
                );
            } else {
                println!("      Reserves: {} {} -> {} {}", 
                    format_amount(pool.reserve1, token_in.decimals),
                    token_in.symbol,
                    format_amount(pool.reserve0, token_out.decimals),
                    token_out.symbol
                );
            }
        }
    }
    
    // Get a price quote for swapping WETH to USDC
    let amount_in = "1000000000000000000"; // 1 WETH (18 decimals)
    let quote = quoter.get_quote(weth_addr, usdc_addr, amount_in, max_hops)?;
    
    println!("\n💰 Price Quote:");
    println!("  Input: {} WETH", format_amount(amount_in.parse::<u128>().unwrap(), weth.decimals));
    println!("  Output: {} USDC", format_amount(quote.amount_out.parse::<u128>().unwrap(), usdc.decimals));
    println!("  Path: {} pools", quote.path.len());
    
    // Calculate price
    let amount_in_f64 = amount_in.parse::<f64>().unwrap() / 10f64.powi(weth.decimals as i32);
    let amount_out_f64 = quote.amount_out.parse::<f64>().unwrap() / 10f64.powi(usdc.decimals as i32);
    let price = amount_out_f64 / amount_in_f64;
    
    println!("  Price: 1 WETH = {:.2} USDC", price);
    
    Ok(())
}

// Helper function to format token amounts with proper decimal places
fn format_amount(amount: u128, decimals: u8) -> String {
    if decimals == 0 {
        return amount.to_string();
    }
    
    let divisor = 10u128.pow(decimals as u32);
    let whole_part = amount / divisor;
    let fractional_part = amount % divisor;
    
    if fractional_part == 0 {
        return whole_part.to_string();
    }
    
    // Format with proper leading zeros
    let fractional_str = format!("{:0width$}", fractional_part, width = decimals as usize);
    
    // Trim trailing zeros
    let trimmed = fractional_str.trim_end_matches('0');
    
    if trimmed.is_empty() {
        whole_part.to_string()
    } else {
        format!("{}.{}", whole_part, trimmed)
    }
}
