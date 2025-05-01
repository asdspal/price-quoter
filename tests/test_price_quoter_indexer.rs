// tests/test_price_quoter_indexer.rs

use tycho_price_quoter::{PriceQuoter, Token, Pool};
use std::collections::HashMap;

// Test that the PriceQuoter can be created with an Indexer
#[test]
fn test_price_quoter_with_indexer() {
    let quoter = PriceQuoter::new()
        .with_indexer(
            "https://tycho-beta.propellerheads.xyz".to_string(),
            Some("sampletoken".to_string()),
        );
    
    // Verify that the Indexer client was created
    assert!(quoter.has_indexer());
}

// Test fetching data from the Indexer
// Use the #[ignore] attribute so it doesn't run during normal test runs
#[tokio::test]
#[ignore]
async fn test_fetch_latest_data() {
    let mut quoter = PriceQuoter::new()
        .with_indexer(
            "https://tycho-beta.propellerheads.xyz".to_string(),
            Some("sampletoken".to_string()),
        );
    
    let result = quoter.fetch_latest_data(vec!["uniswap_v2".to_string()]).await;
    
    assert!(result.is_ok());
    assert!(quoter.pool_count() > 0);
    assert!(quoter.token_count() > 0);
}

// Test finding paths with Indexer data
#[tokio::test]
#[ignore]
async fn test_find_paths_with_indexer_data() {
    let mut quoter = PriceQuoter::new()
        .with_indexer(
            "https://tycho-beta.propellerheads.xyz".to_string(),
            Some("sampletoken".to_string()),
        );
    
    let _ = quoter.fetch_latest_data(vec!["uniswap_v2".to_string()]).await;
    
    // Define token addresses for USDC and WETH
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    
    let paths = quoter.find_paths(&usdc_addr, &weth_addr);
    
    // There should be at least one path between USDC and WETH
    assert!(!paths.is_empty());
}

// Test finding quotes with Indexer data
#[tokio::test]
#[ignore]
async fn test_find_quote_with_indexer_data() {
    let mut quoter = PriceQuoter::new()
        .with_indexer(
            "https://tycho-beta.propellerheads.xyz".to_string(),
            Some("sampletoken".to_string()),
        );
    
    let _ = quoter.fetch_latest_data(vec!["uniswap_v2".to_string()]).await;
    
    // Define token addresses for USDC and WETH
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    
    // Swap 1000 USDC for WETH
    let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
    
    let result = quoter.find_quote(&usdc_addr, &weth_addr, amount_in);
    
    assert!(result.is_ok());
    let quote = result.unwrap();
    assert!(quote.is_some());
    
    let (path, amount_out) = quote.unwrap();
    assert!(!path.hops.is_empty());
    assert!(amount_out > 0);
}
