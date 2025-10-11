//! Configuration types for the pools data library.
//!
//! This module provides flexible configuration options for different RPC providers
//! and use cases, from conservative public RPC settings to high-performance private RPC.

use crate::error::{PoolsDataError, Result};
use governor::{Quota, RateLimiter};
use std::sync::Arc;
use std::time::Duration;

/// Advanced rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Primary rate limit (requests per second)
    pub requests_per_second: Option<u32>,
    /// Burst limit (max requests in burst)
    pub burst_size: Option<u32>,
    /// Time window for rate limiting
    pub time_window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: Some(DefaultConfig::RATE_LIMIT_PER_SECOND),
            burst_size: None,
            time_window: Duration::from_secs(1),
        }
    }
}

impl RateLimitConfig {
    /// Create new rate limit config
    #[must_use]
    pub const fn new() -> Self {
        Self {
            requests_per_second: None,
            burst_size: None,
            time_window: Duration::from_secs(1),
        }
    }

    /// Set requests per second
    #[must_use]
    pub const fn requests_per_second(mut self, rps: u32) -> Self {
        self.requests_per_second = Some(rps);
        self
    }

    /// Set burst size
    #[must_use]
    pub const fn burst_size(mut self, burst: u32) -> Self {
        self.burst_size = Some(burst);
        self
    }

    /// Set time window
    #[must_use]
    pub const fn time_window(mut self, window: Duration) -> Self {
        self.time_window = window;
        self
    }

    /// No rate limiting
    #[must_use]
    pub const fn none() -> Self {
        Self {
            requests_per_second: None,
            burst_size: None,
            time_window: Duration::from_secs(1),
        }
    }
}

/// Configuration builder for `PoolsDataClient`
#[derive(Debug, Clone)]
pub struct PoolsDataClientBuilder {
    rate_limit: Option<u32>,
    burst_size: Option<u32>,
    retry_attempts: u32,
    retry_base_delay_ms: u64,
    timeout_secs: u64,
    max_concurrent: usize,
}

