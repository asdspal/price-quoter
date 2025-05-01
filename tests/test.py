# Create comprehensive test implementations for the Tycho Price Quoter library

with open('types_tests.rs', 'w') as f:
    f.write('''// tests/types_tests.rs
use tycho_price_quoter::types::{Token, Pool, SwapHop, SwapPath, parse_bytes_to_u256, U256, Address};

#[test]
fn test_token_creation() {
    let token = Token {
        address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        decimals: 6,
        symbol: "USDC".to_string(),
    };
    
    assert_eq!(token.address, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    assert_eq!(token.decimals, 6);
    assert_eq!(token.symbol, "USDC");
}

#[test]
fn test_pool_creation() {
    let pool = Pool {
        id: "0xPool1".to_string(),
        token0: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        token1: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        reserve0: 5_000_000 * 10u128.pow(6),
        reserve1: 2_000 * 10u128.pow(18),
        fee: 30,
    };
    
    assert_eq!(pool.id, "0xPool1");
    assert_eq!(pool.token0, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    assert_eq!(pool.token1, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    assert_eq!(pool.reserve0, 5_000_000 * 10u128.pow(6));
    assert_eq!(pool.reserve1, 2_000 * 10u128.pow(18));
    assert_eq!(pool.fee, 30);
}

#[test]
fn test_swap_path_creation() {
    let hop1 = SwapHop {
        pool_id: "0xPool1".to_string(),
        token_in: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        token_out: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
    };
    
    let hop2 = SwapHop {
        pool_id: "0xPool2".to_string(),
        token_in: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        token_out: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
    };
    
    let path = SwapPath {
        hops: vec![hop1.clone(), hop2.clone()],
        token_in: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        token_out: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
    };
    
    assert_eq!(path.hops.len(), 2);
    assert_eq!(path.hops[0], hop1);
    assert_eq!(path.hops[1], hop2);
    assert_eq!(path.token_in, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    assert_eq!(path.token_out, "0x6B175474E89094C44Da98b954EedeAC495271d0F");
}

#[test]
fn test_parse_bytes_to_u256() {
    // Test valid case
    let bytes = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100]; // 100 in big-endian
    let result = parse_bytes_to_u256(&bytes);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100);
    
    // Test empty bytes
    let empty_bytes = vec![];
    let result = parse_bytes_to_u256(&empty_bytes);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
    
    // Test invalid length
    let invalid_bytes = vec![1, 2, 3]; // Too short
    let result = parse_bytes_to_u256(&invalid_bytes);
    assert!(result.is_err());
}
''')

