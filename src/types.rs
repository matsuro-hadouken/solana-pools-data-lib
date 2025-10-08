//! Data types for stake pool information.
//!
//! This module defines the core data structures returned by the library,
//! designed for easy integration with databases, APIs, and data analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::error::PoolError;

/// Complete result from fetching multiple pools
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
    pub fn new() -> Self {
        Self {
            successful: HashMap::new(),
            failed: HashMap::new(),
            summary: PoolsDataSummary::default(),
            fetched_at: Utc::now(),
        }
    }

    /// Check if any pools were fetched successfully
    pub fn has_successful(&self) -> bool {
        !self.successful.is_empty()
    }

    /// Check if any pools failed
    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }

    /// Get total number of pools attempted
    pub fn total_attempted(&self) -> usize {
        self.successful.len() + self.failed.len()
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_attempted() == 0 {
            return 0.0;
        }
        (self.successful.len() as f64 / self.total_attempted() as f64) * 100.0
    }

    /// Get list of pool names that can be retried
    pub fn retryable_pools(&self) -> Vec<String> {
        self.failed
            .iter()
            .filter(|(_, error)| error.retryable)
            .map(|(name, _)| name.clone())
            .collect()
    }
}

/// Data for a single stake pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolData {
    /// Pool name (e.g., "jito", "marinade")
    pub pool_name: String,
    /// Pool authority public key
    pub authority: String,
    /// All stake accounts belonging to this pool
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

    /// Calculate and update statistics from stake accounts
    pub fn calculate_statistics(&mut self) {
        let mut total_lamports = 0u64;
        let mut total_stake = 0u64;
        let mut active_accounts = 0u32;
        let mut unique_validators = std::collections::HashSet::new();

        for account in &self.stake_accounts {
            total_lamports += account.lamports;
            
            if let Some(delegation) = &account.delegation {
                total_stake += delegation.stake;
                if delegation.is_active() {
                    active_accounts += 1;
                }
                unique_validators.insert(&delegation.voter);
            }
        }

        let validator_count = unique_validators.len() as u32;

        self.statistics = PoolStatistics {
            total_stake_accounts: self.stake_accounts.len() as u32,
            active_stake_accounts: active_accounts,
            total_lamports,
            total_staked_lamports: total_stake,
            unique_validators: validator_count,
            average_stake_per_account: if !self.stake_accounts.is_empty() {
                total_stake / self.stake_accounts.len() as u64
            } else {
                0
            },
        };
    }

    /// Get total staked SOL (converted from lamports)
    pub fn total_staked_sol(&self) -> f64 {
        self.statistics.total_staked_lamports as f64 / 1_000_000_000.0
    }

    /// Get total SOL including rent (converted from lamports)
    pub fn total_sol(&self) -> f64 {
        self.statistics.total_lamports as f64 / 1_000_000_000.0
    }

    /// Get validator with most stake in this pool
    pub fn largest_validator_stake(&self) -> Option<&ValidatorStake> {
        self.validator_distribution
            .values()
            .max_by_key(|v| v.total_delegated)
    }

    /// Get validator distribution as percentages
    pub fn validator_distribution_percentages(&self) -> HashMap<String, f64> {
        let total = self.statistics.total_staked_lamports as f64;
        if total == 0.0 {
            return HashMap::new();
        }

        self.validator_distribution
            .iter()
            .map(|(voter, stake)| {
                let percentage = (stake.total_delegated as f64 / total) * 100.0;
                (voter.clone(), percentage)
            })
            .collect()
    }
}

/// Information about a single stake account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeAccountInfo {
    /// The stake account's public key
    pub pubkey: String,
    /// Total lamports in the account
    pub lamports: u64,
    /// Rent exempt reserve amount
    pub rent_exempt_reserve: u64,
    /// Stake delegation information (if delegated)
    pub delegation: Option<StakeDelegation>,
    /// Authority information
    pub authorized: StakeAuthorized,
    /// Lockup information
    pub lockup: StakeLockup,
}

impl StakeAccountInfo {
    /// Get the effective stake amount (lamports - rent reserve)
    pub fn effective_stake(&self) -> u64 {
        self.lamports.saturating_sub(self.rent_exempt_reserve)
    }

