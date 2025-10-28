
# Pools Data Library

Rust library for fetching Solana stake pools data. Supports production and debug formats, RPC configuration, and 31 pools.

## Features
- Pool, validator, and account statistics calculated in-library
- 31 supported pools (Jito, Marinade, Lido, etc.)
- Rate limiting, retries, timeouts, provider presets

## Data Returned
- **ProductionPoolData**:
  - `pool_name`: Pool name
  - `authority`: Pool authority pubkey
  - `stake_accounts`: List of stake accounts (optimized)
  - `validator_distribution`: Validator summary (includes aggregated validator credits)
  - `statistics`: Pool statistics (state logic, account/validator states, edge cases)
  - `fetched_at`: Timestamp
- **PoolStatistics**:
  - State counts and lamports (activating, active, deactivating, deactivated)
  - Validator count and account/stake totals
- **PoolStatisticsFull**:
  - State counts and lamports for the pool (active, activating, deactivating, inactive, waste, unknown)
- **ValidatorStatisticsFull**:
  - State counts and lamports for each validator
  - Validator credits (total credits earned by each validator)
- **AccountStatisticsFull**:
  - State counts and lamports for each account
  - Credits not reported at account level
- **PoolsDataResult** (debug):
  - `successful`: Map of pool name to full pool data
  - `failed`: Map of pool name to error

## Available Methods
- `PoolsDataClient::builder()` - Create a client with custom configuration
- `PoolsDataClient::from_config(config)` - Instantiate client from a config struct
- `PoolsDataClient::list_available_pools()` - Returns all supported pools
- `PoolsDataClient::get_static_field_analysis()` - Returns static field analysis for stake accounts
- `PoolsDataClient::test_connection()` - Tests RPC endpoint connectivity
- `PoolsDataClient::fetch_pools(pool_names)` - Returns production data for specified pools
- `PoolsDataClient::fetch_all_pools()` - Returns production data for all supported pools
- `PoolsDataClient::fetch_pools_debug(pool_names)` - Returns debug data for specified pools with raw RPC fields

## Usage Note

Pass `current_epoch` to the library for state classification and statistics.
Fetch it from RPC to ensure stake/account states are calculated correctly for the current epoch.

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

// Fetch pools with statistics
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
- **Production:** Processed data for databases
- **Debug:** Full RPC response, original fields

## Configuration
Provider presets:
`.auto_config(url)` | `.alchemy_config()` | `.quicknode_config()` | `.helius_config()`
Use-case presets:
`.public_rpc_config()` | `.private_rpc_config()` | `.high_frequency_config()` | `.batch_processing_config()` | `.development_config()` | `.enterprise_config()`
Manual tuning:
`.rate_limit(n)` | `.timeout(secs)` | `.retry_attempts(n)` | `.max_concurrent_requests(n)`

## Supported Pools
31 Solana stake pools. List: `PoolsDataClient::list_available_pools()`

## Error Handling
All API methods return `Result`. Partial failures available in debug format.

## Common Use Cases
- Database storage (production format)
- REST API integration
- Batch processing

## Examples
Run examples:
```bash
cargo run --example quick_test
cargo run --example basic
cargo run --example all_pools_statistics
cargo run --example comprehensive
cargo run --example rpc_configuration
cargo run --example validate_statistics
cargo run --example validator_accounts
cargo run --example validator_map
```

## Documentation
- [Examples](examples/README.md)
- [Getting Started](GETTING_STARTED.md)
- [Integration Guide](INTEGRATION.md)

## License
MIT