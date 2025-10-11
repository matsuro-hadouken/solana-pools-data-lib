# Solana Pools Data Library

Rust library for fetching Solana stake pool data.

> **Note**: This library is currently in development. Install directly from GitHub until published to crates.io.

## Features

- **Two Output Formats**: Production (optimized) and Debug (complete RPC data)
- **RPC Provider Detection**: Automatic configuration for Alchemy, QuickNode, Helius
- **32 Supported Pools**: All major Solana stake pools included
- **Production Ready**: Rate limiting, retries, timeout handling
- **Safe by Design**: Consistent schemas, no breaking changes

## Installation

### From GitHub (Current)

```toml
[dependencies]
solana-pools-data-lib = { git = "https://github.com/matsuro-hadouken/solana-pools-data-lib" }
tokio = { version = "1.0", features = ["full"] }
```

### Future: From Crates.io (When Published)

```toml
[dependencies]
solana-pools-data-lib = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Clone and Test Locally

```bash
git clone https://github.com/matsuro-hadouken/solana-pools-data-lib.git
cd solana-pools-data-lib/pools-data-lib
cargo run --example quick_test
```

## Quick Start

```rust
use solana_pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PoolsDataClient::builder()
        .rate_limit(5)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Production format - optimized for databases
    let production_data = client.fetch_pools(&["jito"]).await?;
    
    for (pool_name, pool_data) in pools {
        println!("Pool: {}", pool_name);
        println!("  Authority: {}", pool_data.authority);
        println!("  Accounts: {}", pool_data.stake_accounts.len());
        println!("  Validators: {}", pool_data.validator_distribution.len());
    }
    
    Ok(())
}
```

## Two Output Formats

### Production Format - Optimized Storage

```rust
let pools = client.fetch_pools(&["jito", "marinade"]).await?;

for (pool_name, pool_data) in pools {
    println!("Pool: {}", pool_name);
    println!("  Authority: {}", pool_data.authority);
    println!("  Accounts: {}", pool_data.stake_accounts.len());
    println!("  Validators: {}", pool_data.validator_distribution.len());
    println!("  Total Staked: {:.2} SOL", 
        pool_data.statistics.total_staked_lamports as f64 / 1_000_000_000.0);
}
```

### Debug Format - Raw RPC Response

```rust
let debug_result = client.fetch_pools_debug(&["jito"]).await?;

// Handle partial failures
for (pool_name, error) in &debug_result.failed {
    println!("Pool {} failed: {}", pool_name, error.error);
}

// Process successful pools
for (pool_name, pool_data) in &debug_result.successful {
    println!("Pool {}: {} accounts", pool_name, pool_data.stake_accounts.len());
    
    // Access all RPC fields for debugging
    for account in &pool_data.stake_accounts.iter().take(3) {
        println!("  Account: {}", account.pubkey);
        println!("    Lamports: {}", account.lamports);
        println!("    Rent Exempt Reserve: {}", account.rent_exempt_reserve);
    }
}
```

## Configuration

### Quick Setup with Provider Presets

```rust
// Auto-detect configuration based on URL (Recommended)
let client = PoolsDataClient::builder()
    .auto_config("https://api.mainnet-beta.solana.com")
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;

// Alchemy RPC - optimized for Alchemy endpoints
let client = PoolsDataClient::builder()
    .alchemy_config()
    .build("https://solana-mainnet.g.alchemy.com/v2/YOUR_API_KEY")
    .and_then(PoolsDataClient::from_config)?;

// QuickNode RPC - optimized for QuickNode endpoints  
let client = PoolsDataClient::builder()
    .quicknode_config()
    .build("https://your-endpoint.solana-mainnet.quiknode.pro/YOUR_TOKEN/")
    .and_then(PoolsDataClient::from_config)?;

// Helius RPC - optimized for Helius endpoints
let client = PoolsDataClient::builder()
    .helius_config()
    .build("https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY")
    .and_then(PoolsDataClient::from_config)?;

// Public RPC - most conservative settings for public endpoints
let client = PoolsDataClient::builder()
    .public_rpc_config()
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;

