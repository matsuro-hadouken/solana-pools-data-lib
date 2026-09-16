//! RPC client for Solana blockchain communication.
//!
//! This module handles the low-level RPC communication with Solana nodes,
//! including request formatting, response parsing, and error handling.

use crate::error::{PoolsDataError, Result};
use crate::types::{StakeAccountInfo, StakeAuthorized, StakeDelegation, StakeLockup};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;

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
    jsonrpc: String, // Used for validation
    id: u64,         // Used for request/response matching
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
    // Never read, and deprecated upstream. Requiring it means the day a node
    // stops emitting it, every pool fails at once — the whole account array
    // decodes in a single pass. That is the warmupCooldownRate outage
    // (2026-08-03) repeating with a different field name, so default it.
    #[serde(rename = "rentEpoch", default)]
    #[allow(dead_code)] // Rent epoch information
    rent_epoch: u64,
    // Defaults to None rather than 0. A 0 default did NOT achieve what an
    // earlier comment here claimed: it still failed `space != 200`, so every
    // pool still failed — just as a non-retryable InvalidStakeData blaming the
    // on-chain account, instead of a retryable ParseError. Absent and wrong are
    // different things, so model them differently: an omitted field skips the
    // size check (the memcmp already restricts results to the stake program),
    // while a present-but-wrong value still fails.
    #[serde(default)]
    space: Option<u64>,
}

/// Parsed stake account data
#[derive(Debug, Deserialize)]
struct RawParsedData {
    parsed: RawParsedInfo,
    #[allow(dead_code)] // Program type, expected to be "stake"
    program: String,
    // Never read — it duplicates the outer account.space. Requiring a field
    // nothing consumes is how one node-side change takes down every pool.
    #[serde(default)]
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
    // Deprecated upstream (Meta::rent_exempt_reserve, since 3.0.1). It is carried
    // through to StakeAccountInfo but never used in any statistic, so defaulting
    // to "0" costs nothing and stops a node-side removal failing every pool.
    #[serde(rename = "rentExemptReserve", default = "zero_string")]
    rent_exempt_reserve: String, // String because it comes as string from RPC
}

fn zero_string() -> String {
    "0".to_string()
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
    // i64, not u64: Solana's UnixTimestamp is signed and the runtime does not
    // require it to be positive. Typing it unsigned meant one account with a
    // negative lockup failed to deserialize, and since the whole account array
    // decodes at once, the ENTIRE pool returned ParseError. That was a cheap
    // denial of service — anyone could create a 200-byte stake account naming a
    // victim pool's authority as staker (keeping themselves as withdrawer, so
    // the pool could never remove it) with a negative lockup, and every fetch of
    // that pool would fail from then on.
    #[serde(rename = "unixTimestamp")]
    unix_timestamp: i64,
}

/// Raw delegation info
#[derive(Debug, Deserialize)]
struct RawStakeData {
    #[serde(rename = "creditsObserved")]
    last_epoch_credits_cumulative: u64,
    delegation: RawDelegation,
}

/// Raw delegation data
#[derive(Debug, Deserialize)]
struct RawDelegation {
    #[serde(rename = "activationEpoch")]
    activation_epoch: String, // String because values can exceed standard integer limits
    #[serde(rename = "deactivationEpoch")]
    deactivation_epoch: String,
    stake: String, // String because values can exceed standard integer limits
    voter: String,
    // Deprecated on-chain; newer Agave nodes omit it from jsonParsed
    // responses while older nodes still send it. Must stay optional.
    #[serde(
        rename = "warmupCooldownRate",
        default = "default_warmup_cooldown_rate"
    )]
    warmup_cooldown_rate: f64,
}

/// Ceiling for any single account balance or delegated amount.
///
/// Total SOL supply is roughly 6e8 SOL = 6e17 lamports; this is 1e18, an order
/// of magnitude of headroom above anything that can exist while still being far
/// below u64::MAX (1.8e19). Its purpose is to catch a response that is wrong,
/// not to cap a pool that is merely large.
const MAX_PLAUSIBLE_LAMPORTS: u64 = 1_000_000_000_000_000_000;

