// src/storage.rs

use std::collections::{HashMap, HashSet};
use crate::types::{Address, Pool, Token};

/// Manages the storage and retrieval of pool data.
/// This is an internal component used by the PriceQuoter.
pub struct PoolStorage {
    // Map pool ID to Pool data
    pools: HashMap<String, Pool>,
    // Map token address to a set of pool IDs containing that token
    token_to_pools: HashMap<Address, HashSet<String>>,
    // Map token address to Token data
    tokens: HashMap<Address, Token>,
}

impl PoolStorage {
    /// Creates a new empty PoolStorage.
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            token_to_pools: HashMap::new(),
            tokens: HashMap::new(),
        }
    }

    /// Creates a PoolStorage with initial data.
    pub fn with_initial_data(pools: Vec<Pool>, tokens: HashMap<Address, Token>) -> Self {
        let mut storage = Self::new();
        
        // Add tokens
        for (addr, token) in tokens {
            storage.add_token(token);
        }
        
        // Add pools
        for pool in pools {
            storage.add_pool(pool);
        }
        
        storage
    }

    /// Adds a token to the storage.
    pub fn add_token(&mut self, token: Token) {
        self.tokens.insert(token.address.clone(), token);
    }

    /// Adds a pool to the storage and updates the token-to-pools mapping.
    pub fn add_pool(&mut self, pool: Pool) {
        // Add to token_to_pools mapping for both tokens
        self.token_to_pools
            .entry(pool.token0.clone())
            .or_insert_with(HashSet::new)
            .insert(pool.id.clone());
        
        self.token_to_pools
            .entry(pool.token1.clone())
            .or_insert_with(HashSet::new)
            .insert(pool.id.clone());
        
        // Add to pools map
        self.pools.insert(pool.id.clone(), pool);
    }

    /// Updates an existing pool's data (e.g., reserves).
    pub fn update_pool(&mut self, pool_id: &str, reserve0: u128, reserve1: u128) -> Result<(), String> {
        if let Some(pool) = self.pools.get_mut(pool_id) {
            pool.reserve0 = reserve0;
            pool.reserve1 = reserve1;
            Ok(())
        } else {
            Err(format!("Pool with ID {} not found", pool_id))
        }
    }

    /// Removes a pool from storage and updates the token-to-pools mapping.
    pub fn remove_pool(&mut self, pool_id: &str) -> Result<(), String> {
        if let Some(pool) = self.pools.remove(pool_id) {
            // Remove from token_to_pools mapping
            if let Some(pools) = self.token_to_pools.get_mut(&pool.token0) {
                pools.remove(pool_id);
                if pools.is_empty() {
                    self.token_to_pools.remove(&pool.token0);
                }
            }
            
            if let Some(pools) = self.token_to_pools.get_mut(&pool.token1) {
                pools.remove(pool_id);
                if pools.is_empty() {
                    self.token_to_pools.remove(&pool.token1);
                }
            }
            
            Ok(())
        } else {
            Err(format!("Pool with ID {} not found", pool_id))
        }
    }

    /// Gets a pool by its ID.
    pub fn get_pool(&self, pool_id: &str) -> Option<&Pool> {
        self.pools.get(pool_id)
    }

    /// Gets a token by its address.
    pub fn get_token(&self, address: &Address) -> Option<&Token> {
        self.tokens.get(address)
    }

    /// Gets all pools containing a specific token.
    pub fn get_pools_for_token(&self, token_address: &Address) -> Vec<&Pool> {
        match self.token_to_pools.get(token_address) {
            Some(pool_ids) => pool_ids
                .iter()
                .filter_map(|id| self.pools.get(id))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Gets all pools.
    pub fn get_all_pools(&self) -> Vec<&Pool> {
        self.pools.values().collect()
    }

    /// Gets all tokens.
    pub fn get_all_tokens(&self) -> Vec<&Token> {
        self.tokens.values().collect()
    }

    /// Checks if a direct pool exists between two tokens.
    pub fn has_direct_pool(&self, token0: &Address, token1: &Address) -> bool {
        // Get pools for token0
        if let Some(pool_ids) = self.token_to_pools.get(token0) {
            // Check if any of these pools also contain token1
            for pool_id in pool_ids {
                if let Some(pool) = self.pools.get(pool_id) {
                    if pool.token0 == *token1 || pool.token1 == *token1 {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Gets the number of pools in storage.
    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }

    /// Gets the number of tokens in storage.
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::U256;

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
    fn test_pool_storage() {
        let (tokens, pools) = create_test_data();
        let storage = PoolStorage::with_initial_data(pools, tokens.clone());
        
        // Test token count
        assert_eq!(storage.token_count(), 3);
        
        // Test pool count
        assert_eq!(storage.pool_count(), 2);
        
        // Test get_pools_for_token
        let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
        let weth_pools = storage.get_pools_for_token(&weth_addr);
        assert_eq!(weth_pools.len(), 2); // WETH is in both pools
        
        // Test has_direct_pool
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
        assert!(storage.has_direct_pool(&usdc_addr, &weth_addr));
        assert!(storage.has_direct_pool(&weth_addr, &dai_addr));
        assert!(!storage.has_direct_pool(&usdc_addr, &dai_addr)); // No direct pool
    }
}
