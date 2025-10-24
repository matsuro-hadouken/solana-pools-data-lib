### `validator_accounts.rs`
Show all stake accounts (open and closed) for a single validator in a pool, with full account details.
```bash
cargo run --example validator_accounts
```
### `validator_map.rs`
Minimal demo showing all available fields for a single pool and a single validator.
```bash
cargo run --example validator_map
```
# Examples

## Available Examples

### `rpc_configuration.rs`
Comprehensive RPC configuration guide for all providers and use cases:
```bash
cargo run --example rpc_configuration
```

### `quick_test.rs`
Library overview and quick test:
```bash
cargo run --example quick_test
```

### `basic.rs`
Configuration examples showing rate limiting, timeouts, and operational delays:
```bash
cargo run --example basic
```

### `comprehensive.rs`
Fetch all supported pools:
```bash
cargo run --example comprehensive
```

## Configuration Options

- `rate_limit(n)` - Requests per second
- `no_rate_limit()` - Remove rate limiting
- `timeout(seconds)` - Request timeout
- `retry_attempts(n)` - Number of retries
- `retry_base_delay(ms)` - Delay between retries
- `max_concurrent_requests(n)` - Parallel request limit

Use appropriate delays between operations for optimal performance.

## **COMPLETE CONFIGURATION REFERENCE**

The library provides comprehensive configuration options for different use cases:

### **Available Configuration Methods:**

```rust
let client = PoolsDataClient::builder()
    .rate_limit(requests_per_second)     // Rate limiting
    .no_rate_limit()                     // Disable rate limiting
    .retry_attempts(count)               // Number of retry attempts
    .retry_base_delay(milliseconds)      // Base delay for retries
    .timeout(seconds)                    // Request timeout
    .max_concurrent_requests(count)      // Concurrent request limit
    .public_rpc_config()                 // Preset for public RPC endpoints
    .private_rpc_config()                // Preset for private RPC endpoints
    .build(rpc_url)?;
```

### **Production Configurations:**

#### **PRESET CONFIGURATIONS (Recommended)**

**Quick Setup for Public RPC:**
```rust
.public_rpc_config()    // All optimized settings for public endpoints
```

**Quick Setup for Private RPC:**
```rust
.private_rpc_config()   // All optimized settings for premium endpoints
```

#### **MANUAL CONFIGURATIONS**

#### **CONSERVATIVE (Public RPC)**
```rust
.rate_limit(1)
.retry_attempts(3)
.retry_base_delay(200)
.timeout(10)
.max_concurrent_requests(1)
```
**Use case**: Public RPC endpoints, maximum reliability, fresh sessions per pool.

#### **MODERATE (Reliable RPC)**
```rust
.rate_limit(5)
.retry_attempts(2)
.retry_base_delay(100)
.timeout(5)
.max_concurrent_requests(3)
```
**Use case**: Reliable RPC providers, balanced performance and reliability.

#### **AGGRESSIVE (Private RPC)**
```rust
.no_rate_limit()
.retry_attempts(1)
.retry_base_delay(50)
.timeout(2)
.max_concurrent_requests(10)
```
**Use case**: Private RPC endpoints, maximum performance.

### **Retry Logic Details:**
- **Automatic retries**: Built-in exponential backoff
- **Base delay**: Starting delay for retries (200ms recommended)
- **Exponential backoff**: Each retry doubles the delay
- **Max attempts**: Default 3 attempts for public RPC

### **Rate Limiting:**
- **Conservative**: 1 request/second (public RPC)
- **Moderate**: 5 requests/second (reliable RPC)
- **Aggressive**: No limit (private RPC)

## **PROVEN OPTIMAL PATTERN**

Based on extensive testing with public RPC endpoints:

```rust
// PRODUCTION PATTERN - Copy this for your applications
for pool_name in pool_list {
    let client = PoolsDataClient::builder()
        .rate_limit(1)                    // Conservative rate limiting
        .retry_attempts(3)                // 3 retry attempts
        .retry_base_delay(200)            // 200ms + exponential backoff
        .timeout(3)                       // 3 second timeout
        .max_concurrent_requests(1)       // Fresh sessions per pool
        .build(rpc_url)?;
    
    match timeout(Duration::from_secs(3),
        client.fetch_pools_debug(&[pool_name])).await {
        Ok(Ok(result)) if !result.successful.is_empty() => {
            // Success with automatic retries
        }
        _ => {
            // Failed after all retries
        }
    }

    sleep(Duration::from_millis(8000)).await; // Prevent rate limiting
}
```

### **Configuration Rationale:**
- **Fresh client per pool** - Eliminates RPC session throttling
- **8-second delays** - Prevents rate limiting completely  
- **3-second timeout** - Fast failure detection (calls are 0.2-0.5s)
- **3 retry attempts** - Handles temporary network issues
- **Exponential backoff** - Smart retry strategy
- **High success rate** - Tested extensively

## Example Selection Guide

### **For comprehensive configuration options**
`cargo run --example basic` - Essential configuration reference

### **For complete pool analysis**
`cargo run --example comprehensive`

### **For library overview**
`cargo run --example quick_test`

### **For validator account mapping**
`cargo run --example validator_accounts` - Show all stake accounts for a single validator

## Running Examples

All examples can be run with:
```bash
cargo run --example <example_name>
```

Replace `<example_name>` with any filename (without `.rs` extension), e.g. `validator_accounts`.

## Production Ready

All examples demonstrate:
- Complete configuration coverage
- Professional output formatting
- Optimal settings
- Comprehensive error handling
- Real Solana mainnet data
- Detailed documentation

Suitable for building reliable Solana pool applications.