//! Parsing utilities for Redis Cluster error responses.
//!
//! Redis Cluster uses special error responses for redirects:
//! - `MOVED <slot> <host>:<port>` - Permanent redirect
//! - `ASK <slot> <host>:<port>` - Temporary redirect during migration
//! - `CLUSTERDOWN` - Cluster is unavailable

use crate::Error;

/// Parses a Redis error message and converts cluster redirects to typed errors.
///
/// # Arguments
///
/// * `error_msg` - The error message bytes from Redis (e.g., b"-MOVED 3999 127.0.0.1:7000\r\n")
///
/// # Returns
///
/// - `Error::Moved` for MOVED redirects
/// - `Error::Ask` for ASK redirects
/// - `Error::ClusterDown` for CLUSTERDOWN errors
/// - `Error::Server` for other errors
///
/// # Examples
///
///
pub fn parse_redis_error(error_msg: &[u8]) -> Error {
    let msg = String::from_utf8_lossy(error_msg);
    let msg = msg.trim();

    // Check for MOVED redirect
    if let Some(stripped) = msg.strip_prefix("MOVED ") {
        if let Some((slot, address)) = parse_redirect(stripped) {
            return Error::Moved { slot, address };
        }
    }

    // Check for ASK redirect
    if let Some(stripped) = msg.strip_prefix("ASK ") {
        if let Some((slot, address)) = parse_redirect(stripped) {
            return Error::Ask { slot, address };
        }
    }

    // Check for CLUSTERDOWN
    if msg.starts_with("CLUSTERDOWN") {
        return Error::ClusterDown;
    }

    // Check for CROSSSLOT
    if msg.contains("CROSSSLOT") {
        return Error::CrossSlot;
    }

    // Default: generic server error
    Error::Server {
        message: msg.to_string(),
    }
}

/// Parses redirect arguments: "<slot> <host>:<port>"
///
/// # Arguments
///
/// * `args` - The redirect arguments (e.g., "3999 127.0.0.1:7000")
///
/// # Returns
///
/// Some((slot, address)) if parsing succeeds, None otherwise
fn parse_redirect(args: &str) -> Option<(u16, String)> {
    let parts: Vec<&str> = args.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let slot: u16 = parts[0].parse().ok()?;
    let address = parts[1].to_string();

    Some((slot, address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_moved_redirect() {
        let error = parse_redis_error(b"MOVED 3999 127.0.0.1:7000");
        match error {
            Error::Moved { slot, address } => {
                assert_eq!(slot, 3999);
                assert_eq!(address, "127.0.0.1:7000");
            }
            _ => panic!("Expected Moved error"),
        }
    }

    #[test]
    fn test_parse_ask_redirect() {
        let error = parse_redis_error(b"ASK 12345 192.168.1.100:6379");
        match error {
            Error::Ask { slot, address } => {
                assert_eq!(slot, 12345);
                assert_eq!(address, "192.168.1.100:6379");
            }
            _ => panic!("Expected Ask error"),
        }
    }

    #[test]
    fn test_parse_clusterdown() {
        let error = parse_redis_error(b"CLUSTERDOWN Hash slot not served");
        assert!(matches!(error, Error::ClusterDown));

        let error2 = parse_redis_error(b"CLUSTERDOWN");
        assert!(matches!(error2, Error::ClusterDown));
    }

    #[test]
    fn test_parse_crossslot() {
        let error = parse_redis_error(b"CROSSSLOT Keys in request don't hash to the same slot");
        assert!(matches!(error, Error::CrossSlot));
    }

    #[test]
    fn test_parse_generic_error() {
        let error = parse_redis_error(b"ERR unknown command");
        match error {
            Error::Server { message } => {
                assert_eq!(message, "ERR unknown command");
            }
            _ => panic!("Expected Server error"),
        }
    }

    #[test]
    fn test_parse_moved_with_whitespace() {
        let error = parse_redis_error(b"  MOVED 100 localhost:7001  ");
        match error {
            Error::Moved { slot, address } => {
                assert_eq!(slot, 100);
                assert_eq!(address, "localhost:7001");
            }
            _ => panic!("Expected Moved error"),
        }
    }

    #[test]
    fn test_parse_moved_invalid_slot() {
        // Invalid slot number should fall back to Server error
        let error = parse_redis_error(b"MOVED invalid 127.0.0.1:7000");
        assert!(matches!(error, Error::Server { .. }));
    }

    #[test]
    fn test_parse_moved_missing_address() {
        // Missing address should fall back to Server error
        let error = parse_redis_error(b"MOVED 3999");
        assert!(matches!(error, Error::Server { .. }));
    }

    #[test]
    fn test_parse_empty_error() {
        let error = parse_redis_error(b"");
        match error {
            Error::Server { message } => {
                assert_eq!(message, "");
            }
            _ => panic!("Expected Server error"),
        }
    }

    #[test]
    fn test_parse_redirect_valid() {
        let result = parse_redirect("3999 127.0.0.1:7000");
        assert_eq!(result, Some((3999, "127.0.0.1:7000".to_string())));
    }

    #[test]
    fn test_parse_redirect_invalid_format() {
        assert_eq!(parse_redirect("3999"), None);
        assert_eq!(parse_redirect(""), None);
        assert_eq!(parse_redirect("invalid 127.0.0.1:7000"), None);
    }

    #[test]
    fn test_parse_redirect_with_ipv6() {
        // IPv6 addresses
        let result = parse_redirect("1234 [::1]:7000");
        assert_eq!(result, Some((1234, "[::1]:7000".to_string())));

        let result2 = parse_redirect("5678 [2001:db8::1]:6379");
        assert_eq!(result2, Some((5678, "[2001:db8::1]:6379".to_string())));
    }

    #[test]
    fn test_parse_redirect_with_hostname() {
        let result = parse_redirect("999 redis-master.local:6379");
        assert_eq!(result, Some((999, "redis-master.local:6379".to_string())));
    }
}
