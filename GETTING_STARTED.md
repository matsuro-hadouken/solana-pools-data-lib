# Getting Started with Solana Pools Data Library

Get up and running with the Solana Pools Data Library in under 2 minutes!

## Quick Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
solana-pools-data-lib = { git = "https://github.com/matsuro-hadouken/solana-pools-data-lib" }
tokio = { version = "1.0", features = ["full"] }
```

## Your First Program

Create a new file `src/main.rs`:

```rust
use solana_pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client (use your private RPC for production!)
    let client = PoolsDataClient::builder()
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Get production-ready data (RECOMMENDED)
    let pools = client.fetch_pools_production(&["jito", "marinade"]).await?;
    
    for (name, data) in &pools {
        println!("Pool: {} - {:.2} SOL staked", name, data.total_staked_sol());
        println!("  Validators: {}", data.statistics.unique_validators);
        println!("  Accounts: {}", data.statistics.total_stake_accounts);
    }

    Ok(())
}
```

Run it:
```bash
cargo run
```

## Private RPC Setup (Production)

For production apps, use a private RPC endpoint:

### QuickNode
```rust
let client = PoolsDataClient::builder()
    .private_rpc_config()  // 50 req/sec, optimized for private RPC
    .build("https://your-endpoint.quiknode.pro/token/")
    .and_then(PoolsDataClient::from_config)?;
```

### Alchemy
```rust
let client = PoolsDataClient::builder()
    .private_rpc_config()
    .build("https://solana-mainnet.g.alchemy.com/v2/your-api-key")
    .and_then(PoolsDataClient::from_config)?;
```

### Helius
```rust
let client = PoolsDataClient::builder()
    .private_rpc_config()
    .build("https://rpc.helius.xyz/?api-key=your-api-key")
    .and_then(PoolsDataClient::from_config)?;
```

## Three Output Formats

Choose the right format for your use case:

### 🏭 Production Format (Recommended)
```rust
// Consistent JSON schema - perfect for databases
let data = client.fetch_pools_production(&["jito"]).await?;
// Always same fields, safe for production
```

### 🔍 Full Format (Debugging)
```rust
// Complete RPC response with error handling
let result = client.fetch_pools(&["jito"]).await?;
// Handle successful and failed requests separately
```

### ⚡ Optimized Format (Special Cases)
```rust
// Variable schema - use with extreme caution
let data = client.fetch_pools_optimized(&["jito"]).await?;
// Only for cached APIs, never direct storage!
```

## Basic Configuration

```rust
let client = PoolsDataClient::builder()
    .rate_limit(10)                    // 10 requests per second
    .max_concurrent_requests(5)        // 5 parallel requests
    .retry_attempts(3)                 // Retry failed requests 3 times
    .timeout(30)                       // 30 second timeout
    .build("https://your-rpc.com")
    .and_then(PoolsDataClient::from_config)?;
```

## Available Pools

The library supports 32+ major Solana stake pools:

```rust
let client = PoolsDataClient::builder()
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;

// Get all available pool names
let all_pools = client.list_available_pools();
for pool in &all_pools {
    println!("Pool: {} (Authority: {})", pool.name, pool.authority);
}

// Fetch data for all pools
let result = client.fetch_all_pools().await?;
println!("Successfully fetched {} pools", result.successful.len());
```

## Error Handling

```rust
use solana_pools_data_lib::PoolsDataError;

match client.fetch_pools_production(&["jito", "invalid_pool"]).await {
    Ok(pools) => {
        println!("Got {} pools", pools.len());
    }
    Err(PoolsDataError::PoolNotFound { pool_name }) => {
        eprintln!("Pool '{}' not found", pool_name);
    }
    Err(PoolsDataError::RpcError { message, .. }) => {
        eprintln!("RPC problem: {}", message);
    }
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## What's Next?

- 📖 Read the [Integration Guide](INTEGRATION.md) for production patterns
- 🔍 Check out the [examples](examples/) folder
- 📚 Run `cargo doc --open` for full API documentation
- 🧪 Try the demo: `cargo run --bin pools-data-demo`

## Need Help?

- Check the [examples](examples/) folder for more use cases
- Read the [Integration Guide](INTEGRATION.md) for production patterns
- All functions have comprehensive documentation with examples

---

**🚀 You're ready to build with Solana pools data!**