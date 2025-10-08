use pools_data_lib::*;

/// Demonstrate the three output formats with real examples
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Production-Ready Output Formats ===\n");
    
    let client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    let test_pools = ["jito", "foundation"];
    
    println!("🔍 Testing with pools: {:?}\n", test_pools);
    
    // Test all three formats
    println!("📊 1. FULL FORMAT (Complete RPC data)");
    let full_result = client.fetch_pools(&test_pools).await?;
    if let Some(pool_data) = full_result.successful.values().next() {
        if let Some(account) = pool_data.stake_accounts.first() {
            let json = serde_json::to_string_pretty(account)?;
            println!("   Sample account (full):");
            println!("{}", json.lines().take(15).collect::<Vec<_>>().join("\n"));
            println!("   ... (truncated)\n");
        }
    }
    
    println!("✅ 2. PRODUCTION FORMAT (Consistent schema - RECOMMENDED)");
    let production_result = client.fetch_pools_production(&test_pools).await?;
    if let Some(pool_data) = production_result.values().next() {
        if let Some(account) = pool_data.stake_accounts.first() {
            let json = serde_json::to_string_pretty(account)?;
            println!("   Sample account (production):");
            println!("{}", json);
            
            println!("\n   ✅ BENEFITS:");
            println!("      • Same JSON schema every time");
            println!("      • Always includes lockup and authority");
            println!("      • Safe for database storage");
            println!("      • Removes only truly static fields");
        }
    }
    
    println!("\n⚠️  3. OPTIMIZED FORMAT (Variable schema - USE WITH CAUTION)");
    let optimized_result = client.fetch_pools_optimized(&test_pools).await?;
    if let Some(pool_data) = optimized_result.values().next() {
        if let Some(account) = pool_data.stake_accounts.first() {
            let json = serde_json::to_string_pretty(account)?;
            println!("   Sample account (optimized):");
            println!("{}", json);
            
            println!("\n   ⚠️  RISKS:");
            println!("      • JSON schema can change suddenly");
            println!("      • Optional fields appear/disappear");
            println!("      • Can break database schemas");
            println!("      • Dangerous for production storage");
        }
    }
    
    // Size comparison
    println!("\n📏 SIZE COMPARISON:");
    let comparison = client.compare_all_output_sizes(&["jito"]).await?;
    
    println!("   Full format:       {:>8} bytes (100%)", comparison.full_size_bytes);
    println!("   Production format: {:>8} bytes ({:.1}% reduction)", 
             comparison.production_size_bytes, 
             comparison.production_reduction_percent);
    println!("   Optimized format:  {:>8} bytes ({:.1}% reduction)", 
             comparison.optimized_size_bytes, 
             comparison.optimized_reduction_percent);
    
    println!("\n🎯 RECOMMENDATION:");
    println!("   ✅ Backend/Database: Use fetch_pools_production()");
    println!("   ✅ Public APIs: Use fetch_pools_production() with caching");
    println!("   ⚠️  Special cases only: fetch_pools_optimized() with error handling");
    println!("   ❌ Never: Direct optimized to database storage");
    
    println!("\n💡 The production format gives you:");
    println!("   • {:.1}% size reduction vs full format", comparison.production_reduction_percent);
    println!("   • Predictable JSON schema");
    println!("   • Safe for all production use cases");
    
    Ok(())
}