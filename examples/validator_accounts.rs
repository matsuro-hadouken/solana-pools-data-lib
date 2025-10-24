use solana_pools_data_lib::*;

#[tokio::main]
async fn main() {
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

    // Choose pool and validator for demo
    let pool_names = vec!["foundation"];
    let pools = client.fetch_pools(&pool_names).await.expect("Fetch failed");

    // Pick the first pool
        if let Some(pool) = pools.values().next() {
            // Use the Solana Foundation pool and the specified validator
            let validator = "5iZ5PQPy5Z9XDnkfoWPi6nvUgtxWnRFwZ36WaftPuaVM";
            println!("Validator: {}", validator);
            println!("Accounts delegated to this validator (open and closed):");
            let mut found = false;
            for account in pool.stake_accounts.iter().filter(|sa| {
                sa.delegation.as_ref().map(|d| d.validator == validator).unwrap_or(false)
            }) {
                found = true;
                println!("  Account: {}", account.pubkey);
                println!("    Lamports: {}", account.lamports);
                println!("    Stake Type: {}", account.stake_type);
                if let Some(delegation) = &account.delegation {
                    println!("    Delegation:");
                    println!("      Stake Lamports: {}", delegation.stake_lamports);
                    println!("      Activation Epoch: {}", delegation.activation_epoch);
                    println!("      Deactivation Epoch: {}", delegation.deactivation_epoch);
                    println!("      Last Epoch Credits Cumulative: {}", delegation.last_epoch_credits_cumulative);
                } else {
                    println!("    Delegation: None");
                }
                println!("    Authority:");
                println!("      Staker: {}", account.authority.staker);
                println!("      Withdrawer: {}", account.authority.withdrawer);
            }
            if !found {
                println!("No accounts found for this validator in the Foundation pool.");
            }
        } else {
            println!("No Foundation pool found.");
        }
}
