use pools_data_lib::PoolsDataClient;

/// Simulate a typical backend developer's database schema
/// This is what they might store in their database
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ApiStakeAccount {
    pub pubkey: String,
    pub lamports: u64,
    pub stake_type: String,
    pub validator: Option<String>,
    pub stake_lamports: Option<u64>,
    pub activation_epoch: Option<u64>,
    pub deactivation_epoch: Option<u64>,
    pub credits_observed: Option<u64>,
    // Notice: NO lockup or custom_authority fields
    // Developer never heard about them and didn't include in their schema
}

/// Simulate developer's API response structure
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ApiPoolResponse {
    pub pool_name: String,
    pub authority: String,
    pub stake_accounts: Vec<ApiStakeAccount>,
    pub total_staked_sol: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Backend Developer Compatibility Test ===\n");
    
    // Simulate developer building their API
    println!("1. Developer builds API using optimized format (no lockups visible initially)");
    
    let client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Test 1: Normal case - no lockups present (what developer expects)
    println!("\n📊 Test 1: Normal operation (no lockups)");
    let optimized_result = client.fetch_pools_optimized(&["jito"]).await;
    
    match optimized_result {
        Ok(pools) => {
            for (pool_name, pool_data) in pools.iter().take(1) {
                println!("   ✅ Pool: {}", pool_name);
                println!("   ✅ Accounts: {}", pool_data.stake_accounts.len());
                
                // Simulate developer converting to their API format
                let mut api_accounts = Vec::new();
                let mut lockup_encountered = false;
                let mut custom_authority_encountered = false;
                
                for account in &pool_data.stake_accounts {
                    // Check if lockup field exists (the dangerous scenario)
                    if account.lockup.is_some() {
                        lockup_encountered = true;
                        println!("   ⚠️  LOCKUP DETECTED: {:?}", account.lockup);
                    }
                    
                    // Check if custom_authority exists
                    if account.custom_authority.is_some() {
                        custom_authority_encountered = true;
                        println!("   ⚠️  CUSTOM AUTHORITY DETECTED: {:?}", account.custom_authority);
                    }
                    
                    // Try to convert to developer's format (this could fail!)
                    let api_account = ApiStakeAccount {
                        pubkey: account.pubkey.clone(),
                        lamports: account.lamports,
                        stake_type: account.stake_type.clone(),
                        validator: account.delegation.as_ref().map(|d| d.validator.clone()),
                        stake_lamports: account.delegation.as_ref().map(|d| d.stake_lamports),
                        activation_epoch: account.delegation.as_ref().map(|d| d.activation_epoch),
                        deactivation_epoch: account.delegation.as_ref().map(|d| d.deactivation_epoch),
                        credits_observed: account.delegation.as_ref().map(|d| d.credits_observed),
                        // Note: lockup and custom_authority are ignored/lost!
                    };
                    
                    api_accounts.push(api_account);
                }
                
                // Test JSON serialization (what goes to database/API)
                let api_response = ApiPoolResponse {
                    pool_name: pool_name.clone(),
                    authority: pool_data.authority.clone(),
                    stake_accounts: api_accounts,
                    total_staked_sol: pool_data.statistics.total_staked_lamports as f64 / 1e9,
                };
                
                let json_size = serde_json::to_string(&api_response)?.len();
                println!("   📊 API Response Size: {} bytes", json_size);
                
                if lockup_encountered {
                    println!("   🚨 CRITICAL: Lockup data lost in conversion!");
                }
                if custom_authority_encountered {
                    println!("   🚨 CRITICAL: Custom authority data lost in conversion!");
                }
                
                if !lockup_encountered && !custom_authority_encountered {
                    println!("   ✅ Safe: No unexpected fields encountered");
                }
            }
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    
    // Test 2: Simulate the nightmare scenario - lockups suddenly appear
    println!("\n🔥 Test 2: Disaster scenario - lockups suddenly appear in production");
    println!("   (Simulating what happens when a pool starts using lockups)");
    
    // Let's examine what happens with full format too
    println!("\n📊 Test 3: Full format comparison");
    let full_result = client.fetch_pools(&["jito"]).await;
    
    match full_result {
        Ok(pools) => {
            for (_pool_name, pool_data) in pools.successful.iter().take(1) {
                println!("   📊 Full format accounts: {}", pool_data.stake_accounts.len());
                
                let mut has_actual_lockups = 0;
                let mut has_custom_authorities = 0;
                
                for account in &pool_data.stake_accounts {
                    if !account.lockup.is_default_lockup() {
                        has_actual_lockups += 1;
                    }
                    if !account.authorized.is_unified_authority() {
                        has_custom_authorities += 1;
                    }
                }
                
                println!("   📊 Accounts with actual lockups: {}", has_actual_lockups);
                println!("   📊 Accounts with custom authorities: {}", has_custom_authorities);
                
                if has_actual_lockups > 0 || has_custom_authorities > 0 {
                    println!("   ⚠️  These would appear as NEW FIELDS in optimized format!");
                    println!("   ⚠️  Developer's API/database could break!");
                }
            }
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    
    println!("\n=== COMPATIBILITY ANALYSIS ===");
    println!("✅ SAFE: Fields are Optional<T> - won't break deserialization");
    println!("⚠️  RISK: New fields appear suddenly in production");
    println!("⚠️  RISK: Data loss if developer ignores optional fields");
    println!("⚠️  RISK: Database schema updates needed for new fields");
    
    println!("\n💡 RECOMMENDATION: Always use full format for production stability");
    println!("💡 RECOMMENDATION: Optimized format only for public APIs with caching");
    
    Ok(())
}