    /// Check if this stake account is currently active
    pub fn is_active(&self) -> bool {
        self.delegation.as_ref().map(|d| d.is_active()).unwrap_or(false)
    }

    /// Get the validator this stake is delegated to
    pub fn delegated_validator(&self) -> Option<&str> {
        self.delegation.as_ref().map(|d| d.voter.as_str())
    }
}

/// Stake delegation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeDelegation {
    /// Validator vote account this stake is delegated to
    pub voter: String,
    /// Amount of stake delegated (in lamports)
    pub stake: u64,
    /// Epoch when stake became active
    pub activation_epoch: u64,
    /// Epoch when stake will deactivate (u64::MAX if not deactivating)
    pub deactivation_epoch: u64,
    /// Validator performance credits observed
    pub credits_observed: u64,
    /// Warmup/cooldown rate
    pub warmup_cooldown_rate: f64,
}

impl StakeDelegation {
    /// Check if this delegation is currently active
    pub fn is_active(&self) -> bool {
        // If deactivation_epoch is max value, it's not deactivating
        self.deactivation_epoch == u64::MAX || 
        self.deactivation_epoch == 18446744073709551615u64 // Common representation
    }

    /// Check if this delegation is deactivating
    pub fn is_deactivating(&self) -> bool {
        !self.is_active()
    }
}

/// Stake account authority information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeAuthorized {
    /// Who can change delegation
    pub staker: String,
    /// Who can withdraw funds
    pub withdrawer: String,
}

impl StakeAuthorized {
    /// Check if staker and withdrawer are the same (common for pools)
    pub fn is_unified_authority(&self) -> bool {
        self.staker == self.withdrawer
    }
}

/// Stake account lockup information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeLockup {
    /// Lockup custodian (often system program)
    pub custodian: String,
    /// Lockup epoch (0 = no lockup)
    pub epoch: u64,
    /// Lockup unix timestamp (0 = no lockup)
    pub unix_timestamp: u64,
}

impl StakeLockup {
    /// Check if stake account has any lockup
    pub fn is_locked(&self) -> bool {
        self.epoch > 0 || self.unix_timestamp > 0
    }

    /// Check if this is the default "no lockup" configuration
    pub fn is_default_lockup(&self) -> bool {
        // Default lockup has system program as custodian and zero constraints
        const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
        
        self.custodian == SYSTEM_PROGRAM && 
        self.epoch == 0 && 
        self.unix_timestamp == 0
    }

    /// Check if lockup has meaningful constraints (not default)
    pub fn has_constraints(&self) -> bool {
        !self.is_default_lockup()
    }

    /// Convert to production format (always includes all fields for consistency)
    pub fn to_production(&self) -> ProductionStakeLockup {
        ProductionStakeLockup {
            custodian: self.custodian.clone(),
            epoch: self.epoch,
            unix_timestamp: self.unix_timestamp,
        }
    }

    /// Convert to optimized format (returns None if default lockup)
    pub fn to_optimized(&self) -> Option<OptimizedStakeLockup> {
        if self.is_default_lockup() {
            // Default lockup - omit entirely
            None
        } else {
            // Has actual constraints - include only non-default values
            const SYSTEM_PROGRAM: &str = "11111111111111111111111111111111";
            
            Some(OptimizedStakeLockup {
                custodian: if self.custodian != SYSTEM_PROGRAM {
                    Some(self.custodian.clone())
                } else {
                    None
                },
                epoch: if self.epoch > 0 {
                    Some(self.epoch)
                } else {
                    None
                },
                unix_timestamp: if self.unix_timestamp > 0 {
                    Some(self.unix_timestamp)
                } else {
                    None
                },
            })
        }
    }
}

/// Production stake account info - consistent schema, removes only static fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStakeAccountInfo {
    /// The stake account's public key  
    pub pubkey: String,
    /// Total lamports in the account
    pub lamports: u64,
    /// Account type: "delegated", "initialized", etc.
    pub stake_type: String,
    /// Delegation details (None if account is not delegated)
    pub delegation: Option<ProductionStakeDelegation>,
    /// Authority info (always included for consistency)
    pub authority: ProductionStakeAuthority,
    /// Lockup info (always included for consistency)
    pub lockup: ProductionStakeLockup,
}

