//! RPC client for Solana blockchain communication.
//!
//! This module handles the low-level RPC communication with Solana nodes,
//! including request formatting, response parsing, and error handling.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use crate::error::{PoolsDataError, Result};
use crate::types::{StakeAccountInfo, StakeAuthorized, StakeDelegation, StakeLockup};

/// RPC request structure
#[derive(Debug, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

impl RpcRequest {
    /// Create a new RPC request
    fn new(id: u64, method: &str, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.to_string(),
            params,
        }
    }

    /// Create getProgramAccounts request for stake accounts
    fn get_program_accounts_stake(id: u64, authority: &str) -> Self {
        let params = json!([
            "Stake11111111111111111111111111111111111111",
            {
                "encoding": "jsonParsed",
                "filters": [
                    {
                        "memcmp": {
                            "offset": 12,
                            "bytes": authority
                        }
                    }
                ]
            }
        ]);

        Self::new(id, "getProgramAccounts", params)
    }
}

/// RPC response structure
#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    jsonrpc: String,  // Used for validation
    id: u64,          // Used for request/response matching
    result: Option<T>,
    error: Option<RpcError>,
}

/// RPC error structure
#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
    data: Option<Value>, // Additional error data for debugging (kept for future use)
}

/// Raw stake account data from RPC
#[derive(Debug, Deserialize)]
struct RawStakeAccount {
    pubkey: String,
    account: RawAccountData,
}

/// Raw account data structure
#[derive(Debug, Deserialize)]
struct RawAccountData {
    lamports: u64,
    data: RawParsedData,
    #[allow(dead_code)] // Account metadata, available for future validation
    executable: bool,
    #[allow(dead_code)] // Program owner, expected to be stake program
    owner: String,
    #[serde(rename = "rentEpoch")]
    #[allow(dead_code)] // Rent epoch information
    rent_epoch: u64,
    #[allow(dead_code)] // Account space, always 200 for stake accounts
    space: u64,
}

/// Parsed stake account data
#[derive(Debug, Deserialize)]
struct RawParsedData {
    parsed: RawParsedInfo,
    #[allow(dead_code)] // Program type, expected to be "stake"
    program: String,
    #[allow(dead_code)] // Data space, same as account space
    space: u64,
}

/// Parsed stake account info
#[derive(Debug, Deserialize)]
struct RawParsedInfo {
    info: RawStakeInfo,
    #[serde(rename = "type")]
    #[allow(dead_code)] // Stake type, expected to be "delegated"
    stake_type: String,
}

/// Raw stake info from blockchain
#[derive(Debug, Deserialize)]
struct RawStakeInfo {
    meta: RawStakeMeta,
    stake: Option<RawStakeData>,
}

/// Raw stake metadata
#[derive(Debug, Deserialize)]
struct RawStakeMeta {
    authorized: RawStakeAuthorized,
    lockup: RawStakeLockup,
    #[serde(rename = "rentExemptReserve")]
    rent_exempt_reserve: String, // String because it comes as string from RPC
}

/// Raw stake authorization info
#[derive(Debug, Deserialize)]
struct RawStakeAuthorized {
    staker: String,
    withdrawer: String,
}

/// Raw stake lockup info
#[derive(Debug, Deserialize)]
struct RawStakeLockup {
    custodian: String,
    epoch: u64,
    #[serde(rename = "unixTimestamp")]
    unix_timestamp: u64,
}

/// Raw delegation info
#[derive(Debug, Deserialize)]
struct RawStakeData {
    #[serde(rename = "creditsObserved")]
    credits_observed: u64,
    delegation: RawDelegation,
}

/// Raw delegation data
#[derive(Debug, Deserialize)]
struct RawDelegation {
    #[serde(rename = "activationEpoch")]
    activation_epoch: String, // String because it can be very large numbers
    #[serde(rename = "deactivationEpoch")]
    deactivation_epoch: String,
    stake: String, // String because it's a large number
    voter: String,
    #[serde(rename = "warmupCooldownRate")]
    warmup_cooldown_rate: f64,
}

/// Internal RPC client for making requests
pub struct RpcClient {
    client: reqwest::Client,
    url: String,
    request_id: std::sync::atomic::AtomicU64,
}

