// src/types.rs

use std::collections::HashMap;
// Re-export or define necessary primitive types.
// In a real scenario, these would likely come from crates like `ethers` or `alloy-primitives`.
// For now, we define placeholders.
pub type Address = String; // Placeholder for ethers::types::Address or alloy_primitives::Address
pub type U256 = u128;    // Placeholder for ethers::types::U256 or alloy_primitives::U256
pub type Bytes = Vec<u8>; // Placeholder for tycho_common::Bytes or ethers::types::Bytes

/// Represents static information about a token.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub address: Address,
    pub decimals: u8,
    pub symbol: String, // Optional, but useful for debugging/logging
    // pub gas: Option<u64>, // Field from simulation codebase, maybe not needed for quoter
}

/// Represents the essential static and dynamic state of a liquidity pool
/// relevant for quoting (simplified internal representation).
/// Assumes derivation from Tycho's `ComponentWithState`.
#[derive(Debug, Clone)]
pub struct Pool {
    pub id: String,         // Unique identifier for the pool (e.g., contract address or Tycho ID)
    pub token0: Address,    // Address of the first token
    pub token1: Address,    // Address of the second token
    pub reserve0: U256,     // Reserve of token0 (scaled according to token0 decimals)
    pub reserve1: U256,     // Reserve of token1 (scaled according to token1 decimals)
    pub fee: u32,           // Fee tier in basis points (e.g., 30 for 0.30%) - Assuming V2/V3 style fee
                            // Add other fields if needed, e.g., tick for V3, pool type enum
}

/// Represents a single step (hop) in a potential multi-hop swap path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwapHop {
    pub pool_id: String,
    pub token_in: Address,
    pub token_out: Address,
}

/// Represents a potential multi-hop swap path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwapPath {
    pub hops: Vec<SwapHop>,
    pub token_in: Address,
    pub token_out: Address,
}

// --- Helper functions (internal or moved to specific modules later) ---

/// Conceptual helper to parse reserve Bytes into U256.
/// The actual implementation depends on Tycho's encoding (e.g., big-endian hex string).
/// This might live in a module responsible for processing Indexer data.
pub(crate) fn parse_bytes_to_u256(bytes: &Bytes) -> Result<U256, String> {
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

// Add other necessary types or enums, e.g., for different pool types if supporting more than V2.
// pub enum PoolType {
//     UniswapV2,
//     UniswapV3,
//     // ... other types
// }