with open('storage_tests.rs', 'w') as f:
    f.write('''// tests/storage_tests.rs
use tycho_price_quoter::{
    storage::{PoolStorage, StorageError},
    types::{Token, Pool, U256, Address},
};
use std::collections::HashMap;

fn create_test_tokens() -> HashMap<Address, Token> {
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();

    let usdc = Token { address: usdc_addr.clone(), decimals: 6, symbol: "USDC".to_string() };
    let weth = Token { address: weth_addr.clone(), decimals: 18, symbol: "WETH".to_string() };
    let dai = Token { address: dai_addr.clone(), decimals: 18, symbol: "DAI".to_string() };

    let mut tokens = HashMap::new();
    tokens.insert(usdc_addr, usdc);
    tokens.insert(weth_addr, weth);
    tokens.insert(dai_addr, dai);
    
    tokens
}

fn create_test_pools() -> Vec<Pool> {
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();

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
        token0: weth_addr,
        token1: dai_addr,
        reserve0: 1_500 * 10u128.pow(18),    // 1.5k WETH
        reserve1: 4_500_000 * 10u128.pow(18), // 4.5M DAI
        fee: 30,
    };

    vec![pool1, pool2]
}

#[test]
fn test_storage_initialization() {
    let storage = PoolStorage::new();
    assert_eq!(storage.token_count(), 0);
    assert_eq!(storage.pool_count(), 0);
}

#[test]
fn test_storage_with_initial_data() {
    let tokens = create_test_tokens();
    let pools = create_test_pools();
    
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    assert_eq!(storage.token_count(), 3);
    assert_eq!(storage.pool_count(), 2);
}

#[test]
fn test_add_and_get_token() {
    let mut storage = PoolStorage::new();
    
    let token = Token {
        address: "0xToken".to_string(),
        decimals: 18,
        symbol: "TKN".to_string(),
    };
    
    storage.add_token(token.clone());
    
    let retrieved = storage.get_token(&token.address).unwrap();
    assert_eq!(retrieved.address, token.address);
    assert_eq!(retrieved.decimals, token.decimals);
    assert_eq!(retrieved.symbol, token.symbol);
}

#[test]
fn test_add_and_get_pool() {
    let mut storage = PoolStorage::new();
    
    let pool = Pool {
        id: "0xPool".to_string(),
        token0: "0xToken0".to_string(),
        token1: "0xToken1".to_string(),
        reserve0: 1000,
        reserve1: 2000,
        fee: 30,
    };
    
    storage.add_pool(pool.clone());
    
    let retrieved = storage.get_pool(&pool.id).unwrap();
    assert_eq!(retrieved.id, pool.id);
    assert_eq!(retrieved.token0, pool.token0);
    assert_eq!(retrieved.token1, pool.token1);
    assert_eq!(retrieved.reserve0, pool.reserve0);
    assert_eq!(retrieved.reserve1, pool.reserve1);
    assert_eq!(retrieved.fee, pool.fee);
}

#[test]
fn test_update_pool() {
    let mut storage = PoolStorage::new();
    
    let pool = Pool {
        id: "0xPool".to_string(),
        token0: "0xToken0".to_string(),
        token1: "0xToken1".to_string(),
        reserve0: 1000,
        reserve1: 2000,
        fee: 30,
    };
    
    storage.add_pool(pool);
    
    // Update the pool reserves
    let result = storage.update_pool("0xPool", 1500, 2500);
    assert!(result.is_ok());
    
    let updated = storage.get_pool("0xPool").unwrap();
    assert_eq!(updated.reserve0, 1500);
    assert_eq!(updated.reserve1, 2500);
}

#[test]
fn test_update_nonexistent_pool() {
    let mut storage = PoolStorage::new();
    
    let result = storage.update_pool("0xNonExistent", 1000, 2000);
    assert!(result.is_err());
}

#[test]
fn test_remove_pool() {
    let mut storage = PoolStorage::new();
    
    let pool = Pool {
        id: "0xPool".to_string(),
        token0: "0xToken0".to_string(),
        token1: "0xToken1".to_string(),
        reserve0: 1000,
        reserve1: 2000,
        fee: 30,
    };
    
    storage.add_pool(pool);
    
    // Remove the pool
    let result = storage.remove_pool("0xPool");
    assert!(result.is_ok());
    
    // Verify it's gone
    assert!(storage.get_pool("0xPool").is_none());
    assert_eq!(storage.pool_count(), 0);
}

#[test]
fn test_get_pools_for_token() {
    let tokens = create_test_tokens();
    let pools = create_test_pools();
    
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let weth_pools = storage.get_pools_for_token(weth_addr);
    
    // WETH should be in both pools
    assert_eq!(weth_pools.len(), 2);
}

#[test]
fn test_has_direct_pool() {
    let tokens = create_test_tokens();
    let pools = create_test_pools();
    
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    
    // USDC and WETH have a direct pool
    assert!(storage.has_direct_pool(usdc_addr, weth_addr));
    
    // WETH and DAI have a direct pool
    assert!(storage.has_direct_pool(weth_addr, dai_addr));
    
    // USDC and DAI don't have a direct pool
    assert!(!storage.has_direct_pool(usdc_addr, dai_addr));
}

#[test]
fn test_get_all_tokens_and_pools() {
    let tokens = create_test_tokens();
    let pools = create_test_pools();
    
    let storage = PoolStorage::with_initial_data(pools.clone(), tokens.clone());
    
    let all_tokens = storage.get_all_tokens();
    let all_pools = storage.get_all_pools();
    
    assert_eq!(all_tokens.len(), tokens.len());
    assert_eq!(all_pools.len(), pools.len());
}
''')