/// Production delegation info with all fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStakeDelegation {
    /// Validator vote account this stake is delegated to
    pub validator: String,
    /// Amount of stake delegated (in lamports)  
    pub stake_lamports: u64,
    /// Epoch when stake became active
    pub activation_epoch: u64,
    /// Epoch when stake will deactivate (u64::MAX if not deactivating)
    pub deactivation_epoch: u64,
    /// Critical: Validator performance credits observed for reward tracking
    pub credits_observed: u64,
    /// Warmup/cooldown rate (standard value 0.25)
    pub warmup_cooldown_rate: f64,
}

/// Authority info (always included for consistent schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStakeAuthority {
    /// Who can change delegation
    pub staker: String,
    /// Who can withdraw funds  
    pub withdrawer: String,
}

/// Lockup info (always included for consistent schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionStakeLockup {
    /// Lockup custodian (system program if no lockup)
    pub custodian: String,
    /// Lockup epoch (0 if no lockup)
    pub epoch: u64,
    /// Lockup unix timestamp (0 if no lockup)
    pub unix_timestamp: u64,
}

/// Production pool data - consistent schema, removes only static fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionPoolData {
    /// Pool name (e.g., "jito", "marinade")
    pub pool_name: String,
    /// Pool authority public key
    pub authority: String,
    /// Production stake accounts (static fields removed, schema consistent)
    pub stake_accounts: Vec<ProductionStakeAccountInfo>,
    /// Validator distribution summary
    pub validator_distribution: HashMap<String, ValidatorStake>,
    /// Pool statistics
    pub statistics: PoolStatistics,
    /// When this data was fetched
    pub fetched_at: DateTime<Utc>,
}

/// LEGACY: Optimized stake account info with only dynamic/relevant fields
/// ⚠️ DEPRECATED: Use ProductionStakeAccountInfo instead for consistent schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedStakeAccountInfo {
    /// The stake account's public key  
    pub pubkey: String,
    /// Total lamports in the account
    pub lamports: u64,
    /// Account type: "delegated", "initialized", etc.
    pub stake_type: String,
    /// Delegation details (None if account is not delegated)
    pub delegation: Option<OptimizedStakeDelegation>,
    /// Authority info (only included if different from pool authority)
    pub custom_authority: Option<OptimizedStakeAuthority>,
    /// Lockup info (only included if account has actual lockup constraints)
    pub lockup: Option<OptimizedStakeLockup>,
}

/// Optimized delegation info with only dynamic fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedStakeDelegation {
    /// Validator vote account this stake is delegated to
    pub validator: String,
    /// Amount of stake delegated (in lamports)  
    pub stake_lamports: u64,
    /// Epoch when stake became active
    pub activation_epoch: u64,
    /// Epoch when stake will deactivate (u64::MAX if not deactivating)
    pub deactivation_epoch: u64,
    /// Critical: Validator performance credits observed for reward tracking
    pub credits_observed: u64,
    /// Warmup/cooldown rate (included for completeness, standard value 0.25)
    pub warmup_cooldown_rate: f64,
}

/// Authority info (only included when non-standard)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedStakeAuthority {
    /// Who can change delegation
    pub staker: String,
    /// Who can withdraw funds  
    pub withdrawer: String,
}

/// Lockup info (only included when account has actual lockup constraints)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedStakeLockup {
    /// Lockup custodian (only if not default system program)
    pub custodian: Option<String>,
    /// Lockup epoch (only if > 0)
    pub epoch: Option<u64>,
    /// Lockup unix timestamp (only if > 0)
    pub unix_timestamp: Option<u64>,
}

/// Optimized pool data with minimal static fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedPoolData {
    /// Pool name (e.g., "jito", "marinade")
    pub pool_name: String,
    /// Pool authority public key
    pub authority: String,
    /// Optimized stake accounts (static fields removed)
    pub stake_accounts: Vec<OptimizedStakeAccountInfo>,
    /// Validator distribution summary
    pub validator_distribution: HashMap<String, ValidatorStake>,
    /// Pool statistics
    pub statistics: PoolStatistics,
    /// When this data was fetched
    pub fetched_at: DateTime<Utc>,
}

