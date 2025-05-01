use crate::graph::TokenGraph;
use crate::pathfinder::find_best_path;
use crate::storage::PoolStorage;
use crate::types::{Address, SwapPath, Token};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuoterError {
    #[error("Token not found: {0}")]
    TokenNotFound(String),
    
    #[error("No path found")]
    NoPathFound,
    
    #[error("Storage error: {0}")]
    StorageError(#[from] crate::storage::StorageError),
    
    #[error("Tycho error: {0}")]
    TychoError(#[from] crate::tycho::TychoError),
}

#[derive(Debug, Clone)]
pub struct PriceQuote {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: String,
    pub amount_out: String,
    pub path: Vec<String>,
}

pub struct PriceQuoter {
    storage: PoolStorage,
    graph: TokenGraph,
}

impl PriceQuoter {
    pub fn new(storage: PoolStorage) -> Self {
        let graph = TokenGraph::from_storage(&storage);
        Self { storage, graph }
    }
    
    pub async fn from_tycho(
        url: &str, 
        api_key: Option<String>
    ) -> Result<Self, QuoterError> {
        let storage = PoolStorage::load_from_tycho(url, api_key).await?;
        Ok(Self::new(storage))
    }
    
    pub fn get_quote(
        &self,
        token_in: &str,
        token_out: &str,
        amount_in: &str,
        max_hops: usize,
    ) -> Result<PriceQuote, QuoterError> {
        // Verify tokens exist
        let token_in = self
            .storage
            .get_token(token_in)
            .ok_or_else(|| QuoterError::TokenNotFound(token_in.to_string()))?;
        
        let token_out = self
            .storage
            .get_token(token_out)
            .ok_or_else(|| QuoterError::TokenNotFound(token_out.to_string()))?;
        
        // Find the best path
        let path = find_best_path(
            &self.graph,
            &self.storage,
            &token_in.address,
            &token_out.address,
            amount_in,
            max_hops,
        ).ok_or(QuoterError::NoPathFound)?;
        
        // Calculate amount out (simplified for now)
        let amount_out = self.calculate_amount_out(&path, amount_in)?;
        
        // Create price quote
        let quote = PriceQuote {
            token_in: token_in.address.clone(),
            token_out: token_out.address.clone(),
            amount_in: amount_in.to_string(),
            amount_out,
            path: path.hops.iter().map(|hop| hop.pool_id.clone()).collect(),
        };
        
        Ok(quote)
    }
    
    fn calculate_amount_out(&self, path: &SwapPath, amount_in: &str) -> Result<String, QuoterError> {
        // Simplified calculation - in a real implementation, this would use the actual pool math
        // For now, just return a dummy value
        Ok("1000000000000000000".to_string())
    }
    
    pub fn get_storage(&self) -> &PoolStorage {
        &self.storage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Pool, U256};
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
    fn test_get_quote() {
        let (tokens, pools) = create_test_data();
        let storage = crate::storage::PoolStorage::with_initial_data(pools, tokens);
        let quoter = PriceQuoter::new(storage);
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
        
        // Get quote for USDC -> DAI
        let quote = quoter.get_quote(usdc_addr, dai_addr, "1000000", 2);
        
        // Should succeed
        assert!(quote.is_ok());
        
        let quote = quote.unwrap();
        assert_eq!(quote.token_in, usdc_addr);
        assert_eq!(quote.token_out, dai_addr);
        assert_eq!(quote.amount_in, "1000000");
        assert_eq!(quote.path.len(), 2); // USDC -> WETH -> DAI
    }
}
