// src/indexer.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::types::{Address, Bytes, Pool, Token, U256};

/// Error types for Indexer operations
#[derive(thiserror::Error, Debug)]
pub enum IndexerError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("Failed to parse response: {0}")]
    ParseError(String),
    
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
    
    #[error("Indexer API error: {0}")]
    ApiError(String),
}

/// Client for interacting with the Tycho Indexer API
pub struct IndexerClient {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

// --- Tycho Indexer API Response Types ---
// These types mirror the structures we found in the Tycho codebase

#[derive(Debug, Deserialize)]
pub struct ResponseProtocolState {
    pub component_id: String,
    pub attributes: HashMap<String, Bytes>,
    pub balances: HashMap<String, Bytes>,
}

#[derive(Debug, Deserialize)]
pub struct ProtocolComponent {
    pub id: String,
    pub tokens: Vec<Address>,
    pub static_attributes: HashMap<String, Bytes>,
    // Other fields as needed
}

#[derive(Debug, Deserialize)]
pub struct ComponentWithState {
    pub component: ProtocolComponent,
    pub state: ResponseProtocolState,
}

#[derive(Debug, Serialize)]
pub struct IndexerRequest {
    pub protocol_ids: Vec<String>,
    // Other request parameters as needed
}

#[derive(Debug, Deserialize)]
pub struct IndexerResponse {
    pub components: Vec<ComponentWithState>,
    // Other response fields as needed
}

impl IndexerClient {
    /// Creates a new IndexerClient.
    pub fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    /// Fetches the current state of all pools for the specified protocols.
    pub async fn fetch_protocol_state(&self, protocol_ids: Vec<String>) -> Result<Vec<ComponentWithState>, IndexerError> {
        let url = format!("{}/api/v1/state", self.base_url);
        
        let mut request_builder = self.client.post(&url)
            .json(&IndexerRequest {
                protocol_ids,
                // Other parameters as needed
            });
        
        // Add API key if provided
        if let Some(api_key) = &self.api_key {
            request_builder = request_builder.header("X-API-Key", api_key);
        }
        
        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(IndexerError::ApiError(format!("API returned error {}: {}", status, error_text)));
        }
        
        let response_data: IndexerResponse = response.json().await?;
        
        Ok(response_data.components)
    }

    /// Converts ComponentWithState to our internal Pool representation.
    pub fn convert_to_pool(&self, component_state: &ComponentWithState) -> Result<Pool, IndexerError> {
        // Extract token addresses
        if component_state.component.tokens.len() < 2 {
            return Err(IndexerError::InvalidResponse(
                format!("Component {} has fewer than 2 tokens", component_state.component.id)
            ));
        }
        
        let token0 = component_state.component.tokens[0].clone();
        let token1 = component_state.component.tokens[1].clone();
        
        // Extract reserves
        let reserve0_bytes = component_state.state.attributes.get("reserve0")
            .ok_or_else(|| IndexerError::InvalidResponse(
                format!("Component {} missing reserve0", component_state.component.id)
            ))?;
        
        let reserve1_bytes = component_state.state.attributes.get("reserve1")
            .ok_or_else(|| IndexerError::InvalidResponse(
                format!("Component {} missing reserve1", component_state.component.id)
            ))?;
        
        // Parse reserves
        let reserve0 = parse_bytes_to_u256(reserve0_bytes)
            .map_err(|e| IndexerError::ParseError(format!("Failed to parse reserve0: {}", e)))?;
        
        let reserve1 = parse_bytes_to_u256(reserve1_bytes)
            .map_err(|e| IndexerError::ParseError(format!("Failed to parse reserve1: {}", e)))?;
        
        // Extract fee (assuming it's a static attribute)
        let fee_bytes = component_state.component.static_attributes.get("fee")
            .ok_or_else(|| IndexerError::InvalidResponse(
                format!("Component {} missing fee", component_state.component.id)
            ))?;
        
        // Parse fee (assuming it's stored as a u32 in basis points)
        let fee_bytes_vec = fee_bytes.clone();
        let fee = if fee_bytes_vec.len() == 4 {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&fee_bytes_vec);
            u32::from_be_bytes(arr)
        } else {
            // Default to 30 (0.3%) if parsing fails
            30
        };
        
