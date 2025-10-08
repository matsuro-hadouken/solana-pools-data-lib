# Pools Data Library

Production-ready Rust library for fetching Solana stake pool data via RPC. Features three output formats for different use cases, configurable rate limiting, retry logic, and enterprise-grade reliability.

## Features

- **Three output formats**: Full (debugging), Production (consistent schema), Optimized (variable schema)
- **Enterprise-ready**: Rate limiting (2-50 req/sec), exponential backoff, partial failure handling
- **Production safety**: Consistent JSON schemas, atomic precision lockup detection
- **Comprehensive**: Support for 32+ major Solana stake pools
- **Zero dependencies**: Pure data fetching interface, integrate with your architecture

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
pools-data-lib = { path = "." }
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // RECOMMENDED: Use production format for consistent schema
    let result = client.fetch_pools_production(&["jito", "marinade"]).await?;
    
    for (name, data) in &result {
        println!("{}: {:.2} SOL staked", name, data.total_staked_sol());
    }
    Ok(())
}
```

## Output Formats

### Three Format Architecture

**🔍 Full Format (Complete RPC data):**
```rust
let full_data = client.fetch_pools(&["jito"]).await?;
// ✅ Complete RPC response with all fields
// ✅ Use for RPC analysis and debugging
// ✅ Returns PoolsResult with successful/failed breakdown
```

**🏭 Production Format (Consistent schema - RECOMMENDED):**
```rust
let production_data = client.fetch_pools_production(&["jito"]).await?;
// ✅ RECOMMENDED for backends and databases
// ✅ Consistent JSON schema - same fields every time
// ✅ Removes only static fields, preserves all dynamic data
// ✅ Always includes lockup and authority for predictability
// ✅ Returns HashMap<String, ProductionPoolData> directly
```

**⚡ Optimized Format (Variable schema - DANGEROUS):**
```rust
let optimized_data = client.fetch_pools_optimized(&["jito"]).await?;
// ⚠️ UNSTABLE schema - fields appear/disappear
// ❌ Do NOT use for primary data storage
// ✅ OK for cached public APIs only
// ⚠️ Returns HashMap<String, OptimizedPoolData> with variable schema
```

### Format Comparison

| Field | Full | Production | Optimized |
|-------|------|------------|-----------|
| Static fields (owner, executable, etc.) | ✅ | ❌ | ❌ |
| Lockup (custodian, epoch, timestamp) | ✅ | ✅ Always | ⚠️ Sometimes |
| Authority (staker, withdrawer) | ✅ | ✅ Always | ⚠️ Sometimes |
| Delegation data | ✅ | ✅ | ✅ |
| Schema consistency | ✅ | ✅ | ❌ |

### Real JSON Examples

**Production format (CONSISTENT):**
```json
{
  "pubkey": "ABC123...",
  "lamports": 5000000000,
  "stake_type": "delegated",
  "delegation": { "validator": "DEF456..." },
  "authority": {
    "staker": "GHI789...",
    "withdrawer": "GHI789..."
  },
  "lockup": {
    "custodian": "11111111111111111111111111111111",
    "epoch": 0,
    "unix_timestamp": 0
  }
}
```

**Optimized format (UNPREDICTABLE):**
```json
// Sometimes (when no custom authority/lockup):
{
  "pubkey": "ABC123...",
  "lamports": 5000000000,
  "stake_type": "delegated",
  "delegation": { "validator": "DEF456..." }
}

