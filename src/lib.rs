//! # Pools Data Library
//!
//! Rust library for fetching Solana stake pool data.
//!
//! ## Two Output Formats
//!
//! - **Production**: Data with static fields removed (suitable for databases)
//! - **Debug**: Complete RPC data with ALL fields (for debugging)
//!
//! ## Example
//!
//! ```rust,no_run
//! use solana_pools_data_lib::PoolsDataClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = PoolsDataClient::builder()
//!         .rate_limit(10)
//!         .build("https://api.mainnet-beta.solana.com")
//!         .and_then(PoolsDataClient::from_config)?;
//!
//!     // Production use - optimized format
//!     let production_data = client.fetch_pools(&["jito", "marinade"]).await?;
//!     
//!     // Debug use - complete RPC data
//!     let debug_data = client.fetch_pools_debug(&["jito"]).await?;
//!     
//!     Ok(())
//! }
//! ```

mod client;
mod config;
mod error;
mod pools;
mod rpc;
mod types;
pub mod statistics;
pub mod statistics_calc;

#[cfg(test)]
mod statistics_calc_tests;

pub use client::*;
pub use config::*;
pub use error::*;
pub use pools::*;
pub use types::*;

// Re-export commonly used types
pub use serde_json;
pub use tokio;
