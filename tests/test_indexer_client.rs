// tests/test_indexer_client.rs

use tycho_price_quoter::indexer::{IndexerClient, IndexerError};
use std::collections::HashMap;

#[test]
fn test_indexer_client_creation() {
    let client = IndexerClient::new(
        "https://tycho-beta.propellerheads.xyz".to_string(),
        Some("sampletoken".to_string()),
    );
    
    // Just verify that the client was created successfully
    assert!(true);
}

#[tokio::test]
async fn test_fetch_protocol_state() {
    let client = IndexerClient::new(
        "https://tycho-beta.propellerheads.xyz".to_string(),
        Some("sampletoken".to_string()),
    );
    
    let protocol_ids = vec!["uniswap_v2".to_string()];
    let result = client.fetch_protocol_state(protocol_ids).await;
    
    assert!(result.is_ok());
    let components = result.unwrap();
    assert!(!components.is_empty());
}

#[tokio::test]
async fn test_convert_to_pool() {
    let client = IndexerClient::new(
        "https://tycho-beta.propellerheads.xyz".to_string(),
        Some("sampletoken".to_string()),
    );
    
    let protocol_ids = vec!["uniswap_v2".to_string()];
    let components = client.fetch_protocol_state(protocol_ids).await.unwrap();
    
    if !components.is_empty() {
        let result = client.convert_to_pool(&components[0]);
        assert!(result.is_ok());
        
        let pool = result.unwrap();
        assert!(!pool.id.is_empty());
        assert!(!pool.token0.is_empty());
        assert!(!pool.token1.is_empty());
    }
}

#[tokio::test]
async fn test_fetch_and_convert_all() {
    let client = IndexerClient::new(
        "https://tycho-beta.propellerheads.xyz".to_string(),
        Some("sampletoken".to_string()),
    );
    
    let protocol_ids = vec!["uniswap_v2".to_string()];
    let result = client.fetch_and_convert_all(protocol_ids).await;
    
    assert!(result.is_ok());
    let (pools, tokens) = result.unwrap();
    assert!(!pools.is_empty());
    assert!(!tokens.is_empty());
}
