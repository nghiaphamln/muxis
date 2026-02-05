use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when interacting with Redis protocol.
#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: io::Error,
    },

    #[error("protocol error: {message}")]
    Protocol { message: String },

    #[error("server error: {message}")]
    Server { message: String },

    #[error("authentication failed")]
    Auth,

    #[error("invalid argument: {message}")]
    InvalidArgument { message: String },

    #[error("encode error: {source}")]
    Encode {
        #[from]
        source: EncodeError,
    },

    #[error("decode error: {source}")]
    Decode {
        #[from]
        source: DecodeError,
    },
}

/// Error returned when frame encoding fails.
#[derive(Debug, Error)]
#[error("encode error: {source}")]
pub struct EncodeError {
    source: io::Error,
}

impl EncodeError {
    /// Creates a new encode error from an IO error.
    pub fn new(source: io::Error) -> Self {
        Self { source }
    }
}

/// Error returned when frame decoding fails.
#[derive(Debug, Error)]
#[error("decode error: {source}")]
pub struct DecodeError {
    source: io::Error,
}

impl DecodeError {
    /// Creates a new decode error from an IO error.
    pub fn new(source: io::Error) -> Self {
        Self { source }
    }
}
