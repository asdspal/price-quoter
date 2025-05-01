// tests/tycho_integration_tests.rs
use tycho_price_quoter::{
    tycho::TychoConnector,
    types::{Token, Pool},
};
use std::env;

// These tests require actual Tycho API access, so they're ignored by default
// To run them, use: cargo test -- --ignored

#[tokio::test]
#[ignore]
async fn test_tycho_connection() {
    let api_key = env::var("TYCHO_API_KEY").ok();
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    let connector = TychoConnector::new(&url, api_key).await;
    assert!(connector.is_ok());
}

#[tokio::test]
#[ignore]
async fn test_fetch_tokens() {
    let api_key = env::var("TYCHO_API_KEY").ok();
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    let connector = TychoConnector::new(&url, api_key).await.unwrap();
    let tokens = connector.fetch_tokens().await;
    
    assert!(tokens.is_ok());
    
    let tokens = tokens.unwrap();
    assert!(!tokens.is_empty());
    
    // Check for some common tokens
    let weth = tokens.iter().find(|t| t.symbol == "WETH");
    let usdc = tokens.iter().find(|t| t.symbol == "USDC");
    
    assert!(weth.is_some());
    assert!(usdc.is_some());
}

#[tokio::test]
#[ignore]
async fn test_fetch_pools() {
    let api_key = env::var("TYCHO_API_KEY").ok();
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    let connector = TychoConnector::new(&url, api_key).await.unwrap();
    let pools = connector.fetch_pools().await;
    
    assert!(pools.is_ok());
    
    let pools = pools.unwrap();
    assert!(!pools.is_empty());
    
    // Check that pools have valid data
    for pool in &pools {
        assert!(!pool.id.is_empty());
        assert!(!pool.token0.is_empty());
        assert!(!pool.token1.is_empty());
        assert!(pool.fee > 0);
    }
}

#[tokio::test]
#[ignore]
async fn test_error_handling_invalid_api_key() {
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    // Use an invalid API key
    let connector = TychoConnector::new(&url, Some("invalid_api_key".to_string())).await;
    
    // Connection should still succeed (API key is validated on requests)
    assert!(connector.is_ok());
    
    let connector = connector.unwrap();
    
    // But requests should fail
    let tokens = connector.fetch_tokens().await;
    assert!(tokens.is_err());
}
