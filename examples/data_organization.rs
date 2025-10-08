use solana_pools_data_lib::*;

/// Simple data structure demonstration
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== Clean Data Organization ===\n");
    
    let client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("🔍 Two Simple Formats:");
    println!("1. Production: Clean data for production use");
    println!("2. Debug: Complete RPC data with all fields");
    
    println!("\n📊 Field Analysis:");
    let analysis = PoolsDataClient::get_static_field_analysis();
    
    println!("Static fields removed in production format:");
    for field in &analysis.static_fields {
        println!("   • {} = {} ({})", field.name, field.value, field.description);
    }
    
    println!("\nDynamic fields kept in both formats:");
    for field in &analysis.dynamic_fields {
        println!("   • {}", field);
    }
    
    println!("\n💾 Size Benefits:");
    println!("   • {} bytes saved per account", analysis.size_analysis.estimated_bytes_saved_per_account);
    println!("   • {:.1}% smaller JSON", analysis.size_analysis.estimated_size_reduction_percent);
    
    // Show live data structure
    match client.fetch_pools(&["jito"]).await {
        Ok(production_data) => {
            if let Some((name, pool)) = production_data.iter().next() {
                println!("\n🏗️  Production Data Structure for '{}':", name);
                println!("Pool {{");
                println!("  pool_name: \"{}\"", pool.pool_name);
                println!("  authority: \"{}\"", pool.authority);
                println!("  stake_accounts: [{} accounts]", pool.stake_accounts.len());
                println!("  validator_distribution: [{} validators]", pool.validator_distribution.len());
                println!("  statistics: {{ total_lamports: {}, active_accounts: {} }}", 
                    pool.statistics.total_lamports, pool.statistics.active_stake_accounts);
                println!("  fetched_at: \"{}\"", pool.fetched_at);
                println!("}}");
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }
    
    println!("\n✅ Simple, clean, and predictable data structures!");
    
    Ok(())
}