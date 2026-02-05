//! RESP frame types.
//!
//! This module defines the frame types used in the Redis protocol,
//! including simple strings, errors, integers, bulk strings, and arrays.

/// Frame type definitions.
pub mod types;

pub use types::Frame;
