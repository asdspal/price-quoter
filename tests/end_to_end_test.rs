// tests/end_to_end_test.rs
use tycho_price_quoter::{
    PriceQuoter,
    storage::PoolStorage,
    types::{Token, Pool, U256, Address},
};
use std::collections::HashMap;
use std::env;

// This test demonstrates the complete functionality of the price quoter
#[test]
fn test_end_to_end_functionality() {
    // 1. Create test data with multiple tokens and pools
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string();
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string();
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string();
    let usdt_addr = "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string();
    let wbtc_addr = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".to_string();

    let usdc = Token { address: usdc_addr.clone(), decimals: 6, symbol: "USDC".to_string() };
    let weth = Token { address: weth_addr.clone(), decimals: 18, symbol: "WETH".to_string() };
    let dai = Token { address: dai_addr.clone(), decimals: 18, symbol: "DAI".to_string() };
    let usdt = Token { address: usdt_addr.clone(), decimals: 6, symbol: "USDT".to_string() };
    let wbtc = Token { address: wbtc_addr.clone(), decimals: 8, symbol: "WBTC".to_string() };

    let mut tokens = HashMap::new();
    tokens.insert(usdc_addr.clone(), usdc);
    tokens.insert(weth_addr.clone(), weth);
    tokens.insert(dai_addr.clone(), dai);
    tokens.insert(usdt_addr.clone(), usdt);
    tokens.insert(wbtc_addr.clone(), wbtc);

    // Create a more complex pool network
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

    let pool3 = Pool {
        id: "0xPoolUsdtWeth".to_string(),
        token0: usdt_addr.clone(),
        token1: weth_addr.clone(),
        reserve0: 4_000_000 * 10u128.pow(6), // 4M USDT
        reserve1: 1_800 * 10u128.pow(18),    // 1.8k WETH
        fee: 30,
    };

    let pool4 = Pool {
        id: "0xPoolWethWbtc".to_string(),
        token0: weth_addr.clone(),
        token1: wbtc_addr.clone(),
        reserve0: 10_000 * 10u128.pow(18),    // 10k WETH
        reserve1: 500 * 10u128.pow(8),        // 500 WBTC
        fee: 30,
    };

    let pool5 = Pool {
        id: "0xPoolUsdcDai".to_string(),
        token0: usdc_addr.clone(),
        token1: dai_addr.clone(),
        reserve0: 10_000_000 * 10u128.pow(6),  // 10M USDC
        reserve1: 10_000_000 * 10u128.pow(18), // 10M DAI
        fee: 30,
    };

    let pools = vec![pool1, pool2, pool3, pool4, pool5];

    // 2. Initialize the storage and price quoter
    let storage = PoolStorage::with_initial_data(pools, tokens);
    let quoter = PriceQuoter::new(storage);

    // 3. Test direct swap quote
    let usdc_to_dai_direct = quoter.get_quote(
        &usdc_addr,
        &dai_addr,
        "1000000", // 1 USDC
        1
    ).unwrap();

    println!("Direct USDC -> DAI quote:");
    println!("  Amount in: {} USDC", usdc_to_dai_direct.amount_in);
    println!("  Amount out: {} DAI", usdc_to_dai_direct.amount_out);
    println!("  Path: {:?}", usdc_to_dai_direct.path);

    // 4. Test multi-hop swap quote
    let usdt_to_wbtc = quoter.get_quote(
        &usdt_addr,
        &wbtc_addr,
        "1000000", // 1 USDT
        2
    ).unwrap();

    println!("Multi-hop USDT -> WBTC quote:");
    println!("  Amount in: {} USDT", usdt_to_wbtc.amount_in);
    println!("  Amount out: {} WBTC", usdt_to_wbtc.amount_out);
    println!("  Path: {:?}", usdt_to_wbtc.path);

    // 5. Test finding multiple paths and selecting the best one
    let usdc_to_dai_multi = quoter.get_quote(
        &usdc_addr,
        &dai_addr,
        "1000000", // 1 USDC
        2
    ).unwrap();

    println!("Best path USDC -> DAI quote:");
    println!("  Amount in: {} USDC", usdc_to_dai_multi.amount_in);
    println!("  Amount out: {} DAI", usdc_to_dai_multi.amount_out);
    println!("  Path: {:?}", usdc_to_dai_multi.path);

    // 6. Test a more complex multi-hop path
    let usdt_to_dai = quoter.get_quote(
        &usdt_addr,
        &dai_addr,
        "1000000", // 1 USDT
        3
    ).unwrap();

    println!("Complex path USDT -> DAI quote:");
    println!("  Amount in: {} USDT", usdt_to_dai.amount_in);
    println!("  Amount out: {} DAI", usdt_to_dai.amount_out);
    println!("  Path: {:?}", usdt_to_dai.path);

    // 7. Verify all quotes have valid output amounts
    assert!(usdc_to_dai_direct.amount_out.parse::<u128>().unwrap() > 0);
    assert!(usdt_to_wbtc.amount_out.parse::<u128>().unwrap() > 0);
    assert!(usdc_to_dai_multi.amount_out.parse::<u128>().unwrap() > 0);
    assert!(usdt_to_dai.amount_out.parse::<u128>().unwrap() > 0);

    // 8. Verify the path lengths are as expected
    assert_eq!(usdc_to_dai_direct.path.len(), 1); // Direct path
    assert_eq!(usdt_to_wbtc.path.len(), 2);       // 2-hop path
    
    // The best path for USDC -> DAI could be either direct or through WETH
    // depending on the reserves and fees
    assert!(usdc_to_dai_multi.path.len() >= 1 && usdc_to_dai_multi.path.len() <= 2);
    
    // USDT -> DAI should be a 2-hop path through WETH
    assert_eq!(usdt_to_dai.path.len(), 2);
}

