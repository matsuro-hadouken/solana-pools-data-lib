use pools_data_lib::*;

/// Demonstrate data organization and field mapping from RPC response
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== POOLS DATA ORGANIZATION ANALYSIS ===\n");
    
    let client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("🔍 1. RPC RESPONSE MAPPING");
    println!("From your RPC example, here's how we map fields:\n");
    
    println!("RPC Response → Our Library:");
    println!("┌─ 'jsonrpc.result[].account.data.parsed.info.meta.authorized' → authority.staker/withdrawer");
    println!("├─ 'jsonrpc.result[].account.data.parsed.info.meta.lockup' → lockup.custodian/epoch/unix_timestamp");
    println!("├─ 'jsonrpc.result[].account.data.parsed.info.stake.delegation' → delegation.*");
    println!("├─ 'jsonrpc.result[].account.lamports' → lamports");
    println!("├─ 'jsonrpc.result[].pubkey' → pubkey");
    println!("└─ Static fields (owner, executable, program, space) → REMOVED in production format\n");

    println!("📊 2. POOL DATA ORGANIZATION");
    println!("Testing with 3 pools to show organization...\n");

    // Test with a few pools to show organization
    let test_pools = ["jito", "marinade", "lido"];
    let result = client.fetch_pools(&test_pools).await?;

    println!("📋 RESULT STRUCTURE:");
    println!("   successful: {} pools", result.successful.len());
    println!("   failed: {} pools", result.failed.len());
    println!("   summary: {} total pools processed\n", result.summary.successful_pools);

    // Show detailed organization for each pool
    for (pool_name, pool_data) in &result.successful {
        println!("🏊 POOL: {} (Authority: {})", pool_name, &pool_data.authority[..8]);
        println!("   📊 Statistics:");
        println!("      • Total accounts: {}", pool_data.statistics.total_stake_accounts);
        println!("      • Active accounts: {}", pool_data.statistics.active_stake_accounts);
        println!("      • Total staked: {:.2} SOL", pool_data.total_staked_sol());
        println!("      • Unique validators: {}", pool_data.statistics.unique_validators);
        
        println!("   🎯 Top 3 Validators by stake:");
        let mut validators: Vec<_> = pool_data.validator_distribution.iter().collect();
        validators.sort_by(|a, b| b.1.total_delegated.cmp(&a.1.total_delegated));
        
        for (i, (validator_pubkey, stake_info)) in validators.iter().take(3).enumerate() {
            println!("      {}. Validator: {}...{}", 
                     i + 1, 
                     &validator_pubkey[..8], 
                     &validator_pubkey[validator_pubkey.len()-8..]);
            println!("         Delegated: {:.2} SOL ({} accounts)", 
                     stake_info.delegated_sol(), 
                     stake_info.account_count);
        }
        
        println!("   📋 Sample Stake Account:");
        if let Some(account) = pool_data.stake_accounts.first() {
            println!("      • Pubkey: {}...{}", &account.pubkey[..8], &account.pubkey[account.pubkey.len()-8..]);
            println!("      • Balance: {:.2} SOL", account.lamports as f64 / 1e9);
            
            if let Some(delegation) = &account.delegation {
                println!("      • Delegated to: {}...{}", &delegation.voter[..8], &delegation.voter[delegation.voter.len()-8..]);
                println!("      • Stake amount: {:.2} SOL", delegation.stake as f64 / 1e9);
                println!("      • Credits observed: {}", delegation.credits_observed);
            }
            
            println!("      • Authority: {} / {}", 
                     if account.authorized.staker == pool_data.authority { "POOL" } else { "CUSTOM" },
                     if account.authorized.withdrawer == pool_data.authority { "POOL" } else { "CUSTOM" });
            
            println!("      • Lockup: {} (epoch: {}, timestamp: {})", 
                     if account.lockup.is_default_lockup() { "NONE" } else { "ACTIVE" },
                     account.lockup.epoch,
                     account.lockup.unix_timestamp);
        }
        println!();
    }

    println!("🏷️  3. POOL IDENTIFICATION");
    println!("Each pool is clearly identified by:");
    println!("   ✅ Pool name (human-readable): 'jito', 'marinade', 'lido'");
    println!("   ✅ Pool authority (unique pubkey): Each pool has unique authority");
    println!("   ✅ Validator breakdown: Clear mapping of which validators each pool uses");
    println!("   ✅ Account organization: All accounts grouped by pool with statistics\n");

    println!("📁 4. PRODUCTION FORMAT ORGANIZATION");
    let production_result = client.fetch_pools_production(&test_pools).await?;
    
    println!("Production format maintains same organization:");
    for (pool_name, pool_data) in production_result.iter().take(1) {
        println!("   Pool: {}", pool_name);
        if let Some(account) = pool_data.stake_accounts.first() {
            let json = serde_json::to_string_pretty(account)?;
            println!("   Sample account structure:");
            println!("{}", json.lines().take(15).collect::<Vec<_>>().join("\n"));
            println!("   ... (showing lockup always included even if default)");
        }
    }

    println!("\n✅ ORGANIZATION SUMMARY:");
    println!("   🎯 Perfect pool identification by name and authority");
    println!("   📊 Clear validator breakdown per pool with stake amounts");
    println!("   📋 All stake accounts grouped by pool with full details");
    println!("   🏷️  Statistics aggregated per pool for easy analysis");
    println!("   🔄 Production format maintains organization with consistent schema");

    Ok(())
}