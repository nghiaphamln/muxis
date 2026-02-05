use std::io;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

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

#[derive(Debug, Error)]
#[error("encode error: {source}")]
pub struct EncodeError {
    source: io::Error,
}

impl EncodeError {
    pub fn new(source: io::Error) -> Self {
        Self { source }
    }
}

#[derive(Debug, Error)]
#[error("decode error: {source}")]
pub struct DecodeError {
    source: io::Error,
}

impl DecodeError {
    pub fn new(source: io::Error) -> Self {
        Self { source }
    }
}
