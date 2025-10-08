# Integration Guide

Production patterns and best practices for integrating the Solana Pools Data Library into your applications.

## Production Architecture Patterns

### Backend API Integration

```rust
use solana_pools_data_lib::PoolsDataClient;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PoolsService {
    client: PoolsDataClient,
    cache: Arc<RwLock<HashMap<String, CachedPoolData>>>,
}

impl PoolsService {
    pub fn new(rpc_url: &str) -> Result<Self, PoolsDataError> {
        let client = PoolsDataClient::builder()
            .private_rpc_config()  // Production settings
            .build(rpc_url)
            .and_then(PoolsDataClient::from_config)?;

        Ok(Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn get_pools(&self, pools: &[&str]) -> Result<HashMap<String, ProductionPoolData>, PoolsDataError> {
        // Always use production format for consistent schema
        self.client.fetch_pools_production(pools).await
    }

    pub async fn get_all_pools_cached(&self, cache_ttl: Duration) -> Result<HashMap<String, ProductionPoolData>, PoolsDataError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get("all_pools") {
                if cached.expires_at > Utc::now() {
                    return Ok(cached.data.clone());
                }
            }
        }

        // Fetch fresh data
        let result = self.client.fetch_all_pools().await?;
        let production_data: HashMap<String, ProductionPoolData> = result.successful
            .iter()
            .map(|(name, pool)| (name.clone(), pool.into()))
            .collect();

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert("all_pools".to_string(), CachedPoolData {
                data: production_data.clone(),
                expires_at: Utc::now() + cache_ttl,
            });
        }

        Ok(production_data)
    }
}

#[derive(Clone)]
struct CachedPoolData {
    data: HashMap<String, ProductionPoolData>,
    expires_at: DateTime<Utc>,
}
```

### Database Storage Pattern

```rust
use serde_json;
use sqlx::{Pool, Postgres, Row};

pub struct PoolsRepository {
    pool: Pool<Postgres>,
}

impl PoolsRepository {
    pub async fn store_pools(&self, pools: &HashMap<String, ProductionPoolData>) -> Result<(), sqlx::Error> {
        for (pool_name, pool_data) in pools {
            // Production format guarantees consistent schema
            let json_data = serde_json::to_value(pool_data)?;
            
            sqlx::query!(
                r#"
                INSERT INTO pools_data (pool_name, authority, data, updated_at)
                VALUES ($1, $2, $3, NOW())
                ON CONFLICT (pool_name) 
                DO UPDATE SET 
                    data = $3,
                    updated_at = NOW()
                "#,
                pool_name,
                pool_data.authority,
                json_data
            )
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn get_pool(&self, pool_name: &str) -> Result<Option<ProductionPoolData>, sqlx::Error> {
        let row = sqlx::query!(
            "SELECT data FROM pools_data WHERE pool_name = $1",
            pool_name
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let pool_data: ProductionPoolData = serde_json::from_value(row.data)?;
            Ok(Some(pool_data))
        } else {
            Ok(None)
        }
    }
}
```

### REST API Pattern

```rust
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};

pub fn create_router(pools_service: Arc<PoolsService>) -> Router {
    Router::new()
        .route("/pools", get(get_all_pools))
        .route("/pools/:name", get(get_pool))
        .route("/pools/:name/validators", get(get_pool_validators))
        .with_state(pools_service)
}

async fn get_all_pools(
    State(service): State<Arc<PoolsService>>,
) -> Result<Json<HashMap<String, ProductionPoolData>>, StatusCode> {
    match service.get_all_pools_cached(Duration::minutes(5)).await {
        Ok(pools) => Ok(Json(pools)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_pool(
    Path(pool_name): Path<String>,
    State(service): State<Arc<PoolsService>>,
) -> Result<Json<ProductionPoolData>, StatusCode> {
    match service.get_pools(&[&pool_name]).await {
        Ok(mut pools) => {
            if let Some(pool_data) = pools.remove(&pool_name) {
                Ok(Json(pool_data))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(serde::Deserialize)]
struct ValidatorsQuery {
    limit: Option<usize>,
}

async fn get_pool_validators(
    Path(pool_name): Path<String>,
    Query(params): Query<ValidatorsQuery>,
    State(service): State<Arc<PoolsService>>,
) -> Result<Json<Vec<ValidatorInfo>>, StatusCode> {
    match service.get_pools(&[&pool_name]).await {
        Ok(mut pools) => {
            if let Some(pool_data) = pools.remove(&pool_name) {
                let mut validators: Vec<_> = pool_data.validator_distribution
                    .into_iter()
                    .map(|(pubkey, stake_info)| ValidatorInfo {
                        pubkey,
                        total_stake: stake_info.total_delegated,
                        account_count: stake_info.account_count,
                        stake_percentage: stake_info.total_delegated as f64 / pool_data.statistics.total_staked_lamports as f64 * 100.0,
                    })
                    .collect();
                
                validators.sort_by(|a, b| b.total_stake.cmp(&a.total_stake));
                
                if let Some(limit) = params.limit {
                    validators.truncate(limit);
                }
                
                Ok(Json(validators))
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(serde::Serialize)]
struct ValidatorInfo {
    pubkey: String,
    total_stake: u64,
    account_count: u32,
    stake_percentage: f64,
}
```

