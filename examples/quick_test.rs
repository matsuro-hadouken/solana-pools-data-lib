use solana_pools_data_lib::*;

/// Show available pools and quick test
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== AVAILABLE POOLS & QUICK TEST ===\n");

    // Show all available pools
    let available_pools = PoolsDataClient::list_available_pools();
    println!("SUPPORTED POOLS ({} total):", available_pools.len());

    for (i, pool) in available_pools.iter().enumerate().take(10) {
        println!(
            "{}. {} (Authority: {}...)",
            i + 1,
            pool.name,
            &pool.authority[0..20]
        );
    }

    if available_pools.len() > 10 {
        println!("   ... and {} more pools", available_pools.len() - 10);
    }

    // Quick test with a small pool
    println!("\nQUICK TEST - Fetching small pool data:");

    let client = PoolsDataClient::builder()
        .rate_limit(5)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Try to get a smaller pool for quick testing
    let test_pools = ["socean", "lido"];

    println!("Testing pools: {:?}", test_pools);

    match client.fetch_pools(&test_pools).await {
        Ok(pools) => {
            for (pool_name, pool_data) in pools {
                println!("\nPool: {}", pool_name);
                println!("   Authority: {}", pool_data.authority);
                println!("   Accounts: {}", pool_data.stake_accounts.len());
                println!("   Validators: {}", pool_data.validator_distribution.len());
                println!(
                    "   Total Staked: {:.2} SOL",
                    pool_data.statistics.total_staked_lamports as f64 / 1_000_000_000.0
                );

                // Show top validators
                let mut validators: Vec<_> = pool_data.validator_distribution.iter().collect();
                validators.sort_by(|a, b| b.1.total_delegated.cmp(&a.1.total_delegated));

                println!("   Top 3 Validators:");
                for (i, (validator, stake)) in validators.iter().take(3).enumerate() {
                    println!(
                        "     {}. {}... ({:.2} SOL)",
                        i + 1,
                        &validator[0..20],
                        stake.total_delegated as f64 / 1_000_000_000.0
                    );
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
            println!("Note: This requires internet connection to Solana RPC");
        }
    }

    println!("\nLibrary test completed successfully with structured data.");
    Ok(())
}