// Other times (when custom authority/lockup present):
{
  "pubkey": "ABC123...",
  "lamports": 5000000000,
  "stake_type": "delegated", 
  "delegation": { "validator": "DEF456..." },
  "custom_authority": { "staker": "...", "withdrawer": "..." },
  "lockup": { "epoch": 500 }
}
```

### Production vs Optimized Logic

**🏭 Production Format (SAFE):**
- **Removes**: `owner`, `executable`, `program`, `space`, `rent_exempt_reserve` (truly static)
- **Always includes**: `lockup`, `authority`, `delegation`, `pubkey`, `lamports`
- **Schema**: **Predictable** - same fields every time
- **Size reduction**: ~5-15% (removes only static fields)
- **Use case**: Backend databases, API endpoints, production systems

**⚡ Optimized Format (RISKY):**  
- **Removes**: Static fields + conditionally removes dynamic fields
- **Sometimes includes**: `lockup`, `custom_authority` (when non-default)
- **Schema**: **Unpredictable** - fields appear/disappear
- **Size reduction**: ~30-50% (aggressive optimization)
- **Use case**: Cached public APIs only, never direct storage

### Production Recommendations

**✅ BACKEND SYSTEMS (Recommended):**
```rust
// Use production format for consistent schema
let data = client.fetch_pools_production(&["jito", "marinade"]).await?;
// Always same JSON structure, safe for databases
```

**✅ PUBLIC APIs (With caching):**
```rust
// Cache production format for best of both worlds
let cached = cache.get_or_compute("pools_data", || async {
    client.fetch_pools_production(&["jito", "marinade"]).await
}).await?;
```

**⚠️ SPECIAL CASES ONLY:**
```rust
// Optimized format only for specific use cases
let optimized = client.fetch_pools_optimized(&["jito"]).await?;
// Use ONLY with proper error handling for schema changes
```

**❌ NEVER DO THIS:**
```rust
// DON'T: Direct optimized to database
let data = client.fetch_pools_optimized(&pools).await?;
database.insert(data)?; // WILL BREAK when schemas change
```

### Size Analysis

```rust
// Compare all three formats
let comparison = client.compare_all_output_sizes(&["jito", "marinade"]).await?;
println!("Full: {} bytes", comparison.full_size_bytes);
println!("Production: {} bytes ({:.1}% reduction)", 
         comparison.production_size_bytes,
         comparison.production_reduction_percent);
println!("Optimized: {} bytes ({:.1}% reduction)",
         comparison.optimized_size_bytes, 
         comparison.optimized_reduction_percent);

// Compare just full vs optimized
let simple = client.compare_output_sizes(&["jito"]).await?;
```

### Atomic Precision Lockup Detection

```rust
// Standard approach (unsafe - makes assumptions)
if lockup.epoch == 0 { /* assumes default */ }

// Atomic precision (safe - verifies all fields)
if lockup.is_default_lockup() { 
    // Verified: custodian=system_program AND epoch=0 AND timestamp=0
}
```

**Detection rules:**
- Default: `custodian="11111...111" AND epoch=0 AND timestamp=0` → omitted
- Active constraints: any field ≠ default → preserved

## Configuration

### RPC Presets

```rust
.public_rpc_config()    // 2 req/sec, 3 concurrent, 3 retries
.private_rpc_config()   // 50 req/sec, 10 concurrent, 2 retries
.no_rate_limit()        // unlimited
```

### Custom Configuration

```rust
let client = PoolsDataClient::builder()
    .rate_limit(10)
    .max_concurrent_requests(5)
    .retry_attempts(3)
    .timeout(30)
    .build("https://your-rpc.com")
    .and_then(PoolsDataClient::from_config)?;
```

## API Methods

```rust
// FULL FORMAT: Complete RPC data with error handling
let result = client.fetch_all_pools().await?;                    // All pools
let result = client.fetch_pools(&["jito", "marinade"]).await?;   // Specific pools
// Returns: PoolsResult with successful/failed breakdown

// PRODUCTION FORMAT: Consistent schema (RECOMMENDED)
let data = client.fetch_pools_production(&["jito", "marinade"]).await?;
// Returns: HashMap<String, ProductionPoolData> directly

// OPTIMIZED FORMAT: Variable schema (use with caution)
let data = client.fetch_pools_optimized(&["jito", "marinade"]).await?;
// Returns: HashMap<String, OptimizedPoolData> with variable schema

// Retry failed requests from full format
let retry_result = client.retry_failed_pools(&result.failed).await?;

// Validator-specific stakes
let stakes = client.get_pools_staking_to_validator("vote_account").await?;

// Summary statistics
let summary = client.get_pools_summary().await?;

