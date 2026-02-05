//! RESP protocol encoder and decoder.
//!
//! This module provides encoding and decoding functionality for Redis
//! Serialization Protocol (RESP) frames.
//!
//! # Modules
//!
//! - [`encoder`] - Frame encoding to bytes
//! - [`decoder`] - Streaming frame decoder from bytes

/// Streaming frame decoder.
pub mod decoder;
/// Frame encoder.
pub mod encoder;

pub use decoder::Decoder;
pub use encoder::{encode_frame, Encoder};
