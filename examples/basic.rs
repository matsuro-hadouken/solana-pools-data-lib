use solana_pools_data_lib::*;
use std::time::Duration;
use tokio::time::sleep;

/// Complete configuration reference - all available options
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Complete Configuration Reference\n");
    
    // Configuration 1: All options with conservative values
    println!("1. Conservative configuration (all options):");
    let conservative = PoolsDataClient::builder()
        .rate_limit(3)                    // 3 requests per second
        .timeout(15)                      // 15 second timeout  
        .retry_attempts(5)                // 5 retries on failure
        .retry_base_delay(3000)           // 3 second base retry delay (exponential backoff)
        .max_concurrent_requests(3)       // Max 3 parallel requests
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    match conservative.fetch_pools(&["socean"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("   Success: {} - {} validators", name, data.validator_distribution.len());
        }
        Err(e) => println!("   Error: {}", e),
    }

    sleep(Duration::from_secs(8)).await;
    
    // Configuration 2: Aggressive settings for premium RPC
    println!("2. Aggressive configuration (premium RPC):");
    let aggressive = PoolsDataClient::builder()
        .rate_limit(25)                   // 25 requests per second
        .timeout(3)                       // 3 second timeout
        .retry_attempts(1)                // Only 1 retry
        .retry_base_delay(500)            // 500ms retry delay
        .max_concurrent_requests(20)      // 20 parallel requests
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    match aggressive.fetch_pools(&["marinade"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("   Success: {} - {} validators", name, data.validator_distribution.len());
        }
        Err(e) => println!("   Error: {}", e),
    }

    sleep(Duration::from_secs(8)).await;
    
    // Configuration 3: No rate limiting for development
    println!("3. Development configuration (no limits):");
    let development = PoolsDataClient::builder()
        .no_rate_limit()                  // No rate limiting
        .timeout(8)                       // 8 second timeout
        .retry_attempts(2)                // 2 retries
        .retry_base_delay(1000)           // 1 second retry delay
        .max_concurrent_requests(15)      // 15 parallel requests
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    match development.fetch_pools(&["jito"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("   Success: {} - {} validators", name, data.validator_distribution.len());
        }
        Err(e) => println!("   Error: {}", e),
    }

    sleep(Duration::from_secs(8)).await;
    
    // Configuration 4: Preset configurations
    println!("4. Preset public RPC configuration:");
    let preset = PoolsDataClient::builder()
        .public_rpc_config()              // Use preset public RPC settings
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    match preset.fetch_pools(&["lido"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("   Success: {} - {} validators", name, data.validator_distribution.len());
        }
        Err(e) => println!("   Error: {}", e),
    }

    sleep(Duration::from_secs(8)).await;
    
    // Configuration 5: Production vs Debug API comparison
    println!("5. Production vs Debug API formats:");
    let api_test = PoolsDataClient::builder()
        .rate_limit(5)
        .timeout(10)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Production API - clean data
    println!("   Production API (clean data):");
    match api_test.fetch_pools(&["socean"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("     Pool: {}", name);
            println!("     Format: Production (database-ready)");
            println!("     Fields: authority, validator_distribution, stake_accounts, statistics");
            println!("     Example validator: {}", data.validator_distribution.keys().next().unwrap());
            println!("     Total staked: {:.2} SOL", data.statistics.total_staked_lamports as f64 / 1e9);
        }
        Err(e) => println!("     Error: {}", e),
    }

    sleep(Duration::from_secs(8)).await;

    // Debug API - full RPC data
    println!("   Debug API (full RPC data):");
    match api_test.fetch_pools_debug(&["socean"]).await {
        Ok(result) => {
            if let Some((name, data)) = result.successful.iter().next() {
                println!("     Pool: {}", name);
                println!("     Format: Debug (full RPC response)");
                println!("     Fields: authority, validator_distribution, stake_accounts, statistics + raw RPC data");
                println!("     Example validator: {}", data.validator_distribution.keys().next().unwrap());
                println!("     Total staked: {:.2} SOL", data.statistics.total_staked_lamports as f64 / 1e9);
                
                // Show the actual data structure difference
                println!("     Debug data includes:");
                println!("       - Raw stake pool account data");
                println!("       - All RPC response fields");
                println!("       - Unprocessed validator info");
                println!("       - Complete account states");
            }
            if !result.failed.is_empty() {
                println!("     Failed pools: {:?}", result.failed.keys().collect::<Vec<_>>());
            }
        }
        Err(e) => println!("     Error: {}", e),
    }

    println!("\nComplete configuration options:");
    println!("  rate_limit(n)               - Requests per second (1-50)");
    println!("  no_rate_limit()             - Remove all rate limiting");
    println!("  timeout(seconds)            - Request timeout (3-30 seconds)");
    println!("  retry_attempts(n)           - Number of retries (0-10)");
    println!("  retry_base_delay(ms)        - Base retry delay in milliseconds");
    println!("  max_concurrent_requests(n)  - Parallel request limit (1-50)");
    println!("  public_rpc_config()         - Preset for public RPC endpoints");
    println!("  private_rpc_config()        - Preset for private RPC endpoints");
    
    println!("\nRecommended values by use case:");
    println!("  Production:   rate_limit(5), timeout(10), retry_attempts(3)");
    println!("  Development:  no_rate_limit(), timeout(5), retry_attempts(2)");
    println!("  Premium RPC:  rate_limit(20), timeout(3), retry_attempts(1)");
    println!("  Mobile/Slow:  rate_limit(2), timeout(15), retry_attempts(8)");
    
    println!("\nAPI formats:");
    println!("  fetch_pools()        - Production: Clean, processed data for databases");
    println!("  fetch_pools_debug()  - Debug: Raw RPC response with all original fields");
    println!("                        Use production for apps, debug for inspection/troubleshooting");
    
    println!("\nAlways use 8-second delays between operations");
    
    Ok(())
}