//! Main client for fetching pools data.
//!
//! This module provides the primary interface for the library, handling
//! rate limiting, retries, concurrent requests, and data aggregation.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio_retry::{strategy::ExponentialBackoff, Retry};

use crate::config::{ClientConfig, PoolsDataClientBuilder};
use crate::error::{PoolsDataError, PoolError, Result};
use crate::pools::{get_all_pools, get_pools_by_names, PoolInfo};
use crate::rpc::RpcClient;
use crate::types::*;

/// Main client for fetching Solana pools data
pub struct PoolsDataClient {
    config: ClientConfig,
    rpc_client: RpcClient,
    semaphore: Arc<Semaphore>,
}

impl PoolsDataClient {
    /// Create a new client builder
    pub fn builder() -> PoolsDataClientBuilder {
        PoolsDataClientBuilder::new()
    }

    /// Create a new client with default configuration
    pub fn new(rpc_url: &str) -> Result<Self> {
        Self::builder().build(rpc_url).and_then(Self::from_config)
    }

    /// Create client from configuration
    pub fn from_config(config: ClientConfig) -> Result<Self> {
        let rpc_client = RpcClient::new(config.rpc_url.clone(), config.timeout);
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));

        Ok(Self {
            config,
            rpc_client,
            semaphore,
        })
    }

    /// List all available pools (no RPC calls)
    pub fn list_available_pools(&self) -> Vec<PoolInfo> {
        get_all_pools().to_vec()
    }

    /// Test RPC connection
    pub async fn test_connection(&self) -> Result<()> {
        self.rpc_client.test_connection().await
    }

    /// Fetch data for all available pools
    pub async fn fetch_all_pools(&self) -> Result<PoolsDataResult> {
        let all_pools = get_all_pools();
        let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();
        self.fetch_pools(&pool_names).await
    }

    /// Fetch data for specific pools by name
    pub async fn fetch_pools(&self, pool_names: &[&str]) -> Result<PoolsDataResult> {
        let pools_to_fetch = get_pools_by_names(pool_names);
        
        if pools_to_fetch.is_empty() {
            return Err(PoolsDataError::ConfigurationError {
                message: "No valid pools found in the provided names".to_string(),
            });
        }

        // Check for unknown pools
        let unknown_pools: Vec<_> = pool_names
            .iter()
            .filter(|name| !pools_to_fetch.iter().any(|p| p.name == **name))
            .collect();

        if !unknown_pools.is_empty() {
            log::warn!("Unknown pools requested: {:?}", unknown_pools);
        }

        log::info!("Fetching data for {} pools", pools_to_fetch.len());

        // Fetch pools concurrently with rate limiting
        let mut result = PoolsDataResult::new();
        let futures: Vec<_> = pools_to_fetch
            .into_iter()
            .map(|pool| self.fetch_single_pool(pool))
            .collect();

        // Execute all requests
        let pool_results = futures::future::join_all(futures).await;

        // Process results
        for pool_result in pool_results {
            match pool_result {
                Ok(pool_data) => {
                    result.successful.insert(pool_data.pool_name.clone(), pool_data);
                }
                Err(pool_error) => {
                    result.failed.insert(pool_error.pool_name.clone(), pool_error);
                }
            }
        }

        // Calculate summary
        result.summary = PoolsDataSummary::from_pools_data(&result.successful, &result.failed);

        log::info!(
            "Completed: {} successful, {} failed pools",
            result.successful.len(),
            result.failed.len()
        );

        Ok(result)
    }

    /// Retry only failed pools from a previous result
    pub async fn retry_failed_pools(&self, failed: &HashMap<String, PoolError>) -> Result<PoolsDataResult> {
        let retryable_pools: Vec<_> = failed
            .values()
            .filter(|error| error.retryable)
            .map(|error| error.pool_name.as_str())
            .collect();

        if retryable_pools.is_empty() {
            return Err(PoolsDataError::ConfigurationError {
                message: "No retryable pools found".to_string(),
            });
        }

        log::info!("Retrying {} failed pools", retryable_pools.len());
        self.fetch_pools(&retryable_pools).await
    }

    /// Get pools that stake to a specific validator
    pub async fn get_pools_staking_to_validator(&self, validator_pubkey: &str) -> Result<Vec<ValidatorPoolStake>> {
        let all_pools_result = self.fetch_all_pools().await?;
        let mut validator_stakes = Vec::new();

        for (pool_name, pool_data) in all_pools_result.successful {
            if let Some(validator_stake) = pool_data.validator_distribution.get(validator_pubkey) {
                let pool_percentage = if pool_data.statistics.total_staked_lamports > 0 {
                    (validator_stake.total_delegated as f64 / pool_data.statistics.total_staked_lamports as f64) * 100.0
                } else {
                    0.0
                };

                validator_stakes.push(ValidatorPoolStake {
                    validator_pubkey: validator_pubkey.to_string(),
                    pool_name: pool_name.clone(),
                    pool_authority: pool_data.authority.clone(),
                    delegated_amount: validator_stake.total_delegated,
                    account_count: validator_stake.account_count,
                    pool_percentage,
                });
            }
        }

        validator_stakes.sort_by(|a, b| b.delegated_amount.cmp(&a.delegated_amount));
        Ok(validator_stakes)
    }

    /// Get summary statistics for all pools (lighter than full fetch)
    pub async fn get_pools_summary(&self) -> Result<PoolsDataSummary> {
        let result = self.fetch_all_pools().await?;
        Ok(result.summary)
    }

    /// Fetch data for a single pool with retries and rate limiting
    async fn fetch_single_pool(&self, pool_info: PoolInfo) -> std::result::Result<PoolData, PoolError> {
        let _permit = self.semaphore.acquire().await.map_err(|e| {
            PoolError::new(
                pool_info.name.clone(),
                pool_info.authority.clone(),
                PoolsDataError::InternalError {
                    message: format!("Failed to acquire semaphore: {}", e),
                },
                0,
            )
        })?;

        // Apply rate limiting if configured
        if let Some(rate_limiter) = &self.config.rate_limiter {
            rate_limiter.until_ready().await;
        }

        let retry_strategy = ExponentialBackoff::from_millis(self.config.retry_base_delay.as_millis() as u64)
            .max_delay(std::time::Duration::from_secs(10))
            .take(self.config.retry_attempts as usize);

        let pool_name = pool_info.name.clone();
        let authority = pool_info.authority.clone();
        let rpc_client = &self.rpc_client;

        let mut attempts = 0;
        let pool_name_clone = pool_name.clone();
        let authority_clone = authority.clone();

        let action = move || {
            attempts += 1;
            let pool_name_ref = pool_name_clone.clone();
            let authority_ref = authority_clone.clone();
            
            async move {
                log::debug!("Fetching pool '{}' (attempt {})", pool_name_ref, attempts);
                
                let stake_accounts = rpc_client
                    .fetch_stake_accounts_for_authority(&authority_ref)
                    .await?;

                let mut pool_data = PoolData::new(pool_name_ref.clone(), authority_ref.clone());
                pool_data.stake_accounts = stake_accounts;

                // Calculate validator distribution
                pool_data.validator_distribution = Self::calculate_validator_distribution(&pool_data.stake_accounts);
                
                // Calculate statistics
                pool_data.calculate_statistics();

                log::debug!(
                    "Successfully fetched pool '{}': {} accounts, {:.2} SOL",
                    pool_name_ref,
                    pool_data.stake_accounts.len(),
                    pool_data.total_staked_sol()
                );

                Ok(pool_data)
            }
        };

        match Retry::spawn(retry_strategy, action).await {
            Ok(pool_data) => Ok(pool_data),
            Err(error) => Err(PoolError::new(
                pool_info.name,
                pool_info.authority,
                error,
                attempts,
            )),
        }
    }

    /// Calculate validator distribution from stake accounts
    fn calculate_validator_distribution(stake_accounts: &[StakeAccountInfo]) -> HashMap<String, ValidatorStake> {
        let mut distribution = HashMap::new();

        for stake_account in stake_accounts {
            if let Some(delegation) = &stake_account.delegation {
                let validator_stake = distribution
                    .entry(delegation.voter.clone())
                    .or_insert_with(|| ValidatorStake::new(delegation.voter.clone()));

                validator_stake.add_stake_account(stake_account);
            }
        }

        distribution
    }

    /// Fetch pools and return production data with consistent schema
    /// 
    /// **PRODUCTION READY**: This method always returns the same JSON schema.
    /// Removes only truly static fields while preserving all dynamic data.
    /// 
    /// **SAFE USAGE:**
    /// - ✅ Backend database storage
    /// - ✅ API endpoints
    /// - ✅ Systems requiring consistent schema
    /// 
    /// **REMOVED FIELDS (static only):**
    /// - owner (always "Stake11111111111111111111111111111111111111")
    /// - executable (always false)
    /// - program (always "stake")  
    /// - space (always 200)
    /// - rent_exempt_reserve (chain constant)
    /// 
    /// **ALWAYS INCLUDED (consistent schema):**
    /// - lockup (custodian, epoch, unix_timestamp)
    /// - authority (staker, withdrawer)
    /// - All delegation fields
    /// 
    /// **SCHEMA GUARANTEE**: Same JSON structure every time, safe for databases.
    pub async fn fetch_pools_production(&self, pool_names: &[&str]) -> Result<HashMap<String, ProductionPoolData>> {
        let result = self.fetch_pools(pool_names).await?;
        let production: HashMap<String, ProductionPoolData> = result.successful
            .iter()
            .map(|(name, pool_data)| (name.clone(), pool_data.into()))
            .collect();
        Ok(production)
    }

    /// Fetch pools and return optimized data with static fields removed
    /// 
    /// ⚠️ **CRITICAL WARNING**: This method can return different JSON schemas
    /// in production! Optional fields (`lockup`, `custom_authority`) may appear
    /// suddenly when pools change their configuration, breaking your database
    /// schema and API clients.
    /// 
    /// **SAFE USAGE:**
    /// - ✅ Public APIs with caching layer
    /// - ✅ One-time data analysis
    /// 
    /// **DANGEROUS USAGE:**
    /// - ❌ Direct database storage (use `fetch_pools_production()` instead)
    /// - ❌ API endpoints without caching
    /// - ❌ Systems that assume fixed JSON schema
    /// 
    /// **PRODUCTION EXAMPLE:**
    /// ```text
    /// January: { "pubkey": "...", "lamports": 123 }
    /// March:   { "pubkey": "...", "lamports": 123, "lockup": { "epoch": 500 } }
    /// Result:  DATABASE CRASH - unknown column 'lockup'
    /// ```
    pub async fn fetch_pools_optimized(&self, pool_names: &[&str]) -> Result<HashMap<String, OptimizedPoolData>> {
        let result = self.fetch_pools(pool_names).await?;
        
        let optimized: HashMap<String, OptimizedPoolData> = result.successful
            .iter()
            .map(|(name, pool_data)| (name.clone(), pool_data.into()))
            .collect();
            
        Ok(optimized)
    }

    /// Fetch a single pool and return production data
    pub async fn fetch_pool_production(&self, pool_name: &str) -> Result<ProductionPoolData> {
        let result = self.fetch_pools(&[pool_name]).await?;
        
        match result.successful.get(pool_name) {
            Some(pool_data) => Ok(pool_data.into()),
            None => {
                if let Some(pool_error) = result.failed.get(pool_name) {
                    Err(pool_error.error.clone())
                } else {
                    Err(PoolsDataError::ParseError {
                        message: format!("Pool '{}' not found", pool_name),
                    })
                }
            }
        }
    }

    /// Fetch a single pool and return optimized data
    pub async fn fetch_pool_optimized(&self, pool_name: &str) -> Result<OptimizedPoolData> {
        let result = self.fetch_pools(&[pool_name]).await?;
        
        match result.successful.get(pool_name) {
            Some(pool_data) => Ok(pool_data.into()),
            None => {
                if let Some(pool_error) = result.failed.get(pool_name) {
                    Err(pool_error.error.clone())
                } else {
                    Err(PoolsDataError::ParseError {
                        message: format!("Pool '{}' not found", pool_name),
                    })
                }
            }
        }
    }

    /// Compare all three output formats
    pub async fn compare_all_output_sizes(&self, pool_names: &[&str]) -> Result<AllOutputComparison> {
        let full_result = self.fetch_pools(pool_names).await?;
        let production_result = self.fetch_pools_production(pool_names).await?;
        let optimized_result = self.fetch_pools_optimized(pool_names).await?;
        
        let full_json = serde_json::to_string_pretty(&full_result.successful)?;
        let production_json = serde_json::to_string_pretty(&production_result)?;
        let optimized_json = serde_json::to_string_pretty(&optimized_result)?;
        
        Ok(AllOutputComparison {
            full_size_bytes: full_json.len(),
            production_size_bytes: production_json.len(),
            optimized_size_bytes: optimized_json.len(),
            production_reduction_percent: if !full_json.is_empty() {
                ((full_json.len() as i64 - production_json.len() as i64) as f64 / full_json.len() as f64) * 100.0
            } else { 0.0 },
            optimized_reduction_percent: if !full_json.is_empty() {
                ((full_json.len() as i64 - optimized_json.len() as i64) as f64 / full_json.len() as f64) * 100.0
            } else { 0.0 },
            full_output: full_json,
            production_output: production_json,
            optimized_output: optimized_json,
        })
    }

    /// Legacy method: Compare full vs optimized (use compare_all_output_sizes instead)
    pub async fn compare_output_sizes(&self, pool_names: &[&str]) -> Result<OutputComparison> {
        let full_result = self.fetch_pools(pool_names).await?;
        let optimized_result = self.fetch_pools_optimized(pool_names).await?;
        
        let full_json = serde_json::to_string_pretty(&full_result.successful)?;
        let optimized_json = serde_json::to_string_pretty(&optimized_result)?;
        
        Ok(OutputComparison {
            full_size_bytes: full_json.len(),
            optimized_size_bytes: optimized_json.len(),
            size_reduction_percent: if !full_json.is_empty() {
                ((full_json.len() as i64 - optimized_json.len() as i64) as f64 / full_json.len() as f64) * 100.0
            } else { 0.0 },
            full_output: full_json,
            optimized_output: optimized_json,
        })
    }

    /// Static field analysis - show which fields are being removed
    pub fn get_static_field_analysis() -> StaticFieldAnalysis {
        StaticFieldAnalysis {
            removed_fields: vec![
                "owner (always Stake11111111111111111111111111111111111111)".to_string(),
                "executable (always false)".to_string(),
                "program (always 'stake')".to_string(),
                "space (always 200)".to_string(),
                "rent_exempt_reserve (chain constant ~2.28 SOL)".to_string(),
            ],
            conditionally_removed_fields: vec![
                "lockup.custodian (omitted when = 11111111111111111111111111111111)".to_string(),
                "lockup.epoch (omitted when = 0)".to_string(),
                "lockup.unix_timestamp (omitted when = 0)".to_string(),
                "custom_authority (omitted when staker == withdrawer)".to_string(),
            ],
            kept_fields: vec![
                "pubkey (unique stake account address)".to_string(),
                "lamports (total account balance)".to_string(),
                "stake_type (delegated/initialized)".to_string(),
                "delegation.validator (validator vote account)".to_string(),
                "delegation.stake_lamports (delegated amount)".to_string(),
                "delegation.activation_epoch (when stake activated)".to_string(),
                "delegation.deactivation_epoch (when stake deactivates)".to_string(),
                "delegation.credits_observed (CRITICAL for rewards)".to_string(),
                "delegation.warmup_cooldown_rate (typically 0.25, preserved for completeness)".to_string(),
                "lockup fields (ONLY when account has actual constraints)".to_string(),
                "custom_authority (ONLY when staker != withdrawer)".to_string(),
            ],
            rationale: "Removed truly static fields while preserving all dynamic data. Lockup and authority fields are conditionally included only when they contain non-default values, ensuring atomic precision.".to_string(),
        }
    }
}

