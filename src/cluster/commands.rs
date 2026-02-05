//! Redis Cluster command builders.
//!
//! This module provides command builders for Redis Cluster management commands
//! used for topology discovery and redirect handling.

use crate::core::command::Cmd;

/// Creates a CLUSTER SLOTS command.
///
/// Returns information about which cluster slots are mapped to which Redis instances.
/// This is the primary method for discovering cluster topology.
///
/// # Response Format
///
/// Returns an array of slot ranges with their corresponding master and replica nodes.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cluster")]
/// # {
/// use muxis::cluster::commands::cluster_slots;
///
/// let cmd = cluster_slots();
/// // Send to Redis: CLUSTER SLOTS
/// # }
/// ```
pub fn cluster_slots() -> Cmd {
    Cmd::new("CLUSTER").arg("SLOTS")
}

/// Creates a CLUSTER NODES command.
///
/// Returns a list of all nodes in the cluster with their ID, address, flags,
/// master/replica status, ping/pong times, and slots served.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cluster")]
/// # {
/// use muxis::cluster::commands::cluster_nodes;
///
/// let cmd = cluster_nodes();
/// // Send to Redis: CLUSTER NODES
/// # }
/// ```
pub fn cluster_nodes() -> Cmd {
    Cmd::new("CLUSTER").arg("NODES")
}

/// Creates a CLUSTER INFO command.
///
/// Returns information about the cluster state, including:
/// - cluster_state (ok/fail)
/// - cluster_slots_assigned
/// - cluster_slots_ok
/// - cluster_slots_pfail
/// - cluster_slots_fail
/// - cluster_known_nodes
/// - cluster_size
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cluster")]
/// # {
/// use muxis::cluster::commands::cluster_info;
///
/// let cmd = cluster_info();
/// // Send to Redis: CLUSTER INFO
/// # }
/// ```
pub fn cluster_info() -> Cmd {
    Cmd::new("CLUSTER").arg("INFO")
}

/// Creates an ASKING command.
///
/// Used before retrying a command that received an ASK redirect.
/// This tells the target node to accept the command even though the slot
/// is being migrated.
///
/// ASKING is a one-time flag - it only affects the immediately following command.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cluster")]
/// # {
/// use muxis::cluster::commands::asking;
///
/// let cmd = asking();
/// // Send to Redis: ASKING
/// # }
/// ```
pub fn asking() -> Cmd {
    Cmd::new("ASKING")
}

/// Creates a READONLY command.
///
/// Enables read queries for a connection to a Redis Cluster replica node.
/// Normally, replica nodes only accept read commands when the connection
/// is in READONLY mode.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cluster")]
/// # {
/// use muxis::cluster::commands::readonly;
///
/// let cmd = readonly();
/// // Send to Redis: READONLY
/// # }
/// ```
pub fn readonly() -> Cmd {
    Cmd::new("READONLY")
}

/// Creates a READWRITE command.
///
/// Disables READONLY mode for a connection. After this, the connection
/// will only accept write commands on master nodes.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cluster")]
/// # {
/// use muxis::cluster::commands::readwrite;
///
/// let cmd = readwrite();
/// // Send to Redis: READWRITE
/// # }
/// ```
pub fn readwrite() -> Cmd {
    Cmd::new("READWRITE")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::frame::Frame;
    use bytes::Bytes;

    #[test]
    fn test_cluster_slots_cmd() {
        let cmd = cluster_slots();
        let frame = cmd.into_frame();

        // Verify it's an array with CLUSTER and SLOTS
        if let Frame::Array(arr) = frame {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Frame::BulkString(Some(Bytes::from("CLUSTER"))));
            assert_eq!(arr[1], Frame::BulkString(Some(Bytes::from("SLOTS"))));
        } else {
            panic!("Expected Array frame");
        }
    }

    #[test]
    fn test_cluster_nodes_cmd() {
        let cmd = cluster_nodes();
        let frame = cmd.into_frame();

        if let Frame::Array(arr) = frame {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Frame::BulkString(Some(Bytes::from("CLUSTER"))));
            assert_eq!(arr[1], Frame::BulkString(Some(Bytes::from("NODES"))));
        } else {
            panic!("Expected Array frame");
        }
    }

    #[test]
    fn test_cluster_info_cmd() {
        let cmd = cluster_info();
        let frame = cmd.into_frame();

        if let Frame::Array(arr) = frame {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Frame::BulkString(Some(Bytes::from("CLUSTER"))));
            assert_eq!(arr[1], Frame::BulkString(Some(Bytes::from("INFO"))));
        } else {
            panic!("Expected Array frame");
        }
    }

    #[test]
    fn test_asking_cmd() {
        let cmd = asking();
        let frame = cmd.into_frame();

        if let Frame::Array(arr) = frame {
            assert_eq!(arr.len(), 1);
            assert_eq!(arr[0], Frame::BulkString(Some(Bytes::from("ASKING"))));
        } else {
            panic!("Expected Array frame");
        }
    }

    #[test]
    fn test_readonly_cmd() {
        let cmd = readonly();
        let frame = cmd.into_frame();

        if let Frame::Array(arr) = frame {
            assert_eq!(arr.len(), 1);
            assert_eq!(arr[0], Frame::BulkString(Some(Bytes::from("READONLY"))));
        } else {
            panic!("Expected Array frame");
        }
    }

    #[test]
    fn test_readwrite_cmd() {
        let cmd = readwrite();
        let frame = cmd.into_frame();

        if let Frame::Array(arr) = frame {
            assert_eq!(arr.len(), 1);
            assert_eq!(arr[0], Frame::BulkString(Some(Bytes::from("READWRITE"))));
        } else {
            panic!("Expected Array frame");
        }
    }
}
