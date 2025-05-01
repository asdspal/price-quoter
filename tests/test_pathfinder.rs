// tests/test_pathfinder.rs

use tycho_price_quoter::{Token, Pool, PoolStorage, pathfinder};
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
fn test_find_direct_paths() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    
    // Find direct paths with max_hops = 1
    let paths = pathfinder::find_paths(&storage, &usdc_addr, &weth_addr, 1);
    
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
    let paths = pathfinder::find_paths(&storage, &usdc_addr, &dai_addr, 2);
    
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
    let paths = pathfinder::find_paths(&storage, &usdt_addr, &dai_addr, 2);
    
    // Should find 1 path: USDT -> WETH -> DAI
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].hops.len(), 2);
}

#[test]
fn test_no_path() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let usdt_addr = "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string();
    
    // Find paths with max_hops = 1
    let paths = pathfinder::find_paths(&storage, &usdc_addr, &usdt_addr, 1);
    
    // Should find no paths (requires 2 hops: USDC -> WETH -> USDT)
    assert_eq!(paths.len(), 0);
    
    // Find paths with max_hops = 2
    let paths = pathfinder::find_paths(&storage, &usdc_addr, &usdt_addr, 2);
    
    // Should find 1 path: USDC -> WETH -> USDT
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].hops.len(), 2);
}
