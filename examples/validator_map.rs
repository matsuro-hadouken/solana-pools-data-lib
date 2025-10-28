use solana_pools_data_lib::*;

#[tokio::main]
async fn main() -> solana_pools_data_lib::Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let current_epoch = fetch_current_epoch(rpc_url).await?;

    // PoolsDataClient autodetects RPC type and configures optimal settings
    let config = PoolsDataClientBuilder::new().build(rpc_url)?;
    let client = PoolsDataClient::from_config(config)?;

    let pool_stats = client.fetch_all_pools_with_stats(current_epoch).await?;
    if let Some(stats) = pool_stats.get("jito") {
        println!("Pool: jito");
        println!("  Total Accounts: {}", stats.summary().total_accounts);
        println!("  Activating Accounts: {}", stats.summary().activating_accounts);
        println!("  Active Accounts: {}", stats.summary().active_accounts);
        println!("  Deactivating Accounts: {}", stats.summary().deactivating_accounts);
        println!("  Deactivated Accounts: {}", stats.summary().deactivated_accounts);
        println!("  Total SOL: {}", stats.summary().total_lamports as f64 / 1_000_000_000.0);
        println!("  Activating Stake SOL: {}", stats.summary().activating_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Active Stake SOL: {}", stats.summary().active_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Deactivating Stake SOL: {}", stats.summary().deactivating_stake_lamports as f64 / 1_000_000_000.0);
        println!("  Deactivated Stake SOL: {}", stats.summary().deactivated_stake_lamports as f64 / 1_000_000_000.0);
    println!("  Validator Count: {}", stats.validators.len());

        // Print first validator and first account for demo
        if let Some(vstat) = stats.validators.first() {
            println!("  Validator: {}", vstat.validator_pubkey);
            if let Some(account) = vstat.accounts.first() {
                println!("    Account: {}", account.account_pubkey);
                println!("      State: {:?}", account.account_state);
                println!("      SOL: {}", account.account_size_in_lamports as f64 / 1_000_000_000.0);
                println!("      Activation Epoch: {:?}", account.activation_epoch.unwrap_or_default());
                println!("      Deactivation Epoch: {:?}", account.deactivation_epoch.unwrap_or_default());
                println!("      Authority: staker={:?}, withdrawer={:?}", account.authorized_staker.clone().unwrap_or_default(), account.authorized_withdrawer.clone().unwrap_or_default());
            }
        }
    } else {
        println!("No Jito pool found.");
    }
    Ok(())
}

// Helper to fetch current epoch from RPC
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
