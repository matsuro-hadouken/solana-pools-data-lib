# Getting Started

Minimal setup for Solana Pools Data Library.

## Installation

Add to `Cargo.toml`:
```toml
[dependencies]
solana-pools-data-lib = { git = "https://github.com/matsuro-hadouken/solana-pools-data-lib" }
tokio = { version = "1.0", features = ["full"] }
```

## Quick Client Setup

```rust
use solana_pools_data_lib::PoolsDataClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Recommended: auto-configure for any RPC endpoint
    let client = PoolsDataClient::builder()
        .auto_config("https://api.mainnet-beta.solana.com")
        .build("https://api.mainnet-beta.solana.com")
        .and_then(PoolsDataClient::from_config)?;

    // Optional: set rate limits manually
    // let client = PoolsDataClient::builder()
    //     .rate_limit(5)
    //     .timeout(10)
    //     .build("https://api.mainnet-beta.solana.com")
    //     .and_then(PoolsDataClient::from_config)?;

    let pools = client.fetch_pools(&["jito"]).await?;
    for (name, data) in pools {
        println!("Pool: {name}, Accounts: {}, Validators: {}",
            data.stake_accounts.len(),
            data.validator_distribution.len()
        );
        // All statistics (active, activating, deactivating, inactive, waste, unknown) are pre-calculated and returned.
        // See data.statistics (PoolStatisticsFull) for canonical state breakdown and engineering details.
        // No manual calculation required.
    }
    Ok(())
}
```

## Debug Data Example

```rust
let debug_result = client.fetch_pools_debug(&["jito"]).await?;
for (pool_name, pool_data) in &debug_result.successful {
    println!("Pool {}: {} accounts", pool_name, pool_data.stake_accounts.len());
}
```

## Next Steps
- See [examples/README.md](examples/README.md) for configuration options
- See [INTEGRATION.md](INTEGRATION.md) for advanced usage and integration patterns