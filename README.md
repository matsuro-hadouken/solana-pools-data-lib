# Pools Data Library

Rust library for fetching Solana stake pools data. Supports production and debug formats, automatic RPC configuration, and all major pools.

## Features
- Pool statistics calculated in-library, returned from API
- 32 supported pools (Jito, Marinade, Lido, etc.)
- Rate limiting, retries, timeouts, provider presets

## Data Returned
- **ProductionPoolData**:
  - `pool_name`: Pool name
  - `authority`: Pool authority pubkey
  - `stake_accounts`: List of stake accounts (optimized)
  - `validator_distribution`: Validator summary
  - `statistics`: Pre-calculated pool statistics (active/deactivating/deactivated accounts, total lamports, validator count, etc.)
  - `fetched_at`: Timestamp
- **PoolStatistics**:
  - `total_accounts`, `active_accounts`, `deactivating_accounts`, `deactivated_accounts`
  - `total_lamports`, `active_stake_lamports`, `deactivating_stake_lamports`, `deactivated_stake_lamports`
  - `validator_count`
- **PoolsDataResult** (debug):
  - `successful`: Map of pool name to full pool data
  - `failed`: Map of pool name to error

## Available Methods
- `PoolsDataClient::builder()` - Create a client with custom configuration
- `PoolsDataClient::from_config(config)` - Instantiate client from a config struct
- `PoolsDataClient::list_available_pools()` - Returns all supported pools
- `PoolsDataClient::get_static_field_analysis()` - Returns static field analysis for stake accounts
- `PoolsDataClient::test_connection()` - Tests RPC endpoint connectivity
- `PoolsDataClient::fetch_pools(pool_names)` - Returns production data for specified pools. All statistics are pre-calculated
- `PoolsDataClient::fetch_all_pools()` - Returns production data for all supported pools
- `PoolsDataClient::fetch_pools_debug(pool_names)` - Returns full debug data for specified pools, including all raw RPC fields

## Installation
```toml
[dependencies]
solana-pools-data-lib = { git = "https://github.com/matsuro-hadouken/solana-pools-data-lib" }
tokio = { version = "1.0", features = ["full"] }
```

## Usage
```rust
use solana_pools_data_lib::PoolsDataClient;

let client = PoolsDataClient::builder()
    .rate_limit(5)
    .build("https://api.mainnet-beta.solana.com")
    .and_then(PoolsDataClient::from_config)?;

// Fetch pools with all statistics pre-calculated
let pools = client.fetch_pools(&["jito", "marinade"]).await?;
for (name, data) in pools {
    println!("Pool: {name}, Accounts: {}, Validators: {}, Total Staked: {} SOL",
        data.stake_accounts.len(),
        data.validator_distribution.len(),
        data.statistics.total_staked_lamports as f64 / 1_000_000_000.0
    );
}
```

## Output Formats
- **Production:** Clean, processed data for databases
- **Debug:** Full RPC response, all original fields

## Configuration
Provider presets:
`.auto_config(url)` | `.alchemy_config()` | `.quicknode_config()` | `.helius_config()`
Use-case presets:
`.public_rpc_config()` | `.private_rpc_config()` | `.high_frequency_config()` | `.batch_processing_config()` | `.development_config()` | `.enterprise_config()`
Manual tuning:
`.rate_limit(n)` | `.timeout(secs)` | `.retry_attempts(n)` | `.max_concurrent_requests(n)`

## Supported Pools
32 major Solana stake pools. List: `PoolsDataClient::list_available_pools()`

## Error Handling
All API methods return `Result`. Partial failures available in debug format.

## Common Use Cases
- Database storage (production format, stable schema)
- REST API integration
- Batch processing

# Examples
Run these to see usage patterns:
```bash
cargo run --example quick_test
cargo run --example basic
cargo run --example all_pools_statistics
cargo run --example comprehensive
cargo run --example rpc_configuration
```

## Documentation
- [Examples](examples/README.md)
- [Getting Started](GETTING_STARTED.md)
- [Integration Guide](INTEGRATION.md)

## License
MIT