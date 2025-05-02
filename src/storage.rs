// src/storage.rs
use crate::types::{Pool, Token, U256};
use crate::tycho::TychoConnector;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    
    #[error("Pool not found: {0}")]
    PoolNotFound(String),
    
    #[error("Tycho error: {0}")]
    TychoError(#[from] crate::tycho::TychoError),
}

pub struct PoolStorage {
    tokens: HashMap<String, Token>,
    pools: HashMap<String, Pool>,
    token_to_pools: HashMap<String, Vec<String>>,
}

impl PoolStorage {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
            pools: HashMap::new(),
            token_to_pools: HashMap::new(),
        }
    }
    
    pub fn with_initial_data(pools: Vec<Pool>, tokens: HashMap<String, Token>) -> Self {
        let mut storage = Self {
            pools: HashMap::new(),
            tokens,
            token_to_pools: HashMap::new(),
        };
        
        for pool in pools {
            storage.add_pool(pool);
        }
        
        storage
    }
    
	pub async fn load_from_tycho(
		url: &str, 
		api_key: Option<String>
	) -> Result<Self, StorageError> {
		let connector = TychoConnector::new(url, api_key).await?;
		
		// Fetch pools and tokens
		let (pools, tokens) = connector.fetch_pools().await?;
		
		// Create a new storage instance with the fetched data
		let mut storage = Self::with_initial_data(pools, tokens);
		
		Ok(storage)
	}
    
    pub fn add_token(&mut self, token: Token) {
        self.tokens.insert(token.address.clone(), token);
    }
    
    pub fn add_pool(&mut self, pool: Pool) {
        // Add pool to token_to_pools mapping
        self.token_to_pools
            .entry(pool.token0.clone())
            .or_insert_with(Vec::new)
            .push(pool.id.clone());
            
        self.token_to_pools
            .entry(pool.token1.clone())
            .or_insert_with(Vec::new)
            .push(pool.id.clone());
            
        // Add pool to pools map
        self.pools.insert(pool.id.clone(), pool);
    }
    
    pub fn update_pool(
        &mut self, 
        pool_id: &str, 
        reserve0: U256, 
        reserve1: U256
    ) -> Result<(), String> {
        let pool = self.pools.get_mut(pool_id)
            .ok_or_else(|| format!("Pool with ID {} not found", pool_id))?;
            
        pool.reserve0 = reserve0;
        pool.reserve1 = reserve1;
        
        Ok(())
    }
    
    pub fn remove_pool(&mut self, pool_id: &str) -> Result<(), String> {
        let pool = self.pools.get(pool_id)
            .ok_or_else(|| format!("Pool with ID {} not found", pool_id))?;
            
        // Remove from token_to_pools
        if let Some(pools) = self.token_to_pools.get_mut(&pool.token0) {
            pools.retain(|id| id != pool_id);
            if pools.is_empty() {
                self.token_to_pools.remove(&pool.token0);
            }
        }
        
        if let Some(pools) = self.token_to_pools.get_mut(&pool.token1) {
            pools.retain(|id| id != pool_id);
            if pools.is_empty() {
                self.token_to_pools.remove(&pool.token1);
            }
        }
        
        // Remove from pools
        self.pools.remove(pool_id);
        
        Ok(())
    }
    
    pub fn get_token(&self, address: &str) -> Option<&Token> {
        self.tokens.get(address)
    }
    
    pub fn get_pool(&self, pool_id: &str) -> Option<&Pool> {
        self.pools.get(pool_id)
    }
    
    pub fn get_pools_for_token(&self, token_address: &str) -> Vec<&Pool> {
        match self.token_to_pools.get(token_address) {
            Some(pool_ids) => {
                pool_ids.iter()
                    .filter_map(|id| self.pools.get(id))
                    .collect()
            },
            None => Vec::new(),
        }
    }
    
    pub fn has_direct_pool(&self, token0: &str, token1: &str) -> bool {
        if let Some(pool_ids) = self.token_to_pools.get(token0) {
            for pool_id in pool_ids {
                if let Some(pool) = self.pools.get(pool_id) {
                    if (pool.token0 == token0 && pool.token1 == token1) ||
                       (pool.token0 == token1 && pool.token1 == token0) {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    pub fn get_all_tokens(&self) -> Vec<&Token> {
        self.tokens.values().collect()
    }
    
    pub fn get_all_pools(&self) -> Vec<&Pool> {
        self.pools.values().collect()
    }
    
    pub fn pool_count(&self) -> usize {
        self.pools.len()
    }
    
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }
}
