use solana_pools_data_lib::*;
use std::time::Instant;

/// Comprehensive RPC configuration examples for different providers and use cases
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    println!("=== COMPREHENSIVE RPC CONFIGURATION GUIDE ===\n");

    // Example 1: Auto-detection based on URL
    println!("1. AUTO-DETECTION (Standard configuration method):");
    let auto_client = PoolsDataClient::builder()
        .auto_config("https://api.mainnet-beta.solana.com")
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Auto-detected public RPC settings");
    
    // Example 2: Provider-specific presets
    println!("\n2. PROVIDER-SPECIFIC PRESETS:");
    
    // Alchemy configuration
    let _alchemy_client = PoolsDataClient::builder()
        .alchemy_config()
        .build("https://solana-mainnet.g.alchemy.com/v2/YOUR_API_KEY")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Alchemy: 25 RPS, 8 concurrent, 20s timeout");

    // QuickNode configuration  
    let _quicknode_client = PoolsDataClient::builder()
        .quicknode_config()
        .build("https://your-endpoint.solana-mainnet.quiknode.pro/YOUR_TOKEN/")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   QuickNode: 20 RPS, 6 concurrent, 25s timeout");

    // Helius configuration
    let _helius_client = PoolsDataClient::builder()
        .helius_config()
        .build("https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Helius: 30 RPS, 10 concurrent, 15s timeout");

    // Public RPC configuration
    let _public_client = PoolsDataClient::builder()
        .public_rpc_config()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Public RPC: 1 RPS, 1 concurrent, 45s timeout (conservative settings)");

    // Example 3: Use case specific configurations
    println!("\n3. USE CASE SPECIFIC CONFIGURATIONS:");

    // High-frequency trading
    let _hft_client = PoolsDataClient::builder()
        .high_frequency_config()
        .build("https://your-premium-rpc.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   High Frequency: 200 RPS, 50 concurrent, 5s timeout");

    // Batch processing
    let _batch_client = PoolsDataClient::builder()
        .batch_processing_config()
        .build("https://your-rpc.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Batch Processing: 10 RPS, 20 concurrent, 60s timeout");

    // Development/testing
    let _dev_client = PoolsDataClient::builder()
        .development_config()
        .build("http://localhost:8899")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Development: 10 RPS, 5 concurrent, 20s timeout");

    // Enterprise configuration
    let _enterprise_client = PoolsDataClient::builder()
        .enterprise_config()
        .build("https://your-enterprise-rpc.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Enterprise: 100 RPS, 20 concurrent, 10s timeout");

    // Example 4: Manual fine-tuning
    println!("\n4. MANUAL CONFIGURATION:");
    
    // Custom configuration for specific needs
    let _custom_client = PoolsDataClient::builder()
        .rate_limit(3)                    // 3 requests per second
        .burst_size(10)                   // Allow burst of 10 requests
        .retry_attempts(5)                // 5 retry attempts
        .retry_base_delay(2000)           // 2 second base delay
        .timeout(30)                      // 30 second timeout
        .max_concurrent_requests(2)       // 2 concurrent requests
        .build("https://your-custom-rpc.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Custom: 3 RPS, burst 10, 5 retries, 2s delay, 30s timeout, 2 concurrent");

    // Example 5: Different scenarios for real-world usage
    println!("\n5. REAL-WORLD IMPLEMENTATION SCENARIOS:");

    // Scenario A: Mobile app in a region with poor connectivity
    let _mobile_client = PoolsDataClient::builder()
        .rate_limit(1)                    // Conservative rate limiting
        .retry_attempts(8)                // Additional retries for poor connections
        .retry_base_delay(3000)           // Extended delays between retries
        .timeout(60)                      // Extended timeout
        .max_concurrent_requests(1)       // Single request at a time
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Mobile/Poor Connection: 1 RPS, 8 retries, 3s delay, 60s timeout");

    // Scenario B: High-volume data analytics platform
    let _analytics_client = PoolsDataClient::builder()
        .rate_limit(50)                   // High throughput
        .retry_attempts(2)                // Quick failures
        .retry_base_delay(100)            // Fast retries
        .timeout(10)                      // Fast timeout
        .max_concurrent_requests(25)      // High concurrency
        .build("https://your-premium-analytics-rpc.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   Analytics Platform: 50 RPS, 2 retries, 100ms delay, 10s timeout, 25 concurrent");

    // Scenario C: DeFi protocol with rate limits
    let _defi_client = PoolsDataClient::builder()
        .rate_limit(15)                   // Moderate rate
        .retry_attempts(3)                // Standard retries
        .retry_base_delay(500)            // Reasonable delays
        .timeout(20)                      // Reasonable timeout
        .max_concurrent_requests(5)       // Moderate concurrency
        .build("https://your-defi-rpc.com")
        .and_then(PoolsDataClient::from_config)?;
    
    println!("   DeFi Protocol: 15 RPS, 3 retries, 500ms delay, 20s timeout, 5 concurrent");

    println!("\n6. CONFIGURATION TESTING:");
    
    // Test the public RPC configuration (most likely to work)
    println!("   Testing public RPC configuration...");
    match auto_client.fetch_pools(&["socean"]).await {
        Ok(pools) => {
            let (pool_name, pool_data) = pools.iter().next().unwrap();
            println!("   Successfully fetched {} with {} validators", 
                pool_name, pool_data.validator_distribution.len());
        }
        Err(e) => {
            println!("   Test failed (this is normal without internet): {}", e);
        }
    }

    println!("\n=== CONFIGURATION RECOMMENDATIONS ===");
    println!("Standard Setup:");
    println!("   .auto_config(url) - Detects provider and applies appropriate settings");
    println!();
    println!("By Provider:");
    println!("   .alchemy_config()     - Optimized for Alchemy RPC");
    println!("   .quicknode_config()   - Optimized for QuickNode RPC");
    println!("   .helius_config()      - Optimized for Helius RPC");
    println!("   .public_rpc_config()  - Conservative for public endpoints");
    println!();
    println!("By Use Case:");
    println!("   .high_frequency_config()    - Real-time trading, minimal latency");
    println!("   .batch_processing_config()  - Bulk operations, high reliability");
    println!("   .development_config()       - Local testing, moderate settings");
    println!("   .enterprise_config()        - High-performance dedicated endpoints");
    println!();
    println!("Custom Configuration:");
    println!("   .rate_limit(n)              - Requests per second (1-1000)");
    println!("   .burst_size(n)              - Allow burst requests");
    println!("   .retry_attempts(n)          - Number of retries (0-10)");
    println!("   .timeout(secs)              - Request timeout (1-300)");
    println!("   .max_concurrent_requests(n) - Parallel requests (1-100)");
    println!();
    println!("Implementation Guidelines:");
    println!("   - Start with auto_config() for automatic configuration");
    println!("   - Use provider presets when available");
    println!("   - Increase rate limits gradually with private RPC");
    println!("   - Monitor RPC provider limits and adjust accordingly");
    println!("   - Use public_rpc_config() as fallback for unknown endpoints");

    let duration = start.elapsed();
    println!("\nExecution time: {:.2?}", duration);

    Ok(())
}