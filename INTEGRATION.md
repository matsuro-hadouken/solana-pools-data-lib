# Integration Guide

Production usage patterns for Solana Pools Data Library.

## Configuration Patterns

```rust
// Auto-detect (recommended)
let client = PoolsDataClient::builder()
    .auto_config("https://api.mainnet-beta.solana.com")
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;

// Provider presets
let client = PoolsDataClient::builder().alchemy_config().build("<alchemy_url>").and_then(PoolsDataClient::from_config)?;
let client = PoolsDataClient::builder().quicknode_config().build("<quicknode_url>").and_then(PoolsDataClient::from_config)?;
let client = PoolsDataClient::builder().helius_config().build("<helius_url>").and_then(PoolsDataClient::from_config)?;

// Manual tuning
let client = PoolsDataClient::builder()
    .rate_limit(5)
    .timeout(10)
    .retry_attempts(2)
    .max_concurrent_requests(3)
    .build("<rpc_url>")
    .and_then(PoolsDataClient::from_config)?;
```

## Database Storage

```rust
let pools = client.fetch_pools(&["jito", "marinade"]).await?;
for (pool_name, pool_data) in pools {
    let json = serde_json::to_string(&pool_data)?;
    database.insert(&pool_name, &json).await?;
}
```

## REST API Integration

```rust
async fn get_pool_handler(pool_name: &str) -> Result<Json<Value>, String> {
    let client = PoolsDataClient::builder().public_rpc_config().build("https://api.mainnet-beta.solana.com").and_then(PoolsDataClient::from_config).map_err(|e| e.to_string())?;
    let pools = client.fetch_pools(&[pool_name]).await.map_err(|e| e.to_string())?;
    pools.into_values().next().map(|data| Json(serde_json::to_value(data).unwrap())).ok_or_else(|| format!("Pool {} not found", pool_name))
}
```

## Error Handling

```rust
match client.fetch_pools(&["jito"]).await {
    Ok(pools) => { /* handle success */ }
    Err(e) => { /* handle error */ }
}
```

## Debug Mode

```rust
let debug_result = client.fetch_pools_debug(&["jito"]).await?;
for (pool_name, error) in &debug_result.failed {
    println!("Pool {} failed: {}", pool_name, error.error);
}
for (pool_name, pool_data) in &debug_result.successful {
    println!("Pool {}: {} stake accounts", pool_name, pool_data.stake_accounts.len());
}
```

## Caching Pattern

```rust
struct CachedPoolClient { /* ... */ }
// Check cache, fetch fresh data if expired, update cache
```

## Batch Processing

```rust
let all_pools = PoolsDataClient::list_available_pools();
let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();
for batch in pool_names.chunks(3) {
    let pools = client.fetch_pools(batch).await?;
    // process pools
    sleep(Duration::from_millis(2000)).await;
}
```

## Custom Configuration

```rust
// Conservative (public RPC)
let client = PoolsDataClient::builder().rate_limit(1).retry_attempts(3).timeout(10).build("https://api.mainnet-beta.solana.com").and_then(PoolsDataClient::from_config)?;
// Aggressive (private RPC)
let client = PoolsDataClient::builder().no_rate_limit().retry_attempts(1).timeout(2).max_concurrent_requests(10).build("<private_url>").and_then(PoolsDataClient::from_config)?;
```

## Production Checklist
- Use preset configs for quick setup
- Use `fetch_pools()` for production (optimized, stable schema)
- Use `fetch_pools_debug()` for troubleshooting only
- Implement error handling and logging
- Add caching for frequent queries
- Use batch processing for multiple pools
- Apply rate limiting between batches
- Test with real data before deployment