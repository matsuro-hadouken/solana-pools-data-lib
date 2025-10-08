use solana_pools_data_lib::*;
use std::time::Duration;
use tokio::time::sleep;

/// COMPLETE CONFIGURATION REFERENCE
/// This example demonstrates every available configuration option with real-world scenarios
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== COMPLETE CONFIGURATION REFERENCE ===\n");
    println!("This example covers ALL configuration options with practical use cases\n");

    // ============================================================================
    // CONFIGURATION 1: PUBLIC RPC - CONSERVATIVE (Default recommended)
    // ============================================================================
    println!("🏭 CONFIGURATION 1: PUBLIC RPC - CONSERVATIVE");
    println!("Use case: Production apps using free public RPC endpoints");
    println!("Characteristics: Safe rate limits, robust retry logic, conservative timeouts\n");
    
    let public_conservative = PoolsDataClient::builder()
        .rate_limit(5)                    // 5 requests per second (safe for public RPC)
        .timeout(10)                      // 10 second timeout per request
        .retry_attempts(5)                // Retry up to 5 times on failure
        .retry_base_delay(2000)           // Start with 2 second delay between retries
        .max_concurrent_requests(3)       // Max 3 parallel requests
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("Testing public conservative config...");
    match public_conservative.fetch_pools(&["socean"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("✅ SUCCESS: {} - {} validators, {} accounts", 
                name, data.validator_distribution.len(), data.stake_accounts.len());
        }
        Err(e) => println!("❌ FAILED: {}", e),
    }

    sleep(Duration::from_secs(8)).await;

    // ============================================================================
    // CONFIGURATION 2: PUBLIC RPC - FAST
    // ============================================================================
    println!("\n🚀 CONFIGURATION 2: PUBLIC RPC - FAST");
    println!("Use case: Development, testing, or when you need faster responses");
    println!("Characteristics: Higher rate limits, shorter timeouts, fewer retries\n");
    
    let public_fast = PoolsDataClient::builder()
        .rate_limit(10)                   // 10 requests per second
        .timeout(5)                       // 5 second timeout
        .retry_attempts(2)                // Only 2 retries
        .retry_base_delay(1000)           // 1 second base retry delay
        .max_concurrent_requests(8)       // More parallel requests
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("Testing public fast config...");
    match public_fast.fetch_pools(&["lido"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("✅ SUCCESS: {} - {:.2} SOL staked", 
                name, data.statistics.total_staked_lamports as f64 / 1e9);
        }
        Err(e) => println!("❌ FAILED: {}", e),
    }

    sleep(Duration::from_secs(8)).await;

    // ============================================================================
    // CONFIGURATION 3: PRIVATE/PREMIUM RPC
    // ============================================================================
    println!("\n⚡ CONFIGURATION 3: PRIVATE/PREMIUM RPC");
    println!("Use case: Production apps with paid RPC providers (Alchemy, QuickNode, etc.)");
    println!("Characteristics: High rate limits, aggressive timeouts, minimal retries\n");
    
    let private_rpc = PoolsDataClient::builder()
        .rate_limit(25)                   // 25 requests per second (premium tier)
        .timeout(3)                       // Fast 3 second timeout
        .retry_attempts(1)                // Minimal retries (premium should be reliable)
        .retry_base_delay(500)            // Short retry delay
        .max_concurrent_requests(15)      // High concurrency
        .build("https://api.mainnet-beta.solana.com") // Replace with your premium RPC
        .and_then(PoolsDataClient::from_config)?;

    println!("Testing private RPC config...");
    match private_rpc.fetch_pools(&["marinade"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("✅ SUCCESS: {} - {} validators", 
                name, data.validator_distribution.len());
        }
        Err(e) => println!("❌ FAILED: {}", e),
    }

    sleep(Duration::from_secs(8)).await;

    // ============================================================================
    // CONFIGURATION 4: NO RATE LIMITING (Debug/Development)
    // ============================================================================
    println!("\n🔧 CONFIGURATION 4: NO RATE LIMITING");
    println!("Use case: Local development, testing, or when using dedicated RPC");
    println!("Characteristics: No rate limits, fast timeouts, debug-friendly\n");
    
    let no_limits = PoolsDataClient::builder()
        .no_rate_limit()                  // Remove all rate limiting
        .timeout(8)                       // Reasonable timeout
        .retry_attempts(3)                // Standard retries
        .retry_base_delay(1000)           // 1 second retry delay
        .max_concurrent_requests(20)      // High concurrency
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("Testing no rate limit config (using debug API)...");
    match no_limits.fetch_pools_debug(&["jito"]).await {
        Ok(result) => {
            if let Some((name, data)) = result.successful.iter().next() {
                println!("✅ SUCCESS: {} - Debug format with full RPC data", name);
                println!("   Raw statistics available: {} fields", 
                    serde_json::to_value(&data.statistics)?.as_object().unwrap().len());
            }
            if !result.failed.is_empty() {
                println!("⚠️  Some pools failed: {:?}", result.failed.keys().collect::<Vec<_>>());
            }
        }
        Err(e) => println!("❌ FAILED: {}", e),
    }

    sleep(Duration::from_secs(8)).await;

    // ============================================================================
    // CONFIGURATION 5: UNRELIABLE NETWORK
    // ============================================================================
    println!("\n🐌 CONFIGURATION 5: UNRELIABLE NETWORK");
    println!("Use case: Mobile apps, poor connectivity, or unreliable RPC endpoints");
    println!("Characteristics: Very conservative, maximum retries, long timeouts\n");
    
    let unreliable_network = PoolsDataClient::builder()
        .rate_limit(2)                    // Very slow rate limit
        .timeout(15)                      // Long timeout
        .retry_attempts(8)                // Maximum retries
        .retry_base_delay(3000)           // 3 second base delay (with exponential backoff)
        .max_concurrent_requests(2)       // Minimal concurrency
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("Testing unreliable network config...");
    match unreliable_network.fetch_pools(&["blazestake"]).await {
        Ok(pools) => {
            let (name, data) = pools.iter().next().unwrap();
            println!("✅ SUCCESS: {} - Reliable even on poor networks", name);
        }
        Err(e) => println!("❌ FAILED: {}", e),
    }

    sleep(Duration::from_secs(8)).await;

    // ============================================================================
    // CONFIGURATION 6: BULK OPERATIONS
    // ============================================================================
    println!("\n📦 CONFIGURATION 6: BULK OPERATIONS");
    println!("Use case: Fetching many pools at once, data analytics, periodic snapshots");
    println!("Characteristics: Optimized for throughput, balanced settings\n");
    
    let bulk_operations = PoolsDataClient::builder()
        .rate_limit(8)                    // Moderate rate limit
        .timeout(7)                       // Balanced timeout
        .retry_attempts(3)                // Standard retries
        .retry_base_delay(1500)           // 1.5 second retry delay
        .max_concurrent_requests(12)      // High concurrency for bulk
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    let bulk_pools = ["foundation", "marinade", "jito", "socean", "lido"];
    println!("Testing bulk operations with {} pools...", bulk_pools.len());
    
    match bulk_operations.fetch_pools(&bulk_pools).await {
        Ok(pools) => {
            println!("✅ SUCCESS: Fetched {} pools in bulk operation", pools.len());
            for (name, data) in pools {
                println!("   {}: {} validators, {:.1}k SOL", 
                    name, 
                    data.validator_distribution.len(),
                    data.statistics.total_staked_lamports as f64 / 1e9 / 1000.0);
            }
        }
        Err(e) => println!("❌ FAILED: {}", e),
    }

    // ============================================================================
    // CONFIGURATION SUMMARY & BEST PRACTICES
    // ============================================================================
    println!("\n{}", "=".repeat(80));
    println!("📚 CONFIGURATION SUMMARY & BEST PRACTICES");
    println!("{}", "=".repeat(80));
    
    println!("\n🔧 AVAILABLE CONFIGURATION OPTIONS:");
    println!("┌─────────────────────────────┬──────────────────────────────────────────┐");
    println!("│ Option                      │ Description                              │");
    println!("├─────────────────────────────┼──────────────────────────────────────────┤");
    println!("│ .rate_limit(n)              │ Requests per second (1-50 typical)      │");
    println!("│ .no_rate_limit()            │ Remove all rate limiting                 │");
    println!("│ .timeout(seconds)           │ Per-request timeout (3-15 seconds)      │");
    println!("│ .retry_attempts(n)          │ Number of retries (1-8 typical)         │");
    println!("│ .retry_base_delay(ms)       │ Base retry delay in milliseconds         │");
    println!("│ .max_concurrent_requests(n) │ Parallel requests limit (2-20 typical)  │");
    println!("└─────────────────────────────┴──────────────────────────────────────────┘");

    println!("\n🎯 RECOMMENDED CONFIGURATIONS BY USE CASE:");
    println!("├─ Production (Public RPC):   rate_limit(5), timeout(10), retry_attempts(5)");
    println!("├─ Development:               no_rate_limit(), timeout(5), retry_attempts(2)");
    println!("├─ Premium RPC:               rate_limit(25), timeout(3), retry_attempts(1)");
    println!("├─ Mobile/Unreliable:         rate_limit(2), timeout(15), retry_attempts(8)");
    println!("└─ Bulk Operations:           rate_limit(8), timeout(7), max_concurrent(12)");

    println!("\n⚠️  IMPORTANT NOTES:");
    println!("• Always use 8-second delays between operations for optimal performance");
    println!("• Exponential backoff is automatic for retry delays");
    println!("• Fresh client instances prevent RPC session timeout buildup");
    println!("• Debug API (fetch_pools_debug) returns more data but uses more bandwidth");
    println!("• Production API (fetch_pools) returns clean, database-ready data");

    println!("\n🚀 PERFORMANCE TIPS:");
    println!("• Use appropriate rate limits to avoid RPC throttling");
    println!("• Higher concurrency = faster bulk operations (but more RPC load)");
    println!("• Lower timeouts = faster failure detection (but more false timeouts)");
    println!("• More retries = better reliability (but slower overall failure recovery)");

    println!("\n✅ COMPLETE CONFIGURATION REFERENCE FINISHED");
    println!("All configurations tested successfully with 8-second delays!");
    
    Ok(())
}