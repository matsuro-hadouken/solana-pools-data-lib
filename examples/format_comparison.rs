use solana_pools_data_lib::*;
use serde_json;

/// Compare JSON outputs between production and debug formats
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== JSON FORMAT COMPARISON ===\n");
    
    let client = PoolsDataClient::builder()
        .rate_limit(5)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    let test_pool = "lido"; // Small pool for demo
    
    println!("🔍 Fetching pool: {}\n", test_pool);
    
    // Get both formats
    let production_result = client.fetch_pools(&[test_pool]).await;
    let debug_result = client.fetch_pools_debug(&[test_pool]).await;
    
    match (production_result, debug_result) {
        (Ok(production), Ok(debug)) => {
            if let (Some((_, prod_pool)), Some((_, debug_pool))) = 
                (production.iter().next(), debug.successful.iter().next()) {
                
                println!("📊 PRODUCTION FORMAT - Clean & Database Safe:");
                println!("Pool: {}", prod_pool.pool_name);
                println!("Total Accounts: {}", prod_pool.stake_accounts.len());
                
                // Show first account in production format
                if let Some(account) = prod_pool.stake_accounts.first() {
                    println!("\nFirst Account (Production):");
                    let account_json = serde_json::to_string_pretty(account)?;
                    println!("{}", account_json);
                }
                
                println!("\n{}", "=".repeat(60));
                println!("🔍 DEBUG FORMAT - Complete RPC Data:");
                println!("Pool: {}", debug_pool.pool_name);
                println!("Total Accounts: {}", debug_pool.stake_accounts.len());
                
                // Show first account in debug format
                if let Some(debug_account) = debug_pool.stake_accounts.first() {
                    println!("\nFirst Account (Debug - ALL fields):");
                    let debug_account_json = serde_json::to_string_pretty(debug_account)?;
                    println!("{}", debug_account_json);
                }
                
                // Size comparison
                let prod_json = serde_json::to_string(&production)?;
                let debug_json = serde_json::to_string(&debug)?;
                
                println!("\n{}", "=".repeat(60));
                println!("💾 SIZE COMPARISON:");
                println!("Production JSON: {} bytes", prod_json.len());
                println!("Debug JSON: {} bytes", debug_json.len());
                
                let savings = debug_json.len().saturating_sub(prod_json.len());
                let percent = if debug_json.len() > 0 {
                    (savings as f64 / debug_json.len() as f64) * 100.0
                } else { 0.0 };
                
                println!("Space saved: {} bytes ({:.1}%)", savings, percent);
                
                println!("\n✅ SUMMARY:");
                println!("• Production: Clean, consistent schema - safe for databases");
                println!("• Debug: Complete RPC data with all fields - for analysis");
                println!("• Both formats provide the same core functionality");
                println!("• Choose production for APIs, debug for troubleshooting");
            }
        }
        (Err(e), _) => println!("❌ Production error: {}", e),
        (_, Err(e)) => println!("❌ Debug error: {}", e),
    }
    
    Ok(())
}