## Format Selection Guide

### 🏭 Production Format (Recommended)
**Use for**: Backend APIs, database storage, production systems

```rust
// ✅ Consistent schema every time
let pools = client.fetch_pools_production(&["jito", "marinade"]).await?;

// Safe to store in database
database.store_pools(&pools).await?;

// Safe to serialize for APIs
let json = serde_json::to_string(&pools)?;
```

**Benefits**:
- Identical JSON structure every time
- Safe for database schemas
- Reliable for API contracts
- ~5-15% size reduction vs full format

### 🔍 Full Format (Error Handling)
**Use for**: Debugging, monitoring, error recovery

```rust
// ✅ Complete error information
let result = client.fetch_pools(&["jito", "invalid"]).await?;

// Handle partial failures
for (pool_name, error) in &result.failed {
    if error.retryable {
        // Retry logic
        let retry_result = client.retry_failed_pools(&result.failed).await?;
    } else {
        log::error!("Permanent failure for pool {}: {}", pool_name, error.error);
    }
}

// Process successful results
for (pool_name, pool_data) in &result.successful {
    database.store_pool(pool_name, pool_data).await?;
}
```

### ⚡ Optimized Format (Special Cases Only)
**Use for**: Cached public APIs, bandwidth optimization

```rust
// ⚠️ ONLY for cached scenarios with proper error handling
async fn get_cached_optimized_data(cache: &Cache) -> Result<OptimizedData, Error> {
    if let Some(cached) = cache.get("optimized_pools").await? {
        return Ok(cached);
    }

    let optimized = client.fetch_pools_optimized(&["jito"]).await?;
    
    // Cache the result
    cache.set("optimized_pools", &optimized, Duration::minutes(5)).await?;
    
    Ok(optimized)
}
```

**⚠️ Never do this**:
```rust
// ❌ DON'T store optimized format directly
let optimized = client.fetch_pools_optimized(&pools).await?;
database.store(optimized).await?; // Schema will break when lockups change!
```

## Error Handling Patterns

### Comprehensive Error Handling

```rust
use solana_pools_data_lib::{PoolsDataError, PoolError};

async fn robust_pool_fetching(client: &PoolsDataClient, pools: &[&str]) -> AppResult<PoolsData> {
    match client.fetch_pools_production(pools).await {
        Ok(pools_data) => Ok(pools_data),
        
        Err(PoolsDataError::PoolNotFound { pool_name }) => {
            log::warn!("Pool '{}' not found, skipping", pool_name);
            // Retry with remaining pools
            let remaining: Vec<&str> = pools.iter()
                .filter(|&&p| p != pool_name)
                .copied()
                .collect();
            client.fetch_pools_production(&remaining).await
                .map_err(Into::into)
        }
        
        Err(PoolsDataError::RpcError { code, message }) => {
            log::error!("RPC error {}: {}", code, message);
            Err(AppError::RpcFailure { code, message })
        }
        
        Err(PoolsDataError::NetworkError(msg)) => {
            log::error!("Network error: {}", msg);
            Err(AppError::NetworkFailure(msg))
        }
        
        Err(e) => {
            log::error!("Unexpected error: {}", e);
            Err(AppError::Unknown(e.to_string()))
        }
    }
}
```

### Retry Logic with Exponential Backoff

```rust
use tokio::time::{sleep, Duration};

pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
        }
    }
}

pub async fn fetch_with_retry(
    client: &PoolsDataClient,
    pools: &[&str],
    config: &RetryConfig,
) -> Result<HashMap<String, ProductionPoolData>, PoolsDataError> {
    let mut last_error = None;
    
    for attempt in 1..=config.max_attempts {
        match client.fetch_pools_production(pools).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                last_error = Some(e);
                
                if attempt < config.max_attempts {
                    let delay = std::cmp::min(
                        config.base_delay * 2_u32.pow(attempt - 1),
                        config.max_delay,
                    );
                    
                    log::warn!("Attempt {} failed, retrying in {:?}", attempt, delay);
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap())
}
```

## Performance Optimization

### Connection Pooling