        Ok(Pool {
            id: component_state.component.id.clone(),
            token0,
            token1,
            reserve0,
            reserve1,
            fee,
        })
    }

    /// Fetches token information from the Indexer.
    /// This is a simplified version - in reality, you might need to fetch this from a separate endpoint
    /// or derive it from other data.
    pub async fn fetch_token_info(&self, address: &Address) -> Result<Token, IndexerError> {
        // In a real implementation, you would fetch this from the Indexer
        // For now, we'll return a placeholder with default values
        
        Ok(Token {
            address: address.clone(),
            decimals: 18, // Default to 18 decimals
            symbol: format!("TOKEN_{}", address.chars().take(6).collect::<String>()), // Placeholder symbol
        })
    }

    /// Fetches all pools and tokens for the specified protocols and converts them to our internal representation.
    pub async fn fetch_and_convert_all(&self, protocol_ids: Vec<String>) -> Result<(Vec<Pool>, HashMap<Address, Token>), IndexerError> {
        let components = self.fetch_protocol_state(protocol_ids).await?;
        
        let mut pools = Vec::new();
        let mut tokens = HashMap::new();
        let mut token_addresses = Vec::new();
        
        // First, collect all pools and token addresses
        for component in &components {
            let pool = self.convert_to_pool(component)?;
            
            // Collect token addresses
            if !token_addresses.contains(&pool.token0) {
                token_addresses.push(pool.token0.clone());
            }
            if !token_addresses.contains(&pool.token1) {
                token_addresses.push(pool.token1.clone());
            }
            
            pools.push(pool);
        }
        
        // Then, fetch token information for all addresses
        for address in token_addresses {
            let token = self.fetch_token_info(&address).await?;
            tokens.insert(address, token);
        }
        
        Ok((pools, tokens))
    }
}

/// Helper function to parse Bytes into U256.
/// This is a copy of the function from types.rs, but we include it here to avoid circular dependencies.
fn parse_bytes_to_u256(bytes: &Bytes) -> Result<U256, String> {
    // Placeholder implementation: Assumes big-endian u128 encoding
    if bytes.len() == 16 {
        let mut arr = [0u8; 16];
        arr.copy_from_slice(bytes);
        Ok(u128::from_be_bytes(arr))
    } else if bytes.is_empty() {
        Ok(0) // Handle empty bytes case if necessary
    } else {
        // In reality, handle hex decoding, potential padding, etc.
        Err(format!("Invalid byte length for U256: {}", bytes.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // These tests would normally use mocks to avoid actual API calls
    // For simplicity, we'll just test the conversion logic
    
    #[test]
    fn test_convert_to_pool() {
        // Create a sample ComponentWithState
        let component = ProtocolComponent {
            id: "pool1".to_string(),
            tokens: vec![
                "0xToken0".to_string(),
                "0xToken1".to_string(),
            ],
            static_attributes: {
                let mut map = HashMap::new();
                // Fee of 30 basis points (0.3%)
                map.insert("fee".to_string(), vec![0, 0, 0, 30]);
                map
            },
        };
        
        let state = ResponseProtocolState {
            component_id: "pool1".to_string(),
            attributes: {
                let mut map = HashMap::new();
                // Reserve0: 1000000 (as 16-byte big-endian)
                let mut reserve0 = vec![0; 16];
                reserve0[15] = 0x10;
                reserve0[14] = 0x27;
                reserve0[13] = 0x00;
                
                // Reserve1: 500000 (as 16-byte big-endian)
                let mut reserve1 = vec![0; 16];
                reserve1[15] = 0x07;
                reserve1[14] = 0xA1;
                reserve1[13] = 0x20;
                
                map.insert("reserve0".to_string(), reserve0);
                map.insert("reserve1".to_string(), reserve1);
                map
            },
            balances: HashMap::new(),
        };
        
        let component_state = ComponentWithState {
            component,
            state,
        };
        
        let client = IndexerClient::new("https://example.com".to_string(), None);
        let pool = client.convert_to_pool(&component_state).unwrap();
        
        assert_eq!(pool.id, "pool1");
        assert_eq!(pool.token0, "0xToken0");
        assert_eq!(pool.token1, "0xToken1");
        assert_eq!(pool.fee, 30);
        
        // The exact values depend on how parse_bytes_to_u256 is implemented
        assert!(pool.reserve0 > 0);
        assert!(pool.reserve1 > 0);
    }
}
