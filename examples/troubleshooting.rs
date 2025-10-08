use solana_pools_data_lib::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// TROUBLESHOOTING & PERFORMANCE ANALYSIS
/// This example helps developers diagnose and optimize their configuration
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== TROUBLESHOOTING & PERFORMANCE ANALYSIS ===\n");
    println!("This example helps you diagnose configuration issues and optimize performance\n");

    // ============================================================================
    // SCENARIO 1: TIMEOUT ISSUES
    // ============================================================================
    println!("🚨 SCENARIO 1: DIAGNOSING TIMEOUT ISSUES");
    println!("Symptom: Requests frequently timeout");
    println!("Solution: Increase timeout, reduce concurrency, add retries\n");

    // BAD: Too aggressive timeout
    let bad_config = PoolsDataClient::builder()
        .rate_limit(10)
        .timeout(1)  // TOO SHORT!
        .retry_attempts(1)
        .max_concurrent_requests(20)  // TOO HIGH!
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("❌ Testing BAD config (1s timeout, 20 concurrent)...");
    let start = Instant::now();
    match bad_config.fetch_pools(&["marinade", "jito", "socean"]).await {
        Ok(pools) => println!("   Unexpectedly succeeded: {} pools in {:?}", pools.len(), start.elapsed()),
        Err(e) => println!("   Failed as expected: {} (took {:?})", e, start.elapsed()),
    }

    sleep(Duration::from_secs(8)).await;

    // GOOD: Conservative timeout settings
    let good_config = PoolsDataClient::builder()
        .rate_limit(5)
        .timeout(10)  // REASONABLE
        .retry_attempts(3)
        .max_concurrent_requests(5)  // REASONABLE
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("✅ Testing GOOD config (10s timeout, 5 concurrent)...");
    let start = Instant::now();
    match good_config.fetch_pools(&["marinade", "jito", "socean"]).await {
        Ok(pools) => println!("   Success: {} pools in {:?}", pools.len(), start.elapsed()),
        Err(e) => println!("   Error: {} (took {:?})", e, start.elapsed()),
    }

    sleep(Duration::from_secs(8)).await;

    // ============================================================================
    // SCENARIO 2: RATE LIMITING ISSUES
    // ============================================================================
    println!("\n🚨 SCENARIO 2: DIAGNOSING RATE LIMITING");
    println!("Symptom: Getting 429 errors or 'Too Many Requests'");
    println!("Solution: Reduce rate_limit, increase delays between calls\n");

    // Test rate limit behavior
    let rate_limited = PoolsDataClient::builder()
        .rate_limit(2)  // Very conservative
        .timeout(5)
        .retry_attempts(2)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("Testing rate limiting with 2 req/s...");
    let pools_to_test = ["lido", "socean"];
    
    for (i, pool) in pools_to_test.iter().enumerate() {
        let start = Instant::now();
        match rate_limited.fetch_pools(&[pool]).await {
            Ok(pools) => {
                let (name, data) = pools.iter().next().unwrap();
                println!("   Request {}: {} success in {:?}", i+1, name, start.elapsed());
            }
            Err(e) => println!("   Request {}: failed - {}", i+1, e),
        }
        
        if i < pools_to_test.len() - 1 {
            println!("   Waiting 8 seconds before next request...");
            sleep(Duration::from_secs(8)).await;
        }
    }

    // ============================================================================
    // SCENARIO 3: PERFORMANCE BENCHMARKING
    // ============================================================================
    println!("\n⚡ SCENARIO 3: PERFORMANCE BENCHMARKING");
    println!("Compare different configurations for your use case\n");

    let test_pools = ["foundation", "marinade"];

    // Configuration A: Balanced
    let config_a = PoolsDataClient::builder()
        .rate_limit(5)
        .timeout(8)
        .retry_attempts(3)
        .max_concurrent_requests(5)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("⏱️  Benchmarking Config A (Balanced)...");
    let start = Instant::now();
    match config_a.fetch_pools(&test_pools).await {
        Ok(pools) => {
            let duration = start.elapsed();
            println!("   ✅ Config A: {} pools in {:?} ({:.2}ms per pool)", 
                pools.len(), duration, duration.as_millis() as f64 / pools.len() as f64);
        }
        Err(e) => println!("   ❌ Config A failed: {}", e),
    }

    sleep(Duration::from_secs(8)).await;

    // Configuration B: Fast
    let config_b = PoolsDataClient::builder()
        .rate_limit(10)
        .timeout(5)
        .retry_attempts(2)
        .max_concurrent_requests(10)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("⏱️  Benchmarking Config B (Fast)...");
    let start = Instant::now();
    match config_b.fetch_pools(&test_pools).await {
        Ok(pools) => {
            let duration = start.elapsed();
            println!("   ✅ Config B: {} pools in {:?} ({:.2}ms per pool)", 
                pools.len(), duration, duration.as_millis() as f64 / pools.len() as f64);
        }
        Err(e) => println!("   ❌ Config B failed: {}", e),
    }

    sleep(Duration::from_secs(8)).await;

    // ============================================================================
    // SCENARIO 4: ERROR HANDLING PATTERNS
    // ============================================================================
    println!("\n🔧 SCENARIO 4: PROPER ERROR HANDLING");
    println!("How to handle different types of errors gracefully\n");

    let error_test_client = PoolsDataClient::builder()
        .rate_limit(5)
        .timeout(5)
        .retry_attempts(2)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Test with both valid and invalid pool names
    let mixed_pools = ["marinade", "nonexistent_pool", "jito"];
    
    println!("Testing error handling with mixed valid/invalid pools...");
    match error_test_client.fetch_pools(&mixed_pools).await {
        Ok(pools) => {
            println!("   ✅ Partial success: {} out of {} pools", pools.len(), mixed_pools.len());
            for (name, _) in pools {
                println!("      - {} ✓", name);
            }
        }
        Err(e) => {
            println!("   ❌ Complete failure: {}", e);
            println!("   💡 Tip: Use debug API for more detailed error information");
        }
    }

    sleep(Duration::from_secs(8)).await;

    // Test debug API for better error reporting
    println!("Testing debug API for detailed error reporting...");
    match error_test_client.fetch_pools_debug(&mixed_pools).await {
        Ok(result) => {
            println!("   📊 Debug Results:");
            println!("      ✅ Successful: {} pools", result.successful.len());
            for name in result.successful.keys() {
                println!("         - {} ✓", name);
            }
            
            if !result.failed.is_empty() {
                println!("      ❌ Failed: {} pools", result.failed.len());
                for (name, error) in &result.failed {
                    println!("         - {}: {:?}", name, error);
                }
            }
        }
        Err(e) => println!("   ❌ Debug API failed: {}", e),
    }

    // ============================================================================
    // TROUBLESHOOTING CHECKLIST
    // ============================================================================
    println!("\n{}", "=".repeat(80));
    println!("🔍 TROUBLESHOOTING CHECKLIST");
    println!("{}", "=".repeat(80));

    println!("\n❓ COMMON ISSUES & SOLUTIONS:");
    
    println!("\n🚨 TIMEOUTS:");
    println!("   Symptoms: Requests take too long and fail");
    println!("   Solutions:");
    println!("   ├─ Increase .timeout() value (try 10-15 seconds)");
    println!("   ├─ Reduce .max_concurrent_requests() (try 3-5)");
    println!("   ├─ Increase .retry_attempts() (try 3-5)");
    println!("   └─ Add longer delays between operations (8+ seconds)");

    println!("\n🚨 RATE LIMITING (429 errors):");
    println!("   Symptoms: 'Too Many Requests' or similar errors");
    println!("   Solutions:");
    println!("   ├─ Reduce .rate_limit() value (try 2-5 req/s)");
    println!("   ├─ Increase delays between operations");
    println!("   ├─ Use .no_rate_limit() only for testing");
    println!("   └─ Consider upgrading to premium RPC");

    println!("\n🚨 INCONSISTENT RESULTS:");
    println!("   Symptoms: Sometimes works, sometimes doesn't");
    println!("   Solutions:");
    println!("   ├─ Use fresh client instances for each operation");
    println!("   ├─ Increase .retry_attempts() and .retry_base_delay()");
    println!("   ├─ Use more conservative settings");
    println!("   └─ Test with debug API for detailed error info");

    println!("\n🚨 SLOW PERFORMANCE:");
    println!("   Symptoms: Operations take too long");
    println!("   Solutions:");
    println!("   ├─ Increase .max_concurrent_requests() (but watch for timeouts)");
    println!("   ├─ Reduce .timeout() for faster failure detection");
    println!("   ├─ Use .no_rate_limit() if your RPC supports it");
    println!("   └─ Consider premium RPC for better performance");

    println!("\n🔧 DEBUGGING STEPS:");
    println!("   1. Start with conservative settings (rate_limit=5, timeout=10)");
    println!("   2. Test with single pool first, then multiple");
    println!("   3. Use debug API for detailed error information");
    println!("   4. Monitor timing with Instant::now() and elapsed()");
    println!("   5. Gradually optimize settings based on your results");

    println!("\n✅ TROUBLESHOOTING COMPLETE");
    println!("Use these patterns to diagnose and fix configuration issues!");

    Ok(())
}