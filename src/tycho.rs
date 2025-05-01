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
    
    pub async fn fetch_pools(&self) -> Result<Vec<Pool>, TychoError> {
        let components_request = ProtocolComponentsRequestBody {
            protocol_system: "uniswap_v2".to_string(),
            component_ids: None,
            tvl_gt: None,
            chain: Chain::Ethereum,
            pagination: PaginationParams { page: 0, page_size: 1000 },
        };
        
        let states_request = ProtocolStateRequestBody {
            protocol_ids: None,
            protocol_system: "uniswap_v2".to_string(),
            chain: Chain::Ethereum,
            include_balances: true,
            version: Default::default(),
            pagination: PaginationParams { page: 0, page_size: 1000 },
        };
        
        let components = self.client.get_protocol_components(&components_request).await?;
        let states = self.client.get_protocol_states(&states_request).await?;
        
        // Create a map of component_id to state
        let state_map: HashMap<String, Arc<serde_json::Value>> = states
            .states
            .into_iter()
            .map(|s| {
                let state_value = serde_json::to_value(&s.attributes)
                    .unwrap_or_else(|_| serde_json::Value::Null);
                (s.component_id.clone(), Arc::new(state_value))
            })
            .collect();
        
        let mut pools = Vec::new();
        
        for component in components.protocol_components {
            if let Some(state) = state_map.get(&component.id) {
                // Convert component and state to Pool
                match self.component_to_pool(&component, state) {
                    Ok(pool) => pools.push(pool),
                    Err(e) => eprintln!("Failed to convert component to pool: {:?}", e),
                }
            }
        }
        
        Ok(pools)
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
