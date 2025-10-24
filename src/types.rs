//! Cle/// Complete result from fetching multiple pools (debug format) data types for stake pool information.

use crate::error::PoolError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete result from fetching multiple pools (debug format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolsDataResult {
    /// Successfully fetched pool data
    pub successful: HashMap<String, PoolData>,
    /// Failed pool fetches with error details
    pub failed: HashMap<String, PoolError>,
    /// Summary statistics
    pub summary: PoolsDataSummary,
    /// Timestamp when data was fetched
    pub fetched_at: DateTime<Utc>,
}

impl Default for PoolsDataResult {
    fn default() -> Self {
        Self::new()
    }
}

impl PoolsDataResult {
    /// Create a new result
    #[must_use]
    pub fn new() -> Self {
        Self {
            successful: HashMap::new(),
            failed: HashMap::new(),
            summary: PoolsDataSummary::default(),
            fetched_at: Utc::now(),
        }
    }

    /// Check if any pools were fetched successfully
    #[must_use]
    pub fn has_successful(&self) -> bool {
        !self.successful.is_empty()
    }

    /// Check if any pools failed
    #[must_use]
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }

    /// Get total number of pools attempted
    #[must_use]
    pub fn total_attempted(&self) -> usize {
        self.successful.len() + self.failed.len()
    }

    /// Get success rate as percentage
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Expected: usize counts converted to f64 for percentage calculation
    pub fn success_rate(&self) -> f64 {
        if self.total_attempted() == 0 {
            return 0.0;
        }
        (self.successful.len() as f64 / self.total_attempted() as f64) * 100.0
    }

    /// Get list of pool names that can be retried
    #[must_use]
    pub fn retryable_pools(&self) -> Vec<String> {
        self.failed
            .iter()
            .filter(|(_, error)| error.retryable)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

/// Complete debug data for a single stake pool (ALL fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolData {
    /// Pool name (e.g., "jito", "marinade")
    pub pool_name: String,
    /// Pool authority public key
    pub authority: String,
    /// All stake accounts belonging to this pool
    ///
    /// Each stake account includes:
    /// - Account public key
    /// - Stake size (lamports)
    /// - Activation epoch
    /// - Deactivation epoch
    /// - Credits observed
    ///
    /// Filtering logic for active stake accounts:
    /// - `deactivation_epoch == u64::MAX` (active)
    /// - `stake > 0`
    pub stake_accounts: Vec<StakeAccountInfo>,
    /// Validator distribution summary
    pub validator_distribution: HashMap<String, ValidatorStake>,
    /// Pool statistics
    pub statistics: PoolStatistics,
    /// When this data was fetched
    pub fetched_at: DateTime<Utc>,
}

impl PoolData {
    /// Create new pool data
    #[must_use]
    pub fn new(pool_name: String, authority: String) -> Self {
        Self {
            pool_name,
            authority,
            stake_accounts: Vec::new(),
            validator_distribution: HashMap::new(),
            statistics: PoolStatistics::default(),
            fetched_at: Utc::now(),
        }
    }

    /// Get total lamports across all accounts
    #[must_use]
    pub fn total_lamports(&self) -> u64 {
        self.stake_accounts.iter().map(|a| a.lamports).sum()
    }

    /// Get total delegated stake
    #[must_use]
    pub fn total_delegated_stake(&self) -> u64 {
        self.stake_accounts
            .iter()
            .filter_map(|a| a.delegation.as_ref().map(|d| d.stake))
            .sum()
    }

    /// Get number of validators this pool delegates to
    #[must_use]
    pub fn validator_count(&self) -> usize {
        self.validator_distribution.len()
    }
}

/// Production pool data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionPoolData {
    /// Pool name (e.g., "jito", "marinade")
    pub pool_name: String,
    /// Pool authority public key
    pub authority: String,
    /// Production stake accounts
    pub stake_accounts: Vec<ProductionStakeAccountInfo>,
    /// Validator distribution summary
    pub validator_distribution: HashMap<String, ValidatorStake>,
    /// Pool statistics
    pub statistics: PoolStatistics,
    /// When this data was fetched
    pub fetched_at: DateTime<Utc>,
}

