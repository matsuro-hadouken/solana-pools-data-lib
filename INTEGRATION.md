# Integration Guide

Simple patterns for using the library in production systems.

## Basic Usage

```rust
use solana_pools_data_lib::PoolsDataClient;

// Production data - clean and safe
let client = PoolsDataClient::builder()
    .rate_limit(10)
    .build("your_rpc_url")
    .and_then(PoolsDataClient::from_config)?;

let pools = client.fetch_pools(&["jito", "marinade"]).await?;
```

## Database Storage

```rust
use serde_json;

// Production format has consistent schema - safe for databases
let pools = client.fetch_pools(&["jito"]).await?;

for (pool_name, pool_data) in pools {
    let json = serde_json::to_string(&pool_data)?;
    
    // Safe to store - schema never changes
    database.insert(&pool_name, &json).await?;
}
```

## REST API

```rust
use axum::{extract::Path, response::Json, Router};

async fn get_pool(Path(pool_name): Path<String>) -> Json<ProductionPoolData> {
    let client = get_client(); // Your client instance
    let pools = client.fetch_pools(&[&pool_name]).await.unwrap();
    
    Json(pools[&pool_name].clone())
}

pub fn routes() -> Router {
    Router::new().route("/pools/:name", axum::routing::get(get_pool))
}
```

## Error Handling

```rust
match client.fetch_pools(&["jito"]).await {
    Ok(pools) => {
        // Success - process data
        process_pools(pools).await?;
    }
    Err(e) => {
        // Handle error
        log::error!("Failed to fetch pools: {}", e);
        return Err(e.into());
    }
}
```

## Debug Mode

```rust
// Use debug format when you need complete information
let debug_result = client.fetch_pools_debug(&["jito"]).await?;

// Check for partial failures
if !debug_result.failed.is_empty() {
    for (pool_name, error) in &debug_result.failed {
        log::warn!("Pool {} failed: {}", pool_name, error.error);
    }
}

// Process successful pools
for (pool_name, pool_data) in &debug_result.successful {
    println!("Pool {}: {} stake accounts", pool_name, pool_data.stake_accounts.len());
    
    // Complete data available for analysis
    for account in &pool_data.stake_accounts {
        println!("  Account {}: {} lamports", account.pubkey, account.lamports);
        // All RPC fields available: rent_exempt_reserve, warmup_cooldown_rate, etc.
    }
}
```

## Caching Pattern

```rust
use std::time::{Duration, Instant};

struct CachedClient {
    client: PoolsDataClient,
    cache: HashMap<String, (ProductionPoolData, Instant)>,
    cache_ttl: Duration,
}

impl CachedClient {
    pub async fn get_pool(&mut self, pool_name: &str) -> Result<ProductionPoolData> {
        // Check cache
        if let Some((data, timestamp)) = self.cache.get(pool_name) {
            if timestamp.elapsed() < self.cache_ttl {
                return Ok(data.clone());
            }
        }
        
        // Fetch fresh data
        let pools = self.client.fetch_pools(&[pool_name]).await?;
        let pool_data = pools[pool_name].clone();
        
        // Update cache
        self.cache.insert(pool_name.to_string(), (pool_data.clone(), Instant::now()));
        
        Ok(pool_data)
    }
}
```

## Batch Processing

```rust
// Process all available pools in batches
let all_pools = PoolsDataClient::list_available_pools();
let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();

for batch in pool_names.chunks(5) {
    match client.fetch_pools(batch).await {
        Ok(pools) => {
            process_batch(pools).await?;
        }
        Err(e) => {
            log::error!("Batch failed: {}", e);
            continue; // Skip failed batch
        }
    }
    
    // Rate limiting between batches
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

## Configuration for Different RPC Providers

```rust
// Public RPC (rate limited)
let client = PoolsDataClient::builder()
    .rate_limit(2)  // Conservative
    .timeout(30)
    .retry_attempts(3)
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;

// Private RPC (higher limits)
let client = PoolsDataClient::builder()
    .rate_limit(20)  // Higher limit
    .timeout(15)     // Faster timeout
    .max_concurrent_requests(10)
    .build("your_private_rpc_url")
    .and_then(PoolsDataClient::from_config)?;
```

## Production Checklist

- ✅ Use `fetch_pools()` for production (clean data)
- ✅ Use `fetch_pools_debug()` only for debugging
- ✅ Set appropriate rate limits for your RPC provider
- ✅ Implement error handling and logging
- ✅ Add caching for frequently accessed data
- ✅ Use batch processing for multiple pools
- ✅ Test with real data before deploying

## Why This Design is Better

**Before (3 confusing formats):**
- Complex warnings and documentation
- Risk of using wrong format
- Inconsistent schemas breaking databases

**Now (2 simple formats):**
- Clear purpose: production vs debug
- No confusing warnings or choices
- Predictable and safe

---

**Clean, simple, and safe to use! 🚀**