// src/graph.rs

use std::collections::{HashMap, HashSet};
use crate::types::{Address, Pool};
use crate::storage::PoolStorage;

/// Represents a directed edge in the token graph.
#[derive(Debug, Clone)]
pub struct Edge {
    /// The ID of the pool this edge represents
    pub pool_id: String,
    /// The destination token address
    pub to: Address,
}

/// A graph representation of the token pools for efficient pathfinding.
/// Each token address is a node, and each pool creates two directed edges
/// (one in each direction).
#[derive(Debug, Clone)]
pub struct TokenGraph {
    /// Maps token addresses to their outgoing edges
    adjacency_list: HashMap<Address, Vec<Edge>>,
    /// Set of all token addresses in the graph
    tokens: HashSet<Address>,
}

impl TokenGraph {
    /// Creates a new empty TokenGraph.
    pub fn new() -> Self {
        Self {
            adjacency_list: HashMap::new(),
            tokens: HashSet::new(),
        }
    }

    /// Creates a TokenGraph from the pools in storage.
    pub fn from_storage(storage: &PoolStorage) -> Self {
        let mut graph = Self::new();
        
        // Add all pools to the graph
        for pool in storage.get_all_pools() {
            graph.add_pool(pool);
        }
        
        graph
    }

    /// Adds a pool to the graph, creating edges in both directions.
    pub fn add_pool(&mut self, pool: &Pool) {
        // Add tokens to the set of all tokens
        self.tokens.insert(pool.token0.clone());
        self.tokens.insert(pool.token1.clone());
        
        // Create edges in both directions
        
        // Edge from token0 to token1
        let edge0_to_1 = Edge {
            pool_id: pool.id.clone(),
            to: pool.token1.clone(),
        };
        
        // Edge from token1 to token0
        let edge1_to_0 = Edge {
            pool_id: pool.id.clone(),
            to: pool.token0.clone(),
        };
        
        // Add edges to adjacency list
        self.adjacency_list
            .entry(pool.token0.clone())
            .or_insert_with(Vec::new)
            .push(edge0_to_1);
        
        self.adjacency_list
            .entry(pool.token1.clone())
            .or_insert_with(Vec::new)
            .push(edge1_to_0);
    }

    /// Removes a pool from the graph.
    pub fn remove_pool(&mut self, pool: &Pool) {
        // Remove edges from token0 to token1
        if let Some(edges) = self.adjacency_list.get_mut(&pool.token0) {
            edges.retain(|edge| edge.pool_id != pool.id);
            
            if edges.is_empty() {
                self.adjacency_list.remove(&pool.token0);
                self.tokens.remove(&pool.token0);
            }
            
        }
        
        // Remove edges from token1 to token0
        if let Some(edges) = self.adjacency_list.get_mut(&pool.token1) {
            edges.retain(|edge| edge.pool_id != pool.id);
            
            if edges.is_empty() {
                self.adjacency_list.remove(&pool.token1);
                self.tokens.remove(&pool.token1);
            }
            
        }
    }

    /// Updates the graph with a new pool, replacing any existing pool with the same ID.
    pub fn update_pool(&mut self, old_pool: &Pool, new_pool: &Pool) {
        // If the pool ID is the same but the tokens have changed, we need to remove the old pool
        // and add the new one. Otherwise, we can just update the reserves.
        if old_pool.id == new_pool.id && 
           old_pool.token0 == new_pool.token0 && 
           old_pool.token1 == new_pool.token1 {
            // Same tokens, just update reserves (nothing to do for the graph)
            return;
        }
        
        // Different tokens, remove old pool and add new one
        self.remove_pool(old_pool);
        self.add_pool(new_pool);
    }

    /// Gets all outgoing edges for a token.
    pub fn get_edges(&self, token: &Address) -> Vec<Edge> {
        self.adjacency_list.get(token).cloned().unwrap_or_default()
    }

    /// Checks if the graph contains a token.
    pub fn contains_token(&self, token: &Address) -> bool {
        self.tokens.contains(token)
    }

    /// Gets the number of tokens in the graph.
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Gets the number of edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.adjacency_list.values().map(|edges| edges.len()).sum()
    }

    /// Gets all tokens in the graph.
    pub fn get_all_tokens(&self) -> HashSet<Address> {
        self.tokens.clone()
    }
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
    fn test_graph_from_storage() {
        let (tokens, pools) = create_test_data();
        let storage = crate::storage::PoolStorage::with_initial_data(pools, tokens);
        
        let graph = TokenGraph::from_storage(&storage);
        
        // Check token count
        assert_eq!(graph.token_count(), 3);
        
        // Check edge count (2 pools * 2 directions = 4 edges)
        assert_eq!(graph.edge_count(), 4);
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
		
		// Check token count (USDC should be removed, WETH and DAI remain)
		assert_eq!(graph.token_count(), 2);
		
		// Check edge count (1 pool * 2 directions = 2 edges)
		assert_eq!(graph.edge_count(), 2);
		
		// Check that WETH only has an edge to DAI now
		let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
		let weth_edges = graph.get_edges(&weth_addr);
		assert_eq!(weth_edges.len(), 1);
		assert_eq!(weth_edges[0].to, "0x6B175474E89094C44Da98b954EedeAC495271d0F");
	}
}
