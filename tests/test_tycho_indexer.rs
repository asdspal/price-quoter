// test_tycho_indexer.rs

use std::env;
use std::error::Error;
use std::time::Duration;

// Import the necessary crates
use tycho_indexer::{
    ProtocolStreamBuilder, Chain, TvlFilter, load_all_tokens, ProtocolStream,
    component::ComponentWithState,
};
use tycho_simulation::prelude::*;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Get environment variables or use defaults
    let tycho_url = env::var("TYCHO_URL")
        .unwrap_or_else(|_| "https://tycho-beta.propellerheads.xyz".to_string());
    
    let tycho_api_key = env::var("TYCHO_API_KEY")
        .unwrap_or_else(|_| "sampletoken".to_string());
    
    // Set TVL threshold (10 ETH for testing)
    let tvl_threshold = 10.0;
    
    println!("Connecting to Tycho Indexer at {}", tycho_url);
    println!("Using API key: {}", tycho_api_key);
    println!("TVL threshold: {} ETH", tvl_threshold);
    
    // Step 1: Load all tokens
    println!("\nStep 1: Loading all tokens...");
    let all_tokens = match load_all_tokens(&tycho_url, &tycho_api_key, Chain::Ethereum).await {
        Ok(tokens) => {
            println!("Successfully loaded {} tokens", tokens.len());
            
            // Print a few example tokens
            println!("Example tokens:");
            for (i, (address, token)) in tokens.iter().take(5).enumerate() {
                println!("  {}: {} ({}) - {} decimals", 
                         i+1, token.symbol, address, token.decimals);
            }
            
            tokens
        },
        Err(e) => {
            println!("Failed to load tokens: {}", e);
            return Err(e.into());
        }
    };
    
    // Step 2: Create a TVL filter
    let tvl_filter = TvlFilter::new(tvl_threshold);
    
    // Step 3: Build the protocol stream
    println!("\nStep 2: Building protocol stream...");
    let mut protocol_stream = match ProtocolStreamBuilder::new(&tycho_url, Chain::Ethereum)
        .exchange::<UniswapV2State>("uniswap_v2", tvl_filter.clone(), None)
        // Uncomment to add more protocols
        // .exchange::<UniswapV3State>("uniswap_v3", tvl_filter.clone(), None)
        .auth_key(Some(tycho_api_key.clone()))
        .set_tokens(all_tokens.clone())
        .await
        .build()
        .await {
            Ok(stream) => {
                println!("Successfully built protocol stream");
                stream
            },
            Err(e) => {
                println!("Failed to build protocol stream: {}", e);
                return Err(e.into());
            }
        };
    
    // Step 4: Wait for the first block update
    println!("\nStep 3: Waiting for first block update...");
    let block_update = match protocol_stream.next().await {
        Ok(update) => {
            println!("Received block update for block {}", update.block_number);
            update
        },
        Err(e) => {
            println!("Failed to get block update: {}", e);
            return Err(e.into());
        }
    };
    
    // Step 5: Analyze the protocols and components
    println!("\nStep 4: Analyzing protocols and components...");
    
    let mut total_components = 0;
    
    for (protocol_id, protocol) in block_update.protocols.iter() {
        let component_count = protocol.components.len();
        total_components += component_count;
        
        println!("Protocol: {} - {} components", protocol_id, component_count);
        
        // Print details of a few components
        for (i, (component_id, component)) in protocol.components.iter().take(3).enumerate() {
            println!("  Component {}: {}", i+1, component_id);
            print_component_details(component);
        }
        
        if component_count > 3 {
            println!("  ... and {} more components", component_count - 3);
        }
    }
    
    println!("\nTotal components across all protocols: {}", total_components);
    
    // Step 6: Find a specific token pair (USDC-WETH)
    println!("\nStep 5: Finding USDC-WETH pools...");
    
    // Define USDC and WETH addresses
    let usdc_address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let weth_address = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    
    let mut usdc_weth_pools = Vec::new();
    
    for (protocol_id, protocol) in block_update.protocols.iter() {
        for (component_id, component) in protocol.components.iter() {
            if component.component.tokens.contains(&usdc_address.to_string()) && 
               component.component.tokens.contains(&weth_address.to_string()) {
                usdc_weth_pools.push((protocol_id, component_id, component));
            }
        }
    }
    
    println!("Found {} USDC-WETH pools", usdc_weth_pools.len());
    
    // Print details of the USDC-WETH pools
    for (i, (protocol_id, component_id, component)) in usdc_weth_pools.iter().enumerate() {
        println!("  Pool {}: {} - {}", i+1, protocol_id, component_id);
        print_component_details(component);
        
        // Try to get the simulation state for this component
        match protocol_stream.get_simulation_state(component) {
            Ok(state) => {
                println!("    Successfully got simulation state");
                
                // Try to simulate a swap
                let amount_in = 1_000_000_000; // 1000 USDC (6 decimals)
                
                match state.get_amount_out(amount_in, usdc_address, weth_address) {
                    Ok(result) => {
                        let amount_out = result.amount_out;
                        let amount_out_f = amount_out as f64 / 1e18; // WETH has 18 decimals
                        
                        println!("    Simulated swap: 1000 USDC -> {} WETH", amount_out_f);
                        println!("    Price: {} WETH per USDC", amount_out_f / 1000.0);
                    },
                    Err(e) => {
                        println!("    Failed to simulate swap: {}", e);
                    }
                }
            },
            Err(e) => {
                println!("    Failed to get simulation state: {}", e);
            }
        }
    }
    
    // Step 7: Listen for more updates
    println!("\nStep 6: Listening for more updates (will exit after 3 updates)...");
    
    let mut update_count = 0;
    while update_count < 3 {
        match protocol_stream.next().await {
            Ok(update) => {
                update_count += 1;
                println!("Received block update for block {}", update.block_number);
                
                let mut total_components = 0;
                for (protocol_id, protocol) in update.protocols.iter() {
                    let component_count = protocol.components.len();
                    total_components += component_count;
                    
                    println!("  Protocol: {} - {} components", protocol_id, component_count);
                }
                
                println!("  Total components in this update: {}", total_components);
            },
            Err(e) => {
                println!("Failed to get block update: {}", e);
                break;
            }
        }
        
        // Wait a bit before the next update
        sleep(Duration::from_secs(5)).await;
    }
    
    println!("\nTest completed successfully!");
    
    Ok(())
}

// Helper function to print component details
fn print_component_details(component: &ComponentWithState) {
    println!("    ID: {}", component.component.id);
    println!("    Tokens: {:?}", component.component.tokens);
    
    // Print a few static attributes
    println!("    Static Attributes:");
    for (key, value) in component.component.static_attributes.iter().take(3) {
        println!("      {}: {:?}", key, value);
    }
    
    // Print a few dynamic attributes
    println!("    Dynamic Attributes:");
    for (key, value) in component.state.attributes.iter().take(3) {
        println!("      {}: {:?}", key, value);
    }
}
