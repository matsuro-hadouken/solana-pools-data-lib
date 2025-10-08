# Integration Guide

Production patterns for using the Solana Pools Data Library in real applications.

## Quick Configuration

```rust
use solana_pools_data_lib::PoolsDataClient;

// Public RPC - use preset configuration
let client = PoolsDataClient::builder()
    .public_rpc_config()
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;

// Private RPC - use preset configuration
let client = PoolsDataClient::builder()
    .private_rpc_config()
    .build("your_private_rpc_url")
    .and_then(PoolsDataClient::from_config)?;
```

## Database Storage

```rust
use serde_json;

// Production format has consistent schema - safe for databases
let pools = client.fetch_pools(&["jito", "marinade"]).await?;

for (pool_name, pool_data) in pools {
    let json = serde_json::to_string(&pool_data)?;
    
    // Safe to store - schema never changes
    database.insert(&pool_name, &json).await?;
    
    println!("Stored pool {} with {} accounts", 
        pool_name, pool_data.stake_accounts.len());
}
```

## REST API Integration

```rust
use axum::{extract::Path, response::Json, Router, routing::get};
use serde_json::Value;

// Simple endpoint that returns pool data
async fn get_pool_handler(Path(pool_name): Path<String>) -> Result<Json<Value>, String> {
    let client = PoolsDataClient::builder()
        .public_rpc_config()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)
        .map_err(|e| e.to_string())?;
        
    let pools = client.fetch_pools(&[&pool_name]).await
        .map_err(|e| e.to_string())?;
    
    if let Some(pool_data) = pools.into_values().next() {
        Ok(Json(serde_json::to_value(pool_data).unwrap()))
    } else {
        Err(format!("Pool {} not found", pool_name))
    }
}

pub fn create_routes() -> Router {
    Router::new().route("/pools/:name", get(get_pool_handler))
}
```

## Error Handling

```rust
match client.fetch_pools(&["jito", "marinade"]).await {
    Ok(pools) => {
        println!("Successfully fetched {} pools", pools.len());
        for (pool_name, pool_data) in pools {
            println!("Pool {}: {} accounts", pool_name, pool_data.stake_accounts.len());
        }
    }
    Err(e) => {
        eprintln!("Failed to fetch pools: {}", e);
        // Handle error appropriately for your application
    }
}
```

## Debug Mode for Troubleshooting

```rust
// Use debug format when you need complete RPC information
let debug_result = client.fetch_pools_debug(&["jito"]).await?;

// Check for partial failures
if !debug_result.failed.is_empty() {
    for (pool_name, error) in &debug_result.failed {
        println!("Pool {} failed: {}", pool_name, error.error);
    }
}

// Process successful pools
for (pool_name, pool_data) in &debug_result.successful {
    println!("Pool {}: {} stake accounts", pool_name, pool_data.stake_accounts.len());
    
    // Complete RPC data available for analysis
    for account in pool_data.stake_accounts.iter().take(3) {
        println!("  Account {}: {} lamports", account.pubkey, account.lamports);
        println!("    Rent exempt reserve: {}", account.rent_exempt_reserve);
        
        if let Some(delegation) = &account.delegation {
            println!("    Delegated to: {}", delegation.voter);
            println!("    Stake: {} lamports", delegation.stake);
        }
    }
}
```

## Caching Pattern

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

struct CachedPoolClient {
    client: PoolsDataClient,
    cache: HashMap<String, (serde_json::Value, Instant)>,
    cache_ttl: Duration,
}

impl CachedPoolClient {
    pub fn new(client: PoolsDataClient, cache_ttl: Duration) -> Self {
        Self {
            client,
            cache: HashMap::new(),
            cache_ttl,
        }
    }
    
    pub async fn get_pool(&mut self, pool_name: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // Check cache
        if let Some((data, timestamp)) = self.cache.get(pool_name) {
            if timestamp.elapsed() < self.cache_ttl {
                return Ok(data.clone());
            }
        }
        
        // Fetch fresh data
        let pools = self.client.fetch_pools(&[pool_name]).await?;
        if let Some(pool_data) = pools.into_values().next() {
            let json_value = serde_json::to_value(pool_data)?;
            
            // Update cache
            self.cache.insert(pool_name.to_string(), (json_value.clone(), Instant::now()));
            
            Ok(json_value)
        } else {
            Err(format!("Pool {} not found", pool_name).into())
        }
    }
}
```

## Batch Processing

```rust
use tokio::time::{sleep, Duration};

// Process all available pools in batches
let all_pools = PoolsDataClient::list_available_pools();
let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();

println!("Processing {} pools in batches of 3...", pool_names.len());

for (batch_num, batch) in pool_names.chunks(3).enumerate() {
    println!("Batch {}: {:?}", batch_num + 1, batch);
    
    match client.fetch_pools(batch).await {
        Ok(pools) => {
            for (pool_name, pool_data) in pools {
                println!("  {} - {} accounts, {} validators", 
                    pool_name, 
                    pool_data.stake_accounts.len(),
                    pool_data.validator_distribution.len()
                );
            }
        }
        Err(e) => {
            println!("  Batch {} failed: {}", batch_num + 1, e);
            continue; // Skip failed batch
        }
    }
    
    // Rate limiting between batches
    sleep(Duration::from_millis(2000)).await;
}
```

## Custom Configuration

```rust
// For different RPC providers, adjust settings accordingly

// Conservative (Public RPC)
let client = PoolsDataClient::builder()
    .rate_limit(1)              // Very conservative
    .retry_attempts(3)          // More retries
    .timeout(10)                // Longer timeout
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;

// Aggressive (Private RPC)
let client = PoolsDataClient::builder()
    .no_rate_limit()            // No limits
    .retry_attempts(1)          // Fewer retries
    .timeout(2)                 // Faster timeout
    .max_concurrent_requests(10) // More concurrent requests
    .build("your_private_rpc_url")
    .and_then(PoolsDataClient::from_config)?;
```

## Production Checklist

- ✅ Use **preset configurations** for quick setup (`.public_rpc_config()`, `.private_rpc_config()`)
- ✅ Use `fetch_pools()` for production (clean, consistent schema)
- ✅ Use `fetch_pools_debug()` only for debugging and analysis
- ✅ Implement proper error handling and logging
- ✅ Add caching for frequently accessed data
- ✅ Use batch processing for multiple pools (chunks of 3-5)
- ✅ Add rate limiting between batches (1-2 second delays)
- ✅ Test with real data before deploying

## Working Examples

```bash
# See all configuration options
cargo run --example basic

# Quick library overview
cargo run --example quick_test

# Production batch processing
cargo run --example comprehensive

# Database integration
cargo run --example backend_compatibility
```