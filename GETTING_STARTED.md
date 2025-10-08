# Getting Started

Quick setup guide

> **Note**: Library is in development - install from GitHub until published to crates.io.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
solana-pools-data-lib = { git = "https://github.com/matsuro-hadouken/solana-pools-data-lib" }
tokio = { version = "1.0", features = ["full"] }
```

Or clone and use locally:

```bash
git clone https://github.com/matsuro-hadouken/solana-pools-data-lib.git
cd solana-pools-data-lib/pools-data-lib
cargo run --example quick_test
```

## Two Simple Formats

**Production**: Clean data, safe for databases

**Debug**: Complete RPC data for debugging

## Quick Start

```rust
use solana_pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with optimized settings for public RPC
    let client = PoolsDataClient::builder()
        .public_rpc_config()  // Optimized for public endpoints
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Production format - clean and safe for databases
    let pools = client.fetch_pools(&["jito", "marinade"]).await?;
    
    for (pool_name, pool_data) in pools {
        println!("Pool: {}", pool_name);
        println!("  Authority: {}", pool_data.authority);
        println!("  Accounts: {}", pool_data.stake_accounts.len());
        println!("  Validators: {}", pool_data.validator_distribution.len());
    }
    
    Ok(())
}
```

## Debug Format (Complete Data)

```rust
// Get complete RPC data with all fields
let debug_result = client.fetch_pools_debug(&["jito"]).await?;

// Handle partial failures
if !debug_result.failed.is_empty() {
    for (pool_name, error) in &debug_result.failed {
        println!("Pool {} failed: {}", pool_name, error.error);
    }
}

// Process successful results
for (pool_name, pool_data) in &debug_result.successful {
    println!("Pool {}: {} accounts", pool_name, pool_data.stake_accounts.len());
    
    // Access all RPC fields for debugging
    for account in pool_data.stake_accounts.iter().take(2) {
        println!("  Account: {}", account.pubkey);
        println!("    Lamports: {}", account.lamports);
        if let Some(delegation) = &account.delegation {
            println!("    Delegated to: {}", delegation.voter);
        }
    }
}
```

## Next Steps

- **See working examples**: `cargo run --example basic`
- **Read integration patterns**: [INTEGRATION.md](INTEGRATION.md)
- **Configuration options**: [examples/README.md](examples/README.md)