//! Configuration types for the pools data library.
//!
//! This module provides flexible configuration options for different RPC providers
//! and use cases, from conservative public RPC settings to high-performance private RPC.

use crate::error::{PoolsDataError, Result};
use governor::{Quota, RateLimiter};
use std::time::Duration;

/// Configuration builder for `PoolsDataClient`
#[derive(Debug, Clone)]
pub struct PoolsDataClientBuilder {
    rate_limit: Option<u32>,
    retry_attempts: u32,
    retry_base_delay_ms: u64,
    timeout_secs: u64,
    max_concurrent: usize,
}

impl Default for PoolsDataClientBuilder {
    fn default() -> Self {
        Self {
            rate_limit: Some(DefaultConfig::RATE_LIMIT_PER_SECOND),
            retry_attempts: DefaultConfig::RETRY_ATTEMPTS,
            retry_base_delay_ms: DefaultConfig::RETRY_BASE_DELAY_MS,
            timeout_secs: DefaultConfig::REQUEST_TIMEOUT_SECS,
            max_concurrent: DefaultConfig::MAX_CONCURRENT_REQUESTS,
        }
    }
}

impl PoolsDataClientBuilder {
    /// Create a new builder with default settings optimized for public RPC
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set rate limit in requests per second
    #[must_use]
    pub const fn rate_limit(mut self, requests_per_second: u32) -> Self {
        self.rate_limit = Some(requests_per_second);
        self
    }

    /// Remove rate limiting
    #[must_use]
    pub const fn no_rate_limit(mut self) -> Self {
        self.rate_limit = None;
        self
    }

    /// Set retry attempts
    #[must_use]
    pub const fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Set retry base delay
    #[must_use]
    pub const fn retry_base_delay(mut self, delay_ms: u64) -> Self {
        self.retry_base_delay_ms = delay_ms;
        self
    }

    /// Set timeout
    #[must_use]
    pub const fn timeout(mut self, seconds: u64) -> Self {
        self.timeout_secs = seconds;
        self
    }

    /// Set maximum concurrent requests
    #[must_use]
    pub const fn max_concurrent_requests(mut self, max: usize) -> Self {
        self.max_concurrent = max;
        self
    }

    /// Apply public RPC configuration
    #[must_use]
    pub const fn public_rpc_config(mut self) -> Self {
        self.rate_limit = Some(DefaultConfig::RATE_LIMIT_PER_SECOND);
        self.retry_attempts = DefaultConfig::RETRY_ATTEMPTS;
        self.retry_base_delay_ms = DefaultConfig::RETRY_BASE_DELAY_MS;
        self.timeout_secs = DefaultConfig::REQUEST_TIMEOUT_SECS;
        self.max_concurrent = DefaultConfig::MAX_CONCURRENT_REQUESTS;
        self
    }

    /// Use preset configuration for private/premium RPC endpoints
    #[must_use]
    pub const fn private_rpc_config(mut self) -> Self {
        self.rate_limit = Some(PrivateRpcConfig::RATE_LIMIT_PER_SECOND);
        self.retry_attempts = PrivateRpcConfig::RETRY_ATTEMPTS;
        self.retry_base_delay_ms = PrivateRpcConfig::RETRY_BASE_DELAY_MS;
        self.timeout_secs = PrivateRpcConfig::REQUEST_TIMEOUT_SECS;
        self.max_concurrent = PrivateRpcConfig::MAX_CONCURRENT_REQUESTS;
        self
    }

    /// Build the configuration
    ///
    /// # Errors
    ///
    /// Returns error if configuration values are invalid:
    /// - Invalid RPC URL format
    /// - Timeout is 0 or greater than 300 seconds
    /// - Max concurrent requests is 0 or greater than 100
    pub fn build(self, rpc_url: &str) -> Result<ClientConfig> {
        if self.retry_attempts > 10 {
            return Err(PoolsDataError::ConfigurationError {
                message: "Retry attempts cannot exceed 10".to_string(),
            });
        }

        if self.timeout_secs == 0 || self.timeout_secs > 300 {
            return Err(PoolsDataError::ConfigurationError {
                message: "Timeout must be between 1 and 300 seconds".to_string(),
            });
        }

        if self.max_concurrent == 0 || self.max_concurrent > 100 {
            return Err(PoolsDataError::ConfigurationError {
                message: "Max concurrent requests must be between 1 and 100".to_string(),
            });
        }

        let rate_limiter = if let Some(rps) = self.rate_limit {
            if rps == 0 || rps > 1000 {
                return Err(PoolsDataError::ConfigurationError {
                    message: "Rate limit must be between 1 and 1000 requests per second"
                        .to_string(),
                });
            }
            match std::num::NonZeroU32::new(rps) {
                Some(nonzero_rps) => Some(RateLimiter::direct(Quota::per_second(nonzero_rps))),
                None => {
                    return Err(PoolsDataError::ConfigurationError {
                        message: "Rate limit must be greater than 0".to_string(),
                    })
                }
            }
        } else {
            None
        };

        Ok(ClientConfig {
            rpc_url: rpc_url.to_string(),
            rate_limiter,
            retry_attempts: self.retry_attempts,
            retry_base_delay: Duration::from_millis(self.retry_base_delay_ms),
            timeout: Duration::from_secs(self.timeout_secs),
            max_concurrent: self.max_concurrent,
        })
    }
}

