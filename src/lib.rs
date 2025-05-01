// src/lib.rs

// Declare modules
pub mod types;
pub mod storage;
pub mod graph;
pub mod pathfinder;
pub mod quoting;
pub mod indexer;

// Re-export public types for easier access by library users
pub use types::{Token, Pool, SwapHop, SwapPath, Address, U256, Bytes};
pub use storage::PoolStorage;
pub use graph::TokenGraph;
pub use indexer::{IndexerClient, IndexerError};

/// The main struct for interacting with the Price Quoter.
/// It holds the pool data and provides methods for finding quotes.
pub struct PriceQuoter {
    // Internal storage for pool and token data
    storage: PoolStorage,
    // Graph representation for efficient pathfinding
    graph: TokenGraph,
    // Maximum number of hops to consider when finding paths
    max_hops: usize,
    // Optional Indexer client for fetching real-time data
    indexer_client: Option<IndexerClient>,
}

impl PriceQuoter {
    /// Creates a new PriceQuoter instance with empty storage.
    pub fn new() -> Self {
        let storage = PoolStorage::new();
        let graph = TokenGraph::new();
        
        Self {
            storage,
            graph,
            max_hops: 2, // Default to 2 hops max
            indexer_client: None,
        }
    }

    /// Creates a new PriceQuoter with initial data.
    pub fn with_initial_data(
        pools: Vec<Pool>, 
        tokens: std::collections::HashMap<Address, Token>,
        max_hops: Option<usize>,
    ) -> Self {
        let storage = PoolStorage::with_initial_data(pools.clone(), tokens);
        let mut graph = TokenGraph::new();
        
        // Add all pools to the graph
        for pool in &pools {
            graph.add_pool(pool);
        }
        
        Self {
            storage,
            graph,
            max_hops: max_hops.unwrap_or(2), // Default to 2 hops max
            indexer_client: None,
        }
    }

    /// Sets the Indexer client for fetching real-time data.
    pub fn with_indexer(mut self, base_url: String, api_key: Option<String>) -> Self {
        self.indexer_client = Some(IndexerClient::new(base_url, api_key));
        self
    }

    /// Sets the maximum number of hops to consider when finding paths.
    pub fn set_max_hops(&mut self, max_hops: usize) {
        self.max_hops = max_hops;
    }

    /// Adds a token to the storage.
    pub fn add_token(&mut self, token: Token) {
        self.storage.add_token(token);
    }

    /// Adds a pool to the storage and graph.
    pub fn add_pool(&mut self, pool: Pool) {
        self.storage.add_pool(pool.clone());
        self.graph.add_pool(&pool);
    }

    /// Updates an existing pool's reserves.
    pub fn update_pool(&mut self, pool_id: &str, reserve0: U256, reserve1: U256) -> Result<(), String> {
        // Get the old pool
        let old_pool = match self.storage.get_pool(pool_id) {
            Some(p) => p.clone(),
            None => return Err(format!("Pool with ID {} not found", pool_id)),
        };
        
        // Update the pool in storage
        self.storage.update_pool(pool_id, reserve0, reserve1)?;
        
        // Get the updated pool
        let new_pool = match self.storage.get_pool(pool_id) {
            Some(p) => p.clone(),
            None => return Err(format!("Pool with ID {} not found after update", pool_id)),
        };
        
        // Update the pool in the graph
        self.graph.update_pool(&old_pool, &new_pool);
        
        Ok(())
    }

    /// Removes a pool from storage and graph.
    pub fn remove_pool(&mut self, pool_id: &str) -> Result<(), String> {
        // Get the pool before removing it
        let pool = match self.storage.get_pool(pool_id) {
            Some(p) => p.clone(),
            None => return Err(format!("Pool with ID {} not found", pool_id)),
        };
        
        // Remove from storage
        self.storage.remove_pool(pool_id)?;
        
        // Remove from graph
        self.graph.remove_pool(&pool);
        
        Ok(())
    }

