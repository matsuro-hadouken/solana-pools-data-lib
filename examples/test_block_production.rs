use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: BlockProductionResult,
}

#[derive(Debug, Deserialize)]
struct BlockProductionResult {
    value: BlockProductionValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BlockProductionValue {
    by_identity: HashMap<String, (u64, u64)>,
    range: SlotRange,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SlotRange {
    first_slot: u64,
    last_slot: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing getBlockProduction RPC call...\n");

    let client = reqwest::Client::new();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBlockProduction",
        "params": []
    });

    let response = client
        .post("https://api.mainnet-beta.solana.com")
        .json(&request)
        .send()
        .await?;

    let rpc_response: RpcResponse = response.json().await?;

    let value = rpc_response.result.value;
    let total_validators = value.by_identity.len();
    let slot_range = value.range.last_slot - value.range.first_slot;

    println!("Slot range: {} - {} ({} slots)",
        value.range.first_slot, value.range.last_slot, slot_range);
    println!("Total validators tracked: {}\n", total_validators);

    let mut skip_rates: Vec<(String, u64, u64, f64)> = value
        .by_identity
        .iter()
        .map(|(pubkey, (leader_slots, blocks_produced))| {
            let missed = leader_slots.saturating_sub(*blocks_produced);
            let skip_rate = if *leader_slots > 0 {
                (missed as f64 / *leader_slots as f64) * 100.0
            } else {
                0.0
            };
            (pubkey.clone(), *leader_slots, missed, skip_rate)
        })
        .collect();

    skip_rates.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    println!("Top 10 validators by skip rate (worst performers):\n");
    for (i, (pubkey, leader_slots, missed, skip_rate)) in skip_rates.iter().take(10).enumerate() {
        println!("{}. {}", i + 1, pubkey);
        println!("   Leader slots: {}", leader_slots);
        println!("   Missed slots: {}", missed);
        println!("   Skip rate: {:.2}%", skip_rate);
        println!();
    }

    skip_rates.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));

    println!("\nTop 10 validators by skip rate (best performers):\n");
    for (i, (pubkey, leader_slots, missed, skip_rate)) in skip_rates.iter().take(10).enumerate() {
        println!("{}. {}", i + 1, pubkey);
        println!("   Leader slots: {}", leader_slots);
        println!("   Missed slots: {}", missed);
        println!("   Skip rate: {:.2}%", skip_rate);
        println!();
    }

    let avg_skip_rate = skip_rates.iter().map(|(_, _, _, sr)| sr).sum::<f64>() / skip_rates.len() as f64;
    println!("Average skip rate across all validators: {:.2}%", avg_skip_rate);

    Ok(())
}
