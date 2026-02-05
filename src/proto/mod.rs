//! # Muxis Proto
//!
//! RESP (Redis Serialization Protocol) codec implementation for Rust.
//! Provides encoding and decoding of Redis protocol messages.
//!
//! ## Features
//!
//! - `resp3` - Enable RESP3 protocol support (default: disabled)
//!
//! ## Modules
//!
//! - [`codec`] - Encoder and decoder for RESP protocol
//! - [`error`] - Error types for protocol operations
//! - [`frame`] - Frame types representing RESP data structures

#![warn(missing_docs)]

pub mod codec;
/// Error types.
pub mod error;
pub mod frame;