with open('graph_tests.rs', 'w') as f:
    f.write('''// tests/graph_tests.rs
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
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let weth_edges = graph.get_edges(weth_addr);
    
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
''')

with open('pathfinder_tests.rs', 'w') as f:
    f.write('''// tests/pathfinder_tests.rs
use tycho_price_quoter::{
    pathfinder::{find_paths, find_best_path},
    storage::PoolStorage,
    graph::TokenGraph,
    types::{Token, Pool, U256, Address},
};
use std::collections::HashMap;

fn create_test_data() -> (HashMap<Address, Token>, Vec<Pool>) {
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    let usdt_addr = "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string();

    let usdc = Token { address: usdc_addr.clone(), decimals: 6, symbol: "USDC".to_string() };
    let weth = Token { address: weth_addr.clone(), decimals: 18, symbol: "WETH".to_string() };
    let dai = Token { address: dai_addr.clone(), decimals: 18, symbol: "DAI".to_string() };
    let usdt = Token { address: usdt_addr.clone(), decimals: 6, symbol: "USDT".to_string() };

    let mut tokens = HashMap::new();
    tokens.insert(usdc_addr.clone(), usdc);
    tokens.insert(weth_addr.clone(), weth);
    tokens.insert(dai_addr.clone(), dai);
    tokens.insert(usdt_addr.clone(), usdt);

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

    let pool3 = Pool {
        id: "0xPoolUsdtWeth".to_string(),
        token0: usdt_addr.clone(),
        token1: weth_addr.clone(),
        reserve0: 4_000_000 * 10u128.pow(6), // 4M USDT
        reserve1: 1_800 * 10u128.pow(18),    // 1.8k WETH
        fee: 30,
    };

    (tokens, vec![pool1, pool2, pool3])
}

#[test]
fn test_find_direct_path() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    
    // Find direct paths with max_hops = 1
    let paths = find_paths(&storage, usdc_addr, weth_addr, 1);
    
    // Should find 1 direct path
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].hops.len(), 1);
    assert_eq!(paths[0].hops[0].token_in, usdc_addr);
    assert_eq!(paths[0].hops[0].token_out, weth_addr);
}

#[test]
fn test_find_multi_hop_path() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    
    // Find paths with max_hops = 2
    let paths = find_paths(&storage, usdc_addr, dai_addr, 2);
    
    // Should find 1 path: USDC -> WETH -> DAI
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].hops.len(), 2);
    
    // First hop: USDC -> WETH
    assert_eq!(paths[0].hops[0].token_in, usdc_addr);
    assert_eq!(paths[0].hops[0].token_out, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    
    // Second hop: WETH -> DAI
    assert_eq!(paths[0].hops[1].token_in, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    assert_eq!(paths[0].hops[1].token_out, dai_addr);
}

#[test]
fn test_find_path_with_nonexistent_token() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let nonexistent_addr = "0xNonExistentToken";
    
    // Try to find paths with a non-existent token
    let paths = find_paths(&storage, usdc_addr, nonexistent_addr, 2);
    
    // Should find no paths
    assert_eq!(paths.len(), 0);
}

#[test]
fn test_find_path_with_disconnected_tokens() {
    let (mut tokens, pools) = create_test_data();
    
    // Add a disconnected token
    let disconnected_addr = "0xDisconnectedToken".to_string();
    tokens.insert(disconnected_addr.clone(), Token {
        address: disconnected_addr.clone(),
        decimals: 18,
        symbol: "DISCONNECTED".to_string(),
    });
    
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    
    // Try to find paths to the disconnected token
    let paths = find_paths(&storage, usdc_addr, &disconnected_addr, 3);
    
    // Should find no paths
    assert_eq!(paths.len(), 0);
}

#[test]
fn test_find_path_with_max_hops_limit() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    
    // Find paths with max_hops = 1 (not enough for USDC -> DAI which needs 2 hops)
    let paths = find_paths(&storage, usdc_addr, dai_addr, 1);
    
    // Should find no paths
    assert_eq!(paths.len(), 0);
}

#[test]
fn test_find_multiple_paths() {
    let (tokens, mut pools) = create_test_data();
    
    // Add a direct pool between USDC and DAI to create multiple paths
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    
    let direct_pool = Pool {
        id: "0xPoolUsdcDai".to_string(),
        token0: usdc_addr.clone(),
        token1: dai_addr.clone(),
        reserve0: 3_000_000 * 10u128.pow(6), // 3M USDC
        reserve1: 3_000_000 * 10u128.pow(18), // 3M DAI
        fee: 30,
    };
    
    pools.push(direct_pool);
    
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    // Find paths with max_hops = 2
    let paths = find_paths(&storage, &usdc_addr, &dai_addr, 2);
    
    // Should find 2 paths: 
    // 1. USDC -> DAI (direct)
    // 2. USDC -> WETH -> DAI
    assert_eq!(paths.len(), 2);
    
    // Check that one path has 1 hop and the other has 2 hops
    let one_hop_paths = paths.iter().filter(|p| p.hops.len() == 1).count();
    let two_hop_paths = paths.iter().filter(|p| p.hops.len() == 2).count();
    
    assert_eq!(one_hop_paths, 1);
    assert_eq!(two_hop_paths, 1);
}

#[test]
fn test_find_best_path() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools.clone(), tokens.clone());
    let graph = TokenGraph::from_storage(&storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    
    // Find the best path for USDC -> DAI
    let amount_in = "1000000000"; // 1000 USDC with 6 decimals
    let best_path = find_best_path(&graph, &storage, usdc_addr, dai_addr, amount_in, 2);
    
    // Should find a path
    assert!(best_path.is_some());
    
    let path = best_path.unwrap();
    
    // Should be a 2-hop path: USDC -> WETH -> DAI
    assert_eq!(path.hops.len(), 2);
    assert_eq!(path.token_in, usdc_addr);
    assert_eq!(path.token_out, dai_addr);
}
''')

