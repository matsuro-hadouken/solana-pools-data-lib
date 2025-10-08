//! Error types for the pools data library.
//!
//! This module provides comprehensive error handling with specific error types
//! for different failure scenarios, enabling developers to handle errors appropriately.

use std::time::Duration;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Main error type for the pools data library
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum PoolsDataError {
    /// HTTP request failed (network issue, connection timeout, etc.)
    #[error("Network error: {message}")]
    NetworkError { message: String },

    /// RPC returned an error response
    #[error("RPC error: {code} - {message}")]
    RpcError { code: i64, message: String },

    /// Failed to parse JSON response
    #[error("Parse error: {message}")]
    ParseError { message: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}")]
    RateLimitExceeded { message: String },

    /// Request timeout
    #[error("Request timeout after {timeout:?}")]
    RequestTimeout { timeout: Duration },

    /// Invalid configuration
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// Pool not found in the known pools list
    #[error("Pool '{pool_name}' not found in available pools")]
    PoolNotFound { pool_name: String },

    /// Invalid stake account data structure
    #[error("Invalid stake account data: {message}")]
    InvalidStakeData { message: String },

    /// Too many failures in batch operation
    #[error("Batch operation failed: {successful} succeeded, {failed} failed")]
    BatchOperationFailed { successful: usize, failed: usize },

    /// Generic error for unexpected issues
    #[error("Internal error: {message}")]
    InternalError { message: String },
}

/// Error information for a specific pool fetch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolError {
    /// Name of the pool that failed
    pub pool_name: String,
    /// Authority key of the pool
    pub authority: String,
    /// The specific error that occurred
    pub error: PoolsDataError,
    /// Whether this error is likely to succeed if retried
    pub retryable: bool,
    /// Number of attempts made
    pub attempts: u32,
}

impl PoolError {
    /// Create a new pool error
    #[must_use] pub const fn new(pool_name: String, authority: String, error: PoolsDataError, attempts: u32) -> Self {
        let retryable = Self::is_retryable(&error);
        Self {
            pool_name,
            authority,
            error,
            retryable,
            attempts,
        }
    }

    /// Determine if an error is retryable
    const fn is_retryable(error: &PoolsDataError) -> bool {
        match error {
            PoolsDataError::NetworkError { .. } => true,
            PoolsDataError::RpcError { code, .. } => {
                // Some RPC errors are retryable
                match code {
                    -32602 => false, // Invalid params - don't retry
                    -32601 => false, // Method not found - don't retry
                    -32603 => true,  // Internal error - might work on retry
                    _ => true,       // Other RPC errors may be temporary
                }
            }
            PoolsDataError::ParseError { .. } => false,      // Parse errors don't fix themselves
            PoolsDataError::RateLimitExceeded { .. } => true, // Rate limits are temporary
            PoolsDataError::RequestTimeout { .. } => true,   // Timeouts might work on retry
            PoolsDataError::ConfigurationError { .. } => false, // Config errors need fixing
            PoolsDataError::PoolNotFound { .. } => false,    // Pool doesn't exist
            PoolsDataError::InvalidStakeData { .. } => false, // Data structure issues
            PoolsDataError::BatchOperationFailed { .. } => false, // Aggregate error
            PoolsDataError::InternalError { .. } => true,    // Internal errors may be temporary
        }
    }
}

// Helper conversions for common error types
impl From<reqwest::Error> for PoolsDataError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            PoolsDataError::RequestTimeout {
                timeout: Duration::from_secs(30), // Default timeout
            }
        } else if error.is_connect() || error.is_request() {
            PoolsDataError::NetworkError {
                message: error.to_string(),
            }
        } else {
            PoolsDataError::InternalError {
                message: error.to_string(),
            }
        }
    }
}

impl From<serde_json::Error> for PoolsDataError {
    fn from(error: serde_json::Error) -> Self {
        PoolsDataError::ParseError {
            message: error.to_string(),
        }
    }
}

// Note: Governor's NotUntil type is complex and version-dependent
// We'll handle rate limiting errors manually in the client code instead

/// Result type for operations that might fail with `PoolsDataError`
pub type Result<T> = std::result::Result<T, PoolsDataError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retryable_errors() {
        let network_error = PoolsDataError::NetworkError {
            message: "Connection refused".to_string(),
        };
        assert!(PoolError::is_retryable(&network_error));

        let parse_error = PoolsDataError::ParseError {
            message: "Invalid JSON".to_string(),
        };
        assert!(!PoolError::is_retryable(&parse_error));

        let rate_limit_error = PoolsDataError::RateLimitExceeded {
            message: "Too many requests".to_string(),
        };
        assert!(PoolError::is_retryable(&rate_limit_error));
    }
}