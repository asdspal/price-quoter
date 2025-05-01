// tests/test_quoting.rs

use tycho_price_quoter::{Token, Pool, PoolStorage, SwapHop, SwapPath, quoting};
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
fn test_calculate_output_amount_single_hop() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    
    // Create a single-hop path: USDC -> WETH
    let hop = SwapHop {
        pool_id: "0xPoolUsdcWeth".to_string(),
        token_in: usdc_addr.clone(),
        token_out: weth_addr.clone(),
    };
    
    let path = SwapPath {
        hops: vec![hop],
        token_in: usdc_addr.clone(),
        token_out: weth_addr.clone(),
    };
    
    // Swap 1000 USDC for WETH
    let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
    
    let amount_out = quoting::calculate_output_amount(&storage, &path, amount_in).unwrap();
    
    // Output should be positive
    assert!(amount_out > 0);
    
    // Rough check: 1000 USDC should get approximately 0.4 WETH
    // (based on the pool reserves: 5M USDC <-> 2k WETH)
    // 1000 / 5M * 2k = 0.4 WETH
    let expected_approx = 0.4 * 10f64.powi(18) as u128;
    let tolerance = 0.05; // 5% tolerance for fees and slippage
    
    let ratio = amount_out as f64 / expected_approx as f64;
    assert!(ratio > (1.0 - tolerance) && ratio < (1.0 + tolerance));
}

#[test]
fn test_calculate_output_amount_multi_hop() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    
    // Create a two-hop path: USDC -> WETH -> DAI
    let hop1 = SwapHop {
        pool_id: "0xPoolUsdcWeth".to_string(),
        token_in: usdc_addr.clone(),
        token_out: weth_addr.clone(),
    };
    
    let hop2 = SwapHop {
        pool_id: "0xPoolWethDai".to_string(),
        token_in: weth_addr.clone(),
        token_out: dai_addr.clone(),
    };
    
    let path = SwapPath {
        hops: vec![hop1, hop2],
        token_in: usdc_addr.clone(),
        token_out: dai_addr.clone(),
    };
    
    // Swap 1000 USDC for DAI
    let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
    
    let amount_out = quoting::calculate_output_amount(&storage, &path, amount_in).unwrap();
    
    // Output should be positive
    assert!(amount_out > 0);
    
    // Rough check: 1000 USDC -> ~0.4 WETH -> ~1200 DAI
    // (based on the pool reserves)
    let expected_approx = 1200.0 * 10f64.powi(18) as u128;
    let tolerance = 0.1; // 10% tolerance for multi-hop fees and slippage
    
    let ratio = amount_out as f64 / expected_approx as f64;
    assert!(ratio > (1.0 - tolerance) && ratio < (1.0 + tolerance));
}

#[test]
fn test_find_best_quote() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    
    // Create two paths: USDC -> WETH -> DAI
    let hop1 = SwapHop {
        pool_id: "0xPoolUsdcWeth".to_string(),
        token_in: usdc_addr.clone(),
        token_out: weth_addr.clone(),
    };
    
    let hop2 = SwapHop {
        pool_id: "0xPoolWethDai".to_string(),
        token_in: weth_addr.clone(),
        token_out: dai_addr.clone(),
    };
    
    let path = SwapPath {
        hops: vec![hop1, hop2],
        token_in: usdc_addr.clone(),
        token_out: dai_addr.clone(),
    };
    
    // Swap 1000 USDC for DAI
    let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
    
    let paths = vec![path];
    let result = quoting::find_best_quote(&storage, &paths, amount_in).unwrap();
    
    assert!(result.is_some());
    let (best_path, amount_out) = result.unwrap();
    
    // Should be a 2-hop path: USDC -> WETH -> DAI
    assert_eq!(best_path.hops.len(), 2);
    assert_eq!(best_path.token_in, usdc_addr);
    assert_eq!(best_path.token_out, dai_addr);
    
    // Output amount should be positive
    assert!(amount_out > 0);
}