```rust
use std::sync::Arc;

pub struct PoolsClientPool {
    clients: Arc<Vec<PoolsDataClient>>,
    current: AtomicUsize,
}

impl PoolsClientPool {
    pub fn new(rpc_urls: &[String], pool_size: usize) -> Result<Self, PoolsDataError> {
        let mut clients = Vec::with_capacity(pool_size);
        
        for _ in 0..pool_size {
            let rpc_url = &rpc_urls[clients.len() % rpc_urls.len()];
            let client = PoolsDataClient::builder()
                .private_rpc_config()
                .build(rpc_url)
                .and_then(PoolsDataClient::from_config)?;
            clients.push(client);
        }
        
        Ok(Self {
            clients: Arc::new(clients),
            current: AtomicUsize::new(0),
        })
    }
    
    pub fn get_client(&self) -> &PoolsDataClient {
        let index = self.current.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        &self.clients[index]
    }
}
```

### Batch Processing

```rust
pub async fn fetch_all_pools_batched(
    client: &PoolsDataClient,
    batch_size: usize,
) -> Result<HashMap<String, ProductionPoolData>, PoolsDataError> {
    let all_pools = client.list_available_pools();
    let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();
    
    let mut all_data = HashMap::new();
    
    for batch in pool_names.chunks(batch_size) {
        match client.fetch_pools_production(batch).await {
            Ok(batch_data) => {
                all_data.extend(batch_data);
            }
            Err(e) => {
                log::error!("Batch failed: {}", e);
                // Continue with remaining batches
                continue;
            }
        }
        
        // Rate limiting between batches
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    
    Ok(all_data)
}
```

## Testing Patterns

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_production_format_consistency() {
        let client = PoolsDataClient::builder()
            .build("https://api.mainnet-beta.solana.com")
            .and_then(PoolsDataClient::from_config)
            .unwrap();

        let pools1 = client.fetch_pools_production(&["jito"]).await.unwrap();
        let pools2 = client.fetch_pools_production(&["jito"]).await.unwrap();

        // Schema should be identical
        let json1 = serde_json::to_string(&pools1).unwrap();
        let json2 = serde_json::to_string(&pools2).unwrap();
        
        // Same structure (though values may differ)
        assert_eq!(json1.matches('{').count(), json2.matches('{').count());
        assert_eq!(json1.matches('[').count(), json2.matches('[').count());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let client = PoolsDataClient::builder()
            .build("https://api.mainnet-beta.solana.com")
            .and_then(PoolsDataClient::from_config)
            .unwrap();

        // Test invalid pool
        let result = client.fetch_pools_production(&["invalid_pool"]).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PoolsDataError::PoolNotFound { pool_name } => {
                assert_eq!(pool_name, "invalid_pool");
            }
            _ => panic!("Expected PoolNotFound error"),
        }
    }
}
```

### Integration Testing

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_integration() {
        let service = PoolsService::new("https://api.mainnet-beta.solana.com").unwrap();
        
        // Test basic functionality
        let pools = service.get_pools(&["jito", "marinade"]).await.unwrap();
        assert!(pools.contains_key("jito"));
        assert!(pools.contains_key("marinade"));
        
        // Test caching
        let cached_pools = service.get_all_pools_cached(Duration::minutes(1)).await.unwrap();
        assert!(!cached_pools.is_empty());
    }
}
```

## Production Deployment Checklist

- [ ] **Use private RPC endpoints** for production traffic
- [ ] **Always use production format** for database storage
- [ ] **Implement proper error handling** with retries
- [ ] **Add monitoring and logging** for RPC failures
- [ ] **Set up health checks** for RPC connectivity
- [ ] **Configure rate limiting** appropriate for your RPC provider
- [ ] **Implement caching** to reduce RPC calls
- [ ] **Test with real data** before deploying
- [ ] **Monitor pool data freshness** and update frequency
- [ ] **Have fallback RPC endpoints** for redundancy

## Common Pitfalls to Avoid

❌ **Don't store optimized format directly**
```rust
// Bad - schema can change
let optimized = client.fetch_pools_optimized(&pools).await?;
database.store(optimized).await?;
```

❌ **Don't ignore partial failures**
```rust
// Bad - loses error information
let result = client.fetch_pools(&pools).await?;
// Should handle result.failed
```

❌ **Don't use public RPC for production**
```rust
// Bad for production - rate limited
let client = PoolsDataClient::builder()
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;
```

✅ **Do use production format consistently**
```rust
// Good - consistent schema
let pools = client.fetch_pools_production(&pools).await?;
database.store(pools).await?;
```

✅ **Do handle errors appropriately**
```rust
// Good - comprehensive error handling
match client.fetch_pools(&pools).await? {
    result => {
        database.store(result.successful).await?;
        for (pool, error) in result.failed {
            if error.retryable {
                // Implement retry logic
            }
        }
    }
}
```

---

**🏆 Following these patterns will ensure your application is production-ready and robust!**