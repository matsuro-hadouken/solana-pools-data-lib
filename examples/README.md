# Examples Reference

All examples demonstrate direct usage of the Solana Pools Data Library. Each example is self-contained and covers a specific technical scenario.

## Example List

- `basic.rs` — Show configuration patterns: rate limiting, timeouts, retries, concurrency.
- `quick_test.rs` — Run a quick library test and print supported pools.
- `comprehensive.rs` — Fetch and print data for all supported pools.
- `validator_accounts.rs` — List all stake accounts (open/closed) for a single validator in a pool, with full account details.
- `validator_map.rs` — Show all available fields for a single pool and validator.
- `all_pools_statistics.rs` — Print active, deactivating, and deactivated stake, total lamports, and account counts for all pools.
- `rpc_configuration.rs` — Demonstrate all RPC configuration options and presets.

## Configuration Reference

| Method                    | Description                        |
|--------------------------|------------------------------------|
| `rate_limit(n)`           | Requests per second                |
| `no_rate_limit()`         | Remove rate limiting               |
| `timeout(seconds)`        | Request timeout                    |
| `retry_attempts(n)`       | Number of retries                  |
| `retry_base_delay(ms)`    | Delay between retries              |
| `max_concurrent_requests(n)` | Parallel request limit           |
| `public_rpc_config()`     | Preset for public RPC endpoints    |
| `private_rpc_config()`    | Preset for private RPC endpoints   |

## Running Examples

Run any example with:
```bash
cargo run --example <example_name>
```
Replace `<example_name>` with the filename (without `.rs`).

## Recommended Patterns

- Use conservative rate limits and retries for public RPC endpoints.
- Use aggressive settings for private RPC endpoints.
- Always apply delays between requests to avoid rate limiting.
- Use fresh client instances for each pool when working with public RPC.

## Retry Logic
- Automatic exponential backoff on failures.
- Configure base delay and max attempts as needed.

## Output
All examples print structured pool data, statistics, and configuration results directly to stdout.