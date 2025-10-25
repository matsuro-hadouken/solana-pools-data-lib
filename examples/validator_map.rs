use solana_pools_data_lib::*;
// ...existing code...

fn main() {
    // Use a runtime for async client
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // Build client (adjust RPC URL as needed)
        let client = PoolsDataClient::builder()
            .rate_limit(5)
            .timeout(10)
            .retry_attempts(3)
            .retry_base_delay(2000)
            .max_concurrent_requests(5)
            .build("https://api.mainnet-beta.solana.com")
            .and_then(PoolsDataClient::from_config)
            .expect("Failed to build client");

        // Choose pools to fetch (e.g., just "jito" for demo)
        let pool_names = vec!["jito"];
        let pools = client.fetch_pools(&pool_names).await.expect("Fetch failed");

        // Print only the first pool and first stake account with delegation, showing all available fields
        if let Some(pool) = pools.values().next() {
            println!("Pool: {}", pool.pool_name);
            println!("  Authority: {}", pool.authority);
            println!("  Fetched At: {}", pool.fetched_at);
            println!("  Pool Statistics:");
            println!("    Total Accounts: {}", pool.statistics.total_accounts);
            println!("    Total Lamports: {}", pool.statistics.total_lamports);
            println!("    Total Staked Lamports: {}", pool.statistics.active_stake_lamports);
            println!("    Active Stake Accounts: {}", pool.statistics.active_accounts);
            println!("    Deactivating Stake Accounts: {}", pool.statistics.deactivating_accounts);
            println!("    Validator Count: {}", pool.statistics.validator_count);

            // Find the first stake account with a delegation
            if let Some(stake_account) = pool.stake_accounts.iter().find(|sa| sa.delegation.is_some()) {
                println!("  Stake Account: {}", stake_account.pubkey);
                println!("    Lamports: {}", stake_account.lamports);
                println!("    Stake Type: {}", stake_account.stake_type);
                if let Some(delegation) = &stake_account.delegation {
                    println!("    Delegation:");
                    println!("      Validator: {}", delegation.validator);
                    println!("      Stake Lamports: {}", delegation.stake_lamports);
                    println!("      Activation Epoch: {}", delegation.activation_epoch);
                    println!("      Deactivation Epoch: {}", delegation.deactivation_epoch);
                    println!("      Last Epoch Credits Cumulative: {}", delegation.last_epoch_credits_cumulative);
                } else {
                    println!("    Delegation: None");
                }
                println!("    Authority:");
                println!("      Staker: {}", stake_account.authority.staker);
                println!("      Withdrawer: {}", stake_account.authority.withdrawer);
                // Lockup details omitted to avoid bloat
            }
        }
    });
}