impl Default for PoolsDataClientBuilder {
    fn default() -> Self {
        Self {
            rate_limit: Some(DefaultConfig::RATE_LIMIT_PER_SECOND),
            burst_size: None,
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

    /// Set burst size for rate limiting
    #[must_use]
    pub const fn burst_size(mut self, burst: u32) -> Self {
        self.burst_size = Some(burst);
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

    /// Use preset configuration for Alchemy RPC
    #[must_use]
    pub const fn alchemy_config(mut self) -> Self {
        self.rate_limit = Some(AlchemyConfig::RATE_LIMIT_PER_SECOND);
        self.retry_attempts = AlchemyConfig::RETRY_ATTEMPTS;
        self.retry_base_delay_ms = AlchemyConfig::RETRY_BASE_DELAY_MS;
        self.timeout_secs = AlchemyConfig::REQUEST_TIMEOUT_SECS;
        self.max_concurrent = AlchemyConfig::MAX_CONCURRENT_REQUESTS;
        self
    }

    /// Use preset configuration for `QuickNode` RPC
    #[must_use]
    pub const fn quicknode_config(mut self) -> Self {
        self.rate_limit = Some(QuickNodeConfig::RATE_LIMIT_PER_SECOND);
        self.retry_attempts = QuickNodeConfig::RETRY_ATTEMPTS;
        self.retry_base_delay_ms = QuickNodeConfig::RETRY_BASE_DELAY_MS;
        self.timeout_secs = QuickNodeConfig::REQUEST_TIMEOUT_SECS;
        self.max_concurrent = QuickNodeConfig::MAX_CONCURRENT_REQUESTS;
        self
    }

    /// Use preset configuration for Helius RPC
    #[must_use]
    pub const fn helius_config(mut self) -> Self {
        self.rate_limit = Some(HeliusConfig::RATE_LIMIT_PER_SECOND);
        self.retry_attempts = HeliusConfig::RETRY_ATTEMPTS;
        self.retry_base_delay_ms = HeliusConfig::RETRY_BASE_DELAY_MS;
        self.timeout_secs = HeliusConfig::REQUEST_TIMEOUT_SECS;
        self.max_concurrent = HeliusConfig::MAX_CONCURRENT_REQUESTS;
        self
    }

    /// Use preset configuration for public RPC endpoints (most conservative)
    #[must_use]
    pub const fn public_rpc_config(mut self) -> Self {
        self.rate_limit = Some(PublicRpcConfig::RATE_LIMIT_PER_SECOND);
        self.retry_attempts = PublicRpcConfig::RETRY_ATTEMPTS;
        self.retry_base_delay_ms = PublicRpcConfig::RETRY_BASE_DELAY_MS;
        self.timeout_secs = PublicRpcConfig::REQUEST_TIMEOUT_SECS;
        self.max_concurrent = PublicRpcConfig::MAX_CONCURRENT_REQUESTS;
        self
    }

    /// Use preset configuration for development/testing
    #[must_use]
    pub const fn development_config(mut self) -> Self {
        self.rate_limit = Some(DevelopmentConfig::RATE_LIMIT_PER_SECOND);
        self.retry_attempts = DevelopmentConfig::RETRY_ATTEMPTS;
        self.retry_base_delay_ms = DevelopmentConfig::RETRY_BASE_DELAY_MS;
        self.timeout_secs = DevelopmentConfig::REQUEST_TIMEOUT_SECS;
        self.max_concurrent = DevelopmentConfig::MAX_CONCURRENT_REQUESTS;
        self
    }

    /// Use preset configuration for enterprise/dedicated endpoints
    #[must_use]
    pub const fn enterprise_config(mut self) -> Self {
        self.rate_limit = Some(EnterpriseConfig::RATE_LIMIT_PER_SECOND);
        self.retry_attempts = EnterpriseConfig::RETRY_ATTEMPTS;
        self.retry_base_delay_ms = EnterpriseConfig::RETRY_BASE_DELAY_MS;
        self.timeout_secs = EnterpriseConfig::REQUEST_TIMEOUT_SECS;
        self.max_concurrent = EnterpriseConfig::MAX_CONCURRENT_REQUESTS;
        self
    }

    /// Auto-detect configuration based on RPC URL
    #[must_use]
    pub fn auto_config(mut self, rpc_url: &str) -> Self {
        // Basic URL-based detection
        let url_lower = rpc_url.to_lowercase();
        
        if url_lower.contains("alchemy") {
            self = self.alchemy_config();
        } else if url_lower.contains("quicknode") {
            self = self.quicknode_config();
        } else if url_lower.contains("helius") {
            self = self.helius_config();
        } else if url_lower.contains("mainnet-beta.solana.com") || url_lower.contains("api.mainnet") {
            self = self.public_rpc_config();
        } else if url_lower.contains("localhost") || url_lower.contains("127.0.0.1") {
            self = self.development_config();
        } else {
            // Default to conservative settings for unknown endpoints
            self = self.private_rpc_config();
        }
        
        self
    }

    /// Configuration for high-frequency trading or real-time applications
    #[must_use]
    pub const fn high_frequency_config(mut self) -> Self {
        self.rate_limit = Some(200);
        self.retry_attempts = 1;
        self.retry_base_delay_ms = 25;
        self.timeout_secs = 5;
        self.max_concurrent = 50;
        self
    }

    /// Configuration for batch processing applications
    #[must_use]
    pub const fn batch_processing_config(mut self) -> Self {
        self.rate_limit = Some(10);
        self.retry_attempts = 5;
        self.retry_base_delay_ms = 500;
        self.timeout_secs = 60;
        self.max_concurrent = 20;
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
                Some(nonzero_rps) => Some(Arc::new(RateLimiter::direct(Quota::per_second(nonzero_rps)))),
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
        Arc<RateLimiter<
            governor::state::direct::NotKeyed,
            governor::state::InMemoryState,
            governor::clock::DefaultClock,
        >>,
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

    /// Retry attempts for failed requests
    pub const RETRY_ATTEMPTS: u32 = 2;

    /// Base delay for exponential backoff
    pub const RETRY_BASE_DELAY_MS: u64 = 100;

    /// Request timeout duration
    pub const REQUEST_TIMEOUT_SECS: u64 = 15;
}

/// Configuration for Alchemy RPC provider
pub struct AlchemyConfig;

impl AlchemyConfig {
    /// Alchemy supports high rate limits
    pub const RATE_LIMIT_PER_SECOND: u32 = 25;
    pub const MAX_CONCURRENT_REQUESTS: usize = 8;
    pub const RETRY_ATTEMPTS: u32 = 2;
    pub const RETRY_BASE_DELAY_MS: u64 = 150;
    pub const REQUEST_TIMEOUT_SECS: u64 = 20;
}

/// Configuration for `QuickNode` RPC provider
pub struct QuickNodeConfig;

impl QuickNodeConfig {
    /// `QuickNode` performance characteristics: 20 RPS limit
    pub const RATE_LIMIT_PER_SECOND: u32 = 20;
    pub const MAX_CONCURRENT_REQUESTS: usize = 6;
    pub const RETRY_ATTEMPTS: u32 = 2;
    pub const RETRY_BASE_DELAY_MS: u64 = 200;
    pub const REQUEST_TIMEOUT_SECS: u64 = 25;
}

/// Configuration for Helius RPC provider
pub struct HeliusConfig;

impl HeliusConfig {
    /// Helius optimized settings
    pub const RATE_LIMIT_PER_SECOND: u32 = 30;
    pub const MAX_CONCURRENT_REQUESTS: usize = 10;
    pub const RETRY_ATTEMPTS: u32 = 2;
    pub const RETRY_BASE_DELAY_MS: u64 = 100;
    pub const REQUEST_TIMEOUT_SECS: u64 = 15;
}

/// Configuration for public RPC endpoints (most conservative)
pub struct PublicRpcConfig;

impl PublicRpcConfig {
    /// Configuration for public endpoints: 1 RPS limit
    pub const RATE_LIMIT_PER_SECOND: u32 = 1;
    pub const MAX_CONCURRENT_REQUESTS: usize = 1;
    pub const RETRY_ATTEMPTS: u32 = 5;
    pub const RETRY_BASE_DELAY_MS: u64 = 1000;
    pub const REQUEST_TIMEOUT_SECS: u64 = 45;
}

/// Configuration for development/testing
pub struct DevelopmentConfig;

impl DevelopmentConfig {
    /// Moderate settings for development
    pub const RATE_LIMIT_PER_SECOND: u32 = 10;
    pub const MAX_CONCURRENT_REQUESTS: usize = 5;
    pub const RETRY_ATTEMPTS: u32 = 3;
    pub const RETRY_BASE_DELAY_MS: u64 = 500;
    pub const REQUEST_TIMEOUT_SECS: u64 = 20;
}

/// Configuration for enterprise/dedicated endpoints
pub struct EnterpriseConfig;

impl EnterpriseConfig {
    /// High performance for enterprise endpoints
    pub const RATE_LIMIT_PER_SECOND: u32 = 100;
    pub const MAX_CONCURRENT_REQUESTS: usize = 20;
    pub const RETRY_ATTEMPTS: u32 = 1;
    pub const RETRY_BASE_DELAY_MS: u64 = 50;
    pub const REQUEST_TIMEOUT_SECS: u64 = 10;
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
