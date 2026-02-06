use std::io;

use thiserror::Error;

/// Result type alias for muxis operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when interacting with Redis protocol.
#[derive(Debug, Error)]
#[non_exhaustive]
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

    /// Redis Cluster: key moved to another node (permanent redirect).
    ///
    /// This error indicates that the slot for the requested key has been
    /// migrated to a different node. The client should update its slot map
    /// and retry the command on the new node.
    #[cfg(feature = "cluster")]
    #[error("MOVED to slot {slot} at {address}")]
    Moved {
        /// The slot number (0-16383).
        slot: u16,
        /// The address of the node owning this slot (e.g., "127.0.0.1:7001").
        address: String,
    },

    /// Redis Cluster: temporary redirect during migration (ASK redirect).
    ///
    /// This error occurs during slot migration. The client should send an
    /// ASKING command to the target node, then retry the command. The slot
    /// map should NOT be updated for ASK redirects.
    #[cfg(feature = "cluster")]
    #[error("ASK to slot {slot} at {address}")]
    Ask {
        /// The slot number (0-16383).
        slot: u16,
        /// The address of the node temporarily handling this slot.
        address: String,
    },

    /// Redis Cluster is down or unavailable.
    #[cfg(feature = "cluster")]
    #[error("CLUSTERDOWN cluster is down")]
    ClusterDown,

    /// Multi-key operation with keys in different slots (cluster mode).
    ///
    /// In Redis Cluster, multi-key commands (MGET, MSET, DEL, etc.) require
    /// all keys to map to the same slot. Use hash tags `{...}` to ensure
    /// keys are in the same slot.
    #[cfg(feature = "cluster")]
    #[error("CROSSSLOT keys in multi-key operation map to different slots")]
    CrossSlot,
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