with open('quoting_tests.rs', 'w') as f:
    f.write('''// tests/quoting_tests.rs
use tycho_price_quoter::{
    quoting::{PriceQuoter, QuoterError},
    storage::PoolStorage,
    types::{Token, Pool, U256, Address},
};
use std::collections::HashMap;

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
fn test_price_quoter_initialization() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    assert_eq!(quoter.get_storage().token_count(), 3);
    assert_eq!(quoter.get_storage().pool_count(), 2);
}

#[test]
fn test_get_quote_direct_swap() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    
    // Get quote for USDC -> WETH (direct swap)
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, weth_addr, amount_in, 1);
    
    // Should succeed
    assert!(quote.is_ok());
    
    let quote = quote.unwrap();
    assert_eq!(quote.token_in, usdc_addr);
    assert_eq!(quote.token_out, weth_addr);
    assert_eq!(quote.amount_in, amount_in);
    assert_eq!(quote.path.len(), 1); // Direct path
}

#[test]
fn test_get_quote_multi_hop_swap() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    
    // Get quote for USDC -> DAI (multi-hop swap)
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, dai_addr, amount_in, 2);
    
    // Should succeed
    assert!(quote.is_ok());
    
    let quote = quote.unwrap();
    assert_eq!(quote.token_in, usdc_addr);
    assert_eq!(quote.token_out, dai_addr);
    assert_eq!(quote.amount_in, amount_in);
    assert_eq!(quote.path.len(), 2); // 2-hop path
}

#[test]
fn test_get_quote_nonexistent_token() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let nonexistent_addr = "0xNonExistentToken";
    
    // Try to get quote with a non-existent token
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, nonexistent_addr, amount_in, 2);
    
    // Should fail with TokenNotFound error
    assert!(quote.is_err());
    match quote.unwrap_err() {
        QuoterError::TokenNotFound(_) => {}, // Expected error
        err => panic!("Unexpected error: {:?}", err),
    }
}

#[test]
fn test_get_quote_no_path() {
    let (mut tokens, pools) = create_test_data();
    
    // Add a disconnected token
    let disconnected_addr = "0xDisconnectedToken".to_string();
    tokens.insert(disconnected_addr.clone(), Token {
        address: disconnected_addr.clone(),
        decimals: 18,
        symbol: "DISCONNECTED".to_string(),
    });
    
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    
    // Try to get quote to the disconnected token
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, &disconnected_addr, amount_in, 3);
    
    // Should fail with NoPathFound error
    assert!(quote.is_err());
    match quote.unwrap_err() {
        QuoterError::NoPathFound => {}, // Expected error
        err => panic!("Unexpected error: {:?}", err),
    }
}

#[test]
fn test_get_quote_different_decimal_scales() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"; // 6 decimals
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // 18 decimals
    
    // Get quote for USDC -> WETH (different decimal scales)
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, weth_addr, amount_in, 1);
    
    // Should succeed
    assert!(quote.is_ok());
    
    let quote = quote.unwrap();
    
    // Output amount should be positive
    let amount_out = quote.amount_out.parse::<u128>().unwrap();
    assert!(amount_out > 0);
}
''')