// Private RPC - optimized for premium endpoints  
let client = PoolsDataClient::builder()
    .private_rpc_config()
    .build("your_private_rpc_url")
    .and_then(PoolsDataClient::from_config)?;
```

### Use Case Specific Configurations

```rust
// High-frequency trading or real-time applications
let client = PoolsDataClient::builder()
    .high_frequency_config()
    .build("https://your-premium-rpc.com")
    .and_then(PoolsDataClient::from_config)?;

// Batch processing applications
let client = PoolsDataClient::builder()
    .batch_processing_config()
    .build("https://your-rpc.com")
    .and_then(PoolsDataClient::from_config)?;

// Development and testing
let client = PoolsDataClient::builder()
    .development_config()
    .build("http://localhost:8899")
    .and_then(PoolsDataClient::from_config)?;

// Enterprise/dedicated endpoints
let client = PoolsDataClient::builder()
    .enterprise_config()
    .build("https://your-enterprise-rpc.com")
    .and_then(PoolsDataClient::from_config)?;
```

### Manual Fine-tuning

```rust
// Custom configuration for specific requirements
let client = PoolsDataClient::builder()
    .rate_limit(3)                    // 3 requests per second
    .burst_size(10)                   // Allow burst of 10 requests
    .retry_attempts(5)                // 5 retry attempts
    .retry_base_delay(2000)           // 2 second base delay  
    .timeout(30)                      // 30 second timeout
    .max_concurrent_requests(2)       // 2 concurrent requests
    .build("https://your-custom-rpc.com")
    .and_then(PoolsDataClient::from_config)?;

// No rate limiting for local development
let client = PoolsDataClient::builder()
    .no_rate_limit()
    .retry_attempts(1)
    .timeout(5)
    .max_concurrent_requests(50)
    .build("http://localhost:8899")
    .and_then(PoolsDataClient::from_config)?;
```

## Supported Pools

The library supports **32 major Solana stake pools**:

```rust
// Get list of all supported pools
let available_pools = PoolsDataClient::list_available_pools();
for pool in available_pools {
    println!("Pool: {} (Authority: {})", pool.name, pool.authority);
}
```

**Major pools include**: Jito, Marinade, Lido, Blazestake, Jupiter, Sanctum, and more.

## Error Handling

```rust
match client.fetch_pools(&["jito"]).await {
    Ok(pools) => {
        for (pool_name, pool_data) in pools {
            println!("Pool {} has {} accounts", pool_name, pool_data.stake_accounts.len());
        }
    }
    Err(e) => {
        eprintln!("Failed to fetch pools: {}", e);
    }
}
```

## Common Use Cases

### Database Storage

```rust
// Production format is safe for databases - schema never changes
let pools = client.fetch_pools(&["jito", "marinade"]).await?;

for (pool_name, pool_data) in pools {
    let json = serde_json::to_string(&pool_data)?;
    database.insert(&pool_name, &json).await?;
}
```

### REST API Integration

```rust
// Integration with web frameworks
async fn get_pool_data(pool_name: &str) -> Result<PoolData, Error> {
    let client = PoolsDataClient::builder()
        .public_rpc_config()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;
        
    let pools = client.fetch_pools(&[pool_name]).await?;
    Ok(pools.into_values().next().unwrap())
}
```

### Batch Processing

```rust
// Process multiple pools efficiently
let all_pools = PoolsDataClient::list_available_pools();
let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();

// Process in small batches to avoid rate limiting
for batch in pool_names.chunks(3) {
    let pools = client.fetch_pools(batch).await?;
    println!("Processed {} pools", pools.len());
    
    // Brief pause between batches
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
}
```

## Examples

Run these examples to see the library in action:

```bash
# Quick overview
cargo run --example quick_test

# Complete configuration reference
cargo run --example basic

# All 32 pools
cargo run --example comprehensive

# Format comparison
cargo run --example format_comparison
```

## Additional Documentation

- **[Examples](examples/README.md)** - Complete configuration reference
- **[Getting Started](GETTING_STARTED.md)** - Quick 2-minute setup
- **[Integration Guide](INTEGRATION.md)** - Production patterns

## License

MIT License - see LICENSE file for details.