impl From<&PoolData> for ProductionPoolData {
    fn from(pool: &PoolData) -> Self {
        Self {
            pool_name: pool.pool_name.clone(),
            authority: pool.authority.clone(),
            stake_accounts: pool.stake_accounts.iter().map(Into::into).collect(),
            validator_distribution: pool.validator_distribution.clone(),
            statistics: pool.statistics.clone(),
            fetched_at: pool.fetched_at,
        }
    }
}

/// Complete stake account info with ALL fields (debug format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeAccountInfo {
    /// Account public key
    pub pubkey: String,
    /// Account balance in lamports
    pub lamports: u64,
    /// Rent exempt reserve
    pub rent_exempt_reserve: u64,
    /// Delegation information (if delegated)
    pub delegation: Option<StakeDelegation>,
    /// Authority configuration
    pub authorized: StakeAuthorized,
    /// Lockup configuration
    pub lockup: StakeLockup,
}

/// Production stake account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStakeAccountInfo {
    /// Account public key
    pub pubkey: String,
    /// Account balance in lamports
    pub lamports: u64,
    /// Stake type ("delegated" or "initialized")
    pub stake_type: String,
    /// Delegation information (if delegated)
    pub delegation: Option<ProductionStakeDelegation>,
    /// Authority configuration
    pub authority: ProductionStakeAuthority,
    /// Lockup configuration
    pub lockup: ProductionStakeLockup,
}

impl From<&StakeAccountInfo> for ProductionStakeAccountInfo {
    fn from(account: &StakeAccountInfo) -> Self {
        let delegation = account
            .delegation
            .as_ref()
            .map(|d| ProductionStakeDelegation {
                validator: d.voter.clone(),
                stake_lamports: d.stake,
                activation_epoch: d.activation_epoch,
                deactivation_epoch: d.deactivation_epoch,
                last_epoch_credits_cumulative: d.last_epoch_credits_cumulative,
            });

        let stake_type = if account.delegation.is_some() {
            "delegated".to_string()
        } else {
            "initialized".to_string()
        };

        let authority = ProductionStakeAuthority {
            staker: account.authorized.staker.clone(),
            withdrawer: account.authorized.withdrawer.clone(),
        };

        let lockup = ProductionStakeLockup {
            custodian: account.lockup.custodian.clone(),
            epoch: account.lockup.epoch,
            unix_timestamp: account.lockup.unix_timestamp,
        };

        Self {
            pubkey: account.pubkey.clone(),
            lamports: account.lamports,
            stake_type,
            delegation,
            authority,
            lockup,
        }
    }
}

/// Complete stake delegation info (debug format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeDelegation {
    /// Validator vote account public key
    pub voter: String,
    /// Stake amount in lamports
    pub stake: u64,
    /// Epoch when stake became active
    pub activation_epoch: u64,
    /// Epoch when stake will deactivate (`u64::MAX` if not deactivating)
    pub deactivation_epoch: u64,
    /// Last epoch credits cumulative from this validator
    pub last_epoch_credits_cumulative: u64,
    /// Warmup/cooldown rate
    pub warmup_cooldown_rate: f64,
}

/// Production stake delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStakeDelegation {
    /// Validator vote account public key
    pub validator: String,
    /// Stake amount in lamports
    pub stake_lamports: u64,
    /// Epoch when stake became active
    pub activation_epoch: u64,
    /// Epoch when stake will deactivate (`u64::MAX` if not deactivating)
    pub deactivation_epoch: u64,
    /// Last epoch credits cumulative from this validator
    pub last_epoch_credits_cumulative: u64,
}

/// Complete stake authorization (debug format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeAuthorized {
    /// Authorized staker public key
    pub staker: String,
    /// Authorized withdrawer public key
    pub withdrawer: String,
}

