#[cfg(test)]
mod tests {
    use price_quoter::PriceQuoter;
    use std::env;
    
    #[tokio::test]
    #[ignore] // Ignore by default as it requires network access
    async fn test_tycho_integration() {
        // Get API key from environment
        let api_key = env::var("TYCHO_API_KEY").ok();
        if api_key.is_none() {
            println!("Skipping test: TYCHO_API_KEY not set");
            return;
        }
        
        // Create price quoter from Tycho
        let quoter = PriceQuoter::from_tycho(
            "https://tycho-beta.propellerheads.xyz",
            api_key
        ).await.expect("Failed to create price quoter");
        
        // Verify we have tokens and pools
        let tokens = quoter.get_storage().get_all_tokens();
        let pools = quoter.get_storage().get_all_pools();
        
        assert!(!tokens.is_empty(), "No tokens loaded from Tycho");
        assert!(!pools.is_empty(), "No pools loaded from Tycho");
        
        println!("Loaded {} tokens and {} pools from Tycho", tokens.len(), pools.len());
    }
}