// This test requires actual Tycho API access, so it's ignored by default
// To run it, use: cargo test -- --ignored
#[tokio::test]
#[ignore]
async fn test_end_to_end_with_tycho() {
    let api_key = env::var("TYCHO_API_KEY").ok();
    let url = env::var("TYCHO_API_URL").unwrap_or_else(|_| "https://indexer.tycho.network".to_string());
    
    // 1. Initialize the price quoter with data from Tycho
    let quoter = PriceQuoter::from_tycho(&url, api_key).await.unwrap();
    
    println!("Loaded {} tokens and {} pools from Tycho", 
             quoter.get_storage().token_count(),
             quoter.get_storage().pool_count());
    
    // 2. Define some well-known token addresses
    let weth_addr = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let usdc_addr = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
    let dai_addr = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    
    // 3. Get a quote for USDC -> WETH
    let usdc_to_weth = quoter.get_quote(
        usdc_addr,
        weth_addr,
        "1000000", // 1 USDC
        1
    );
    
    if let Ok(quote) = usdc_to_weth {
        println!("USDC -> WETH quote:");
        println!("  Amount in: {} USDC", quote.amount_in);
        println!("  Amount out: {} WETH", quote.amount_out);
        println!("  Path: {:?}", quote.path);
    } else {
        println!("Failed to get USDC -> WETH quote: {:?}", usdc_to_weth.err());
    }
    
    // 4. Get a quote for USDC -> DAI
    let usdc_to_dai = quoter.get_quote(
        usdc_addr,
        dai_addr,
        "1000000", // 1 USDC
        2
    );
    
    if let Ok(quote) = usdc_to_dai {
        println!("USDC -> DAI quote:");
        println!("  Amount in: {} USDC", quote.amount_in);
        println!("  Amount out: {} DAI", quote.amount_out);
        println!("  Path: {:?}", quote.path);
    } else {
        println!("Failed to get USDC -> DAI quote: {:?}", usdc_to_dai.err());
    }
}
