// tests/graph_tests.rs
use tycho_price_quoter::{
    graph::TokenGraph,
    storage::PoolStorage,
    types::{Token, Pool, U256, Address},
};
use std::collections::{HashMap, HashSet};

fn create_test_data() -> (HashMap<Address, Token>, Vec<Pool>) {
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
        reserve0: 1_500 * 10u128.pow(18),    // 1.5k WETH
        reserve1: 4_500_000 * 10u128.pow(18), // 4.5M DAI
        fee: 30,
    };

    (tokens, vec![pool1, pool2])
}

#[test]
fn test_empty_graph_creation() {
    let graph = TokenGraph::new();
    
    assert_eq!(graph.token_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn test_graph_from_storage() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let graph = TokenGraph::from_storage(&storage);
    
    // 3 tokens
    assert_eq!(graph.token_count(), 3);
    
    // 2 pools * 2 directions = 4 edges
    assert_eq!(graph.edge_count(), 4);
}

#[test]
fn test_add_pool_to_graph() {
    let mut graph = TokenGraph::new();
    
    let pool = Pool {
        id: "0xPool".to_string(),
        token0: "0xToken0".to_string(),
        token1: "0xToken1".to_string(),
        reserve0: 1000,
        reserve1: 2000,
        fee: 30,
    };
    
    graph.add_pool(&pool);
    
    // 2 tokens
    assert_eq!(graph.token_count(), 2);
    
    // 1 pool * 2 directions = 2 edges
    assert_eq!(graph.edge_count(), 2);
    
    // Check edges
    let edges0 = graph.get_edges(&pool.token0);
    let edges1 = graph.get_edges(&pool.token1);
    
    assert_eq!(edges0.len(), 1);
    assert_eq!(edges1.len(), 1);
    
    assert_eq!(edges0[0].to, pool.token1);
    assert_eq!(edges1[0].to, pool.token0);
}

#[test]
fn test_remove_pool_from_graph() {
    let (_, pools) = create_test_data();
    let mut graph = TokenGraph::new();
    
    // Add both pools
    for pool in &pools {
        graph.add_pool(pool);
    }
    
    // Initial state: 3 tokens, 4 edges
    assert_eq!(graph.token_count(), 3);
    assert_eq!(graph.edge_count(), 4);
    
    // Remove the first pool
    graph.remove_pool(&pools[0]);
    
    // After removal: still 3 tokens (all tokens are still in the graph via the second pool)
    // but only 2 edges (from the remaining pool)
    assert_eq!(graph.token_count(), 3);
    assert_eq!(graph.edge_count(), 2);
    
    // Check that WETH only has edges to DAI now
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let weth_edges = graph.get_edges(&weth_addr);
    
    assert_eq!(weth_edges.len(), 1);
    assert_eq!(weth_edges[0].to, "0x6B175474E89094C44Da98b954EedeAC495271d0F");
}

#[test]
fn test_update_pool_in_graph() {
    let (_, pools) = create_test_data();
    let mut graph = TokenGraph::new();
    
    // Add the first pool
    graph.add_pool(&pools[0]);
    
    // Create a new pool with the same ID but different tokens
    let new_pool = Pool {
        id: pools[0].id.clone(),
        token0: "0xNewToken0".to_string(),
        token1: "0xNewToken1".to_string(),
        reserve0: 1000,
        reserve1: 2000,
        fee: 30,
    };
    
    // Update the pool
    graph.update_pool(&pools[0], &new_pool);
    
    // Check that the old tokens are gone and new ones are added
    assert!(!graph.contains_token(&pools[0].token0));
    assert!(!graph.contains_token(&pools[0].token1));
    assert!(graph.contains_token(&new_pool.token0));
    assert!(graph.contains_token(&new_pool.token1));
}

#[test]
fn test_get_all_tokens() {
    let (_, pools) = create_test_data();
    let mut graph = TokenGraph::new();
    
    // Add both pools
    for pool in &pools {
        graph.add_pool(pool);
    }
    
    let all_tokens = graph.get_all_tokens();
    
    // Should have 3 unique tokens
    assert_eq!(all_tokens.len(), 3);
    
    // Check specific tokens
    assert!(all_tokens.contains(&"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string()));
    assert!(all_tokens.contains(&"0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string()));
    assert!(all_tokens.contains(&"0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string()));
}
