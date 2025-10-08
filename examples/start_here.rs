use solana_pools_data_lib::*;

/// DEVELOPER ONBOARDING - START HERE
/// Quick 2-minute overview to get developers started immediately
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 SOLANA POOLS DATA LIBRARY - DEVELOPER ONBOARDING");
    println!("{}", "=".repeat(60));
    
    println!("\n👋 Welcome! Here's everything you need in 2 minutes:\n");

    // STEP 1: Basic Usage
    println!("📝 STEP 1: BASIC USAGE");
    let client = PoolsDataClient::builder()
        .rate_limit(5)  // Safe for production
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    match client.fetch_pools(&["marinade", "jito"]).await {
        Ok(pools) => {
            println!("✅ SUCCESS: Fetched {} pools", pools.len());
            for (name, data) in &pools {
                println!("   {}: {} validators, {:.1}k SOL", 
                    name, 
                    data.validator_distribution.len(),
                    data.statistics.total_staked_lamports as f64 / 1e9 / 1000.0);
            }
        }
        Err(e) => println!("❌ ERROR: {}", e),
    }

    println!("\n📚 NEXT STEPS - RUN THESE EXAMPLES:");
    println!("┌──────────────────────────┬────────────────────────────────────────┐");
    println!("│ Example                  │ Purpose                                │");
    println!("├──────────────────────────┼────────────────────────────────────────┤");
    println!("│ cargo run --example      │ 📖 COMPLETE configuration guide       │");
    println!("│   complete_config        │    (6 scenarios, all options)         │");
    println!("├──────────────────────────┼────────────────────────────────────────┤");
    println!("│ cargo run --example      │ 🔧 TROUBLESHOOTING & optimization     │");
    println!("│   troubleshooting        │    (debug issues, performance)        │");
    println!("├──────────────────────────┼────────────────────────────────────────┤");
    println!("│ cargo run --example      │ 🌍 ALL 32 pools comprehensive fetch   │");
    println!("│   comprehensive          │    (production-ready implementation)  │");
    println!("├──────────────────────────┼────────────────────────────────────────┤");
    println!("│ cargo run --example      │ 💾 Backend integration patterns       │");
    println!("│   backend_compatibility  │    (database, API, storage)           │");
    println!("└──────────────────────────┴────────────────────────────────────────┘");

    println!("\n🎯 QUICK CONFIGURATION CHEAT SHEET:");
    println!("Production:  .rate_limit(5).timeout(10).retry_attempts(5)");
    println!("Development: .no_rate_limit().timeout(5).retry_attempts(2)");
    println!("Premium RPC: .rate_limit(25).timeout(3).retry_attempts(1)");
    println!("Unreliable:  .rate_limit(2).timeout(15).retry_attempts(8)");

    println!("\n⚡ PERFORMANCE NOTES:");
    println!("• Always use 8-second delays between operations");
    println!("• Fresh client instances prevent session timeout buildup");
    println!("• Debug API gives full RPC data, Production API gives clean data");
    println!("• Exponential backoff is automatic on retries");

    println!("\n📋 AVAILABLE POOLS:");
    let available = PoolsDataClient::list_available_pools();
    print!("   {} pools: ", available.len());
    for (i, pool) in available.iter().take(8).enumerate() {
        print!("{}", pool.name);
        if i < 7 && i < available.len() - 1 { print!(", "); }
    }
    if available.len() > 8 {
        println!(" + {} more", available.len() - 8);
    } else {
        println!();
    }

    println!("\n🎉 You're ready to build! Start with complete_config example.");
    
    Ok(())
}