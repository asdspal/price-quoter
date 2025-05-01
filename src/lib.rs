// src/lib.rs
pub mod graph;
pub mod pathfinder;
pub mod quoting;
pub mod storage;
pub mod types;
pub mod tycho;

pub use quoting::PriceQuoter;
pub use quoting::QuoterError;
pub use storage::PoolStorage;
pub use types::{SwapPath, Pool, Token, U256, Address};

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
            reserve0: 1_500 * 10u128.pow(18),    // 1.5k WETH
            reserve1: 4_500_000 * 10u128.pow(18), // 4.5M DAI
            fee: 30,
        };

        (tokens, vec![pool1, pool2])
    }

    #[test]
    fn test_price_quoter_initialization() {
        let (tokens, pools) = create_test_data();
        let quoter = PriceQuoter::with_storage(PoolStorage::with_initial_data(pools, tokens));
        
        assert_eq!(quoter.get_storage().token_count(), 3);
        assert_eq!(quoter.get_storage().pool_count(), 2);
    }

    #[test]
    fn test_find_paths() {
        let (tokens, pools) = create_test_data();
        let quoter = PriceQuoter::with_storage(PoolStorage::with_initial_data(pools, tokens));
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
        
        let paths = quoter.find_paths(usdc_addr, dai_addr);
        
        // Should find 1 path: USDC -> WETH -> DAI
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].hops.len(), 2);
    }

    #[test]
    fn test_find_quote() {
        let (tokens, pools) = create_test_data();
        let quoter = PriceQuoter::with_storage(PoolStorage::with_initial_data(pools, tokens));
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
        
        // Swap 1000 USDC for DAI
        let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
        
        let result = quoter.get_quote(usdc_addr, dai_addr, amount_in);
        
        assert!(result.is_ok());
        let (path, amount_out) = result.unwrap();
        
        // Should be a 2-hop path: USDC -> WETH -> DAI
        assert_eq!(path.hops.len(), 2);
        assert_eq!(path.token_in, usdc_addr);
        assert_eq!(path.token_out, dai_addr);
        
        // Output amount should be positive
        assert!(amount_out > 0);
    }
}
