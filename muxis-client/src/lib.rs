//! # Muxis Client
//!
//! High-performance Redis client library for Rust with multiplexing,
//! automatic standalone/cluster detection, and full feature coverage.
//!
//! ## Features
//!
//! - `tls` - TLS/SSL support via native-tls
//! - `resp3` - RESP3 protocol support
//! - `cluster` - Cluster mode support
//! - `json` - RedisJSON commands
//! - `streams` - Redis Streams commands
//! - `tracing` - OpenTelemetry tracing support
//!
//! ## Example
//!
//! ```rust,ignore
//! use muxis_client::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::connect("redis://localhost:6379").await?;
//!     let _ = client.ping().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Crate Architecture
//!
//! This crate re-exports types from the following crates:
//! - [`muxis_core`] - Core connection handling
//! - [`muxis_cluster`] - Cluster support
//! - [`muxis_proto`] - Protocol codec

pub use muxis_core::{Client, ClientPool};
