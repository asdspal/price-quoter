// tests/test_price_quoter.rs

use tycho_price_quoter::{Token, Pool, PriceQuoter};
use std::collections::HashMap;

fn create_test_data() -> (HashMap<String, Token>, Vec<Pool>) {
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
fn test_price_quoter_initialization() {
    let (tokens, pools) = create_test_data();
    let quoter = PriceQuoter::with_initial_data(pools, tokens, None);
    
    assert_eq!(quoter.token_count(), 4);
    assert_eq!(quoter.pool_count(), 3);
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

#[test]
fn test_update_pool() {
    let (tokens, pools) = create_test_data();
    let mut quoter = PriceQuoter::with_initial_data(pools, tokens, Some(2));
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    
    // Update pool reserves
    let new_reserve0 = 6_000_000 * 10u128.pow(6); // 6M USDC
    let new_reserve1 = 2_500 * 10u128.pow(18);    // 2.5k WETH
    
    quoter.update_pool("0xPoolUsdcWeth", new_reserve0, new_reserve1).unwrap();
    
    // Swap 1000 USDC for WETH
    let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
    
    let result = quoter.find_quote(&usdc_addr, &weth_addr, amount_in).unwrap();
    
    assert!(result.is_some());
    let (_, amount_out) = result.unwrap();
    
    // Rough check: 1000 USDC should get approximately 0.417 WETH
    // (based on the updated pool reserves: 6M USDC <-> 2.5k WETH)
    // 1000 / 6M * 2.5k = 0.417 WETH
    let expected_approx = 0.417 * 10f64.powi(18) as u128;
    let tolerance = 0.05; // 5% tolerance for fees and slippage
    
    let ratio = amount_out as f64 / expected_approx as f64;
    assert!(ratio > (1.0 - tolerance) && ratio < (1.0 + tolerance));
}

#[test]
fn test_remove_pool() {
    let (tokens, pools) = create_test_data();
    let mut quoter = PriceQuoter::with_initial_data(pools, tokens, Some(2));
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    
    // Remove the WETH-DAI pool
    quoter.remove_pool("0xPoolWethDai").unwrap();
    
    // Try to find a path from USDC to DAI
    let paths = quoter.find_paths(&usdc_addr, &dai_addr);
    
    // Should find no paths
    assert_eq!(paths.len(), 0);
}
