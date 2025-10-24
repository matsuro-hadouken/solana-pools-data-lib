use solana_pools_data_lib::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: VoteAccountsResult,
}

#[derive(Debug, Deserialize)]
struct VoteAccountsResult {
    current: Vec<VoteAccount>,
    delinquent: Vec<VoteAccount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VoteAccount {
    vote_pubkey: String,
    epoch_credits: Vec<(u64, u64, u64)>,
}

struct ValidatorMetrics {
    stake: u64,
    credits: u64,
    accounts: usize,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    println!("=== POOLS ANALYSIS ===\n");

    let rpc_url = "https://api.mainnet-beta.solana.com";

    println!("Fetching validator vote credits...");
    let validator_credits = fetch_all_validator_credits(rpc_url).await?;
    println!("Retrieved {} validators\n", validator_credits.len());

    let client = PoolsDataClient::builder()
        .helius_config()
        .build(rpc_url)
        .and_then(PoolsDataClient::from_config)?;

    let available_pools = PoolsDataClient::list_available_pools();
    let pool_names: Vec<&str> = available_pools.iter().map(|p| p.name.as_str()).collect();

    println!("Analyzing {} pools\n", pool_names.len());

    match client.fetch_pools(&pool_names).await {
        Ok(pools) => {
            let mut all_pools_stats = Vec::new();

            for (pool_name, pool_data) in &pools {
                let mut validator_metrics: HashMap<String, ValidatorMetrics> = HashMap::new();

                for account in &pool_data.stake_accounts {
                    if let Some(delegation) = &account.delegation {
                        let entry = validator_metrics
                            .entry(delegation.validator.clone())
                            .or_insert_with(|| ValidatorMetrics {
                                stake: 0,
                                credits: 0,
                                accounts: 0,
                            });

                        entry.stake += delegation.stake_lamports;
                        entry.accounts += 1;

                        if entry.credits == 0 {
                            if let Some(&credits) = validator_credits.get(&delegation.validator) {
                                entry.credits = credits;
                            }
                        }
                    }
                }

                let mut validators_by_stake: Vec<_> = validator_metrics.iter().collect();
                validators_by_stake.sort_by(|a, b| b.1.stake.cmp(&a.1.stake));

                let mut validators_by_credits: Vec<_> = validator_metrics
                    .iter()
                    .filter(|(_, m)| m.credits > 0)
                    .collect();
                validators_by_credits.sort_by(|a, b| b.1.credits.cmp(&a.1.credits));

                let total_stake: u64 = validator_metrics.values().map(|m| m.stake).sum();
                let total_credits: u64 = validator_metrics.values().map(|m| m.credits).sum();
                let validators_with_credits = validator_metrics.values().filter(|m| m.credits > 0).count();

                let avg_stake = if !validators_by_stake.is_empty() {
                    total_stake / validators_by_stake.len() as u64
                } else {
                    0
                };

                let avg_credits = if validators_with_credits > 0 {
                    total_credits / validators_with_credits as u64
                } else {
                    0
                };

                let concentration = calculate_gini_coefficient(&validators_by_stake);
                let correlation = calculate_stake_credit_correlation(&validator_metrics);

                let top_10_stake_total: u64 = validators_by_stake
                    .iter()
                    .take(10)
                    .map(|(_, m)| m.stake)
                    .sum();
                let top_10_stake_percent = if total_stake > 0 {
                    (top_10_stake_total as f64 / total_stake as f64) * 100.0
                } else {
                    0.0
                };

                println!("Pool: {}", pool_name);
                println!("  Validators: {}", validator_metrics.len());
                println!("  Accounts: {}", pool_data.stake_accounts.len());
                println!("  Total Stake: {} SOL", format_sol(total_stake));
                println!("  Total Credits: {}", format_number(total_credits));
                println!("  Avg Stake/Validator: {} SOL", format_sol(avg_stake));
                println!("  Avg Credits/Validator: {}", format_number(avg_credits));
                println!("  Concentration Score: {:.3}", concentration);
                println!("  Stake-Credit Correlation: {:.3}", correlation);
                println!("  Top 10 Validators Hold: {:.1}% of stake", top_10_stake_percent);
                println!();

                all_pools_stats.push((
                    pool_name.clone(),
                    total_stake,
                    total_credits,
                    avg_credits,
                    concentration,
                    correlation,
                    validator_metrics.len(),
                ));
            }

            println!("\n=== SUMMARY: By Total Stake ===\n");
            let mut by_stake = all_pools_stats.clone();
            by_stake.sort_by(|a, b| b.1.cmp(&a.1));

            for (i, (name, stake, credits, _avg_credits, conc, corr, validators)) in by_stake.iter().enumerate() {
                println!(
                    "{}. {} - {} SOL ({} credits, Conc: {:.3}, Corr: {:.3}, {} val)",
                    i + 1,
                    name,
                    format_sol(*stake),
                    format_number(*credits),
                    conc,
                    corr,
                    validators
                );
            }

            println!("\n=== SUMMARY: By Stake-Credit Correlation (High = Credits Drive Stake) ===\n");
            let mut by_correlation = all_pools_stats.clone();
            by_correlation.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal));

