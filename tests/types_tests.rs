// tests/types_tests.rs
use tycho_price_quoter::types::{Token, Pool, SwapHop, SwapPath, U256, Address};

#[test]
fn test_token_creation() {
    let token = Token {
        address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        decimals: 6,
        symbol: "USDC".to_string(),
    };
    
    assert_eq!(token.address, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    assert_eq!(token.decimals, 6);
    assert_eq!(token.symbol, "USDC");
}

#[test]
fn test_pool_creation() {
    let pool = Pool {
        id: "0xPool1".to_string(),
        token0: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        token1: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        reserve0: 5_000_000 * 10u128.pow(6),
        reserve1: 2_000 * 10u128.pow(18),
        fee: 30,
    };
    
    assert_eq!(pool.id, "0xPool1");
    assert_eq!(pool.token0, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    assert_eq!(pool.token1, "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    assert_eq!(pool.reserve0, 5_000_000 * 10u128.pow(6));
    assert_eq!(pool.reserve1, 2_000 * 10u128.pow(18));
    assert_eq!(pool.fee, 30);
}

#[test]
fn test_swap_path_creation() {
    let hop1 = SwapHop {
        pool_id: "0xPool1".to_string(),
        token_in: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        token_out: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
    };
    
    let hop2 = SwapHop {
        pool_id: "0xPool2".to_string(),
        token_in: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
        token_out: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
    };
    
    let path = SwapPath {
        hops: vec![hop1.clone(), hop2.clone()],
        token_in: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
        token_out: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
    };
    
    assert_eq!(path.hops.len(), 2);
    assert_eq!(path.hops[0], hop1);
    assert_eq!(path.hops[1], hop2);
    assert_eq!(path.token_in, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    assert_eq!(path.token_out, "0x6B175474E89094C44Da98b954EedeAC495271d0F");
}


