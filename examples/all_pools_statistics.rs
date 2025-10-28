use solana_pools_data_lib::*;

async fn fetch_current_epoch(rpc_url: &str) -> solana_pools_data_lib::Result<u64> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getEpochInfo",
        "params": []
    });
    let resp = client.post(rpc_url)
        .json(&body)
        .send()
        .await?;
    let resp_json: serde_json::Value = resp.json().await?;
    let epoch = resp_json["result"]["epoch"].as_u64()
        .ok_or_else(|| PoolsDataError::ParseError { message: "No epoch in response".to_string() })?;
    Ok(epoch)
}

#[tokio::main]
async fn main() -> solana_pools_data_lib::Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let current_epoch = fetch_current_epoch(rpc_url).await?;
    println!("Current epoch: {}", current_epoch);

    // PoolsDataClient autodetects RPC type and configures optimal rate limits, timeouts, and concurrency
    // for public or private endpoints. No manual tuning required for best performance and reliability.
    let config = PoolsDataClientBuilder::new().build(rpc_url)?;
    let client = PoolsDataClient::from_config(config)?;

    let pool_stats = client.fetch_all_pools_with_stats(current_epoch).await?;
    println!("Fetched {} pools.", pool_stats.len());

    for (pool_name, stats) in pool_stats.iter() {
        let summary = stats.summary();
        println!("Pool: {}", pool_name);
        println!("  Total stake accounts: {}", summary.total_accounts);
        println!("  Activating accounts: {}", summary.activating_accounts);
        println!("    Activating stake (lamports): {}", summary.activating_stake_lamports);
        println!("    Activating stake (SOL): {:.2}", summary.activating_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Active accounts: {}", summary.active_accounts);
        println!("    Active stake (lamports): {}", summary.active_stake_lamports);
        println!("    Active stake (SOL): {:.2}", summary.active_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Deactivating accounts: {}", summary.deactivating_accounts);
        println!("    Deactivating stake (lamports): {}", summary.deactivating_stake_lamports);
        println!("    Deactivating stake (SOL): {:.2}", summary.deactivating_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Fully deactivated accounts: {}", summary.deactivated_accounts);
        println!("    Fully deactivated stake (lamports): {}", summary.deactivated_stake_lamports);
        println!("    Fully deactivated stake (SOL): {:.2}", summary.deactivated_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Total lamports in all stake accounts: {}", summary.total_lamports);
        println!("  Total lamports (SOL): {:.2}", summary.total_lamports as f64 / 1_000_000_000.0);
        println!("---");
    }
    Ok(())
}
