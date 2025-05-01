// tests/storage_tests.rs
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
