# Getting Started

Quick setup guide for the Solana Pools Data Library. **Ready in 2 minutes!**

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
solana-pools-data-lib = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Two Simple Formats

**Production**: Clean data, safe for databases  
**Debug**: Complete RPC data for debugging

## Quick Start

```rust
use solana_pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = PoolsDataClient::builder()
        .rate_limit(10)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Production use - clean and safe
    let pools = client.fetch_pools(&["jito", "marinade"]).await?;
    println!("Fetched {} pools", pools.len());
    
    // Safe to store in database
    database.store(pools).await?;
    
    Ok(())
}
```

## Debug Format (Complete Data)

```rust
// Get complete RPC data with all fields
let debug_result = client.fetch_pools_debug(&["jito"]).await?;

// Handle partial failures
for (pool_name, error) in &debug_result.failed {
    println!("Pool {} failed: {}", pool_name, error.error);
}

// Process successful results
for (pool_name, pool_data) in &debug_result.successful {
    println!("Pool {}: {} accounts", pool_name, pool_data.stake_accounts.len());
}
```