impl From<&StakeAccountInfo> for ProductionStakeAccountInfo {
    fn from(account: &StakeAccountInfo) -> Self {
        // Convert delegation if present
        let delegation = account.delegation.as_ref().map(|d| ProductionStakeDelegation {
            validator: d.voter.clone(),
            stake_lamports: d.stake,
            activation_epoch: d.activation_epoch,
            deactivation_epoch: d.deactivation_epoch,
            credits_observed: d.credits_observed,
            warmup_cooldown_rate: d.warmup_cooldown_rate,
        });

        // Determine stake type based on delegation status
        let stake_type = if account.delegation.is_some() {
            "delegated".to_string()
        } else {
            "initialized".to_string()
        };

        // Always include authority and lockup for consistent schema
        let authority = ProductionStakeAuthority {
            staker: account.authorized.staker.clone(),
            withdrawer: account.authorized.withdrawer.clone(),
        };

        let lockup = account.lockup.to_production();

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

impl From<&PoolData> for ProductionPoolData {
    fn from(pool_data: &PoolData) -> Self {
        Self {
            pool_name: pool_data.pool_name.clone(),
            authority: pool_data.authority.clone(),
            stake_accounts: pool_data.stake_accounts.iter().map(|a| a.into()).collect(),
            validator_distribution: pool_data.validator_distribution.clone(),
            statistics: pool_data.statistics.clone(),
            fetched_at: pool_data.fetched_at,
        }
    }
}

impl From<&StakeAccountInfo> for OptimizedStakeAccountInfo {
    fn from(account: &StakeAccountInfo) -> Self {
        // Determine if we need to include custom authority
        let custom_authority = if !account.authorized.is_unified_authority() {
            Some(OptimizedStakeAuthority {
                staker: account.authorized.staker.clone(),
                withdrawer: account.authorized.withdrawer.clone(),
            })
        } else {
            None
        };

        // Convert delegation if present
        let delegation = account.delegation.as_ref().map(|d| OptimizedStakeDelegation {
            validator: d.voter.clone(),
            stake_lamports: d.stake,
            activation_epoch: d.activation_epoch,
            deactivation_epoch: d.deactivation_epoch,
            credits_observed: d.credits_observed,
            warmup_cooldown_rate: d.warmup_cooldown_rate,
        });

        // Determine stake type based on delegation status
        let stake_type = if account.delegation.is_some() {
            "delegated".to_string()
        } else {
            "initialized".to_string()
        };

        // Convert lockup (only include if has actual constraints)
        let lockup = account.lockup.to_optimized();

        Self {
            pubkey: account.pubkey.clone(),
            lamports: account.lamports,
            stake_type,
            delegation,
            custom_authority,
            lockup,
        }
    }
}

impl From<&PoolData> for OptimizedPoolData {
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

/// Aggregated validator stake information for a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorStake {
    /// Validator vote account public key
    pub voter_pubkey: String,
    /// Total lamports delegated to this validator from this pool
    pub total_delegated: u64,
    /// Number of stake accounts delegated to this validator
    pub account_count: u32,
    /// Average stake per account for this validator
    pub average_stake_per_account: u64,
    /// List of stake account pubkeys
    pub stake_account_pubkeys: Vec<String>,
}

impl ValidatorStake {
    /// Create new validator stake entry
    pub fn new(voter_pubkey: String) -> Self {
        Self {
            voter_pubkey,
            total_delegated: 0,
            account_count: 0,
            average_stake_per_account: 0,
            stake_account_pubkeys: Vec::new(),
        }
    }

    /// Add a stake account to this validator's aggregation
    pub fn add_stake_account(&mut self, stake_account: &StakeAccountInfo) {
        if let Some(delegation) = &stake_account.delegation {
            self.total_delegated += delegation.stake;
            self.account_count += 1;
            self.stake_account_pubkeys.push(stake_account.pubkey.clone());
            
            // Recalculate average
            self.average_stake_per_account = if self.account_count > 0 {
                self.total_delegated / self.account_count as u64
            } else {
                0
            };
        }
    }

    /// Get delegated SOL amount
    pub fn delegated_sol(&self) -> f64 {
        self.total_delegated as f64 / 1_000_000_000.0
    }
}

/// Statistics for a pool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolStatistics {
    /// Total number of stake accounts
    pub total_stake_accounts: u32,
    /// Number of active stake accounts
    pub active_stake_accounts: u32,
    /// Total lamports across all accounts
    pub total_lamports: u64,
    /// Total staked lamports (excluding rent reserves)
    pub total_staked_lamports: u64,
    /// Number of unique validators this pool delegates to
    pub unique_validators: u32,
    /// Average stake per account
    pub average_stake_per_account: u64,
}

/// Summary statistics across multiple pools
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolsDataSummary {
    /// Number of pools successfully fetched
    pub successful_pools: u32,
    /// Number of pools that failed
    pub failed_pools: u32,
    /// Total stake accounts across all successful pools
    pub total_stake_accounts: u32,
    /// Total lamports across all successful pools
    pub total_lamports: u64,
    /// Total staked lamports across all successful pools  
    pub total_staked_lamports: u64,
    /// Total unique validators across all pools
    pub total_unique_validators: u32,
}

impl PoolsDataSummary {
    /// Calculate summary from pools data
    pub fn from_pools_data(successful: &HashMap<String, PoolData>, failed: &HashMap<String, PoolError>) -> Self {
        let mut summary = Self {
            successful_pools: successful.len() as u32,
            failed_pools: failed.len() as u32,
            ..Default::default()
        };

        let mut all_validators = std::collections::HashSet::new();

        for pool_data in successful.values() {
            summary.total_stake_accounts += pool_data.statistics.total_stake_accounts;
            summary.total_lamports += pool_data.statistics.total_lamports;
            summary.total_staked_lamports += pool_data.statistics.total_staked_lamports;
            
            // Collect unique validators across all pools
            for validator_pubkey in pool_data.validator_distribution.keys() {
                all_validators.insert(validator_pubkey);
            }
        }

        summary.total_unique_validators = all_validators.len() as u32;
        summary
    }