            for (i, (name, stake, _credits, avg_credits, _conc, corr, validators)) in by_correlation.iter().enumerate() {
                println!(
                    "{}. {} - Corr: {:.3} ({} SOL, Avg Credits: {}, {} val)",
                    i + 1,
                    name,
                    corr,
                    format_sol(*stake),
                    format_number(*avg_credits),
                    validators
                );
            }

            println!("\n=== SUMMARY: By Concentration (Low = More Equal Distribution) ===\n");
            let mut by_concentration = all_pools_stats.clone();
            by_concentration.sort_by(|a, b| a.4.partial_cmp(&b.4).unwrap_or(std::cmp::Ordering::Equal));

            for (i, (name, stake, _credits, _avg_credits, conc, corr, validators)) in by_concentration.iter().enumerate() {
                println!(
                    "{}. {} - Conc: {:.3} (Corr: {:.3}, {} SOL, {} val)",
                    i + 1,
                    name,
                    conc,
                    corr,
                    format_sol(*stake),
                    validators
                );
            }

            println!("\n=== SUMMARY: By Validator Quality (Avg Credits) ===\n");
            let mut by_quality = all_pools_stats.clone();
            by_quality.sort_by(|a, b| b.3.cmp(&a.3));

            for (i, (name, stake, _credits, avg_credits, _conc, corr, validators)) in by_quality.iter().enumerate() {
                println!(
                    "{}. {} - Avg Credits: {} ({} SOL, Corr: {:.3}, {} val)",
                    i + 1,
                    name,
                    format_number(*avg_credits),
                    format_sol(*stake),
                    corr,
                    validators
                );
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    println!("\nExecution time: {:.2?}", start.elapsed());
    Ok(())
}

async fn fetch_all_validator_credits(
    rpc_url: &str,
) -> std::result::Result<HashMap<String, u64>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getVoteAccounts",
        "params": []
    });

    let response = client.post(rpc_url).json(&request).send().await?;
    let rpc_response: RpcResponse = response.json().await?;

    let mut credits_map = HashMap::new();

    for validator in rpc_response
        .result
        .current
        .iter()
        .chain(&rpc_response.result.delinquent)
    {
        if let Some(&( _epoch, cum, prev )) = validator.epoch_credits.last() {
            credits_map.insert(validator.vote_pubkey.clone(), cum.saturating_sub(prev));
        }
    }

    Ok(credits_map)
}

fn calculate_gini_coefficient(validators: &[(&String, &ValidatorMetrics)]) -> f64 {
    if validators.is_empty() {
        return 0.0;
    }

    let mut stakes: Vec<f64> = validators.iter().map(|(_, m)| m.stake as f64).collect();
    stakes.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = stakes.len() as f64;
    let total: f64 = stakes.iter().sum();

    if total == 0.0 {
        return 0.0;
    }

    let mut gini_sum = 0.0;

    for (i, &stake) in stakes.iter().enumerate() {
        gini_sum += (2.0 * (i as f64 + 1.0) - n - 1.0) * stake;
    }

    gini_sum / (n * total)
}

fn calculate_stake_credit_correlation(validators: &HashMap<String, ValidatorMetrics>) -> f64 {
    let mut stakes = Vec::new();
    let mut credits = Vec::new();

    for metrics in validators.values() {
        if metrics.credits > 0 {
            stakes.push(metrics.stake as f64);
            credits.push(metrics.credits as f64);
        }
    }

    if stakes.len() < 2 {
        return 0.0;
    }

    let n = stakes.len() as f64;
    let mean_stake: f64 = stakes.iter().sum::<f64>() / n;
    let mean_credits: f64 = credits.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut stake_variance = 0.0;
    let mut credits_variance = 0.0;

    for i in 0..stakes.len() {
        let stake_diff = stakes[i] - mean_stake;
        let credits_diff = credits[i] - mean_credits;
        numerator += stake_diff * credits_diff;
        stake_variance += stake_diff * stake_diff;
        credits_variance += credits_diff * credits_diff;
    }

    if stake_variance == 0.0 || credits_variance == 0.0 {
        return 0.0;
    }

    numerator / (stake_variance.sqrt() * credits_variance.sqrt())
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut count = 0;

    for c in s.chars().rev() {
        if count == 3 {
            result.push(',');
            count = 0;
        }
        result.push(c);
        count += 1;
    }

    result.chars().rev().collect()
}

fn format_sol(lamports: u64) -> String {
    let sol = lamports as f64 / 1_000_000_000.0;
    format_number(sol as u64)
}
