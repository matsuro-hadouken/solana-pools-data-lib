use solana_pools_data_lib::*;
use serde_json;

/// Show actual data samples and structure
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== ACTUAL DATA SAMPLES ===\n");
    
    let client = PoolsDataClient::builder()
        .rate_limit(5) // Conservative for demo
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Get a small sample
    match client.fetch_pools(&["jito"]).await {
        Ok(production_data) => {
            if let Some((pool_name, pool_data)) = production_data.iter().next() {
                println!("🎯 PRODUCTION FORMAT SAMPLE (Pool: {})", pool_name);
                println!("Authority: {}", pool_data.authority);
                println!("Total Accounts: {}", pool_data.stake_accounts.len());
                println!("Total Validators: {}", pool_data.validator_distribution.len());
                
                // Show first stake account structure
                if let Some(account) = pool_data.stake_accounts.first() {
                    println!("\n📋 STAKE ACCOUNT STRUCTURE:");
                    let account_json = serde_json::to_string_pretty(account)?;
                    println!("{}", account_json);
                }
                
                // Show validator distribution sample
                if let Some((validator, stake_info)) = pool_data.validator_distribution.iter().next() {
                    println!("\n🗳️  VALIDATOR DISTRIBUTION SAMPLE:");
                    println!("Validator: {}", validator);
                    println!("Total Delegated: {} lamports", stake_info.total_delegated);
                    println!("Account Count: {}", stake_info.account_count);
                    println!("Stake Accounts: {:?}", stake_info.accounts.iter().take(3).collect::<Vec<_>>());
                }
                
                // Show statistics
                println!("\n📊 POOL STATISTICS:");
                let stats_json = serde_json::to_string_pretty(&pool_data.statistics)?;
                println!("{}", stats_json);
                
                println!("\n💾 JSON SIZE COMPARISON:");
                let production_json = serde_json::to_string(&pool_data)?;
                println!("Production JSON size: {} bytes", production_json.len());
                
                // Get debug format for comparison
                if let Ok(debug_result) = client.fetch_pools_debug(&[pool_name]).await {
                    if let Some((_, debug_pool)) = debug_result.successful.iter().next() {
                        let debug_json = serde_json::to_string(debug_pool)?;
                        println!("Debug JSON size: {} bytes", debug_json.len());
                        let savings = debug_json.len() - production_json.len();
                        let percent = (savings as f64 / debug_json.len() as f64) * 100.0;
                        println!("Space saved: {} bytes ({:.1}%)", savings, percent);
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Error fetching data: {}", e);
            println!("Note: This requires internet connection to Solana RPC");
        }
    }
    
    println!("\n✅ This shows the actual clean, structured data your applications will receive!");
    Ok(())
}