    /// Get total SOL across all pools
    pub fn total_sol(&self) -> f64 {
        self.total_lamports as f64 / 1_000_000_000.0
    }

    /// Get total staked SOL across all pools
    pub fn total_staked_sol(&self) -> f64 {
        self.total_staked_lamports as f64 / 1_000_000_000.0
    }
}

/// Information for validator-focused queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPoolStake {
    /// Validator vote account public key
    pub validator_pubkey: String,
    /// Pool name that stakes to this validator
    pub pool_name: String,
    /// Pool authority
    pub pool_authority: String,
    /// Amount delegated from this pool to this validator
    pub delegated_amount: u64,
    /// Number of stake accounts
    pub account_count: u32,
    /// Percentage of pool's total stake going to this validator
    pub pool_percentage: f64,
}

impl ValidatorPoolStake {
    /// Get delegated SOL amount
    pub fn delegated_sol(&self) -> f64 {
        self.delegated_amount as f64 / 1_000_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_data_statistics() {
        let mut pool_data = PoolData::new("test_pool".to_string(), "authority123".to_string());
        
        // Add a test stake account
        let stake_account = StakeAccountInfo {
            pubkey: "stake123".to_string(),
            lamports: 5_000_000_000, // 5 SOL
            rent_exempt_reserve: 2_282_880,
            delegation: Some(StakeDelegation {
                voter: "validator123".to_string(),
                stake: 4_997_717_120, // 5 SOL - rent
                activation_epoch: 100,
                deactivation_epoch: u64::MAX,
                credits_observed: 1000,
                warmup_cooldown_rate: 0.25,
            }),
            authorized: StakeAuthorized {
                staker: "authority123".to_string(),
                withdrawer: "authority123".to_string(),
            },
            lockup: StakeLockup {
                custodian: "11111111111111111111111111111111".to_string(),
                epoch: 0,
                unix_timestamp: 0,
            },
        };

        pool_data.stake_accounts.push(stake_account);
        pool_data.calculate_statistics();

        assert_eq!(pool_data.statistics.total_stake_accounts, 1);
        assert_eq!(pool_data.statistics.active_stake_accounts, 1);
        assert_eq!(pool_data.statistics.total_lamports, 5_000_000_000);
        assert!(pool_data.total_staked_sol() > 4.99);
    }

    #[test]
    fn test_stake_delegation_active() {
        let active_delegation = StakeDelegation {
            voter: "validator123".to_string(),
            stake: 1000,
            activation_epoch: 100,
            deactivation_epoch: u64::MAX,
            credits_observed: 1000,
            warmup_cooldown_rate: 0.25,
        };

        assert!(active_delegation.is_active());
        assert!(!active_delegation.is_deactivating());
    }

    #[test]
    fn test_pools_data_result() {
        let mut result = PoolsDataResult::new();
        
        let pool_data = PoolData::new("test".to_string(), "auth".to_string());
        result.successful.insert("test".to_string(), pool_data);

        assert!(result.has_successful());
        assert!(!result.has_failures());
        assert_eq!(result.success_rate(), 100.0);
    }

    #[test]
    fn test_optimized_stake_account_conversion() {
        let original = StakeAccountInfo {
            pubkey: "test_account".to_string(),
            lamports: 5000000000,
            rent_exempt_reserve: 2282880,
            delegation: Some(StakeDelegation {
                voter: "validator123".to_string(),
                stake: 4997717120,
                activation_epoch: 100,
                deactivation_epoch: u64::MAX,
                credits_observed: 248271389,
                warmup_cooldown_rate: 0.25,
            }),
            authorized: StakeAuthorized {
                staker: "pool_authority".to_string(),
                withdrawer: "pool_authority".to_string(),
            },
            lockup: StakeLockup {
                custodian: "11111111111111111111111111111111".to_string(),
                epoch: 0,
                unix_timestamp: 0,
            },
        };

        let optimized: OptimizedStakeAccountInfo = (&original).into();

        // Test that important fields are preserved
        assert_eq!(optimized.pubkey, "test_account");
        assert_eq!(optimized.lamports, 5000000000);
        assert_eq!(optimized.stake_type, "delegated");
        
        // Test delegation conversion
        let opt_delegation = optimized.delegation.unwrap();
        assert_eq!(opt_delegation.validator, "validator123");
        assert_eq!(opt_delegation.stake_lamports, 4997717120);
        assert_eq!(opt_delegation.activation_epoch, 100);
        assert_eq!(opt_delegation.deactivation_epoch, u64::MAX);
        assert_eq!(opt_delegation.credits_observed, 248271389);
        assert_eq!(opt_delegation.warmup_cooldown_rate, 0.25);

        // Test that unified authority is omitted (since staker == withdrawer)
        assert!(optimized.custom_authority.is_none());

        // Test that default lockup is omitted
        assert!(optimized.lockup.is_none());
    }

    #[test]
    fn test_optimized_stake_account_with_custom_authority() {
        let original = StakeAccountInfo {
            pubkey: "test_account".to_string(),
            lamports: 5000000000,
            rent_exempt_reserve: 2282880,
            delegation: None, // Initialized but not delegated
            authorized: StakeAuthorized {
                staker: "different_staker".to_string(),
                withdrawer: "different_withdrawer".to_string(),
            },
            lockup: StakeLockup {
                custodian: "11111111111111111111111111111111".to_string(),
                epoch: 0,
                unix_timestamp: 0,
            },
        };

        let optimized: OptimizedStakeAccountInfo = (&original).into();

        // Test stake type for non-delegated account
        assert_eq!(optimized.stake_type, "initialized");
        assert!(optimized.delegation.is_none());

        // Test that non-unified authority is preserved
        let custom_auth = optimized.custom_authority.unwrap();
        assert_eq!(custom_auth.staker, "different_staker");
        assert_eq!(custom_auth.withdrawer, "different_withdrawer");

        // Test that default lockup is omitted
        assert!(optimized.lockup.is_none());
    }

    #[test]
    fn test_optimized_pool_data_conversion() {
        let mut original = PoolData::new("jito".to_string(), "jito_authority".to_string());
        
        // Add a test stake account
        original.stake_accounts.push(StakeAccountInfo {
            pubkey: "stake1".to_string(),
            lamports: 3000000000,
            rent_exempt_reserve: 2282880,
            delegation: Some(StakeDelegation {
                voter: "validator1".to_string(),
                stake: 2997717120,
                activation_epoch: 50,
                deactivation_epoch: u64::MAX,
                credits_observed: 123456789,
                warmup_cooldown_rate: 0.25,
            }),
            authorized: StakeAuthorized {
                staker: "jito_authority".to_string(),
                withdrawer: "jito_authority".to_string(),
            },
            lockup: StakeLockup {
                custodian: "11111111111111111111111111111111".to_string(),
                epoch: 0,
                unix_timestamp: 0,
            },
        });

        original.calculate_statistics();

        let optimized: OptimizedPoolData = (&original).into();

        // Test pool-level fields are preserved
        assert_eq!(optimized.pool_name, "jito");
        assert_eq!(optimized.authority, "jito_authority");
        assert_eq!(optimized.stake_accounts.len(), 1);
        assert_eq!(optimized.statistics.total_stake_accounts, 1);
        assert_eq!(optimized.fetched_at, original.fetched_at);

        // Test stake account optimization
        let opt_stake = &optimized.stake_accounts[0];
        assert_eq!(opt_stake.pubkey, "stake1");
        assert_eq!(opt_stake.stake_type, "delegated");
        assert!(opt_stake.custom_authority.is_none()); // Unified authority omitted
        assert!(opt_stake.lockup.is_none()); // Default lockup omitted
    }

    #[test]
    fn test_lockup_detection_and_optimization() {
        // Test default lockup (should be omitted)
        let default_lockup = StakeLockup {
            custodian: "11111111111111111111111111111111".to_string(),
            epoch: 0,
            unix_timestamp: 0,
        };
        
        assert!(default_lockup.is_default_lockup());
        assert!(!default_lockup.has_constraints());
        assert!(default_lockup.to_optimized().is_none());

        // Test lockup with epoch constraint
        let epoch_lockup = StakeLockup {
            custodian: "11111111111111111111111111111111".to_string(),
            epoch: 500,
            unix_timestamp: 0,
        };
        
        assert!(!epoch_lockup.is_default_lockup());
        assert!(epoch_lockup.has_constraints());
        let opt = epoch_lockup.to_optimized().unwrap();
        assert!(opt.custodian.is_none()); // System program custodian omitted
        assert_eq!(opt.epoch, Some(500));
        assert!(opt.unix_timestamp.is_none());

        // Test lockup with timestamp constraint
        let timestamp_lockup = StakeLockup {
            custodian: "11111111111111111111111111111111".to_string(),
            epoch: 0,
            unix_timestamp: 1640995200, // Some timestamp
        };
        
        assert!(!timestamp_lockup.is_default_lockup());
        assert!(timestamp_lockup.has_constraints());
        let opt = timestamp_lockup.to_optimized().unwrap();
        assert!(opt.custodian.is_none());
        assert!(opt.epoch.is_none());
        assert_eq!(opt.unix_timestamp, Some(1640995200));

        // Test lockup with custom custodian
        let custom_custodian_lockup = StakeLockup {
            custodian: "CustomCustodian1234567890123456789012".to_string(),
            epoch: 0,
            unix_timestamp: 0,
        };
        
        assert!(!custom_custodian_lockup.is_default_lockup());
        assert!(custom_custodian_lockup.has_constraints());
        let opt = custom_custodian_lockup.to_optimized().unwrap();
        assert_eq!(opt.custodian, Some("CustomCustodian1234567890123456789012".to_string()));
        assert!(opt.epoch.is_none());
        assert!(opt.unix_timestamp.is_none());

        // Test full lockup (all constraints)
        let full_lockup = StakeLockup {
            custodian: "CustomCustodian1234567890123456789012".to_string(),
            epoch: 500,
            unix_timestamp: 1640995200,
        };
        
        assert!(!full_lockup.is_default_lockup());
        assert!(full_lockup.has_constraints());
        let opt = full_lockup.to_optimized().unwrap();
        assert_eq!(opt.custodian, Some("CustomCustodian1234567890123456789012".to_string()));
        assert_eq!(opt.epoch, Some(500));
        assert_eq!(opt.unix_timestamp, Some(1640995200));
    }
}