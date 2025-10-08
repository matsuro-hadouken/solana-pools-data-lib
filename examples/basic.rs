use solana_pools_data_lib::*;
use std::time::Duration;
use tokio::time::sleep;

/// Configuration examples showing rate limiting, timeouts, and delays
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Configuration Examples\n");
    
    // Production configuration - conservative settings for public RPC
    println!("1. Production configuration:");
    let production_client = PoolsDataClient::builder()
        .rate_limit(5)                    // 5 requests per second
        .timeout(10)                      // 10 second timeout
        .retry_attempts(3)                // 3 retries on failure
        .retry_base_delay(2000)           // 2 second base retry delay
        .max_concurrent_requests(5)       // Max 5 parallel requests
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    match production_client.fetch_pools(&["marinade"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("   Success: {} - {} validators", name, data.validator_distribution.len());
        }
        Err(e) => println!("   Error: {}", e),
    }

    // 8 second delay between operations
    sleep(Duration::from_secs(8)).await;
    
    // Development configuration - faster settings for testing
    println!("2. Development configuration:");
    let dev_client = PoolsDataClient::builder()
        .no_rate_limit()                  // No rate limiting
        .timeout(5)                       // 5 second timeout
        .retry_attempts(2)                // 2 retries
        .max_concurrent_requests(10)      // Higher concurrency
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    match dev_client.fetch_pools(&["jito"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("   Success: {} - {} validators", name, data.validator_distribution.len());
        }
        Err(e) => println!("   Error: {}", e),
    }

    sleep(Duration::from_secs(8)).await;
    
    // Premium RPC configuration - aggressive settings for paid endpoints
    println!("3. Premium RPC configuration:");
    let premium_client = PoolsDataClient::builder()
        .rate_limit(20)                   // 20 requests per second
        .timeout(3)                       // 3 second timeout
        .retry_attempts(1)                // Minimal retries
        .max_concurrent_requests(15)      // High concurrency
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    match premium_client.fetch_pools(&["socean"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("   Success: {} - {} validators", name, data.validator_distribution.len());
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\nConfiguration options:");
    println!("  rate_limit(n) - Requests per second limit");
    println!("  no_rate_limit() - Remove rate limiting");
    println!("  timeout(seconds) - Request timeout");
    println!("  retry_attempts(n) - Number of retries");
    println!("  retry_base_delay(ms) - Delay between retries");
    println!("  max_concurrent_requests(n) - Parallel request limit");
    println!("  Use 8 second delays between operations");
    
    Ok(())
}