//! # Muxis
//!
//! High-performance Redis client library for Rust with multiplexing,
//! automatic standalone/cluster detection, and full feature coverage.
//!
//! ## Features
//!
//! - `tls` - TLS/SSL support
//! - `resp3` - RESP3 protocol support
//! - `cluster` - Cluster mode support
//! - `json` - RedisJSON commands
//! - `streams` - Redis Streams commands
//! - `tracing` - Observability
//!
//! ## Example
//!
//! ```no_run
//! use muxis::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = Client::connect("redis://localhost:6379").await?;
//!     let _ = client.ping().await?;
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

pub(crate) mod core;
pub(crate) mod proto;

#[cfg(feature = "cluster")]
pub(crate) mod cluster;

#[cfg(test)]
mod stress;

#[cfg(feature = "test-utils")]
pub mod testing;

// Re-export high-level client types for convenience
pub use crate::core::builder::ClientBuilder;
pub use crate::core::{Client, Error, Result};

#[cfg(feature = "cluster")]
pub use crate::cluster::key_slot;
#[cfg(feature = "cluster")]
pub use crate::cluster::ClusterClient;
