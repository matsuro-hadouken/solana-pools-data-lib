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
```

## What Each Example Shows

- **production_formats**: Two-format comparison with size metrics
- **data_organization**: Field analysis and data structure overview
- **backend_compatibility**: Database integration patterns
- **data_samples**: Raw JSON output and stake account structure
- **quick_test**: Available pools list and validator data
- **format_comparison**: Side-by-side JSON format differences

## Requirements

- Internet connection (fetches live Solana mainnet data)
- Examples use conservative rate limiting for public RPC

## QA Validation

All examples should run without errors and display structured pool data. Each example fetches real data from different pools to validate library functionality.