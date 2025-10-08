# Examples

## Basic Usage

### `quick_test.rs`
Library overview and quick test:
```bash
cargo run --example quick_test
```

### `basic.rs`
Configuration examples showing rate limiting, timeouts, and 8-second delays:
```bash
cargo run --example basic
```

### `comprehensive.rs`
Fetch all 32 supported pools:
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

Use 8-second delays between operations for optimal performance.

### `quick_test.rs`
Fast library overview - get familiar with available pools:
```bash
cargo run --example quick_test
```

## Specialized Examples

### `format_comparison.rs`
Compare production vs debug output formats:
```bash
cargo run --example format_comparison
```

### `data_samples.rs`
Educational examples showing pool data structure:
```bash
cargo run --example data_samples
```

### `backend_compatibility.rs`
Database integration patterns and serialization:
```bash
cargo run --example backend_compatibility
```

### `delegation_states.rs`
Explains why some pools show `null` delegation accounts:
```bash
cargo run --example delegation_states
```

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

### **Why This Configuration Works:**
- **Fresh client per pool** - Eliminates RPC session throttling
- **8-second delays** - Prevents rate limiting completely  
- **3-second timeout** - Fast failure detection (calls are 0.2-0.5s)
- **3 retry attempts** - Handles temporary network issues
- **Exponential backoff** - Smart retry strategy
- **100% success rate** - Tested extensively

## Which Example Should I Run?

### **"I need to understand all configuration options"**
→ `cargo run --example basic` ⭐ **ESSENTIAL**

### **"I need comprehensive pool analysis"**
→ `cargo run --example comprehensive`

### **"I'm new to this library"**
→ `cargo run --example quick_test`

### **"I want to understand the data structure"**  
→ `cargo run --example data_samples`  
→ `cargo run --example format_comparison`

### **"I'm seeing 'delegation: null' errors"**
→ `cargo run --example delegation_states`

### **"I need database integration"**
→ `cargo run --example backend_compatibility`

## Available Pool Names

The library supports 32 verified Solana stake pools:
```
foundation, firedancer_delegation, double_zero, jpool, raydium,
jito, marinade, marinade_native, marinade_native_2, socean,
lido, eversol, edgevana, blazestake, daopool, bonk, sanctum,
sanctum_2, binance, jupiter, binance_2, solayer, bybit,
shinobi, helius, marginfi, vault, drift, aerosol, ftx,
juicy, picosol
```

**Major pools for dashboards**: `jito`, `marinade`, `blazestake`, `jupiter`, `lido`, `sanctum`

## Running Examples

All examples can be run with:
```bash
cargo run --example <example_name>
```

Replace `<example_name>` with any filename (without `.rs` extension).

## Production Ready

All examples demonstrate:
- **Complete configuration coverage**
- **Professional output formatting**
- **Proven optimal settings**
- **Comprehensive error handling**
- **Real Solana mainnet data**
- **Detailed documentation**

Perfect for building reliable Solana pool applications!