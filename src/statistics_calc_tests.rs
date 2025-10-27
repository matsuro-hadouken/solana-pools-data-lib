//! Unit tests for statistics_calc.rs

#[cfg(test)]
mod tests {
    #[test]
    fn test_error_empty_pool_name() {
        use crate::types::ProductionPoolData;
        use crate::types::PoolStatistics;
        use std::collections::HashMap;
        let pool = ProductionPoolData {
            pool_name: "".to_string(),
            authority: "testauth".to_string(),
            stake_accounts: vec![],
            validator_distribution: HashMap::new(),
            statistics: PoolStatistics::default(),
            fetched_at: chrono::Utc::now(),
        };
        let result = crate::statistics_calc::calculate_pool_statistics_full(&pool, 1);
        assert!(matches!(result, Err(crate::error::PoolsDataError::ConfigurationError { .. })), "Expected ConfigurationError for empty pool name");
    }

    #[test]
    fn test_error_empty_authority() {
        use crate::types::ProductionPoolData;
        use crate::types::PoolStatistics;
        use std::collections::HashMap;
        let pool = ProductionPoolData {
            pool_name: "testpool".to_string(),
            authority: "".to_string(),
            stake_accounts: vec![],
            validator_distribution: HashMap::new(),
            statistics: PoolStatistics::default(),
            fetched_at: chrono::Utc::now(),
        };
        let result = crate::statistics_calc::calculate_pool_statistics_full(&pool, 1);
        assert!(matches!(result, Err(crate::error::PoolsDataError::ConfigurationError { .. })), "Expected ConfigurationError for empty authority");
    }
    use crate::statistics_calc::calculate_pool_statistics_full;
    use crate::types::ProductionPoolData;
    use crate::types::PoolStatistics;
    use std::collections::HashMap;
    use chrono::Utc;

    #[test]
    fn test_empty_pool_statistics() {
        let pool = ProductionPoolData {
            pool_name: "testpool".to_string(),
            authority: "testauth".to_string(),
            stake_accounts: vec![],
            validator_distribution: HashMap::new(),
            statistics: PoolStatistics::default(),
            fetched_at: Utc::now(),
        };
            let stats = calculate_pool_statistics_full(&pool, 123).unwrap();
            assert_eq!(stats.summary().total_accounts, 0);
            assert_eq!(stats.summary().active_accounts, 0);
            assert_eq!(stats.summary().deactivating_accounts, 0);
            assert_eq!(stats.summary().deactivated_accounts, 0);
            assert_eq!(stats.summary().total_lamports, 0);
    }

    #[test]
    fn test_active_account_statistics() {
        use crate::types::{ProductionStakeAccountInfo, ProductionStakeDelegation, ProductionStakeAuthority, ProductionStakeLockup};
        let account = ProductionStakeAccountInfo {
            pubkey: "active_account".to_string(),
            lamports: 1000,
            stake_type: "delegated".to_string(),
            delegation: Some(ProductionStakeDelegation {
                validator: "validator1".to_string(),
                stake_lamports: 1000,
                activation_epoch: 0,
                deactivation_epoch: u64::MAX,
                last_epoch_credits_cumulative: 0,
            }),
            authority: ProductionStakeAuthority {
                staker: "staker1".to_string(),
                withdrawer: "withdrawer1".to_string(),
            },
            lockup: ProductionStakeLockup {
                custodian: "".to_string(),
                epoch: 0,
                unix_timestamp: 0,
            },
        };
        let pool = ProductionPoolData {
            pool_name: "testpool".to_string(),
            authority: "testauth".to_string(),
            stake_accounts: vec![account],
            validator_distribution: HashMap::new(),
            statistics: PoolStatistics::default(),
            fetched_at: Utc::now(),
        };
            let stats = calculate_pool_statistics_full(&pool, 1).unwrap();
            let summary = stats.summary();
            assert_eq!(summary.total_accounts, 1);
            assert_eq!(summary.active_accounts, 1);
            assert_eq!(summary.active_stake_lamports, 1000);
            assert_eq!(summary.deactivating_accounts, 0);
            assert_eq!(summary.deactivated_accounts, 0);
            assert_eq!(summary.total_lamports, 1000);
    }
}