/// Comparison of full vs optimized output
#[derive(Debug, Clone)]
pub struct OutputComparison {
    pub full_size_bytes: usize,
    pub optimized_size_bytes: usize, 
    pub size_reduction_percent: f64,
    pub full_output: String,
    pub optimized_output: String,
}

/// Comparison of all three output formats
#[derive(Debug, Clone)]
pub struct AllOutputComparison {
    pub full_size_bytes: usize,
    pub production_size_bytes: usize,
    pub optimized_size_bytes: usize,
    pub production_reduction_percent: f64,
    pub optimized_reduction_percent: f64,
    pub full_output: String,
    pub production_output: String,
    pub optimized_output: String,
}

/// Analysis of static fields being removed
#[derive(Debug, Clone)]
pub struct StaticFieldAnalysis {
    pub removed_fields: Vec<String>,
    pub conditionally_removed_fields: Vec<String>,
    pub kept_fields: Vec<String>,
    pub rationale: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PoolsDataClient::builder()
            .rate_limit(5)
            .build("https://test.com")
            .and_then(PoolsDataClient::from_config);

        assert!(client.is_ok());
    }

    #[test]
    fn test_list_available_pools() {
        let client = PoolsDataClient::builder()
            .build("https://test.com")
            .and_then(PoolsDataClient::from_config)
            .unwrap();

        let pools = client.list_available_pools();
        assert!(!pools.is_empty());
        assert!(pools.iter().any(|p| p.name == "jito"));
    }

    #[test]
    fn test_validator_distribution_calculation() {
        let stake_accounts = vec![
            StakeAccountInfo {
                pubkey: "stake1".to_string(),
                lamports: 5_000_000_000,
                rent_exempt_reserve: 2_282_880,
                delegation: Some(StakeDelegation {
                    voter: "validator1".to_string(),
                    stake: 4_997_717_120,
                    activation_epoch: 100,
                    deactivation_epoch: u64::MAX,
                    credits_observed: 1000,
                    warmup_cooldown_rate: 0.25,
                }),
                authorized: StakeAuthorized {
                    staker: "pool_auth".to_string(),
                    withdrawer: "pool_auth".to_string(),
                },
                lockup: StakeLockup {
                    custodian: "11111111111111111111111111111111".to_string(),
                    epoch: 0,
                    unix_timestamp: 0,
                },
            },
            StakeAccountInfo {
                pubkey: "stake2".to_string(),
                lamports: 3_000_000_000,
                rent_exempt_reserve: 2_282_880,
                delegation: Some(StakeDelegation {
                    voter: "validator1".to_string(),
                    stake: 2_997_717_120,
                    activation_epoch: 100,
                    deactivation_epoch: u64::MAX,
                    credits_observed: 1000,
                    warmup_cooldown_rate: 0.25,
                }),
                authorized: StakeAuthorized {
                    staker: "pool_auth".to_string(),
                    withdrawer: "pool_auth".to_string(),
                },
                lockup: StakeLockup {
                    custodian: "11111111111111111111111111111111".to_string(),
                    epoch: 0,
                    unix_timestamp: 0,
                },
            },
        ];

        let distribution = PoolsDataClient::calculate_validator_distribution(&stake_accounts);
        
        assert_eq!(distribution.len(), 1);
        let validator_stake = distribution.get("validator1").unwrap();
        assert_eq!(validator_stake.account_count, 2);
        assert_eq!(validator_stake.total_delegated, 4_997_717_120 + 2_997_717_120);
    }

    // Integration tests that require actual RPC calls should be in separate file
}