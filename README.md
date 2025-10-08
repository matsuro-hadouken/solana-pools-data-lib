# Solana Pools Data Library

Clean, simple Rust library for fetching Solana stake pool data.

## Two Output Formats

- **Production**: Clean data, safe for databases and APIs
- **Debug**: Complete RPC data with all fields for debugging

## Installation

```toml
[dependencies]
solana-pools-data-lib = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use solana_pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PoolsDataClient::builder()
        .rate_limit(10)
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Production use - clean and safe
    let pools = client.fetch_pools(&["jito", "marinade"]).await?;
    println!("Fetched {} pools", pools.len());
    
    Ok(())
}
```

## Usage

### Production Format

Clean data with static fields removed. Safe for databases and APIs.

```rust
let pools = client.fetch_pools(&["jito", "marinade"]).await?;

for (pool_name, pool_data) in pools {
    println!("Pool: {}", pool_name);
    println!("Authority: {}", pool_data.authority);
    println!("Accounts: {}", pool_data.stake_accounts.len());
    
    // Safe to store in database - schema never changes
    database.insert(&pool_name, &pool_data).await?;
}
```

### Debug Format

Complete RPC data with all fields for debugging and analysis.

```rust
let debug_result = client.fetch_pools_debug(&["jito"]).await?;

// Handle partial failures
for (pool_name, error) in &debug_result.failed {
    println!("Pool {} failed: {}", pool_name, error.error);
}

// Process successful pools
for (pool_name, pool_data) in &debug_result.successful {
    println!("Pool {}: {} accounts", pool_name, pool_data.stake_accounts.len());
    
    // All RPC fields available for analysis
    for account in &pool_data.stake_accounts {
        println!("  {} lamports", account.lamports);
        // Access all fields: rent_exempt_reserve, warmup_cooldown_rate, etc.
    }
}
```

## Configuration

### Public RPC (Rate Limited)

```rust
let client = PoolsDataClient::builder()
    .rate_limit(2)      // Conservative for public RPC
    .timeout(30)
    .retry_attempts(3)
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;
```

### Private RPC (Higher Limits)

```rust
let client = PoolsDataClient::builder()
    .rate_limit(20)     // Higher limit for private RPC
    .timeout(15)
    .max_concurrent_requests(10)
    .build("your_private_rpc_url")
    .and_then(PoolsDataClient::from_config)?;
```

## Supported Pools

The library supports all major Solana stake pools:

- Jito
- Marinade
- Lido
- Cogent
- Socean
- daoPool
- And 25+ more

```rust
// Get list of all supported pools
let available_pools = PoolsDataClient::list_available_pools();
for pool in available_pools {
    println!("Pool: {} ({})", pool.name, pool.authority);
}
```

## Error Handling

```rust
match client.fetch_pools(&["jito"]).await {
    Ok(pools) => {
        // Process successful results
        for (pool_name, pool_data) in pools {
            process_pool(pool_name, pool_data).await?;
        }
    }
    Err(e) => {
        // Handle error
        eprintln!("Failed to fetch pools: {}", e);
        return Err(e.into());
    }
}
```

## Integration Examples

### REST API

```rust
use axum::{extract::Path, response::Json};

async fn get_pool(Path(pool_name): Path<String>) -> Json<ProductionPoolData> {
    let client = get_client(); // Your client instance
    let pools = client.fetch_pools(&[&pool_name]).await.unwrap();
    Json(pools[&pool_name].clone())
}
```

### Database Storage

```rust
// Production format has consistent schema - safe for databases
let pools = client.fetch_pools(&["jito"]).await?;

for (pool_name, pool_data) in pools {
    let json = serde_json::to_string(&pool_data)?;
    database.insert(&pool_name, &json).await?; // Always safe
}
```

### Batch Processing

```rust
let all_pools = PoolsDataClient::list_available_pools();
let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();

for batch in pool_names.chunks(5) {
    let pools = client.fetch_pools(batch).await?;
    process_batch(pools).await?;
    
    // Rate limiting between batches
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

## Features

- **Clean API**: Two simple methods, no confusing choices
- **Safe by Design**: Production format never changes schema
- **Complete Data**: Debug format includes all RPC fields
- **Configurable**: Rate limiting, timeouts, retries
- **Reliable**: Exponential backoff, partial failure handling
- **Production Ready**: Used in enterprise applications

## Documentation

- [Getting Started](GETTING_STARTED.md) - Quick 2-minute setup
- [Integration Guide](INTEGRATION.md) - Production patterns and examples

## License

MIT License - see LICENSE file for details.