with open('tycho_integration_tests.rs', 'w') as f:
    f.write('''// tests/tycho_integration_tests.rs
use tycho_price_quoter::{
    tycho::TychoConnector,
    types::{Token, Pool},
};
use std::env;

// These tests require actual Tycho API access, so they're ignored by default
// To run them, use: cargo test -- --ignored

#[tokio::test]
#[ignore]
async fn test_tycho_connection() {
    let api_key = env::var("TYCHO_API_KEY").ok();
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    let connector = TychoConnector::new(&url, api_key).await;
    assert!(connector.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_fetch_tokens() {
    let api_key = env::var("TYCHO_API_KEY").ok();
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    let connector = TychoConnector::new(&url, api_key).await.unwrap();
    let tokens = connector.fetch_tokens().await;
    
    assert!(tokens.is_ok());
    
    let tokens = tokens.unwrap();
    assert!(!tokens.is_empty());
    
    // Check for some common tokens
    let weth = tokens.iter().find(|t| t.symbol == "WETH");
    let usdc = tokens.iter().find(|t| t.symbol == "USDC");
    
    assert!(weth.is_some());
    assert!(usdc.is_some());
}

#[tokio::test]
#[ignore]
async fn test_fetch_pools() {
    let api_key = env::var("TYCHO_API_KEY").ok();
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    let connector = TychoConnector::new(&url, api_key).await.unwrap();
    let pools = connector.fetch_pools().await;
    
    assert!(pools.is_ok());
    
    let pools = pools.unwrap();
    assert!(!pools.is_empty());
    
    // Check that pools have valid data
    for pool in &pools {
        assert!(!pool.id.is_empty());
        assert!(!pool.token0.is_empty());
        assert!(!pool.token1.is_empty());
        assert!(pool.fee > 0);
    }
}

#[tokio::test]
#[ignore]
async fn test_error_handling_invalid_api_key() {
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    // Use an invalid API key
    let connector = TychoConnector::new(&url, Some("invalid_api_key".to_string())).await;
    
    // Connection should still succeed (API key is validated on requests)
    assert!(connector.is_ok());
    
    let connector = connector.unwrap();
    
    // But requests should fail
    let tokens = connector.fetch_tokens().await;
    assert!(tokens.is_err());
}
''')

