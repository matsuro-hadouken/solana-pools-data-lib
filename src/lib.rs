//! # Pools Data Library
//!
//! A Rust library for fetching Solana stake pool data from the blockchain.
//! This library provides a clean, configurable interface for retrieving stake account
//! information from various liquid staking pools.
//!
//! ## Features
//!
//! - Configurable rate limiting for different RPC providers
//! - Retry logic with exponential backoff
//! - Partial success handling - get what works, retry what fails
//! - Granular error handling for better debugging
//! - Support for all major Solana stake pools
//! - No caching - pure data fetching library
//!
//! ## Example
//!
//! ```rust,no_run
//! use pools_data_lib::PoolsDataClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = PoolsDataClient::builder()
//!         .rate_limit(5) // 5 requests per second
//!         .build("https://api.mainnet-beta.solana.com")
//!         .and_then(PoolsDataClient::from_config)?;
//!
//!     let result = client.fetch_pools(&["jito", "marinade"]).await?;
//!     println!("Fetched {} pools successfully", result.successful.len());
//!     
//!     Ok(())
//! }
//! ```

mod client;
mod config;
mod error;
mod types;
mod pools;
mod rpc;

pub use client::*;
pub use config::*;
pub use error::*;
pub use types::*;
pub use pools::*;

// Re-export commonly used types
pub use tokio;
pub use serde_json;