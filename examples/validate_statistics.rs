use solana_pools_data_lib::PoolsDataClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Engineers must fetch current_epoch from RPC or other source
    let rpc_url = "https://api.mainnet-beta.solana.com";
    let current_epoch = fetch_current_epoch(rpc_url).await?;
    println!("Current epoch: {}", current_epoch);

    let client = PoolsDataClient::builder()
        .auto_config(rpc_url)
        .build(rpc_url)
        .and_then(PoolsDataClient::from_config)?;

    let pool_stats = client.fetch_all_pools_with_stats(current_epoch).await?;
    for (pool_name, stats) in pool_stats.iter() {
        println!("Pool: {}", pool_name);
        for validator in &stats.validators {
            println!("  Validator: {}", validator.validator_pubkey);
            for account in &validator.accounts {
                println!("    Account: {} | State: {:?} | Lamports: {}", account.account_pubkey, account.account_state, account.account_size_in_lamports);
            }
        }
    }
    Ok(())
}

async fn fetch_current_epoch(rpc_url: &str) -> Result<u64, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getEpochInfo",
        "params": []
    });
    let resp = client.post(rpc_url).json(&body).send().await?;
    let resp_json: serde_json::Value = resp.json().await?;
    let epoch = resp_json["result"]["epoch"].as_u64().ok_or("No epoch in response")?;
    Ok(epoch)
}
