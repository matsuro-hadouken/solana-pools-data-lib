#[derive(Debug, Default, Clone)]
pub struct PoolStatisticsSummary {
    pub total_accounts: usize,
    pub activating_accounts: usize,
    pub active_accounts: usize,
    pub deactivating_accounts: usize,
    pub deactivated_accounts: usize,
    pub activating_stake_lamports: u64,
    pub active_stake_lamports: u64,
    pub deactivating_stake_lamports: u64,
    pub deactivated_stake_lamports: u64,
    pub total_lamports: u64,
}

impl PoolStatisticsFull {
    #[must_use]
    pub fn summary(&self) -> PoolStatisticsSummary {
        use crate::statistics::StakeState;
        let mut summary = PoolStatisticsSummary::default();
        for validator in &self.validators {
            for account in &validator.accounts {
                summary.total_accounts += 1;
                summary.total_lamports += account.account_size_in_lamports;
                match account.account_state {
                    StakeState::Activating => {
                        summary.activating_accounts += 1;
                        summary.activating_stake_lamports += account.account_size_in_lamports;
                    }
                    StakeState::Active => {
                        summary.active_accounts += 1;
                        summary.active_stake_lamports += account.account_size_in_lamports;
                    }
                    StakeState::Deactivating => {
                        summary.deactivating_accounts += 1;
                        summary.deactivating_stake_lamports += account.account_size_in_lamports;
                    }
                    StakeState::Inactive | StakeState::Waste | StakeState::Unknown => {
                        summary.deactivated_accounts += 1;
                        summary.deactivated_stake_lamports += account.account_size_in_lamports;
                    }
                }
            }
        }
        summary
    }
}
// Canonical pool/validator/account statistics for engineering use
// This module is NOT public API yet, for internal validation and migration only

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum StakeState {
    Active,
    Activating,
    Deactivating,
    Inactive,
    Unknown,
    Waste,
}

#[derive(Debug, Clone)]
pub struct AccountStatisticsFull {
    pub account_pubkey: String,
    pub account_state: StakeState,
    pub account_size_in_lamports: u64,
    pub validator_pubkey: String,
    pub activation_epoch: Option<u64>,
    pub deactivation_epoch: Option<u64>,
    pub rent_exempt_reserve: Option<u64>,
    pub authorized_staker: Option<String>,
    pub authorized_withdrawer: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidatorStatisticsFull {
    pub validator_pubkey: String,
    pub accounts: Vec<AccountStatisticsFull>,
    pub last_epoch_credits_cumulative: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct PoolStatisticsFull {
    pub pool_name: String,
    pub validators: Vec<ValidatorStatisticsFull>,
}

// Helper: classify canonical state
#[must_use]
pub fn classify_stake_state(
    delegation: Option<&crate::types::ProductionStakeDelegation>,
    current_epoch: u64,
) -> StakeState {
    delegation.map_or(StakeState::Inactive, |d| {
        if d.activation_epoch == current_epoch && d.deactivation_epoch != u64::MAX {
            StakeState::Waste
        } else if d.activation_epoch > u64::MAX - 100 {
            StakeState::Unknown
        } else if d.activation_epoch == current_epoch && d.deactivation_epoch == u64::MAX {
            StakeState::Activating
        } else if d.deactivation_epoch == current_epoch {
            StakeState::Deactivating
        } else if d.deactivation_epoch < current_epoch {
            StakeState::Inactive
        } else if d.activation_epoch > d.deactivation_epoch {
            StakeState::Unknown
        } else {
            StakeState::Active
        }
    })
}
