//! # Muxis Cluster
//!
//! Cluster support for Redis, including automatic topology discovery,
//! slot mapping, and redirect handling.
//!
//! ## Features
//!
//! - `cluster` - Enable cluster mode support (default: disabled)
//!
//! ## Modules
//!
//! - [`cluster`] - Cluster connection and management
//! - [`slot_map`] - Slot-based routing logic

/// Cluster implementation.
pub mod cluster;
/// Slot map for routing.
pub mod slot_map;