/// Production stake authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStakeAuthority {
    /// Authorized staker public key
    pub staker: String,
    /// Authorized withdrawer public key
    pub withdrawer: String,
}

/// Complete stake lockup info (debug format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeLockup {
    /// Lockup custodian public key
    pub custodian: String,
    /// Lockup epoch
    pub epoch: u64,
    /// Lockup unix timestamp
    pub unix_timestamp: i64,
}

/// Production stake lockup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStakeLockup {
    /// Lockup custodian public key
    pub custodian: String,
    /// Lockup epoch
    pub epoch: u64,
    /// Lockup unix timestamp
    pub unix_timestamp: i64,
}

/// Validator stake information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStake {
    /// Total lamports delegated to this validator from this pool
    pub total_delegated: u64,
    /// Number of stake accounts delegated to this validator
    pub account_count: u32,
    /// List of stake account pubkeys
    pub accounts: Vec<String>,
}

impl ValidatorStake {
    /// Create new validator stake entry
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_delegated: 0,
            account_count: 0,
            accounts: Vec::new(),
        }
    }

    /// Add a stake account to this validator
    pub fn add_account(&mut self, pubkey: String, stake: u64) {
        self.total_delegated += stake;
        self.account_count += 1;
        self.accounts.push(pubkey);
    }

    /// Get average stake per account
    #[must_use]
    pub const fn average_stake_per_account(&self) -> u64 {
        if self.account_count == 0 {
            0
        } else {
            self.total_delegated / self.account_count as u64
        }
    }
}

impl Default for ValidatorStake {
    fn default() -> Self {
        Self::new()
    }
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolStatistics {
    /// Total number of stake accounts
    pub total_accounts: usize,
    /// Total lamports across all accounts
    pub total_lamports: u64,
    /// Total delegated stake lamports
    pub total_staked_lamports: u64,
    /// Number of active stake accounts
    pub active_stake_accounts: usize,
    /// Number of deactivating stake accounts
    pub deactivating_stake_accounts: usize,
    /// Number of unique validators
    pub validator_count: usize,
}

/// Summary of pools data operation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolsDataSummary {
    /// Total pools attempted to fetch
    pub total_pools_attempted: usize,
    /// Number of successfully fetched pools
    pub successful_pools: usize,
    /// Number of failed pools
    pub failed_pools: usize,
    /// Total processing time in milliseconds
    pub total_processing_time_ms: u64,
}

/// Field analysis for understanding static vs dynamic fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAnalysis {
    /// Fields that are always the same value
    pub static_fields: Vec<StaticField>,
    /// Fields that change between accounts
    pub dynamic_fields: Vec<String>,
    /// Field size analysis
    pub size_analysis: SizeAnalysis,
}

impl FieldAnalysis {
    #[must_use]
    pub fn new() -> Self {
        Self {
            static_fields: vec![
                StaticField {
                    name: "rent_exempt_reserve".to_string(),
                    value: "2282880".to_string(),
                    description: "Always the same for all stake accounts".to_string(),
                },
                StaticField {
                    name: "warmup_cooldown_rate".to_string(),
                    value: "0.25".to_string(),
                    description: "Network constant".to_string(),
                },
            ],
            dynamic_fields: vec![
                "pubkey".to_string(),
                "lamports".to_string(),
                "delegation".to_string(),
                "authorized".to_string(),
                "lockup".to_string(),
            ],
            size_analysis: SizeAnalysis::default(),
        }
    }
}

impl Default for FieldAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a static field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticField {
    /// Field name
    pub name: String,
    /// Constant value
    pub value: String,
    /// Description of static field characteristics
    pub description: String,
}

/// Size analysis information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeAnalysis {
    /// Estimated bytes saved by removing static fields
    pub estimated_bytes_saved_per_account: usize,
    /// Percentage reduction in data size
    pub estimated_size_reduction_percent: f64,
}

impl Default for SizeAnalysis {
    fn default() -> Self {
        Self {
            estimated_bytes_saved_per_account: 50,
            estimated_size_reduction_percent: 15.0,
        }
    }
}