// Static field analysis
let analysis = PoolsDataClient::get_static_field_analysis();
```

## Supported Pools

32+ major Solana stake pools: jito, marinade, lido, blazestake, sanctum, binance, jupiter, and others.

## Error Handling

```rust
// Full format provides comprehensive error handling
let result = client.fetch_pools(&["jito", "invalid_pool"]).await?;

// Process successful results
for (name, data) in result.successful {
    println!("{}: {} accounts", name, data.stake_accounts.len());
}

// Handle failures with retry capability
for (name, error) in result.failed {
    if error.retryable {
        println!("Retryable failure for {}: {}", name, error.error);
    } else {
        println!("Permanent failure for {}: {}", name, error.error);
    }
}

// Production format: simple success or error
match client.fetch_pools_production(&["jito"]).await {
    Ok(data) => println!("Success: {} pools", data.len()),
    Err(e) => println!("Error: {}", e),
}
```

## Data Structures

```rust
// Full format structures
pub struct PoolData {
    pub pool_name: String,
    pub authority: String,
    pub stake_accounts: Vec<StakeAccountInfo>,
    pub validator_distribution: HashMap<String, ValidatorStake>,
    pub statistics: PoolStatistics,
    pub fetched_at: DateTime<Utc>,
}

pub struct StakeAccountInfo {
    pub pubkey: String,
    pub lamports: u64,
    pub rent_exempt_reserve: u64,
    pub delegation: Option<StakeDelegation>,
    pub authorized: StakeAuthorized,
    pub lockup: StakeLockup,
}

// Production format structures (consistent schema)
pub struct ProductionPoolData {
    pub pool_name: String,
    pub authority: String,
    pub stake_accounts: Vec<ProductionStakeAccountInfo>, // Consistent schema
    pub validator_distribution: HashMap<String, ValidatorStake>,
    pub statistics: PoolStatistics,
    pub fetched_at: DateTime<Utc>,
}

pub struct ProductionStakeAccountInfo {
    pub pubkey: String,
    pub lamports: u64,
    pub stake_type: String,
    pub delegation: Option<ProductionStakeDelegation>,
    pub authority: ProductionStakeAuthority,  // Always included
    pub lockup: ProductionStakeLockup,        // Always included
}

// Optimized format structures (variable schema)
pub struct OptimizedStakeAccountInfo {
    pub pubkey: String,
    pub lamports: u64,
    pub stake_type: String,
    pub delegation: Option<OptimizedStakeDelegation>,
    pub custom_authority: Option<OptimizedStakeAuthority>, // Sometimes missing
    pub lockup: Option<OptimizedStakeLockup>,              // Sometimes missing
}
```

## Performance Metrics

- **Memory**: ~500KB-1MB for all pools data
- **Network**: ~21KB per pool authority query
- **Rate limits**: 2 req/sec (public) / 50 req/sec (private)
- **Size optimization**: 5-15% (production) / 30-50% (optimized)
- **Concurrency**: Configurable concurrent requests (3-10)

## Examples

Run the included examples:

```bash
cargo run --example production_formats    # Compare all three formats
cargo run --example backend_compatibility # Backend integration patterns
cargo run --example data_organization     # Pool data structure overview
cargo run --example rpc_mapping          # RPC field mapping explanation
```

## Testing

```bash
cargo test                    # All tests (25 tests)
cargo test --lib             # Unit tests only  
cargo test -- --nocapture    # With output
cargo clippy                  # Linting
```

## Dependencies

- reqwest (HTTP), tokio (async), serde (JSON)
- governor (rate limiting), tokio-retry, thiserror

## Architecture

Production-ready data-fetching library designed for integration:

- **Zero caching**: Integrate with your cache layer
- **Zero persistence**: Integrate with your database
- **Three output formats**: Full (debugging), Production (consistent), Optimized (risky)
- **Enterprise features**: Rate limiting, timeouts, retries, partial failure recovery
- **Atomic precision**: Lockup detection without field assumptions
- **Comprehensive testing**: 25 unit tests covering all edge cases

## License

MIT