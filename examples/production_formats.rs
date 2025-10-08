use solana_pools_data_lib::*;

/// Clean demonstration of the two output formats
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Clean Output Formats ===\n");
    
    let client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    let test_pools = ["jito", "marinade"];
    
    println!("🔍 Testing with pools: {:?}\n", test_pools);
    
    // Production format - clean and safe
    println!("📊 1. PRODUCTION FORMAT (Clean data for databases/APIs)");
    match client.fetch_pools(&test_pools).await {
        Ok(production_data) => {
            let json = serde_json::to_string_pretty(&production_data)?;
            println!("✅ Success! Size: {} bytes", json.len());
            
            if let Some((name, pool_data)) = production_data.iter().next() {
                println!("   Pool: {}", name);
                println!("   Authority: {}", pool_data.authority);
                println!("   Accounts: {}", pool_data.stake_accounts.len());
                println!("   Validators: {}", pool_data.validator_distribution.len());
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    println!();
    
    // Debug format - complete data
    println!("🔍 2. DEBUG FORMAT (Complete RPC data)");
    match client.fetch_pools_debug(&test_pools).await {
        Ok(debug_result) => {
            let json = serde_json::to_string_pretty(&debug_result)?;
            println!("✅ Success! Size: {} bytes", json.len());
            println!("   Successful: {}", debug_result.successful.len());
            println!("   Failed: {}", debug_result.failed.len());
            
            if let Some((name, pool_data)) = debug_result.successful.iter().next() {
                println!("   Pool: {}", name);
                println!("   Authority: {}", pool_data.authority);
                println!("   Accounts: {}", pool_data.stake_accounts.len());
                
                if let Some(account) = pool_data.stake_accounts.first() {
                    println!("   First account has {} fields", 
                        serde_json::to_value(account)?.as_object().unwrap().len());
                }
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    println!("\n✅ Clean and simple - no confusing warnings or complex choices!");
    
    Ok(())
}