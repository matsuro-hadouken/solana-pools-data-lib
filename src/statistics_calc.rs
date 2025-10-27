// Calculation logic for PoolStatisticsFull, ValidatorStatisticsFull, AccountStatisticsFull
// Uses canonical state classification and current_epoch

use crate::statistics::{AccountStatisticsFull, ValidatorStatisticsFull, PoolStatisticsFull, classify_stake_state};
use crate::types::ProductionPoolData;
use crate::error::PoolsDataError;

/// Calculate canonical pool statistics, grouping by validator and account state
/// Clippy pedantic/nursery compliant
///
/// # Errors
/// Returns `PoolsDataError::ConfigurationError` if pool name or authority is empty.
pub fn calculate_pool_statistics_full(pool: &ProductionPoolData, current_epoch: u64) -> Result<PoolStatisticsFull, PoolsDataError> {
    if pool.pool_name.trim().is_empty() {
        return Err(PoolsDataError::ConfigurationError { message: "Pool name is empty".to_string() });
    }
    if pool.authority.trim().is_empty() {
        return Err(PoolsDataError::ConfigurationError { message: "Pool authority is empty".to_string() });
    }
    // stake_accounts cannot be None, but can be empty
    let mut validator_map: std::collections::HashMap<String, (Vec<AccountStatisticsFull>, Option<u64>)> = std::collections::HashMap::new();
    for account in &pool.stake_accounts {
        let delegation = account.delegation.as_ref();
        let state = classify_stake_state(delegation, current_epoch);
        let validator_pubkey = delegation.map_or_else(String::new, |d| d.validator.clone());
        let credits = delegation.map(|d| d.last_epoch_credits_cumulative);
        let account_stats = AccountStatisticsFull {
            account_pubkey: account.pubkey.clone(),
            account_state: state,
            account_size_in_lamports: account.lamports,
            validator_pubkey: validator_pubkey.clone(),
            activation_epoch: delegation.map(|d| d.activation_epoch),
            deactivation_epoch: delegation.map(|d| d.deactivation_epoch),
            rent_exempt_reserve: None,
            authorized_staker: Some(account.authority.staker.clone()),
            authorized_withdrawer: Some(account.authority.withdrawer.clone()),
        };
        let entry = validator_map.entry(validator_pubkey).or_insert((Vec::new(), credits));
        entry.0.push(account_stats);
        // If credits is Some, always set it (should be same for all accounts)
        if credits.is_some() {
            entry.1 = credits;
        }
    }
    let validators: Vec<ValidatorStatisticsFull> = validator_map
        .into_iter()
        .map(|(validator_pubkey, (accounts, credits))| ValidatorStatisticsFull {
            validator_pubkey,
            accounts,
            last_epoch_credits_cumulative: credits,
        })
        .collect();
    Ok(PoolStatisticsFull {
        pool_name: pool.pool_name.clone(),
        validators,
    })
}
