// tests/test_storage.rs

use tycho_price_quoter::{Token, Pool, PoolStorage};
use std::collections::HashMap;

fn create_test_data() -> (HashMap<String, Token>, Vec<Pool>) {
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
fn test_storage_initialization() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    assert_eq!(storage.token_count(), 3);
    assert_eq!(storage.pool_count(), 2);
}

#[test]
fn test_get_token() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let token = storage.get_token(&usdc_addr).unwrap();
    
    assert_eq!(token.symbol, "USDC");
    assert_eq!(token.decimals, 6);
}

#[test]
fn test_get_pool() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let pool = storage.get_pool("0xPoolUsdcWeth").unwrap();
    
    assert_eq!(pool.token0, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    assert_eq!(pool.token1, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
}

#[test]
fn test_get_pools_for_token() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let weth_pools = storage.get_pools_for_token(&weth_addr);
    
    assert_eq!(weth_pools.len(), 2);
}

#[test]
fn test_has_direct_pool() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    
    assert!(storage.has_direct_pool(&usdc_addr, &weth_addr));
    assert!(storage.has_direct_pool(&weth_addr, &dai_addr));
    assert!(!storage.has_direct_pool(&usdc_addr, &dai_addr));
}

#[test]
fn test_update_pool() {
    let (tokens, pools) = create_test_data();
    let mut storage = PoolStorage::with_initial_data(pools, tokens);
    
    // Update pool reserves
    let new_reserve0 = 6_000_000 * 10u128.pow(6); // 6M USDC
    let new_reserve1 = 2_500 * 10u128.pow(18);    // 2.5k WETH
    
    storage.update_pool("0xPoolUsdcWeth", new_reserve0, new_reserve1).unwrap();
    
    let updated_pool = storage.get_pool("0xPoolUsdcWeth").unwrap();
    assert_eq!(updated_pool.reserve0, new_reserve0);
    assert_eq!(updated_pool.reserve1, new_reserve1);
}

#[test]
fn test_remove_pool() {
    let (tokens, pools) = create_test_data();
    let mut storage = PoolStorage::with_initial_data(pools, tokens);
    
    assert_eq!(storage.pool_count(), 2);
    
    storage.remove_pool("0xPoolUsdcWeth").unwrap();
    
    assert_eq!(storage.pool_count(), 1);
    assert!(storage.get_pool("0xPoolUsdcWeth").is_none());
}