impl Clone for RpcClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            url: self.url.clone(),
            request_id: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl RpcClient {
    /// Create a new RPC client
    pub fn new(url: String, timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("pools-data-lib/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            url,
            request_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Get next request ID
    fn next_request_id(&self) -> u64 {
        self.request_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Fetch stake accounts for a specific pool authority
    pub async fn fetch_stake_accounts_for_authority(&self, authority: &str) -> Result<Vec<StakeAccountInfo>> {
        let request_id = self.next_request_id();
        let request = RpcRequest::get_program_accounts_stake(request_id, authority);

        log::debug!("Sending RPC request for authority: {}", authority);

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        // Check for HTTP errors
        if !response.status().is_success() {
            return Err(PoolsDataError::NetworkError {
                message: format!("HTTP error: {}", response.status()),
            });
        }

        let response_text = response.text().await?;

        // Try to parse as RPC response
        let rpc_response: RpcResponse<Vec<RawStakeAccount>> = serde_json::from_str(&response_text)
            .map_err(|e| PoolsDataError::ParseError {
                message: format!("Failed to parse RPC response: {e}"),
            })?;

        // Validate RPC response format
        self.validate_rpc_response(&rpc_response, request_id)?;

        // Check for RPC errors
        if let Some(error) = rpc_response.error {
            // Validate the error structure before using it
            if let Err(validation_error) = self.validate_rpc_error(&error) {
                eprintln!("Warning: RPC error validation failed: {validation_error}");
            }
            
            return Err(PoolsDataError::RpcError {
                code: error.code,
                message: error.message,
            });
        }

        let raw_accounts = rpc_response.result.ok_or_else(|| PoolsDataError::ParseError {
            message: "Missing result in RPC response".to_string(),
        })?;

        log::debug!("Received {} stake accounts for authority: {}", raw_accounts.len(), authority);

        // Convert raw accounts to our types
        let mut stake_accounts = Vec::new();
        for raw_account in raw_accounts {
            let pubkey = raw_account.pubkey.clone(); // Clone before moving
            match self.parse_stake_account(raw_account) {
                Ok(stake_account) => stake_accounts.push(stake_account),
                Err(e) => {
                    log::warn!("Failed to parse stake account {}: {}", pubkey, e);
                    // Continue processing other accounts instead of failing completely
                }
            }
        }

        Ok(stake_accounts)
    }

    /// Parse raw stake account data into our types
    fn parse_stake_account(&self, raw: RawStakeAccount) -> Result<StakeAccountInfo> {
        // Validate that this is actually a stake account
        self.validate_stake_account(&raw)?;

        let rent_exempt_reserve = raw.account.data.parsed.info.meta.rent_exempt_reserve
            .parse::<u64>()
            .map_err(|e| PoolsDataError::InvalidStakeData {
                message: format!("Invalid rent exempt reserve: {e}"),
            })?;

        let authorized = StakeAuthorized {
            staker: raw.account.data.parsed.info.meta.authorized.staker,
            withdrawer: raw.account.data.parsed.info.meta.authorized.withdrawer,
        };

        let lockup = StakeLockup {
            custodian: raw.account.data.parsed.info.meta.lockup.custodian,
            epoch: raw.account.data.parsed.info.meta.lockup.epoch,
            #[allow(clippy::cast_possible_wrap)] // Unix timestamps are typically positive and fit in i64
            unix_timestamp: raw.account.data.parsed.info.meta.lockup.unix_timestamp as i64,
        };

        let delegation = if let Some(stake_data) = raw.account.data.parsed.info.stake {
            Some(self.parse_delegation(stake_data)?)
        } else {
            None
        };

        Ok(StakeAccountInfo {
            pubkey: raw.pubkey,
            lamports: raw.account.lamports,
            rent_exempt_reserve,
            delegation,
            authorized,
            lockup,
        })
    }

    /// Validate that the raw account is a proper stake account
    fn validate_stake_account(&self, raw: &RawStakeAccount) -> Result<()> {
        // Validate owner is stake program
        if raw.account.owner != "Stake11111111111111111111111111111111111111" {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!("Account owner is not stake program: {}", raw.account.owner),
            });
        }

        // Validate account is not executable
        if raw.account.executable {
            return Err(PoolsDataError::InvalidStakeData {
                message: "Stake account should not be executable".to_string(),
            });
        }

        // Validate account space (stake accounts are always 200 bytes)
        if raw.account.space != 200 {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!("Invalid stake account space: {} (expected 200)", raw.account.space),
            });
        }

        // Validate program type
        if raw.account.data.program != "stake" {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!("Invalid program type: {} (expected 'stake')", raw.account.data.program),
            });
        }

        // Validate stake type for delegated accounts
        if raw.account.data.parsed.info.stake.is_some() && raw.account.data.parsed.stake_type != "delegated" {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!("Invalid stake type: {} (expected 'delegated')", raw.account.data.parsed.stake_type),
            });
        }

        Ok(())
    }

    /// Parse delegation data
    fn parse_delegation(&self, raw: RawStakeData) -> Result<StakeDelegation> {
        let stake = raw.delegation.stake
            .parse::<u64>()
            .map_err(|e| PoolsDataError::InvalidStakeData {
                message: format!("Invalid stake amount: {e}"),
            })?;

        let activation_epoch = raw.delegation.activation_epoch
            .parse::<u64>()
            .map_err(|e| PoolsDataError::InvalidStakeData {
                message: format!("Invalid activation epoch: {e}"),
            })?;

        let deactivation_epoch = raw.delegation.deactivation_epoch
            .parse::<u64>()
            .map_err(|e| PoolsDataError::InvalidStakeData {
                message: format!("Invalid deactivation epoch: {e}"),
            })?;

        Ok(StakeDelegation {
            voter: raw.delegation.voter,
            stake,
            activation_epoch,
            deactivation_epoch,
            credits_observed: raw.credits_observed,
            warmup_cooldown_rate: raw.delegation.warmup_cooldown_rate,
        })
    }

    /// Test RPC connection
    pub async fn test_connection(&self) -> Result<()> {
        let request = RpcRequest::new(1, "getHealth", json!([]));

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PoolsDataError::NetworkError {
                message: format!("Health check failed: {}", response.status()),
            });
        }

        let response_text = response.text().await?;
        let rpc_response: RpcResponse<String> = serde_json::from_str(&response_text)?;

        if let Some(error) = rpc_response.error {
            // Validate the error structure before using it
            if let Err(validation_error) = self.validate_rpc_error(&error) {
                eprintln!("Warning: RPC error validation failed: {validation_error}");
            }
            
            return Err(PoolsDataError::RpcError {
                code: error.code,
                message: error.message,
            });
        }

        log::debug!("RPC connection test successful");
        Ok(())
    }

    /// Validate RPC response format and content
    fn validate_rpc_response<T>(&self, response: &RpcResponse<T>, expected_id: u64) -> Result<()> {
        // Validate JSON-RPC version
        if response.jsonrpc != "2.0" {
            return Err(PoolsDataError::RpcError {
                code: -32600,
                message: format!("Invalid JSON-RPC version: {} (expected '2.0')", response.jsonrpc),
            });
        }

        // Validate response ID matches request ID
        if response.id != expected_id {
            return Err(PoolsDataError::RpcError {
                code: -32603,
                message: format!("Response ID mismatch: {} (expected {})", response.id, expected_id),
            });
        }

        Ok(())
    }

    /// Validate RPC error structure and content
    fn validate_rpc_error(&self, error: &RpcError) -> Result<()> {
        // Validate error code is within expected ranges
        // Standard JSON-RPC error codes: -32768 to -32000 are reserved
        // Solana-specific codes are negative but outside this range
        if error.code == 0 {
            return Err(PoolsDataError::ParseError {
                message: "Invalid RPC error code: 0 is not a valid error code".to_string(),
            });
        }

        // Validate error message is not empty
        if error.message.trim().is_empty() {
            return Err(PoolsDataError::ParseError {
                message: "Invalid RPC error: empty message".to_string(),
            });
        }

        // If data field is present, it should be valid JSON
        if let Some(data) = &error.data {
            // Basic validation that data is a valid JSON value
            // The data field is already parsed as serde_json::Value, so it's valid JSON
            log::debug!("RPC error includes additional data: {}", data);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_request_creation() {
        let request = RpcRequest::get_program_accounts_stake(1, "test_authority");
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, 1);
        assert_eq!(request.method, "getProgramAccounts");
    }

    #[test]
    fn test_delegation_parsing() {
        let client = RpcClient::new("http://test".to_string(), Duration::from_secs(30));
        
        let raw_stake_data = RawStakeData {
            credits_observed: 1000,
            delegation: RawDelegation {
                activation_epoch: "100".to_string(),
                deactivation_epoch: "18446744073709551615".to_string(),
                stake: "5000000000".to_string(),
                voter: "validator123".to_string(),
                warmup_cooldown_rate: 0.25,
            },
        };

        let delegation = client.parse_delegation(raw_stake_data).unwrap();
        
        assert_eq!(delegation.voter, "validator123");
        assert_eq!(delegation.stake, 5000000000);
        assert_eq!(delegation.activation_epoch, 100);
        assert_eq!(delegation.deactivation_epoch, 18446744073709551615);
        assert_eq!(delegation.deactivation_epoch, u64::MAX); // Active delegation
    }

    // Note: Integration tests that require actual RPC calls should be in a separate file
    // and marked with #[ignore] or run only in CI with real endpoints
}