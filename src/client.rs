//! Client for fetching pools data.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_retry::{strategy::ExponentialBackoff, Retry};

use crate::config::{ClientConfig, PoolsDataClientBuilder};
use crate::error::{PoolError, PoolsDataError, Result};
use crate::pools::{get_all_pools, get_pools_by_names, PoolInfo};
use crate::rpc::RpcClient;
use crate::types::{
    FieldAnalysis, PoolData, PoolStatistics, PoolsDataResult, ProductionPoolData, StakeAccountInfo,
    ValidatorStake,
};

/// Main client for fetching Solana pools data
pub struct PoolsDataClient {
    config: ClientConfig,
    rpc_client: RpcClient,
    semaphore: Arc<Semaphore>,
}

impl PoolsDataClient {
    /// Create a new client builder
    #[must_use]
    pub fn builder() -> PoolsDataClientBuilder {
        PoolsDataClientBuilder::new()
    }

    /// Create a new client from configuration
    ///
    /// # Errors
    ///
    /// Returns error if the configuration is invalid or if system resources cannot be allocated.
    pub fn from_config(config: ClientConfig) -> Result<Self> {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        let rpc_client = RpcClient::new(config.rpc_url.clone(), config.timeout);

        Ok(Self {
            config,
            rpc_client,
            semaphore,
        })
    }

    /// Get list of all available pools
    #[must_use]
    pub fn list_available_pools() -> Vec<PoolInfo> {
        get_all_pools().to_vec()
    }

    /// Get static field analysis
    #[must_use]
    pub fn get_static_field_analysis() -> FieldAnalysis {
        FieldAnalysis::new()
    }

    /// Test RPC connection
    ///
    /// # Errors
    ///
    /// Returns error if the RPC endpoint is unreachable or returns invalid responses.
    pub async fn test_connection(&self) -> Result<()> {
        self.rpc_client.test_connection().await
    }

    /// Fetch stake pool data for production use
    ///
    /// Returns data with static/redundant fields removed.
    /// Use this method for production databases where storage size matters.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Any requested pool is not found
    /// - Network connection fails
    /// - RPC endpoint returns invalid data
    pub async fn fetch_pools(
        &self,
        pool_names: &[&str],
    ) -> Result<HashMap<String, ProductionPoolData>> {
        let debug_result = self.fetch_pools_debug(pool_names).await?;

        // Convert to production format
        let production_data: HashMap<String, ProductionPoolData> = debug_result
            .successful
            .iter()
            .map(|(name, pool)| (name.clone(), pool.into()))
            .collect();

        Ok(production_data)
    }

    /// Fetch data for all available pools
    ///
    /// # Errors
    ///
    /// Returns error if any pool fails to fetch or if network issues occur.
    pub async fn fetch_all_pools(&self) -> Result<HashMap<String, ProductionPoolData>> {
        let all_pools = get_all_pools();
        let pool_names: Vec<&str> = all_pools.iter().map(|p| p.name.as_str()).collect();
        self.fetch_pools(&pool_names).await
    }

    /// Fetch stake pool data with complete debugging information
    ///
    /// Returns ALL fields from RPC response - use for debugging and development.
    /// Contains complete raw data including static/redundant fields.
    ///
    /// # Errors
    ///
    /// Returns error if all requested pools fail to fetch.
    ///
    /// # Panics
    ///
    /// Panics if the result contains failed pools but the failed map is unexpectedly empty.
    /// This should never happen in normal operation.
    pub async fn fetch_pools_debug(&self, pool_names: &[&str]) -> Result<PoolsDataResult> {
        let pools_to_fetch = get_pools_by_names(pool_names);

        if pools_to_fetch.is_empty() {
            return Err(PoolsDataError::PoolNotFound {
                pool_name: format!("None of the requested pools found: {pool_names:?}"),
            });
        }

        log::info!("Fetching {} pools", pools_to_fetch.len());

        let mut tasks = Vec::new();
        for pool_info in pools_to_fetch {
            let rpc_client = self.rpc_client.clone();
            let semaphore = Arc::clone(&self.semaphore);
            let retry_attempts = self.config.retry_attempts;
            let retry_base_delay = self.config.retry_base_delay;
            let rate_limiter = self.config.rate_limiter.clone();

            let task = tokio::spawn(async move {
                Self::fetch_single_pool_impl(
                    rpc_client,
                    semaphore,
                    pool_info,
                    retry_attempts,
                    retry_base_delay,
                    rate_limiter,
                )
                .await
            });
            tasks.push(task);
        }

        let mut result = PoolsDataResult::new();

        for task in tasks {
            match task.await {
                Ok(Ok(pool_data)) => {
                    result
                        .successful
                        .insert(pool_data.pool_name.clone(), pool_data);
                }
                Ok(Err(pool_error)) => {
                    result
                        .failed
                        .insert(pool_error.pool_name.clone(), pool_error);
                }
                Err(join_error) => {
                    log::error!("Task join error: {join_error}");
                    result.failed.insert(
                        "unknown".to_string(),
                        PoolError::new(
                            "unknown".to_string(),
                            "unknown".to_string(),
                            PoolsDataError::InternalError {
                                message: format!("Task failed: {join_error}"),
                            },
                            0,
                        ),
                    );
                }
            }
        }

        // Update summary
        result.summary.total_pools_attempted = result.successful.len() + result.failed.len();
        result.summary.successful_pools = result.successful.len();
        result.summary.failed_pools = result.failed.len();

        if result.successful.is_empty() && !result.failed.is_empty() {
            let first_error = result.failed.values().next().unwrap();
            return Err(first_error.error.clone());
        }

        Ok(result)
    }

