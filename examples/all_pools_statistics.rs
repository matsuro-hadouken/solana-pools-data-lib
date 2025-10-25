use solana_pools_data_lib::*;

#[tokio::main]
async fn main() {
    let client = PoolsDataClient::builder()
        .rate_limit(5)
        .timeout(10)
        .retry_attempts(3)
        .retry_base_delay(2000)
        .max_concurrent_requests(5)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)
        .expect("Failed to build client");

    // Fetch current epoch from Solana RPC using HTTP
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getEpochInfo"
    });
    let resp = reqwest::Client::new()
        .post(rpc_url)
        .json(&payload)
        .send()
        .await
        .expect("Failed to fetch epoch info");
    let json: serde_json::Value = resp.json().await.expect("Invalid epoch info response");
    let current_epoch = json["result"]["epoch"].as_u64().expect("No epoch in response");
    println!("Current Solana epoch: {}", current_epoch);

    // Check all available pools for account states
    let all_pools = PoolsDataClient::list_available_pools();
    let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();
    println!("Fetching {} pools...", pool_names.len());
    let pools = client.fetch_pools(&pool_names).await.expect("Fetch failed");
    println!("Fetched {} pools.", pools.len());

    for (pool_name, pool) in pools.iter() {
        let mut total_accounts = 0;
        let mut active_accounts = 0;
        let mut deactivated_accounts = 0;
        let mut other_states = 0;
        let mut active_stake: u64 = 0;
        let mut deactivating_stake: u64 = 0;
        let mut fully_deactivated_stake: u64 = 0;
        let mut total_lamports: u64 = 0;
        for account in pool.stake_accounts.iter() {
            total_lamports += account.lamports;
            if let Some(delegation) = &account.delegation {
                total_accounts += 1;
                if delegation.deactivation_epoch == u64::MAX {
                    active_accounts += 1;
                    active_stake += delegation.stake_lamports;
                } else if delegation.deactivation_epoch > current_epoch {
                    other_states += 1;
                    deactivating_stake += delegation.stake_lamports;
                } else if delegation.deactivation_epoch <= current_epoch {
                    deactivated_accounts += 1;
                    fully_deactivated_stake += delegation.stake_lamports;
                }
            }
        }
        println!("Pool: {}", pool_name);
        println!("  Total stake accounts: {}", total_accounts);
        println!("  Active accounts: {}", active_accounts);
        println!("    Active stake (lamports): {}", active_stake);
        println!("    Active stake (SOL): {:.2}", active_stake as f64 / 1_000_000_000.0);
        println!("  Deactivating accounts: {}", other_states);
        println!("    Deactivating stake (lamports): {}", deactivating_stake);
        println!("    Deactivating stake (SOL): {:.2}", deactivating_stake as f64 / 1_000_000_000.0);
        println!("  Fully deactivated accounts: {}", deactivated_accounts);
        println!("    Fully deactivated stake (lamports): {}", fully_deactivated_stake);
        println!("    Fully deactivated stake (SOL): {:.2}", fully_deactivated_stake as f64 / 1_000_000_000.0);
        println!("  Total lamports in all stake accounts: {}", total_lamports);
        println!("  Total lamports (SOL): {:.2}", total_lamports as f64 / 1_000_000_000.0);
        println!("---");
    }
}