    /// Gets the number of pools in storage.
    pub fn pool_count(&self) -> usize {
        self.storage.pool_count()
    }

    /// Gets the number of tokens in storage.
    pub fn token_count(&self) -> usize {
        self.storage.token_count()
    }

    /// Fetches the latest pool data from the Indexer for the specified protocols.
    pub async fn fetch_latest_data(&mut self, protocol_ids: Vec<String>) -> Result<(), IndexerError> {
        if let Some(client) = &self.indexer_client {
            let (pools, tokens) = client.fetch_and_convert_all(protocol_ids).await?;
            
            // Update storage and graph with new data
            for token in tokens.values() {
                self.storage.add_token(token.clone());
            }
            
            for pool in pools {
                self.add_pool(pool);
            }
            
            Ok(())
        } else {
            Err(IndexerError::ApiError("Indexer client not configured".to_string()))
        }
    }

    /// Finds all possible swap paths between two tokens.
    pub fn find_paths(
        &self,
        token_in: &Address,
        token_out: &Address,
    ) -> Vec<SwapPath> {
        pathfinder::find_paths(&self.storage, token_in, token_out, self.max_hops)
    }

    /// Calculates the expected output amount for a specific path.
    pub fn calculate_output_amount(
        &self,
        path: &SwapPath,
        amount_in: U256,
    ) -> Result<U256, String> {
        quoting::calculate_output_amount(&self.storage, path, amount_in)
    }

    /// Finds the best quote for swapping `amount_in` of `token_in` to `token_out`.
    pub fn find_quote(
        &self,
        token_in: &Address,
        token_out: &Address,
        amount_in: U256,
    ) -> Result<Option<(SwapPath, U256)>, String> {
        // Find all possible paths
        let paths = self.find_paths(token_in, token_out);
        
        if paths.is_empty() {
            return Ok(None);
        }
        
        // Find the best quote among all paths
        quoting::find_best_quote(&self.storage, &paths, amount_in)
    }

    /// Finds the best quote after fetching the latest data from the Indexer.
    pub async fn find_quote_with_latest_data(
        &mut self,
        token_in: &Address,
        token_out: &Address,
        amount_in: U256,
        protocol_ids: Vec<String>,
    ) -> Result<Option<(SwapPath, U256)>, String> {
        // Fetch latest data
        self.fetch_latest_data(protocol_ids).await
            .map_err(|e| format!("Failed to fetch latest data: {}", e))?;
        
        // Find the best quote
        self.find_quote(token_in, token_out, amount_in)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_price_quoter_initialization() {
        let (tokens, pools) = create_test_data();
        let quoter = PriceQuoter::with_initial_data(pools, tokens, None);
        
        assert_eq!(quoter.token_count(), 3);
        assert_eq!(quoter.pool_count(), 2);
    }

    #[test]
    fn test_find_paths() {
        let (tokens, pools) = create_test_data();
        let quoter = PriceQuoter::with_initial_data(pools, tokens, Some(2));
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
        
        let paths = quoter.find_paths(&usdc_addr, &dai_addr);
        
        // Should find 1 path: USDC -> WETH -> DAI
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].hops.len(), 2);
    }

    #[test]
    fn test_find_quote() {
        let (tokens, pools) = create_test_data();
        let quoter = PriceQuoter::with_initial_data(pools, tokens, Some(2));
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
        
        // Swap 1000 USDC for DAI
        let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
        
        let result = quoter.find_quote(&usdc_addr, &dai_addr, amount_in).unwrap();
        
        assert!(result.is_some());
        let (path, amount_out) = result.unwrap();
        
        // Should be a 2-hop path: USDC -> WETH -> DAI
        assert_eq!(path.hops.len(), 2);
        assert_eq!(path.token_in, usdc_addr);
        assert_eq!(path.token_out, dai_addr);
        
        // Output amount should be positive
        assert!(amount_out > 0);
    }
}
