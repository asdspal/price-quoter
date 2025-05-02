use crate::types::{Pool, Token};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tycho_client::rpc::{HttpRPCClient, RPCClient};
use tycho_common::dto::{
    Chain, 
    ProtocolComponent,
    ProtocolComponentsRequestBody, 
    ProtocolStateRequestBody,
    TokensRequestBody,
    PaginationParams,
};

#[derive(Error, Debug)]
pub enum TychoError {
    #[error("RPC error: {0}")]
    RpcError(#[from] tycho_client::rpc::RPCError),
    
    #[error("Failed to parse component data: {0}")]
    ParseError(String),
}

pub struct TychoConnector {
    client: HttpRPCClient,
}

impl TychoConnector {
    pub async fn new(url: &str, api_key: Option<String>) -> Result<Self, TychoError> {
        let client = HttpRPCClient::new(url, api_key.as_deref()).map_err(TychoError::RpcError)?;
        Ok(Self { client })
    }
    
    pub async fn fetch_tokens(&self) -> Result<Vec<Token>, TychoError> {
        let request = TokensRequestBody {
            token_addresses: None,
            min_quality: Some(51),
            traded_n_days_ago: Some(30),
            pagination: PaginationParams { page: 0, page_size: 1000 },
            chain: Chain::Ethereum,
        };
        
        let response = self.client.get_tokens(&request).await?;
        
        let result = response.tokens
            .into_iter()
            .map(|t| Token {
                address: hex::encode(&t.address),
                symbol: t.symbol,
                decimals: t.decimals as u8, // Convert u32 to u8
            })
            .collect();
        
        Ok(result)
    }
    
	pub async fn fetch_pools(&self) -> Result<(Vec<Pool>, HashMap<String, Token>), TychoError> {
		// First fetch tokens
		println!("Fetching tokens...");
		let tokens = match self.client.get_all_tokens(
		    Chain::Ethereum,
		    Some(51),  // min token quality
		    Some(30),  // days since last traded
		    1000,      // chunk size
		).await {
		    Ok(tokens) => tokens,
		    Err(e) => {
		        eprintln!("Error fetching tokens: {:?}", e);
		        return Err(TychoError::RpcError(e));
		    }
		};
		
		// Convert to our Token type and store in a HashMap
		let mut token_map = HashMap::new();
		for token in tokens {
		    let address = hex::encode(&token.address);
		    let our_token = Token {
		        address: address.clone(),
		        symbol: token.symbol.clone(),
		        decimals: token.decimals as u8,
		    };
		    token_map.insert(address, our_token);
		}
		
		println!("Fetched {} tokens", token_map.len());
		
		// Then get components
		println!("Fetching components...");
		let components_request = ProtocolComponentsRequestBody {
		    protocol_system: "uniswap_v2".to_string(),
		    component_ids: None,
		    tvl_gt: None,
		    chain: Chain::Ethereum,
		    pagination: PaginationParams { page: 0, page_size: 500 },
		};
		
		let components = match self.client.get_protocol_components(&components_request).await {
		    Ok(c) => c,
		    Err(e) => {
		        eprintln!("Error fetching components: {:?}", e);
		        return Err(TychoError::RpcError(e));
		    }
		};
		
		println!("Fetched {} components", components.protocol_components.len());
		
		// Extract component IDs
		let component_ids: Vec<String> = components.protocol_components
		    .iter()
		    .map(|c| c.id.clone())
		    .collect();
		
		if component_ids.is_empty() {
		    return Ok((vec![], token_map));
		}
		
		// Get states using component_ids
		println!("Fetching states...");
		let states_request = ProtocolStateRequestBody {
		    protocol_ids: Some(component_ids),
		    protocol_system: "uniswap_v2".to_string(),
		    chain: Chain::Ethereum,
		    include_balances: true,
		    version: Default::default(),
		    pagination: PaginationParams { page: 0, page_size: 100 },
		};
		
		let states = match self.client.get_protocol_states(&states_request).await {
		    Ok(s) => s,
		    Err(e) => {
		        eprintln!("Error fetching states: {:?}", e);
		        return Err(TychoError::RpcError(e));
		    }
		};
		
		println!("Fetched {} states", states.states.len());
		
		// Process components and states to create Pool objects
		let mut pools = Vec::new();
		
		for component in components.protocol_components {
		    // Find the corresponding state
		    if let Some(state) = states.states.iter().find(|s| s.component_id == component.id) {
		        // Extract token addresses
		        if component.tokens.len() != 2 {
		            continue; // Skip pools that don't have exactly 2 tokens
		        }
		        
		        let token0 = hex::encode(&component.tokens[0]);
		        let token1 = hex::encode(&component.tokens[1]);
		        
		        // Make sure we have these tokens in our map
		        // If not, create placeholder tokens
		        if !token_map.contains_key(&token0) {
		            let placeholder = Token {
		                address: token0.clone(),
		                symbol: format!("Token-{}", &token0[0..8]),
		                decimals: 18, // Default to 18 decimals
		            };
		            token_map.insert(token0.clone(), placeholder);
		        }
		        
		        if !token_map.contains_key(&token1) {
		            let placeholder = Token {
		                address: token1.clone(),
		                symbol: format!("Token-{}", &token1[0..8]),
		                decimals: 18, // Default to 18 decimals
		            };
		            token_map.insert(token1.clone(), placeholder);
		        }
		        
		        // Extract reserves from state
		        let reserve0 = match state.attributes.get("reserve0") {
		            Some(bytes) => {
		                let bytes = bytes.as_ref();
		                if bytes.len() <= 32 {
		                    let mut buf = [0u8; 32];
		                    let start = 32 - bytes.len();
		                    buf[start..].copy_from_slice(bytes);
		                    u128::from_be_bytes(buf[16..32].try_into().unwrap_or_default())
		                } else {
		                    0
		                }
		            },
		            None => 0,
		        };
		        
		        let reserve1 = match state.attributes.get("reserve1") {
		            Some(bytes) => {
		                let bytes = bytes.as_ref();
		                if bytes.len() <= 32 {
		                    let mut buf = [0u8; 32];
		                    let start = 32 - bytes.len();
		                    buf[start..].copy_from_slice(bytes);
		                    u128::from_be_bytes(buf[16..32].try_into().unwrap_or_default())
		                } else {
		                    0
		                }
		            },
		            None => 0,
		        };
		        
		        // Extract fee from component static attributes
		        let fee = match component.static_attributes.get("fee") {
		            Some(bytes) => {
		                let bytes = bytes.as_ref();
		                if !bytes.is_empty() {
		                    bytes[0] as u32  // Convert to u32
		                } else {
		                    30u32 // Default to 0.3%
		                }
		            },
		            None => 30u32, // Default to 0.3%
		        };
		        
		        // Create Pool object
		        let pool = Pool {
		            id: component.id.clone(),
		            token0: token0,
		            token1: token1,
		            reserve0,
		            reserve1,
		            fee,
		        };
		        
		        pools.push(pool);
		    }
		}
		
		println!("Created {} pools", pools.len());
		
		Ok((pools, token_map))
	}
	    