/// Internal configuration for the client
#[derive(Debug)]
pub struct ClientConfig {
    pub rpc_url: String,
    pub rate_limiter: Option<
        RateLimiter<
            governor::state::direct::NotKeyed,
            governor::state::InMemoryState,
            governor::clock::DefaultClock,
        >,
    >,
    pub retry_attempts: u32,
    pub retry_base_delay: Duration,
    pub timeout: Duration,
    pub max_concurrent: usize,
}

/// Default configuration optimized for public Solana RPC
pub struct DefaultConfig;

impl DefaultConfig {
    /// Conservative rate limit safe for public RPC endpoints
    pub const RATE_LIMIT_PER_SECOND: u32 = 2;

    /// Maximum concurrent requests to avoid overwhelming public RPC
    pub const MAX_CONCURRENT_REQUESTS: usize = 3;

    /// Number of retry attempts for failed requests
    pub const RETRY_ATTEMPTS: u32 = 3;

    /// Base delay for exponential backoff (200ms, 400ms, 800ms)
    pub const RETRY_BASE_DELAY_MS: u64 = 200;

    /// Request timeout - getProgramAccounts can be slow
    pub const REQUEST_TIMEOUT_SECS: u64 = 30;
}

/// Configuration optimized for private/premium RPC endpoints
pub struct PrivateRpcConfig;

impl PrivateRpcConfig {
    /// Higher rate limit for private RPC
    pub const RATE_LIMIT_PER_SECOND: u32 = 50;

    /// More concurrent requests for private RPC
    pub const MAX_CONCURRENT_REQUESTS: usize = 10;

    /// Fewer retries needed with reliable private RPC
    pub const RETRY_ATTEMPTS: u32 = 2;

    /// Faster retry with reliable private RPC
    pub const RETRY_BASE_DELAY_MS: u64 = 100;

    /// Shorter timeout with fast private RPC
    pub const REQUEST_TIMEOUT_SECS: u64 = 15;
}

/// No limits configuration for local testing
pub struct NoLimitsConfig;

impl NoLimitsConfig {
    pub const MAX_CONCURRENT_REQUESTS: usize = 50;
    pub const RETRY_ATTEMPTS: u32 = 1;
    pub const RETRY_BASE_DELAY_MS: u64 = 50;
    pub const REQUEST_TIMEOUT_SECS: u64 = 10;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let builder = PoolsDataClientBuilder::new();
        let config = builder.build("https://test.com").unwrap();

        assert_eq!(config.rpc_url, "https://test.com");
        assert!(config.rate_limiter.is_some());
        assert_eq!(config.retry_attempts, DefaultConfig::RETRY_ATTEMPTS);
        assert_eq!(
            config.max_concurrent,
            DefaultConfig::MAX_CONCURRENT_REQUESTS
        );
    }

    #[test]
    fn test_no_rate_limit() {
        let builder = PoolsDataClientBuilder::new().no_rate_limit();
        let config = builder.build("https://test.com").unwrap();

        assert!(config.rate_limiter.is_none());
    }

    #[test]
    fn test_invalid_config() {
        let result = PoolsDataClientBuilder::new()
            .retry_attempts(20)
            .build("https://test.com");

        assert!(result.is_err());
    }

    #[test]
    fn test_private_rpc_config() {
        let builder = PoolsDataClientBuilder::new().private_rpc_config();
        let config = builder.build("https://private-rpc.com").unwrap();

        assert_eq!(
            config.max_concurrent,
            PrivateRpcConfig::MAX_CONCURRENT_REQUESTS
        );
        assert_eq!(config.retry_attempts, PrivateRpcConfig::RETRY_ATTEMPTS);
    }
}
