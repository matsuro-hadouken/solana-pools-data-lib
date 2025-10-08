use solana_pools_data_lib::*;

/// Explain delegation states in stake accounts
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== DELEGATION STATES EXPLAINED ===\n");
    
    let client = PoolsDataClient::builder()
        .rate_limit(5)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("🔍 Stake accounts can be in different states:\n");
    println!("1. 📋 INITIALIZED: Created but not delegated yet");
    println!("2. ✅ DELEGATED: Actively staked to a validator");
    println!("3. 🔄 DEACTIVATING: Being withdrawn from validator\n");
    
    match client.fetch_pools(&["jito"]).await {
        Ok(pools) => {
            if let Some((pool_name, pool_data)) = pools.iter().next() {
                println!("Pool: {} ({} total accounts)", pool_name, pool_data.stake_accounts.len());
                
                // Find examples of different states
                let mut initialized_account = None;
                let mut delegated_account = None;
                let mut deactivating_account = None;
                
                for account in &pool_data.stake_accounts {
                    match (&account.delegation, &account.stake_type) {
                        (None, stake_type) if stake_type == "initialized" => {
                            if initialized_account.is_none() {
                                initialized_account = Some(account);
                            }
                        }
                        (Some(delegation), _) if delegation.deactivation_epoch == u64::MAX => {
                            if delegated_account.is_none() {
                                delegated_account = Some(account);
                            }
                        }
                        (Some(delegation), _) if delegation.deactivation_epoch != u64::MAX => {
                            if deactivating_account.is_none() {
                                deactivating_account = Some(account);
                            }
                        }
                        _ => {}
                    }
                }
                
                // Show examples
                if let Some(account) = initialized_account {
                    println!("📋 INITIALIZED ACCOUNT (delegation: null):");
                    println!("   Pubkey: {}", account.pubkey);
                    println!("   Lamports: {}", account.lamports);
                    println!("   State: {}", account.stake_type);
                    println!("   Delegation: {:?}", account.delegation);
                    println!("   ➡️  This account is created but not yet staked to any validator\n");
                }
                
                if let Some(account) = delegated_account {
                    println!("✅ DELEGATED ACCOUNT (delegation: populated):");
                    println!("   Pubkey: {}", account.pubkey);
                    println!("   Lamports: {}", account.lamports);
                    println!("   State: {}", account.stake_type);
                    if let Some(delegation) = &account.delegation {
                        println!("   Delegation:");
                        println!("     ├─ Validator: {}", delegation.validator);
                        println!("     ├─ Stake: {} lamports", delegation.stake_lamports);
                        println!("     ├─ Active since epoch: {}", delegation.activation_epoch);
                        println!("     └─ Deactivation epoch: {}", delegation.deactivation_epoch);
                    }
                    println!("   ➡️  This account is actively staking to a validator\n");
                }
                
                if let Some(account) = deactivating_account {
                    println!("🔄 DEACTIVATING ACCOUNT:");
                    println!("   Pubkey: {}", account.pubkey);
                    if let Some(delegation) = &account.delegation {
                        println!("   Deactivating at epoch: {}", delegation.deactivation_epoch);
                        println!("   ➡️  This stake is being withdrawn from the validator\n");
                    }
                }
                
                // Statistics
                let initialized_count = pool_data.stake_accounts.iter()
                    .filter(|a| a.delegation.is_none())
                    .count();
                let delegated_count = pool_data.stake_accounts.iter()
                    .filter(|a| a.delegation.as_ref().map_or(false, |d| d.deactivation_epoch == u64::MAX))
                    .count();
                let deactivating_count = pool_data.stake_accounts.iter()
                    .filter(|a| a.delegation.as_ref().map_or(false, |d| d.deactivation_epoch != u64::MAX))
                    .count();
                
                println!("📊 ACCOUNT STATE BREAKDOWN:");
                println!("   📋 Initialized (delegation: null): {}", initialized_count);
                println!("   ✅ Actively delegated: {}", delegated_count);
                println!("   🔄 Deactivating: {}", deactivating_count);
                println!("   📝 Total: {}", pool_data.stake_accounts.len());
                
                println!("\n✅ EXPLANATION:");
                println!("• delegation: null = Account not currently staked");
                println!("• delegation: {{...}} = Account actively staking");
                println!("• This is normal - pools manage accounts in different states");
                println!("• Initialized accounts are reserves ready for delegation");
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    Ok(())
}