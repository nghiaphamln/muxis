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
pub fn cluster_slots() -> Cmd {
    Cmd::new("CLUSTER").arg("SLOTS")
}

/// Creates a CLUSTER NODES command.
///
/// Returns a list of all nodes in the cluster with their ID, address, flags,
/// master/replica status, ping/pong times, and slots served.
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
pub fn asking() -> Cmd {
    Cmd::new("ASKING")
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
}
