// tests/quoting_tests.rs
use tycho_price_quoter::{
    quoting::{PriceQuoter, QuoterError},
    storage::PoolStorage,
    types::{Token, Pool, U256, Address},
};
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
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    assert_eq!(quoter.get_storage().token_count(), 3);
    assert_eq!(quoter.get_storage().pool_count(), 2);
}

#[test]
fn test_get_quote_direct_swap() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    
    // Get quote for USDC -> WETH (direct swap)
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, weth_addr, amount_in, 1);
    
    // Should succeed
    assert!(quote.is_ok());
    
    let quote = quote.unwrap();
    assert_eq!(quote.token_in, usdc_addr);
    assert_eq!(quote.token_out, weth_addr);
    assert_eq!(quote.amount_in, amount_in);
    assert_eq!(quote.path.len(), 1); // Direct path
}

#[test]
fn test_get_quote_multi_hop_swap() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    
    // Get quote for USDC -> DAI (multi-hop swap)
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, dai_addr, amount_in, 2);
    
    // Should succeed
    assert!(quote.is_ok());
    
    let quote = quote.unwrap();
    assert_eq!(quote.token_in, usdc_addr);
    assert_eq!(quote.token_out, dai_addr);
    assert_eq!(quote.amount_in, amount_in);
    assert_eq!(quote.path.len(), 2); // 2-hop path
}

#[test]
fn test_get_quote_nonexistent_token() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let nonexistent_addr = "0xNonExistentToken";
    
    // Try to get quote with a non-existent token
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, nonexistent_addr, amount_in, 2);
    
    // Should fail with TokenNotFound error
    assert!(quote.is_err());
    match quote.unwrap_err() {
        QuoterError::TokenNotFound(_) => {}, // Expected error
        err => panic!("Unexpected error: {:?}", err),
    }
}

#[test]
fn test_get_quote_no_path() {
    let (mut tokens, pools) = create_test_data();
    
    // Add a disconnected token
    let disconnected_addr = "0xDisconnectedToken".to_string();
    tokens.insert(disconnected_addr.clone(), Token {
        address: disconnected_addr.clone(),
        decimals: 18,
        symbol: "DISCONNECTED".to_string(),
    });
    
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    
    // Try to get quote to the disconnected token
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, &disconnected_addr, amount_in, 3);
    
    // Should fail with NoPathFound error
    assert!(quote.is_err());
    match quote.unwrap_err() {
        QuoterError::NoPathFound => {}, // Expected error
        err => panic!("Unexpected error: {:?}", err),
    }
}

#[test]
fn test_get_quote_different_decimal_scales() {
    let (tokens, pools) = create_test_data();
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);
    
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"; // 6 decimals
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"; // 18 decimals
    
    // Get quote for USDC -> WETH (different decimal scales)
    let amount_in = "1000000"; // 1 USDC with 6 decimals
    let quote = quoter.get_quote(usdc_addr, weth_addr, amount_in, 1);
    
    // Should succeed
    assert!(quote.is_ok());
    
    let quote = quote.unwrap();
    
    // Output amount should be positive
    let amount_out = quote.amount_out.parse::<u128>().unwrap();
    assert!(amount_out > 0);
}
