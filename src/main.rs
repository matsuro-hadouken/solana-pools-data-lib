//! Main demonstration of the Pools Data Library
//!
//! This example shows the key features and output optimization capabilities
//! of the library without making actual RPC calls.

use pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Solana Pools Data Library - Production-Ready Output Formats");
    println!("===========================================================");
    
    println!("\nThree Output Formats Available:");
    println!("   1. Full Format - Complete RPC data (use for debugging)");
    println!("   2. Production Format - Consistent schema (RECOMMENDED for backends)");
    println!("   3. Optimized Format - Variable schema (use with extreme caution)");
    
    println!("\nUsage Examples:");
    println!("   // Full format (includes all RPC fields)");
    println!("   let full_data = client.fetch_pools(&[\"jito\"]).await?;");
    println!();
    
    println!("   // Production format (consistent schema - RECOMMENDED)");
    println!("   let production_data = client.fetch_pools_production(&[\"jito\"]).await?;");
    println!();
    
    println!("   // Optimized format (variable schema - use with caution)");
    println!("   let optimized_data = client.fetch_pools_optimized(&[\"jito\"]).await?;");
    println!();
    
    println!("   // Compare all three formats");
    println!("   let comparison = client.compare_all_output_sizes(&[\"jito\"]).await?;");
    
    // Show static field analysis
    let analysis = PoolsDataClient::get_static_field_analysis();
    println!("\nStatic Fields Analysis:");
    println!("   Rationale: {}\n", analysis.rationale);
    
    println!("Removed Fields (static/irrelevant - {} fields):", analysis.removed_fields.len());
    for (i, field) in analysis.removed_fields.iter().enumerate() {
        println!("   {}. {}", i + 1, field);
    }
    
    println!("\nDynamic fields kept ({} fields):", analysis.kept_fields.len());
    for (i, field) in analysis.kept_fields.iter().enumerate() {
        println!("   {}. {}", i + 1, field);
    }
    
    println!("\nKey Benefits:");
    println!("   • Reduced JSON payload size");
    println!("   • Faster network transmission");
    println!("   • Cleaner API responses");
    println!("   • Focus on dynamic/relevant data");
    println!("   • Preserves critical reward tracking (credits_observed)");
    
    println!("\nLibrary Setup Example:");
    println!("   let client = PoolsDataClient::builder()");
    println!("       .rate_limit(5) // 5 requests per second");
    println!("       .retry_attempts(3)");
    println!("       .timeout(30)");
    println!("       .max_concurrent_requests(3)");
    println!("       .build(\"https://api.mainnet-beta.solana.com\")");
    println!("       .and_then(PoolsDataClient::from_config)?;");
    
    // Show available pools without making RPC calls
    let client = PoolsDataClient::builder()
        .build("https://dummy.com") // Won't be used for this demo
        .and_then(PoolsDataClient::from_config)?;
    
    let pools = client.list_available_pools();
    println!("\nAvailable Pools ({} total):", pools.len());
    for pool in pools.iter().take(10) {
        println!("   • {} -> {}", pool.name, pool.authority);
    }
    if pools.len() > 10 {
        println!("   ... and {} more pools", pools.len() - 10);
    }
    
    println!("\n🎯 RECOMMENDATIONS:");
    println!("   ✅ Backend/Database: Use fetch_pools_production()");
    println!("   ✅ Public APIs: Use fetch_pools_production() with caching");
    println!("   ⚠️  Special cases: fetch_pools_optimized() with error handling");
    println!("   ❌ Never: Direct optimized to database storage");
    
    println!("\nFor real usage examples, run:");
    println!("   cargo run --example production_formats");
    println!("   cargo run --example backend_compatibility");
    println!("   cargo run --example data_organization");
    
    println!("\nThis library is ready for production use.");
    println!("Configure with your RPC endpoint and start fetching pool data.");
    
    Ok(())
}
