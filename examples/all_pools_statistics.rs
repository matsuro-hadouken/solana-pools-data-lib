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

    let all_pools = PoolsDataClient::list_available_pools();
    let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();
    println!("Fetching {} pools...", pool_names.len());
    let pools = client.fetch_pools(&pool_names).await.expect("Fetch failed");
    println!("Fetched {} pools.", pools.len());

    for (pool_name, pool) in pools.iter() {
        let stats = &pool.statistics;
        println!("Pool: {}", pool_name);
        println!("  Total stake accounts: {}", stats.total_accounts);
        println!("  Active accounts: {}", stats.active_accounts);
        println!("    Active stake (lamports): {}", stats.active_stake_lamports);
        println!("    Active stake (SOL): {:.2}", stats.active_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Deactivating accounts: {}", stats.deactivating_accounts);
        println!("    Deactivating stake (lamports): {}", stats.deactivating_stake_lamports);
        println!("    Deactivating stake (SOL): {:.2}", stats.deactivating_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Fully deactivated accounts: {}", stats.deactivated_accounts);
        println!("    Fully deactivated stake (lamports): {}", stats.deactivated_stake_lamports);
        println!("    Fully deactivated stake (SOL): {:.2}", stats.deactivated_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Total lamports in all stake accounts: {}", stats.total_lamports);
        println!("  Total lamports (SOL): {:.2}", stats.total_lamports as f64 / 1_000_000_000.0);
        println!("---");
    }
}
