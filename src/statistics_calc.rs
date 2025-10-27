// Calculation logic for PoolStatisticsFull, ValidatorStatisticsFull, AccountStatisticsFull
// Uses canonical state classification and current_epoch

use crate::statistics::{AccountStatisticsFull, ValidatorStatisticsFull, PoolStatisticsFull, classify_stake_state};
use crate::types::ProductionPoolData;

/// Calculate canonical pool statistics, grouping by validator and account state
/// Clippy pedantic/nursery compliant
#[must_use]
pub fn calculate_pool_statistics_full(pool: &ProductionPoolData, current_epoch: u64) -> PoolStatisticsFull {
    let mut validator_map = std::collections::HashMap::<String, Vec<AccountStatisticsFull>>::new();
    for account in &pool.stake_accounts {
        let delegation = account.delegation.as_ref();
        let state = classify_stake_state(delegation, current_epoch);
        // Use explicit empty string for non-delegated accounts
            let validator_pubkey = delegation.map_or_else(String::new, |d| d.validator.clone());
        let account_stats = AccountStatisticsFull {
            account_pubkey: account.pubkey.clone(),
            account_state: state,
            account_size_in_lamports: account.lamports,
            validator_pubkey: validator_pubkey.clone(),
            activation_epoch: delegation.map(|d| d.activation_epoch),
            deactivation_epoch: delegation.map(|d| d.deactivation_epoch),
            last_epoch_credits_cumulative: delegation.map(|d| d.last_epoch_credits_cumulative),
            rent_exempt_reserve: None,
            authorized_staker: Some(account.authority.staker.clone()),
            authorized_withdrawer: Some(account.authority.withdrawer.clone()),
        };
        validator_map.entry(validator_pubkey).or_default().push(account_stats);
    }
    let validators: Vec<ValidatorStatisticsFull> = validator_map
        .into_iter()
        .map(|(validator_pubkey, accounts)| ValidatorStatisticsFull {
            validator_pubkey,
            accounts,
        })
        .collect();
    PoolStatisticsFull {
        pool_name: pool.pool_name.clone(),
        validators,
    }
}
