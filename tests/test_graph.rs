// tests/test_graph.rs

use tycho_price_quoter::{Token, Pool, TokenGraph};
use std::collections::{HashMap, HashSet};

fn create_test_data() -> (HashMap<String, Token>, Vec<Pool>) {
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

#[test]
fn test_graph_creation() {
    let (_, pools) = create_test_data();
    let mut graph = TokenGraph::new();
    
    // Add pools to graph
    for pool in &pools {
        graph.add_pool(pool);
    }
    
    // Check token count
    assert_eq!(graph.token_count(), 3);
    
    // Check edge count (2 pools * 2 directions = 4 edges)
    assert_eq!(graph.edge_count(), 4);
}

#[test]
fn test_get_edges() {
    let (_, pools) = create_test_data();
    let mut graph = TokenGraph::new();
    
    // Add pools to graph
    for pool in &pools {
        graph.add_pool(pool);
    }
    
    // Check edges for WETH
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let weth_edges = graph.get_edges(&weth_addr);
    
    assert_eq!(weth_edges.len(), 2);
    
    // Check that WETH has edges to both USDC and DAI
    let edge_destinations: HashSet<_> = weth_edges.iter().map(|e| &e.to).collect();
    assert!(edge_destinations.contains(&"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()));
    assert!(edge_destinations.contains(&"0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string()));
}

#[test]
fn test_remove_pool() {
    let (_, pools) = create_test_data();
    let mut graph = TokenGraph::new();
    
    // Add pools to graph
    for pool in &pools {
        graph.add_pool(pool);
    }
    
    // Remove the first pool
    graph.remove_pool(&pools[0]);
    
    // Check edge count (1 pool * 2 directions = 2 edges)
    assert_eq!(graph.edge_count(), 2);
    
    // Check that WETH only has an edge to DAI now
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let weth_edges = graph.get_edges(&weth_addr);
    
    assert_eq!(weth_edges.len(), 1);
    assert_eq!(weth_edges[0].to, "0x6B175474E89094C44Da98b954EedeAC495271d0F");
}

#[test]
fn test_update_pool() {
    let (_, pools) = create_test_data();
    let mut graph = TokenGraph::new();
    
    // Add pools to graph
    for pool in &pools {
        graph.add_pool(pool);
    }
    
    // Create a new pool with the same ID but different tokens
    let new_pool = Pool {
        id: pools[0].id.clone(),
        token0: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(), // USDT
        token1: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(), // WETH
        reserve0: 4_000_000 * 10u128.pow(6), // 4M USDT
        reserve1: 1_800 * 10u128.pow(18),    // 1.8k WETH
        fee: 30,
    };
    
    // Update the pool
    graph.update_pool(&pools[0], &new_pool);
    
    // Check token count (should now be 4: USDC, WETH, DAI, USDT)
    assert_eq!(graph.token_count(), 4);
    
    // Check that WETH has edges to both USDT and DAI
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let weth_edges = graph.get_edges(&weth_addr);
    
    assert_eq!(weth_edges.len(), 2);
    
    let edge_destinations: HashSet<_> = weth_edges.iter().map(|e| &e.to).collect();
    assert!(edge_destinations.contains(&"0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string()));
    assert!(edge_destinations.contains(&"0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string()));
}
