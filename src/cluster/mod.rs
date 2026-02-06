//! # Muxis Cluster
//!
//! Redis Cluster support with automatic slot-based routing, topology discovery,
//! and redirect handling (MOVED/ASK).
//!
//! ## Features
//!
//! - **Slot-based routing**: Automatically routes commands to correct nodes
//! - **Topology discovery**: Uses CLUSTER SLOTS to map slots to nodes
//! - **Redirect handling**: Handles MOVED and ASK redirects transparently
//! - **Connection pooling**: Maintains connections to all cluster nodes
//! - **Hash tags**: Supports Redis hash tags `{...}` for multi-key operations
//!
//! ## Usage
//!
//! Enable the `cluster` feature in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! muxis = { version = "0.4", features = ["cluster"] }
//! ```
//!
//! ## Example
//!
//! ```no_run
//! # #[cfg(feature = "cluster")]
//! # async fn example() -> muxis::Result<()> {
//! use muxis::ClusterClient;
//! use bytes::Bytes;
//!
//! // Connect to Redis Cluster (comma-separated seed nodes)
//! let client = ClusterClient::connect("127.0.0.1:7000,127.0.0.1:7001").await?;
//!
//! // Commands are automatically routed to correct node
//! client.set("key", Bytes::from("value")).await?;
//! let value = client.get("key").await?;
//! # Ok(())
//! # }
//! ```

mod client;
pub mod commands;
mod errors;
mod pool;
mod slot;
mod topology;

pub use client::ClusterClient;
pub use slot::key_slot;