    fn component_to_pool(
        &self, 
        component: &tycho_common::dto::ProtocolComponent,
        state: &serde_json::Value
    ) -> Result<Pool, TychoError> {
        // Extract token addresses from component
        if component.tokens.len() < 2 {
            return Err(TychoError::ParseError("Component has less than 2 tokens".to_string()));
        }
        
        let token0 = hex::encode(&component.tokens[0]);
        let token1 = hex::encode(&component.tokens[1]);
        
        // Extract reserves from state - fixed borrowing issues
        let reserve0 = if let Some(str_val) = state["reserve0"].as_str() {
            str_val.to_string()
        } else if let Some(arr) = state["reserve0"].as_array() {
            self.bytes_to_hex_string(arr)
        } else {
            return Err(TychoError::ParseError("reserve0 not found or not in expected format".to_string()));
        };
        
        let reserve1 = if let Some(str_val) = state["reserve1"].as_str() {
            str_val.to_string()
        } else if let Some(arr) = state["reserve1"].as_array() {
            self.bytes_to_hex_string(arr)
        } else {
            return Err(TychoError::ParseError("reserve1 not found or not in expected format".to_string()));
        };
        
        // Get fee from component static attributes or default to 30 (0.3%)
        let fee = component.static_attributes.get("fee")
            .and_then(|v| {
                let bytes = v.as_ref();
                if bytes.len() <= 4 {
                    let mut buf = [0u8; 4];
                    let start = 4 - bytes.len();
                    buf[start..].copy_from_slice(bytes);
                    Some(u32::from_be_bytes(buf))
                } else {
                    None
                }
            })
            .unwrap_or(30);
        
        // Create Pool
        Ok(Pool {
            id: component.id.clone(),
            token0,
            token1,
            reserve0: reserve0.parse().unwrap_or(0),
            reserve1: reserve1.parse().unwrap_or(0),
            fee,
        })
    }
    
    fn bytes_to_hex_string(&self, bytes: &[serde_json::Value]) -> String {
        let mut result = String::from("0x");
        for v in bytes {
            if let Some(byte) = v.as_u64() {
                let hex_str = hex::encode([byte as u8]);
                result.push_str(&hex_str);
            }
        }
        result
    }
}
