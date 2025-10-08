# Examples

Quick reference for testing the library with real Solana mainnet data.

## Run Examples

```bash
# Show both output formats side-by-side
cargo run --example production_formats

# Demonstrate clean data structure and field analysis
cargo run --example data_organization

# Show database-safe storage format
cargo run --example backend_compatibility

# Display actual JSON structure and stake account details
cargo run --example data_samples

# List all 32 supported pools and show validator distribution
cargo run --example quick_test

# Direct comparison of production vs debug JSON
cargo run --example format_comparison

# Explain delegation states (why some accounts have "delegation: null")
cargo run --example delegation_states
```

## What Each Example Shows

- **production_formats**: Two-format comparison with size metrics
- **data_organization**: Field analysis and data structure overview
- **backend_compatibility**: Database integration patterns
- **data_samples**: Raw JSON output and stake account structure
- **quick_test**: Available pools list and validator data
- **format_comparison**: Side-by-side JSON format differences
- **delegation_states**: Explains stake account states and null delegations

## Understanding Delegation States (Important!)

When you see `"delegation": null` in the JSON output, **this is normal behavior** for certain stake accounts:

### Three Account States:

1. **Initialized** (`delegation: null`): Account created but not yet staked
2. **Delegated** (`delegation: {...}`): Actively staked to a validator
3. **Deactivating** (`delegation: {...}` with deactivation info): Stake being withdrawn

### Why This Matters:

- Stake pools contain accounts in all three states
- `null` delegation **does not** mean broken data
- These accounts contribute to total pool value but aren't actively earning rewards
- Run `cargo run --example delegation_states` to see real examples

### Real Pool Data Example:
```
Jito Pool: 941 total accounts
- 2 accounts: Initialized (delegation: null)
- 937 accounts: Actively delegated 
- 2 accounts: Deactivating
```

This mixed state is normal pool operation - users deposit/withdraw continuously.

## Requirements

- Internet connection (fetches live Solana mainnet data)
- Examples use conservative rate limiting for public RPC

## QA Validation

All examples should run without errors and display structured pool data. Each example fetches real data from different pools to validate library functionality.