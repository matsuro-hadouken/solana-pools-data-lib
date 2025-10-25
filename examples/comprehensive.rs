use solana_pools_data_lib::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Fetch all supported pools with proper configuration
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    println!("Comprehensive pools data fetch\n");

    let available_pools = PoolsDataClient::list_available_pools();
    println!(
        "Fetching data for {} supported pools\n",
        available_pools.len()
    );

    let mut successful = 0;
    let mut failed = 0;

    let client = PoolsDataClient::builder()
        .rate_limit(5)
        .timeout(10)
        .retry_attempts(3)
        .retry_base_delay(2000)
        .max_concurrent_requests(5)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    for (index, pool_info) in available_pools.iter().enumerate() {
    println!("{}/{}: Fetching {}...", index + 1, available_pools.len(), pool_info.name);
    match client.fetch_pools(&[&pool_info.name]).await {
            Ok(pools) => {
                if let Some((_name, data)) = pools.iter().next() {
                    successful += 1;
                    let stats = &data.statistics;
                    println!(
                        "   Success: validators={}, total_accounts={}, active_accounts={}, deactivating_accounts={}, deactivated_accounts={}, total_lamports={}, active_stake_lamports={}, deactivating_stake_lamports={}, deactivated_stake_lamports={}, validator_count={}, active_stake_SOL={:.2}",
                        data.validator_distribution.len(),
                        stats.total_accounts,
                        stats.active_accounts,
                        stats.deactivating_accounts,
                        stats.deactivated_accounts,
                        stats.total_lamports,
                        stats.active_stake_lamports,
                        stats.deactivating_stake_lamports,
                        stats.deactivated_stake_lamports,
                        stats.validator_count,
                        stats.active_stake_lamports as f64 / 1e9
                    );
                }
            }
            Err(e) => {
                failed += 1;
                println!("   Failed: {}", e);
            }
        }

        if index < available_pools.len() - 1 {
            sleep(Duration::from_secs(8)).await;
        }
    }

    println!("\nSummary:");
    println!("  Successful: {}", successful);
    println!("  Failed: {}", failed);
    println!(
        "  Success rate: {:.1}%",
        (successful as f64 / available_pools.len() as f64) * 100.0
    );

    let duration = start.elapsed();
    println!("\nExecution time: {:.2?}", duration);

    Ok(())
}