fn default_warmup_cooldown_rate() -> f64 {
    0.25
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
            // reqwest follows up to 10 redirects by default, which silently
            // multiplies requests below the layer that counts them: one pool's
            // query becomes 11 HTTP requests (measured), so a 294-pool refresh
            // becomes 3,234 before retries compound it. A JSON-RPC POST endpoint
            // has no legitimate reason to redirect, so surface it as an error
            // rather than paying for it.
            .redirect(reqwest::redirect::Policy::none())
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
        self.request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Fetch stake accounts for a specific pool authority
    pub async fn fetch_stake_accounts_for_authority(
        &self,
        authority: &str,
    ) -> Result<Vec<StakeAccountInfo>> {
        let request_id = self.next_request_id();
        let request = RpcRequest::get_program_accounts_stake(request_id, authority);

        log::debug!("Sending RPC request for authority: {authority}");

        let response = self
            .client
            .post(&self.url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        // Check for HTTP errors
        if response.status().is_redirection() {
            // Redirects are not followed (see the client builder). A 3xx means
            // the configured URL is not canonical, which no amount of retrying
            // fixes — classify it as configuration so the retry loop stops at
            // the first attempt instead of spending the whole budget per pool.
            return Err(PoolsDataError::ConfigurationError {
                message: format!(
                    "RPC endpoint redirected ({}) to {:?}; configure the canonical URL directly. \
                     Redirects are not followed because they multiply every request.",
                    response.status(),
                    response
                        .headers()
                        .get(reqwest::header::LOCATION)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("<no Location header>")
                ),
            });
        }
        // 4xx is the client's fault and retrying cannot fix it: a bad API key
        // (401/403), a wrong path (404), a malformed body (400). Classifying it
        // as NetworkError made it retryable, so a deterministic permanent
        // failure cost 1+retry_attempts requests per pool — the same
        // amplification already closed for 3xx and RPC -32602. 429 is the
        // exception: it is explicitly a "come back later".
        if response.status().is_client_error()
            && response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            return Err(PoolsDataError::ConfigurationError {
                message: format!(
                    "RPC endpoint returned {}; check the URL and credentials. Not retried.",
                    response.status()
                ),
            });
        }
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
        Self::validate_rpc_response(&rpc_response, request_id)?;

        // Check for RPC errors
        if let Some(error) = rpc_response.error {
            // Validate the error structure before using it
            if let Err(validation_error) = Self::validate_rpc_error(&error) {
                eprintln!("Warning: RPC error validation failed: {validation_error}");
            }

            return Err(PoolsDataError::RpcError {
                code: error.code,
                message: error.message,
            });
        }

        let raw_accounts = rpc_response
            .result
            .ok_or_else(|| PoolsDataError::ParseError {
                message: "Missing result in RPC response".to_string(),
            })?;

        log::debug!(
            "Received {} stake accounts for authority: {}",
            raw_accounts.len(),
            authority
        );

        // Convert raw accounts to our types
        let mut stake_accounts = Vec::new();
        for raw_account in raw_accounts {
            let pubkey = raw_account.pubkey.clone(); // Clone before moving
            match Self::parse_stake_account(raw_account) {
                Ok(stake_account) => stake_accounts.push(stake_account),
                Err(e) => {
                    // Skipping used to be a warn-and-continue, which let a pool
                    // succeed with understated stake — even under
                    // fetch_pools_strict. A silently wrong balance is worse than
                    // an absent row.
                    //
                    // In practice this arm is close to unreachable: every
                    // remaining check (owner, executable, space, program type,
                    // stake type, the three string->u64 parses) is satisfied by
                    // definition for an account a getProgramAccounts query on
                    // the stake program returned, and a live fetch of all 294
                    // pools produced zero hits. The real within-pool guarantee
                    // comes one step earlier, from decoding the whole array at
                    // once. This is the backstop for a shape that slips past
                    // that, not the primary defence.
                    log::error!("stake account {pubkey} failed to parse: {e}");
                    return Err(PoolsDataError::InvalidStakeData {
                        message: format!(
                            "stake account {pubkey} for authority {authority} did not parse ({e}); \
                             refusing to report this pool with an understated balance"
                        ),
                    });
                }
            }
        }

        Ok(stake_accounts)
    }

    /// Parse raw stake account data into our types
    fn parse_stake_account(raw: RawStakeAccount) -> Result<StakeAccountInfo> {
        // Validate that this is actually a stake account
        Self::validate_stake_account(&raw)?;

        // Reject balances that cannot exist before they reach any accumulator.
        // The totals are u64 sums; a release build wraps silently, so an
        // out-of-range value from a lying or buggy RPC would be returned as a
        // successful pool with corrupted figures — measured: u64::MAX + 10 came
        // back as total_lamports = 9, Ok. Total SOL supply is ~6e8 (6e17
        // lamports), so anything past MAX_PLAUSIBLE_LAMPORTS is impossible on
        // chain and means the response is wrong, not that the pool is enormous.
        if raw.account.lamports > MAX_PLAUSIBLE_LAMPORTS {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!(
                    "stake account {} reports {} lamports, which exceeds the total SOL supply; \
                     refusing to fold an impossible balance into pool totals",
                    raw.pubkey, raw.account.lamports
                ),
            });
        }

        let rent_exempt_reserve = raw
            .account
            .data
            .parsed
            .info
            .meta
            .rent_exempt_reserve
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
            // No cast: the field is already i64, matching Solana's UnixTimestamp.
            unix_timestamp: raw.account.data.parsed.info.meta.lockup.unix_timestamp,
        };

        let delegation = if let Some(stake_data) = raw.account.data.parsed.info.stake {
            Some(Self::parse_delegation(stake_data)?)
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
    fn validate_stake_account(raw: &RawStakeAccount) -> Result<()> {
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
        if raw.account.space.is_some_and(|s| s != 200) {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!(
                    "Invalid stake account space: {} (expected 200)",
                    raw.account.space.unwrap_or_default()
                ),
            });
        }

        // Validate program type
        if raw.account.data.program != "stake" {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!(
                    "Invalid program type: {} (expected 'stake')",
                    raw.account.data.program
                ),
            });
        }

        // Validate stake type for delegated accounts
        if raw.account.data.parsed.info.stake.is_some()
            && raw.account.data.parsed.stake_type != "delegated"
        {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!(
                    "Invalid stake type: {} (expected 'delegated')",
                    raw.account.data.parsed.stake_type
                ),
            });
        }

        Ok(())
    }

    /// Parse delegation data
    fn parse_delegation(raw: RawStakeData) -> Result<StakeDelegation> {
        let stake =
            raw.delegation
                .stake
                .parse::<u64>()
                .map_err(|e| PoolsDataError::InvalidStakeData {
                    message: format!("Invalid stake amount: {e}"),
                })?;

        // Same ceiling as account lamports: a delegated amount past the total
        // SOL supply means the response is wrong, and these values are summed
        // into u64 totals that wrap silently in release builds.
        if stake > MAX_PLAUSIBLE_LAMPORTS {
            return Err(PoolsDataError::InvalidStakeData {
                message: format!(
                    "delegation reports {stake} lamports staked, which exceeds the total SOL \
                     supply; refusing to fold an impossible amount into pool totals"
                ),
            });
        }

        let activation_epoch = raw
            .delegation
            .activation_epoch
            .parse::<u64>()
            .map_err(|e| PoolsDataError::InvalidStakeData {
                message: format!("Invalid activation epoch: {e}"),
            })?;

        let deactivation_epoch = raw
            .delegation
            .deactivation_epoch
            .parse::<u64>()
            .map_err(|e| PoolsDataError::InvalidStakeData {
                message: format!("Invalid deactivation epoch: {e}"),
            })?;

        Ok(StakeDelegation {
            voter: raw.delegation.voter,
            stake,
            activation_epoch,
            deactivation_epoch,
            last_epoch_credits_cumulative: raw.last_epoch_credits_cumulative,
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

        // Same classification as the fetch path: a 3xx means the configured URL
        // is not canonical, which retrying never fixes. Leaving it as a retryable
        // NetworkError here would have a caller consulting is_retryable() retry a
        // permanent 308 that normal fetches correctly stop on.
        if response.status().is_redirection() {
            return Err(PoolsDataError::ConfigurationError {
                message: format!(
                    "RPC endpoint redirected ({}) during health check; configure the canonical URL",
                    response.status()
                ),
            });
        }
        if !response.status().is_success() {
            return Err(PoolsDataError::NetworkError {
                message: format!("Health check failed: {}", response.status()),
            });
        }

        let response_text = response.text().await?;
        let rpc_response: RpcResponse<String> = serde_json::from_str(&response_text)?;

        if let Some(error) = rpc_response.error {
            // Validate the error structure before using it
            if let Err(validation_error) = Self::validate_rpc_error(&error) {
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
    fn validate_rpc_response<T>(response: &RpcResponse<T>, expected_id: u64) -> Result<()> {
        // Validate JSON-RPC version
        if response.jsonrpc != "2.0" {
            return Err(PoolsDataError::RpcError {
                code: -32600,
                message: format!(
                    "Invalid JSON-RPC version: {} (expected '2.0')",
                    response.jsonrpc
                ),
            });
        }

        // Validate response ID matches request ID
        if response.id != expected_id {
            return Err(PoolsDataError::RpcError {
                code: -32603,
                message: format!(
                    "Response ID mismatch: {} (expected {})",
                    response.id, expected_id
                ),
            });
        }

        Ok(())
    }

    /// Validate RPC error structure and content
    fn validate_rpc_error(error: &RpcError) -> Result<()> {
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
            // The data field is already parsed as serde_json::Value, ensuring valid JSON
            log::debug!("RPC error includes additional data: {data}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_delegation_parses_without_warmup_cooldown_rate() {
        // Newer Agave nodes omit the deprecated warmupCooldownRate field from
        // jsonParsed stake responses; older nodes still send it. Both shapes
        // must parse (regression: 2026-08-03 pools-daemon outage).
        let omitted: RawDelegation = serde_json::from_str(
            r#"{"activationEpoch":"862","deactivationEpoch":"18446744073709551615","stake":"1000000000","voter":"validator123"}"#,
        )
        .expect("delegation without warmupCooldownRate must parse");
        assert!((omitted.warmup_cooldown_rate - 0.25).abs() < f64::EPSILON);

        let present: RawDelegation = serde_json::from_str(
            r#"{"activationEpoch":"862","deactivationEpoch":"0","stake":"1","voter":"validator123","warmupCooldownRate":0.09}"#,
        )
        .expect("delegation with warmupCooldownRate must parse");
        assert!((present.warmup_cooldown_rate - 0.09).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rpc_request_creation() {
        let request = RpcRequest::get_program_accounts_stake(1, "test_authority");

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, 1);
        assert_eq!(request.method, "getProgramAccounts");
    }

    #[test]
    fn test_delegation_parsing() {
        let _client = RpcClient::new("http://test".to_string(), Duration::from_secs(30));

        let raw_stake_data = RawStakeData {
            last_epoch_credits_cumulative: 1000,
            delegation: RawDelegation {
                activation_epoch: "100".to_string(),
                deactivation_epoch: "18446744073709551615".to_string(),
                stake: "5000000000".to_string(),
                voter: "validator123".to_string(),
                warmup_cooldown_rate: 0.25,
            },
        };

        let delegation = RpcClient::parse_delegation(raw_stake_data).unwrap();

        assert_eq!(delegation.voter, "validator123");
        assert_eq!(delegation.stake, 5000000000);
        assert_eq!(delegation.activation_epoch, 100);
        assert_eq!(delegation.deactivation_epoch, 18446744073709551615);
        assert_eq!(delegation.deactivation_epoch, u64::MAX); // Active delegation
    }

    // Note: Integration tests that require actual RPC calls should be in a separate file
    // and marked with #[ignore] or run only in CI with real endpoints
}
