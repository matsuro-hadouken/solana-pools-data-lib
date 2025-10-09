//! Clean demonstration of the Pools Data Library

use solana_pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("Solana Pools Data Library - Demo");
    println!("================================");

    println!("\nTwo Output Formats Available:");
    println!("   1. fetch_pools() - Production data (static fields removed)");
    println!("   2. fetch_pools_debug() - Complete data (ALL fields for debugging)");

    println!("\nUsage Examples:");
    println!("   // Production use");
    println!("   let production_data = client.fetch_pools(&[\"jito\"]).await?;");
    println!("   database.store(production_data).await?; // Safe");
    println!();
    println!("   // Debugging");
    println!("   let debug_data = client.fetch_pools_debug(&[\"jito\"]).await?;");
    println!("   // Contains ALL RPC fields for analysis");

    // Show static field analysis
    let analysis = PoolsDataClient::get_static_field_analysis();

    println!("\nStatic Fields Removed in Production Format:");
    for field in &analysis.static_fields {
        println!(
            "   - {} = {} ({})",
            field.name, field.value, field.description
        );
    }

    println!("\nSize Optimization:");
    println!(
        "   - {} bytes saved per account",
        analysis.size_analysis.estimated_bytes_saved_per_account
    );
    println!(
        "   - {:.1}% size reduction",
        analysis.size_analysis.estimated_size_reduction_percent
    );

    println!("\nInitialization:");
    println!("   let client = PoolsDataClient::builder()");
    println!("       .rate_limit(10)");
    println!("       .build(\"your_rpc_url\")");
    println!("       .and_then(PoolsDataClient::from_config)?;");

    // Example with public RPC (no actual calls)
    let _client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    println!("\nClient ready. Available methods:");
    println!("   - client.fetch_pools(&pools) for production");
    println!("   - client.fetch_pools_debug(&pools) for debugging");

    Ok(())
}