with open('end_to_end_test.rs', 'w') as f:
    f.write('''// tests/end_to_end_test.rs
use tycho_price_quoter::{
    PriceQuoter,
    storage::PoolStorage,
    types::{Token, Pool, U256, Address},
};
use std::collections::HashMap;
use std::env;

// This test demonstrates the complete functionality of the price quoter
#[test]
fn test_end_to_end_functionality() {
    // 1. Create test data with multiple tokens and pools
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    let usdt_addr = "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string();
    let wbtc_addr = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string();

    let usdc = Token { address: usdc_addr.clone(), decimals: 6, symbol: "USDC".to_string() };
    let weth = Token { address: weth_addr.clone(), decimals: 18, symbol: "WETH".to_string() };
    let dai = Token { address: dai_addr.clone(), decimals: 18, symbol: "DAI".to_string() };
    let usdt = Token { address: usdt_addr.clone(), decimals: 6, symbol: "USDT".to_string() };
    let wbtc = Token { address: wbtc_addr.clone(), decimals: 8, symbol: "WBTC".to_string() };

    let mut tokens = HashMap::new();
    tokens.insert(usdc_addr.clone(), usdc);
    tokens.insert(weth_addr.clone(), weth);
    tokens.insert(dai_addr.clone(), dai);
    tokens.insert(usdt_addr.clone(), usdt);
    tokens.insert(wbtc_addr.clone(), wbtc);

    // Create a more complex pool network
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

    let pool3 = Pool {
        id: "0xPoolUsdtWeth".to_string(),
        token0: usdt_addr.clone(),
        token1: weth_addr.clone(),
        reserve0: 4_000_000 * 10u128.pow(6), // 4M USDT
        reserve1: 1_800 * 10u128.pow(18),    // 1.8k WETH
        fee: 30,
    };

    let pool4 = Pool {
        id: "0xPoolWethWbtc".to_string(),
        token0: weth_addr.clone(),
        token1: wbtc_addr.clone(),
        reserve0: 10_000 * 10u128.pow(18),    // 10k WETH
        reserve1: 500 * 10u128.pow(8),        // 500 WBTC
        fee: 30,
    };

    let pool5 = Pool {
        id: "0xPoolUsdcDai".to_string(),
        token0: usdc_addr.clone(),
        token1: dai_addr.clone(),
        reserve0: 10_000_000 * 10u128.pow(6),  // 10M USDC
        reserve1: 10_000_000 * 10u128.pow(18), // 10M DAI
        fee: 30,
    };

    let pools = vec![pool1, pool2, pool3, pool4, pool5];

    // 2. Initialize the storage and price quoter
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);

    // 3. Test direct swap quote
    let usdc_to_dai_direct = quoter.get_quote(
        &usdc_addr,
        &dai_addr,
        "1000000", // 1 USDC
        1
    ).unwrap();

    println!("Direct USDC -> DAI quote:");
    println!("  Amount in: {} USDC", usdc_to_dai_direct.amount_in);
    println!("  Amount out: {} DAI", usdc_to_dai_direct.amount_out);
    println!("  Path: {:?}", usdc_to_dai_direct.path);

    // 4. Test multi-hop swap quote
    let usdt_to_wbtc = quoter.get_quote(
        &usdt_addr,
        &wbtc_addr,
        "1000000", // 1 USDT
        2
    ).unwrap();

    println!("Multi-hop USDT -> WBTC quote:");
    println!("  Amount in: {} USDT", usdt_to_wbtc.amount_in);
    println!("  Amount out: {} WBTC", usdt_to_wbtc.amount_out);
    println!("  Path: {:?}", usdt_to_wbtc.path);

    // 5. Test finding multiple paths and selecting the best one
    let usdc_to_dai_multi = quoter.get_quote(
        &usdc_addr,
        &dai_addr,
        "1000000", // 1 USDC
        2
    ).unwrap();

    println!("Best path USDC -> DAI quote:");
    println!("  Amount in: {} USDC", usdc_to_dai_multi.amount_in);
    println!("  Amount out: {} DAI", usdc_to_dai_multi.amount_out);
    println!("  Path: {:?}", usdc_to_dai_multi.path);

    // 6. Test a more complex multi-hop path
    let usdt_to_dai = quoter.get_quote(
        &usdt_addr,
        &dai_addr,
        "1000000", // 1 USDT
        3
    ).unwrap();

    println!("Complex path USDT -> DAI quote:");
    println!("  Amount in: {} USDT", usdt_to_dai.amount_in);
    println!("  Amount out: {} DAI", usdt_to_dai.amount_out);
    println!("  Path: {:?}", usdt_to_dai.path);

    // 7. Verify all quotes have valid output amounts
    assert!(usdc_to_dai_direct.amount_out.parse::<u128>().unwrap() > 0);
    assert!(usdt_to_wbtc.amount_out.parse::<u128>().unwrap() > 0);
    assert!(usdc_to_dai_multi.amount_out.parse::<u128>().unwrap() > 0);
    assert!(usdt_to_dai.amount_out.parse::<u128>().unwrap() > 0);

    // 8. Verify the path lengths are as expected
    assert_eq!(usdc_to_dai_direct.path.len(), 1); // Direct path
    assert_eq!(usdt_to_wbtc.path.len(), 2);       // 2-hop path
    
    // The best path for USDC -> DAI could be either direct or through WETH
    // depending on the reserves and fees
    assert!(usdc_to_dai_multi.path.len() >= 1 && usdc_to_dai_multi.path.len() <= 2);
    
    // USDT -> DAI should be a 2-hop path through WETH
    assert_eq!(usdt_to_dai.path.len(), 2);
}

// This test requires actual Tycho API access, so it's ignored by default
// To run it, use: cargo test -- --ignored
#[tokio::test]
#[ignore]
async fn test_end_to_end_with_tycho() {
    let api_key = env::var("TYCHO_API_KEY").ok();
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    // 1. Initialize the price quoter with data from Tycho
    let quoter = PriceQuoter::from_tycho(&url, api_key).await.unwrap();
    
    println!("Loaded {} tokens and {} pools from Tycho", 
             quoter.get_storage().token_count(),
             quoter.get_storage().pool_count());
    
    // 2. Define some well-known token addresses
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    
    // 3. Get a quote for USDC -> WETH
    let usdc_to_weth = quoter.get_quote(
        usdc_addr,
        weth_addr,
        "1000000", // 1 USDC
        1
    );
    
    if let Ok(quote) = usdc_to_weth {
        println!("USDC -> WETH quote:");
        println!("  Amount in: {} USDC", quote.amount_in);
        println!("  Amount out: {} WETH", quote.amount_out);
        println!("  Path: {:?}", quote.path);
    } else {
        println!("Failed to get USDC -> WETH quote: {:?}", usdc_to_weth.err());
    }
    
    // 4. Get a quote for USDC -> DAI
    let usdc_to_dai = quoter.get_quote(
        usdc_addr,
        dai_addr,
        "1000000", // 1 USDC
        2
    );
    
    if let Ok(quote) = usdc_to_dai {
        println!("USDC -> DAI quote:");
        println!("  Amount in: {} USDC", quote.amount_in);
        println!("  Amount out: {} DAI", quote.amount_out);
        println!("  Path: {:?}", quote.path);
    } else {
        println!("Failed to get USDC -> DAI quote: {:?}", usdc_to_dai.err());
    }
}
''')

print("Created comprehensive test files for the Tycho Price Quoter library")