    /// Fetch data for a single pool with retries and rate limiting
    async fn fetch_single_pool_impl(
        rpc_client: RpcClient,
        semaphore: Arc<Semaphore>,
        pool_info: PoolInfo,
        retry_attempts: u32,
        retry_base_delay: Duration,
        rate_limiter: Option<Arc<governor::RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>>,
    ) -> std::result::Result<PoolData, PoolError> {
        let _permit = semaphore.acquire().await.map_err(|e| {
            PoolError::new(
                pool_info.name.clone(),
                pool_info.authority.clone(),
                PoolsDataError::InternalError {
                    message: format!("Failed to acquire semaphore: {e}"),
                },
                0,
            )
        })?;

        // Apply rate limiting if configured
        if let Some(limiter) = &rate_limiter {
            limiter.until_ready().await;
        }

        log::debug!("Fetching pool: {}", pool_info.name);

        #[allow(clippy::cast_possible_truncation)]
        // Duration as_millis() to u64 is intentional for retry delays
        let retry_strategy = ExponentialBackoff::from_millis(retry_base_delay.as_millis() as u64)
            .max_delay(std::time::Duration::from_secs(30))
            .take(retry_attempts as usize);

        let pool_name = pool_info.name.clone();
        let authority = pool_info.authority.clone();

        let result = Retry::spawn(retry_strategy, || async {
            rpc_client
                .fetch_stake_accounts_for_authority(&pool_info.authority)
                .await
        })
        .await;

        match result {
            Ok(stake_accounts) => {
                if stake_accounts.is_empty() {
                    return Err(PoolError::new(
                        pool_name,
                        authority,
                        PoolsDataError::NoStakeAccounts { 
                            pool_name: pool_info.name.clone() 
                        },
                        0,
                    ));
                }

                let validator_distribution =
                    Self::calculate_validator_distribution(&stake_accounts);
                let statistics = Self::calculate_pool_statistics(&stake_accounts);

                Ok(PoolData {
                    pool_name: pool_info.name,
                    authority: pool_info.authority,
                    stake_accounts,
                    validator_distribution,
                    statistics,
                    fetched_at: chrono::Utc::now(),
                })
            }
            Err(e) => {
                log::error!("Failed to fetch pool {pool_name}: {e}");
                Err(PoolError::new(pool_name, authority, e, 0))
            }
        }
    }

    /// Calculate validator distribution from stake accounts
    fn calculate_validator_distribution(
        stake_accounts: &[StakeAccountInfo],
    ) -> HashMap<String, ValidatorStake> {
        let mut distribution = HashMap::new();

        for account in stake_accounts {
            if let Some(delegation) = &account.delegation {
                // Only sum stake accounts that are active (deactivation_epoch == u64::MAX) and stake > 0
                if delegation.deactivation_epoch == u64::MAX && delegation.stake > 0 {
                    let entry =
                        distribution
                            .entry(delegation.voter.clone())
                            .or_insert(ValidatorStake {
                                total_delegated: 0,
                                account_count: 0,
                                accounts: Vec::new(),
                            });

                    entry.total_delegated += delegation.stake;
                    entry.account_count += 1;
                    entry.accounts.push(account.pubkey.clone());
                }
            }
        }

        distribution
    }

    /// Calculate pool statistics
    fn calculate_pool_statistics(stake_accounts: &[StakeAccountInfo]) -> PoolStatistics {
        let total_accounts = stake_accounts.len();
        let total_lamports = stake_accounts.iter().map(|a| a.lamports).sum();
        let total_staked_lamports = stake_accounts
            .iter()
            .filter_map(|a| a.delegation.as_ref().map(|d| d.stake))
            .sum();

        let active_stake_accounts = stake_accounts
            .iter()
            .filter(|a| {
                a.delegation.is_some()
                    && a.delegation.as_ref().unwrap().deactivation_epoch == u64::MAX
            })
            .count();

        let deactivating_stake_accounts = stake_accounts
            .iter()
            .filter(|a| {
                a.delegation.is_some()
                    && a.delegation.as_ref().unwrap().deactivation_epoch != u64::MAX
            })
            .count();

        let validator_count = stake_accounts
            .iter()
            .filter_map(|a| a.delegation.as_ref().map(|d| &d.voter))
            .collect::<std::collections::HashSet<_>>()
            .len();

        PoolStatistics {
            total_accounts,
            total_lamports,
            total_staked_lamports,
            active_stake_accounts,
            deactivating_stake_accounts,
            validator_count,
        }
    }
}
