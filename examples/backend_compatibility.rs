use solana_pools_data_lib::*;

/// Simple backend compatibility demonstration
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Backend Database Integration ===\n");
    
    let client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("🏭 Production Format - Safe for Database Storage");
    
    match client.fetch_pools(&["jito"]).await {
        Ok(production_data) => {
            println!("✅ Fetched {} pools", production_data.len());
            
            // Show consistent schema
            for (pool_name, pool_data) in &production_data {
                println!("\nPool: {}", pool_name);
                println!("├─ Authority: {}", pool_data.authority);
                println!("├─ Total Accounts: {}", pool_data.stake_accounts.len());
                println!("├─ Total Lamports: {}", pool_data.statistics.total_lamports);
                println!("└─ Validators: {}", pool_data.validator_distribution.len());
                
                // Show first stake account structure
                if let Some(account) = pool_data.stake_accounts.first() {
                    println!("\nFirst Stake Account Structure:");
                    println!("├─ pubkey: {}", account.pubkey);
                    println!("├─ lamports: {}", account.lamports);
                    println!("├─ stake_type: {}", account.stake_type);
                    println!("├─ authority: {{staker: {}, withdrawer: {}}}", 
                        account.authority.staker, account.authority.withdrawer);
                    println!("├─ lockup: {{custodian: {}, epoch: {}, unix_timestamp: {}}}", 
                        account.lockup.custodian, account.lockup.epoch, account.lockup.unix_timestamp);
                    if let Some(delegation) = &account.delegation {
                        println!("└─ delegation: {{validator: {}, stake_lamports: {}, activation_epoch: {}}}", 
                            delegation.validator, delegation.stake_lamports, delegation.activation_epoch);
                    } else {
                        println!("└─ delegation: null");
                    }
                }
                break; // Only show first pool
            }
            
            // JSON serialization test
            let json = serde_json::to_string_pretty(&production_data)?;
            println!("\n💾 JSON Size: {} bytes", json.len());
            println!("✅ Safe to store in database - consistent schema every time!");
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    Ok(())
}