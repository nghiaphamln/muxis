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
//!
//! // Connect to Redis Cluster
//! let mut client = ClusterClient::connect(&["127.0.0.1:7000"]).await?;
//!
//! // Commands are automatically routed to correct node
//! client.set("key", "value").await?;
//! let value = client.get("key").await?;
//! # Ok(())
//! # }
//! ```

pub mod commands;
mod errors;
mod slot;
mod topology;

pub use errors::parse_redis_error;
pub use slot::{key_slot, SLOT_COUNT};
pub use topology::{ClusterTopology, NodeFlags, NodeId, NodeInfo, SlotRange};
