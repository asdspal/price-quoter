# Tycho Price Quoter

A Rust library for finding optimal token swap paths and calculating price quotes using real-time liquidity data from the Tycho Indexer.

## Overview

The Tycho Price Quoter is a lightweight, efficient library that provides price quoting functionality for token swaps on Ethereum. It connects to the Tycho Indexer to fetch real-time liquidity data from various DEXs (currently supporting Uniswap V2-style pools), builds a graph representation of the token ecosystem, and finds the most efficient paths for token swaps.

## Features

- **Real-time Data**: Fetches up-to-date pool and token data from the Tycho Indexer
- **Multi-hop Routing**: Finds optimal paths between token pairs, supporting direct and multi-hop routes
- **Price Quoting**: Calculates expected output amounts for token swaps using constant product formulas
- **Efficient Graph Representation**: Maintains an in-memory graph of tokens and pools for fast path finding
- **Modular Design**: Cleanly separated components for storage, graph representation, path finding, and price calculation

## Architecture

The library is organized into several modules:

- **types.rs**: Core data structures like `Token`, `Pool`, and `SwapPath`
- **storage.rs**: Manages token and pool data with efficient lookups
- **graph.rs**: Represents the token ecosystem as a directed graph
- **pathfinder.rs**: Implements BFS algorithm to find swap paths
- **quoting.rs**: Calculates price quotes using pool math
- **tycho.rs**: Handles communication with the Tycho Indexer

## Usage

### Basic Usage

```rust
use tycho_price_quoter::PriceQuoter;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment variable
    let api_key = env::var("TYCHO_API_KEY").ok();
    let tycho_url = "https://tycho-beta.propellerheads.xyz";
    
    // Initialize the price quoter with data from Tycho
    let quoter = PriceQuoter::from_tycho(tycho_url, api_key).await?;
    
    // Define token addresses (without 0x prefix, lowercase)
    let weth_addr = "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
    let usdc_addr = "a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
    
    // Get a price quote for swapping 1 WETH to USDC
    let amount_in = "1000000000000000000"; // 1 WETH (18 decimals)
    let max_hops = 2;
    let quote = quoter.get_quote(weth_addr, usdc_addr, amount_in, max_hops)?;
    
    println!("Price Quote:");
    println!("  Input: {} WETH", amount_in);
    println!("  Output: {} USDC", quote.amount_out);
    println!("  Path: {} pools", quote.path.len());
    
    Ok(())
}
```

### Advanced Usage

```rust
use tycho_price_quoter::{PriceQuoter, pathfinder::find_paths};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the price quoter
    let api_key = env::var("TYCHO_API_KEY").ok();
    let tycho_url = "https://tycho-beta.propellerheads.xyz";
    let quoter = PriceQuoter::from_tycho(tycho_url, api_key).await?;
    
    // Define token addresses
    let weth_addr = "c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
    let usdc_addr = "a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
    
    // Find all paths between WETH and USDC with max 2 hops
    let max_hops = 2;
    let paths = find_paths(
        quoter.get_storage(),
        &weth_addr.to_string(),
        &usdc_addr.to_string(),
        max_hops
    );
    
    // Print details of each path
    for (i, path) in paths.iter().enumerate() {
        println!("Path #{} ({} hops):", i + 1, path.hops.len());
        
        for (j, hop) in path.hops.iter().enumerate() {
            let pool = quoter.get_storage().get_pool(&hop.pool_id).unwrap();
            let token_in = quoter.get_storage().get_token(&hop.token_in).unwrap();
            let token_out = quoter.get_storage().get_token(&hop.token_out).unwrap();
            
            println!("  Hop #{}: {} -> {} (Pool: {})", 
                j + 1,
                token_in.symbol,
                token_out.symbol,
                &hop.pool_id[0..10]
            );
        }
    }
    
    Ok(())
}
```

## Implementation Details

### Token and Pool Representation

Tokens and pools are represented as simple structs:

```rust
pub struct Token {
    pub address: Address,
    pub decimals: u8,
    pub symbol: String,
}

pub struct Pool {
    pub id: String,
    pub token0: Address,
    pub token1: Address,
    pub reserve0: U256,
    pub reserve1: U256,
    pub fee: u32,
}
```

### Graph Representation

The token ecosystem is represented as a directed graph where:
- Nodes are tokens
- Edges are pools connecting tokens
- Each edge has attributes like pool ID and reserves

### Path Finding

The library uses a Breadth-First Search (BFS) algorithm to find all possible paths between two tokens, respecting the maximum hop constraint.

### Price Calculation

Price quotes are calculated using the constant product formula (x * y = k) for Uniswap V2-style pools:
1. For each hop in the path, apply the pool's fee to the input amount
2. Calculate the output amount using the formula: `amount_out = (reserve_out * amount_in) / (reserve_in + amount_in)`
3. Use the output amount as the input for the next hop

## Important Notes

1. **Token Addresses**: Token addresses should be provided without the "0x" prefix and in lowercase.
2. **API Key**: A Tycho API key is required to fetch data from the Tycho Indexer.
3. **Pool Types**: Currently, only Uniswap V2-style constant product pools are supported.

## Future Improvements

- Support for additional DEX protocols (Uniswap V3, Balancer, Curve, etc.)
- Gas-aware routing to optimize for gas costs
- Slippage estimation and handling
- Support for token-specific features (fee-on-transfer tokens, rebasing tokens, etc.)
- Streaming updates for real-time price changes
- Parallel path finding for improved performance

## Dependencies

- `tycho-client`: For communicating with the Tycho Indexer
- `tycho-common`: For common data types and utilities
- `tokio`: For async runtime
- `thiserror`: For error handling
- `hex`: For encoding/decoding hex strings

## License

[MIT License](LICENSE)
