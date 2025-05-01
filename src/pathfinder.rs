// src/pathfinder.rs

use std::collections::{HashMap, HashSet, VecDeque};
use crate::types::{Address, Pool, SwapHop, SwapPath};
use crate::storage::PoolStorage;
use crate::graph::TokenGraph;

/// Finds all possible swap paths between two tokens using BFS.
/// 
/// # Arguments
/// * `storage` - The pool storage containing all pools and tokens
/// * `token_in` - The address of the input token
/// * `token_out` - The address of the output token
/// * `max_hops` - The maximum number of hops (pools) to traverse
/// 
/// # Returns
/// A vector of possible swap paths, each containing a sequence of hops
pub fn find_paths(
    storage: &PoolStorage,
    token_in: &Address,
    token_out: &Address,
    max_hops: usize,
) -> Vec<SwapPath> {
    // Early return if tokens are the same
    if token_in == token_out {
        return Vec::new();
    }

    // Check if both tokens exist in our storage
    if storage.get_token(token_in).is_none() || storage.get_token(token_out).is_none() {
        return Vec::new();
    }

    // Create a graph from the storage
    let graph = TokenGraph::from_storage(storage);
    
    // Check if both tokens exist in the graph
    if !graph.contains_token(token_in) || !graph.contains_token(token_out) {
        return Vec::new();
    }

    // Direct pool check (optimization)
    if storage.has_direct_pool(token_in, token_out) && max_hops >= 1 {
        // Find the direct pool(s)
        let direct_pools = find_direct_pools(storage, token_in, token_out);
        
        // Convert to paths
        let mut paths = Vec::new();
        for pool in direct_pools {
            let hop = SwapHop {
                pool_id: pool.id.clone(),
                token_in: token_in.clone(),
                token_out: token_out.clone(),
            };
            
            paths.push(SwapPath {
                hops: vec![hop],
                token_in: token_in.clone(),
                token_out: token_out.clone(),
            });
        }
        
        // If max_hops is 1, return only direct paths
        if max_hops == 1 {
            return paths;
        }
    }

    // BFS to find all paths
    let mut paths = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    
    // Start with the input token
    // Each entry in the queue is (current_token, path_so_far, visited_tokens)
    queue.push_back((
        token_in.clone(),
        Vec::new(),
        HashSet::from([token_in.clone()]),
    ));
    
    while let Some((current_token, path_so_far, visited_tokens)) = queue.pop_front() {
        // Skip if we've reached max hops
        if path_so_far.len() >= max_hops {
            continue;
        }
        
        // Get all outgoing edges from the current token
        let edges = graph.get_edges(&current_token);
        
        for edge in edges {
            // Skip if we've already visited this token in this path
            if visited_tokens.contains(&edge.to) {
                continue;
            }
            
            // Get the pool for this edge
            let pool = match storage.get_pool(&edge.pool_id) {
                Some(p) => p,
                None => continue, // Skip if pool not found
            };
            
            // Create a new hop
            let hop = SwapHop {
                pool_id: pool.id.clone(),
                token_in: current_token.clone(),
                token_out: edge.to.clone(),
            };
            
            // Create a new path by extending the current path
            let mut new_path = path_so_far.clone();
            new_path.push(hop);
            
            // If we've reached the target token, add the path to results
            if edge.to == *token_out {
                paths.push(SwapPath {
                    hops: new_path,
                    token_in: token_in.clone(),
                    token_out: token_out.clone(),
                });
                continue; // No need to explore further from the target
            }
            
            // Otherwise, add the new state to the queue for further exploration
            let mut new_visited = visited_tokens.clone();
            new_visited.insert(edge.to.clone());
            
            // Create a unique key for this path to avoid duplicates
            let path_key = format!("{}-{}-{}", token_in, edge.to, new_path.len());
            if !visited.contains(&path_key) {
                visited.insert(path_key);
                queue.push_back((edge.to, new_path, new_visited));
            }
        }
    }
    
    paths
}

/// Finds direct pools between two tokens.
fn find_direct_pools<'a>(
    storage: &'a PoolStorage,
    token0: &Address,
    token1: &Address,
) -> Vec<&'a Pool> {
    let pools0 = storage.get_pools_for_token(token0);
    
    pools0.into_iter()
        .filter(|pool| {
            (pool.token0 == *token0 && pool.token1 == *token1) ||
            (pool.token0 == *token1 && pool.token1 == *token0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Token, U256};
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
            reserve0: 1_500 * 10u128.pow(18),     // 1.5k WETH
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
    fn test_find_direct_paths() {
        let (tokens, pools) = create_test_data();
        let storage = PoolStorage::with_initial_data(pools, tokens);
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
        let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
        
        // Find direct paths with max_hops = 1
        let paths = find_paths(&storage, &usdc_addr, &weth_addr, 1);
        
        // Should find 1 direct path
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].hops.len(), 1);
        assert_eq!(paths[0].hops[0].token_in, usdc_addr);
        assert_eq!(paths[0].hops[0].token_out, weth_addr);
    }

    #[test]
    fn test_find_multi_hop_paths() {
        let (tokens, pools) = create_test_data();
        let storage = PoolStorage::with_initial_data(pools, tokens);
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
        
        // Find paths with max_hops = 2
        let paths = find_paths(&storage, &usdc_addr, &dai_addr, 2);
        
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
    fn test_multiple_paths() {
        let (tokens, pools) = create_test_data();
        let storage = PoolStorage::with_initial_data(pools, tokens);
        
        let usdt_addr = "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string();
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
        
        // Find paths with max_hops = 2
        let paths = find_paths(&storage, &usdt_addr, &dai_addr, 2);
        
        // Should find 1 path: USDT -> WETH -> DAI
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].hops.len(), 2);
    }
}
