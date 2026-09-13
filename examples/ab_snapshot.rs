//! Dump exactly what the production pipeline would store, to a local file.
//!
//! Read-only: fetches from RPC and writes one local JSON file. It never writes
//! to a database or any remote service. Use it to diff the new registry's output
//! against your backend before deploying.
//!
//!   # everything the new code would produce (294 pools)
//!   cargo run --example ab_snapshot -- --all --out snapshot-new.json
//!
//!   # only the 61 names that existed before auto-discovery
//!   cargo run --example ab_snapshot -- --names-file legacy61.txt --out snapshot-legacy.json
//!
//! Set SOLANA_RPC_URL to use a private endpoint; the public one will rate-limit
//! badly across ~294 getProgramAccounts calls.
//!
//! `fetched_at` is normalised to the epoch so a diff shows real data changes
//! rather than timestamps.

use solana_pools_data_lib::*;
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let out = flag("--out").unwrap_or_else(|| "snapshot.json".to_string());
    let names_file = flag("--names-file");
    let all = args.iter().any(|a| a == "--all");

    if !all && names_file.is_none() {
        eprintln!("usage: --all | --names-file <path>  [--out <path>]");
        std::process::exit(2);
    }

    let rpc_url =
        std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into());

    // Owned names first, then borrow — fetch_pools takes &[&str].
    let owned: Vec<String> = match &names_file {
        Some(p) => std::fs::read_to_string(p)
            .map_err(|e| PoolsDataError::InternalError {
                message: format!("cannot read {p}: {e}"),
            })?
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect(),
        None => get_active_pools().iter().map(|p| p.name.clone()).collect(),
    };
    let names: Vec<&str> = owned.iter().map(String::as_str).collect();

    eprintln!("rpc      : {rpc_url}");
    eprintln!("pools    : {}", names.len());
    eprintln!("output   : {out}  (no remote writes)\n");

    let client = PoolsDataClient::builder()
        .build(&rpc_url)
        .and_then(PoolsDataClient::from_config)?;

    // fetch_pools_debug reports per-pool failures instead of aborting the run,
    // which is what an A/B needs — a partial result is still comparable.
    let result = client.fetch_pools_debug(&names).await?;

    let mut snapshot: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut rows: Vec<(String, String, usize, u64, usize)> = Vec::new();

    for (name, pool) in &result.successful {
        let prod: ProductionPoolData = pool.into();
        let mut v = serde_json::to_value(&prod)?;
        // Normalise the timestamp so diffs show data, not clock drift.
        v["fetched_at"] = serde_json::json!("1970-01-01T00:00:00Z");
        snapshot.insert(name.clone(), v);
        rows.push((
            name.clone(),
            pool.authority.clone(),
            pool.stake_accounts.len(),
            pool.total_delegated_stake(),
            pool.validator_distribution.len(),
        ));
    }

    rows.sort_by_key(|r| std::cmp::Reverse(r.3));
    println!("{:<26} {:>7} {:>16} {:>6}  authority", "pool", "accts", "delegated SOL", "vals");
    for (name, auth, accts, lamports, vals) in &rows {
        println!(
            "{name:<26} {accts:>7} {:>16.3} {vals:>6}  {auth}",
            *lamports as f64 / 1e9
        );
    }

    std::fs::write(&out, serde_json::to_string_pretty(&snapshot)?).map_err(|e| {
        PoolsDataError::InternalError {
            message: format!("cannot write {out}: {e}"),
        }
    })?;

    eprintln!("\nok       : {} pools", result.successful.len());
    if !result.failed.is_empty() {
        eprintln!("failed   : {}", result.failed.len());
        for (name, err) in &result.failed {
            eprintln!("  {name}: {}", err.error);
        }
    }
    eprintln!("wrote    : {out}");
    Ok(())
}
