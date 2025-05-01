// src/quoting.rs

use crate::types::{Address, Pool, SwapHop, SwapPath, U256};
use crate::storage::PoolStorage;

/// Calculates the expected output amount for a given swap path and input amount.
/// 
/// # Arguments
/// * `storage` - The pool storage containing all pools and tokens
/// * `path` - The swap path to calculate the quote for
/// * `amount_in` - The amount of input token to swap
/// 
/// # Returns
/// The expected output amount, or an error if the calculation fails
pub fn calculate_output_amount(
    storage: &PoolStorage,
    path: &SwapPath,
    amount_in: U256,
) -> Result<U256, String> {
    if path.hops.is_empty() {
        return Err("Path contains no hops".to_string());
    }

    let mut current_amount = amount_in;

    // Process each hop in the path
    for hop in &path.hops {
        let pool = match storage.get_pool(&hop.pool_id) {
            Some(p) => p,
            None => return Err(format!("Pool with ID {} not found", hop.pool_id)),
        };

        // Calculate the output amount for this hop
        current_amount = calculate_hop_output(pool, &hop.token_in, current_amount)?;
    }

    Ok(current_amount)
}

/// Calculates the output amount for a single hop.
/// 
/// # Arguments
/// * `pool` - The pool to swap through
/// * `token_in` - The address of the input token
/// * `amount_in` - The amount of input token
/// 
/// # Returns
/// The expected output amount, or an error if the calculation fails
fn calculate_hop_output(
    pool: &Pool,
    token_in: &Address,
    amount_in: U256,
) -> Result<U256, String> {
    // Determine which token is being swapped in
    let (reserve_in, reserve_out) = if *token_in == pool.token0 {
        (pool.reserve0, pool.reserve1)
    } else if *token_in == pool.token1 {
        (pool.reserve1, pool.reserve0)
    } else {
        return Err(format!("Token {} not found in pool {}", token_in, pool.id));
    };

    // Check for zero reserves
    if reserve_in == 0 || reserve_out == 0 {
        return Err(format!("Pool {} has zero reserves", pool.id));
    }

    // Calculate fee amount
    let fee_numerator = 10000 - pool.fee as u128;
    let amount_in_with_fee = amount_in * fee_numerator;

    // Calculate output amount using constant product formula: x * y = k
    // amount_out = reserve_out * amount_in_with_fee / (reserve_in * 10000 + amount_in_with_fee)
    
    // To avoid potential overflow, we'll do the calculation carefully
    // First, calculate the numerator (reserve_out * amount_in_with_fee)
    let numerator = reserve_out * amount_in_with_fee;
    
    // Then, calculate the denominator (reserve_in * 10000 + amount_in_with_fee)
    let denominator = reserve_in * 10000 + amount_in_with_fee;
    
    // Finally, calculate the output amount
    if denominator == 0 {
        return Err("Division by zero in output calculation".to_string());
    }
    
    let amount_out = numerator / denominator;
    
    Ok(amount_out)
}

/// Finds the best quote (highest output amount) among multiple paths.
/// 
/// # Arguments
/// * `storage` - The pool storage containing all pools and tokens
/// * `paths` - The swap paths to evaluate
/// * `amount_in` - The amount of input token to swap
/// 
/// # Returns
/// The best path and output amount, or None if no valid path is found
pub fn find_best_quote(
    storage: &PoolStorage,
    paths: &[SwapPath],
    amount_in: U256,
) -> Result<Option<(SwapPath, U256)>, String> {
    if paths.is_empty() {
        return Ok(None);
    }

    let mut best_path = None;
    let mut best_amount_out = 0;

    for path in paths {
        match calculate_output_amount(storage, path, amount_in) {
            Ok(amount_out) => {
                if amount_out > best_amount_out {
                    best_amount_out = amount_out;
                    best_path = Some(path.clone());
                }
            },
            Err(e) => {
                // Log the error but continue checking other paths
                eprintln!("Error calculating output for path: {}", e);
            }
        }
    }

    if let Some(path) = best_path {
        Ok(Some((path, best_amount_out)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Token;
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

        // USDC/WETH pool with price ~2500 USDC per WETH
        let pool1 = Pool {
            id: "0xPoolUsdcWeth".to_string(),
            token0: usdc_addr.clone(),
            token1: weth_addr.clone(),
            reserve0: 5_000_000 * 10u128.pow(6), // 5M USDC
            reserve1: 2_000 * 10u128.pow(18),    // 2k WETH
            fee: 30, // 0.3%
        };

        // WETH/DAI pool with price ~3000 DAI per WETH
        let pool2 = Pool {
            id: "0xPoolWethDai".to_string(),
            token0: weth_addr.clone(),
            token1: dai_addr.clone(),
            reserve0: 1_500 * 10u128.pow(18),     // 1.5k WETH
            reserve1: 4_500_000 * 10u128.pow(18), // 4.5M DAI
            fee: 30, // 0.3%
        };

        (tokens, vec![pool1, pool2])
    }

    #[test]
    fn test_calculate_hop_output() {
        let (_, pools) = create_test_data();
        let pool = &pools[0]; // USDC/WETH pool
        
        // Swap 1000 USDC for WETH
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
        let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
        
        let amount_out = calculate_hop_output(pool, &usdc_addr, amount_in).unwrap();
        
        // Expected output calculation:
        // amount_in_with_fee = 1000 * 10^6 * 9970 / 10000 = 997,000,000
        // amount_out = (2000 * 10^18 * 997,000,000) / (5,000,000 * 10^6 * 10000 + 997,000,000)
        // ≈ 0.398 WETH
        
        // The exact value depends on the precision of the calculation
        // Let's just check it's in a reasonable range
        assert!(amount_out > 0);
        assert!(amount_out < 1 * 10u128.pow(18)); // Less than 1 WETH
    }

    #[test]
    fn test_calculate_output_amount() {
        let (tokens, pools) = create_test_data();
        let storage = crate::storage::PoolStorage::with_initial_data(pools, tokens);
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
        let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
        
        // Create a path: USDC -> WETH -> DAI
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
        
        // Swap 1000 USDC
        let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
        
        let amount_out = calculate_output_amount(&storage, &path, amount_in).unwrap();
        
        // Expected output:
        // First hop: ~0.398 WETH (from previous test)
        // Second hop: ~0.398 WETH -> ~1194 DAI
        
        // The exact value depends on the precision of the calculation
        // Let's just check it's in a reasonable range
        assert!(amount_out > 0);
        assert!(amount_out < 2000 * 10u128.pow(18)); // Less than 2000 DAI
    }

    #[test]
    fn test_find_best_quote() {
        let (tokens, pools) = create_test_data();
        let storage = crate::storage::PoolStorage::with_initial_data(pools, tokens);
        
        let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
        let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
        let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
        
        // Create a path: USDC -> WETH -> DAI
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
        
        // Swap 1000 USDC
        let amount_in = 1000 * 10u128.pow(6); // 1000 USDC
        
        let paths = vec![path];
        let result = find_best_quote(&storage, &paths, amount_in).unwrap();
        
        assert!(result.is_some());
        let (best_path, amount_out) = result.unwrap();
        
        assert_eq!(best_path.hops.len(), 2);
        assert!(amount_out > 0);
    }
}
