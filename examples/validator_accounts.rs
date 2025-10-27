use solana_pools_data_lib::*;

// Minimal example: show accounts delegated to a validator in a pool, using canonical API and autodetected RPC config
#[tokio::main]
async fn main() -> solana_pools_data_lib::Result<()> {
    let rpc_url = "https://api.mainnet-beta.solana.com";
    // Fetch current epoch for canonical state classification
    let current_epoch = fetch_current_epoch(rpc_url).await?;

    // PoolsDataClient autodetects RPC type and configures optimal settings
    let config = PoolsDataClientBuilder::new().build(rpc_url)?;
    let client = PoolsDataClient::from_config(config)?;

    let pool_stats = client.fetch_all_pools_with_stats(current_epoch).await?;

    if let Some(stats) = pool_stats.get("foundation") {
        let validator = "5iZ5PQPy5Z9XDnkfoWPi6nvUgtxWnRFwZ36WaftPuaVM";
        println!("Validator: {}", validator);
        println!("Accounts delegated to this validator (open and closed):");
        let mut found = false;
        for vstat in &stats.validators {
            if vstat.validator_pubkey == validator {
                for account in &vstat.accounts {
                    found = true;
                    println!("  Account: {}", account.account_pubkey);
                    println!("    State: {:?}", account.account_state);
                    println!("    Lamports: {}", account.account_size_in_lamports);
                    println!("    Activation Epoch: {:?}", account.activation_epoch);
                    println!("    Deactivation Epoch: {:?}", account.deactivation_epoch);
                    println!("    Authority: staker={:?}, withdrawer={:?}", account.authorized_staker, account.authorized_withdrawer);
                }
            }
        }
        if !found {
            println!("No accounts found for this validator in the Foundation pool.");
        }
    } else {
        println!("No Foundation pool found.");
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