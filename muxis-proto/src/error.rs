use std::io;

use thiserror::Error;

/// Result type alias for muxis operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when interacting with Redis protocol.
#[derive(Debug, Error)]
pub enum Error {
    /// An IO error occurred.
    #[error("IO error: {source}")]
    Io {
        /// The underlying IO error.
        #[from]
        source: io::Error,
    },

    /// A protocol error occurred.
    #[error("protocol error: {message}")]
    Protocol {
        /// Description of the error.
        message: String,
    },

    /// The server returned an error.
    #[error("server error: {message}")]
    Server {
        /// Error message from server.
        message: String,
    },

    /// Authentication failed.
    #[error("authentication failed")]
    Auth,

    /// Invalid argument provided.
    #[error("invalid argument: {message}")]
    InvalidArgument {
        /// Description of invalid argument.
        message: String,
    },

    /// Encoding failed.
    #[error("encode error: {source}")]
    Encode {
        /// Underlying encode error.
        #[from]
        source: EncodeError,
    },

    /// Decoding failed.
    #[error("decode error: {source}")]
    Decode {
        /// Underlying decode error.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_io() {
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "connection refused");
        let error = Error::Io { source: io_err };
        assert!(error.to_string().contains("IO error"));
    }

    #[test]
    fn test_error_display_protocol() {
        let error = Error::Protocol {
            message: "invalid frame".to_string(),
        };
        assert_eq!(error.to_string(), "protocol error: invalid frame");
    }

    #[test]
    fn test_error_display_server() {
        let error = Error::Server {
            message: "ERR wrong type".to_string(),
        };
        assert_eq!(error.to_string(), "server error: ERR wrong type");
    }

    #[test]
    fn test_error_display_auth() {
        let error = Error::Auth;
        assert_eq!(error.to_string(), "authentication failed");
    }

    #[test]
    fn test_error_display_invalid_argument() {
        let error = Error::InvalidArgument {
            message: "missing required field".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "invalid argument: missing required field"
        );
    }

    #[test]
    fn test_encode_error_new() {
        let io_err = io::Error::new(io::ErrorKind::Other, "encode failed");
        let enc_error = EncodeError::new(io_err);
        assert!(enc_error.to_string().contains("encode error"));
    }

    #[test]
    fn test_decode_error_new() {
        let io_err = io::Error::new(io::ErrorKind::Other, "decode failed");
        let dec_error = DecodeError::new(io_err);
        assert!(dec_error.to_string().contains("decode error"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::Other, "test");
        let error: Error = io_err.into();
        assert!(matches!(error, Error::Io { .. }));
    }

    #[test]
    fn test_error_from_encode() {
        let enc_err = EncodeError::new(io::Error::new(io::ErrorKind::Other, "test"));
        let error: Error = enc_err.into();
        assert!(matches!(error, Error::Encode { .. }));
    }

    #[test]
    fn test_error_from_decode() {
        let dec_err = DecodeError::new(io::Error::new(io::ErrorKind::Other, "test"));
        let error: Error = dec_err.into();
        assert!(matches!(error, Error::Decode { .. }));
    }
}
