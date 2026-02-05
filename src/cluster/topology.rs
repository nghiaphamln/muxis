//! Cluster topology data structures and parsers.
//!
//! This module provides types for representing Redis Cluster topology,
//! including node information, slot ranges, and parsers for CLUSTER SLOTS
//! and CLUSTER NODES responses.

use crate::core::{Error, Result};
use crate::proto::frame::Frame;
use std::collections::HashMap;

/// Unique identifier for a Redis node in the cluster.
///
/// Node IDs are 40-character hex strings assigned by Redis.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(String);

impl NodeId {
    /// Creates a new NodeId from a string.
    ///
    /// # Arguments
    ///
    /// * `id` - The node ID string (typically 40 hex characters)
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the node ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Flags indicating the role and state of a cluster node.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeFlags {
    /// Node is a master
    pub master: bool,
    /// Node is a replica
    pub slave: bool,
    /// Node is myself (the current connection)
    pub myself: bool,
    /// Node is in PFAIL state (possibly failing)
    pub pfail: bool,
    /// Node is in FAIL state (confirmed failed)
    pub fail: bool,
    /// Node is a new node, not yet properly configured
    pub handshake: bool,
    /// Node has no assigned slots
    pub noaddr: bool,
}

impl NodeFlags {
    /// Parses node flags from a comma-separated string.
    ///
    /// # Arguments
    ///
    /// * `flags_str` - Comma-separated flags (e.g., "master,myself")
    ///
    /// # Examples
    ///
    /// ```
    /// # use muxis::cluster::NodeFlags;
    /// let flags = NodeFlags::parse("master,myself");
    /// assert!(flags.master);
    /// assert!(flags.myself);
    /// assert!(!flags.slave);
    /// ```
    pub fn parse(flags_str: &str) -> Self {
        let mut flags = Self::default();
        for flag in flags_str.split(',') {
            match flag.trim() {
                "master" => flags.master = true,
                "slave" => flags.slave = true,
                "myself" => flags.myself = true,
                "fail?" | "pfail" => flags.pfail = true,
                "fail" => flags.fail = true,
                "handshake" => flags.handshake = true,
                "noaddr" => flags.noaddr = true,
                _ => {}
            }
        }
        flags
    }

    /// Returns true if the node is a master and not in a failed state.
    pub fn is_available_master(&self) -> bool {
        self.master && !self.fail && !self.pfail
    }

    /// Returns true if the node is a replica and not in a failed state.
    pub fn is_available_replica(&self) -> bool {
        self.slave && !self.fail && !self.pfail
    }
}

/// Information about a node in the Redis Cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeInfo {
    /// Node ID (40-character hex string)
    pub id: NodeId,
    /// Network address (host:port or IP:port)
    pub address: String,
    /// Node flags (master, replica, myself, etc.)
    pub flags: NodeFlags,
    /// Master node ID (for replicas)
    pub master_id: Option<NodeId>,
    /// Ping sent timestamp
    pub ping_sent: u64,
    /// Pong received timestamp
    pub pong_recv: u64,
    /// Configuration epoch
    pub config_epoch: u64,
    /// Link state (connected or disconnected)
    pub link_state: String,
    /// Slot ranges assigned to this node
    pub slots: Vec<(u16, u16)>,
}

impl NodeInfo {
    /// Returns true if this node is a master.
    pub fn is_master(&self) -> bool {
        self.flags.master
    }

    /// Returns true if this node is a replica.
    pub fn is_replica(&self) -> bool {
        self.flags.slave
    }

    /// Returns true if this node is available (not failed).
    pub fn is_available(&self) -> bool {
        !self.flags.fail && !self.flags.pfail
    }
}

/// A range of hash slots assigned to a node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRange {
    /// Start of the slot range (inclusive)
    pub start: u16,
    /// End of the slot range (inclusive)
    pub end: u16,
    /// Master node serving this slot range
    pub master: NodeInfo,
    /// Replica nodes for this slot range
    pub replicas: Vec<NodeInfo>,
}

impl SlotRange {
    /// Returns true if the given slot is within this range.
    ///
    /// # Arguments
    ///
    /// * `slot` - The slot number to check
    pub fn contains(&self, slot: u16) -> bool {
        slot >= self.start && slot <= self.end
    }

    /// Returns the number of slots in this range.
    pub fn len(&self) -> usize {
        (self.end - self.start + 1) as usize
    }

    /// Returns true if this range is empty (invalid).
    pub fn is_empty(&self) -> bool {
        self.end < self.start
    }
}

/// Complete cluster topology information.
///
/// Maps each hash slot to its master and replica nodes.
#[derive(Debug, Clone)]
pub struct ClusterTopology {
    /// Slot ranges with their master and replica nodes
    pub slot_ranges: Vec<SlotRange>,
    /// All nodes in the cluster, indexed by node ID
    pub nodes: HashMap<NodeId, NodeInfo>,
}

impl ClusterTopology {
    /// Creates a new empty cluster topology.
    pub fn new() -> Self {
        Self {
            slot_ranges: Vec::new(),
            nodes: HashMap::new(),
        }
    }

    /// Finds the master node responsible for a given slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - The hash slot number (0-16383)
    ///
    /// # Returns
    ///
    /// Returns the master node info if found, or None if the slot is not covered.
    pub fn get_master_for_slot(&self, slot: u16) -> Option<&NodeInfo> {
        self.slot_ranges
            .iter()
            .find(|range| range.contains(slot))
            .map(|range| &range.master)
    }

    /// Finds all replica nodes for a given slot.
    ///
    /// # Arguments
    ///
    /// * `slot` - The hash slot number (0-16383)
    pub fn get_replicas_for_slot(&self, slot: u16) -> Option<&[NodeInfo]> {
        self.slot_ranges
            .iter()
            .find(|range| range.contains(slot))
            .map(|range| range.replicas.as_slice())
    }

    /// Gets node information by node ID.
    pub fn get_node(&self, node_id: &NodeId) -> Option<&NodeInfo> {
        self.nodes.get(node_id)
    }

    /// Parses cluster topology from CLUSTER SLOTS response.
    ///
    /// # Arguments
    ///
    /// * `frame` - The Frame returned by CLUSTER SLOTS command
    ///
    /// # Returns
    ///
    /// Returns a ClusterTopology on success, or an error if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The frame is not an array
    /// - The frame structure is invalid
    /// - Node addresses are missing or malformed
    pub fn from_cluster_slots(frame: Frame) -> Result<Self> {
        let mut topology = Self::new();

        let ranges = match frame {
            Frame::Array(arr) => arr,
            _ => {
                return Err(Error::Protocol {
                    message: "CLUSTER SLOTS response must be an array".to_string(),
                })
            }
        };

        for range_frame in ranges {
            let range_arr = match range_frame {
                Frame::Array(arr) => arr,
                _ => continue,
            };

            if range_arr.len() < 3 {
                continue;
            }

            // Parse slot range
            let start = match &range_arr[0] {
                Frame::Integer(n) => *n as u16,
                _ => continue,
            };

            let end = match &range_arr[1] {
                Frame::Integer(n) => *n as u16,
                _ => continue,
            };

            // Parse master node (index 2)
            let master = Self::parse_node_from_array(&range_arr[2])?;

            // Parse replica nodes (index 3+)
            let mut replicas = Vec::new();
            for node_frame in range_arr.iter().skip(3) {
                if let Ok(replica) = Self::parse_node_from_array(node_frame) {
                    replicas.push(replica);
                }
            }

            topology.slot_ranges.push(SlotRange {
                start,
                end,
                master: master.clone(),
                replicas: replicas.clone(),
            });

            // Add nodes to the nodes map
            topology.nodes.insert(master.id.clone(), master);
            for replica in replicas {
                topology.nodes.insert(replica.id.clone(), replica);
            }
        }

        Ok(topology)
    }

    /// Helper function to parse a node from a Frame array.
    fn parse_node_from_array(frame: &Frame) -> Result<NodeInfo> {
        let node_arr = match frame {
            Frame::Array(arr) => arr,
            _ => {
                return Err(Error::Protocol {
                    message: "Node info must be an array".to_string(),
                })
            }
        };

        if node_arr.len() < 3 {
            return Err(Error::Protocol {
                message: "Node info array must have at least 3 elements".to_string(),
            });
        }

        // Parse IP address
        let ip = match &node_arr[0] {
            Frame::BulkString(Some(data)) => String::from_utf8_lossy(data).to_string(),
            _ => {
                return Err(Error::Protocol {
                    message: "Node IP must be a bulk string".to_string(),
                })
            }
        };

        // Parse port
        let port = match &node_arr[1] {
            Frame::Integer(n) => *n,
            _ => {
                return Err(Error::Protocol {
                    message: "Node port must be an integer".to_string(),
                })
            }
        };

        // Parse node ID (if available, index 2)
        let id = match &node_arr[2] {
            Frame::BulkString(Some(data)) => NodeId::new(String::from_utf8_lossy(data).to_string()),
            _ => NodeId::new(format!("{}:{}", ip, port)),
        };

        let address = format!("{}:{}", ip, port);

        Ok(NodeInfo {
            id,
            address,
            flags: NodeFlags::default(),
            master_id: None,
            ping_sent: 0,
            pong_recv: 0,
            config_epoch: 0,
            link_state: "connected".to_string(),
            slots: Vec::new(),
        })
    }

    /// Parses cluster topology from CLUSTER NODES response.
    ///
    /// CLUSTER NODES returns a text-based response where each line represents
    /// a node in the cluster.
    ///
    /// # Arguments
    ///
    /// * `frame` - The Frame returned by CLUSTER NODES command (BulkString)
    ///
    /// # Returns
    ///
    /// Returns a ClusterTopology on success, or an error if parsing fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The frame is not a bulk string
    /// - The response format is invalid
    pub fn from_cluster_nodes(frame: Frame) -> Result<Self> {
        let data = match frame {
            Frame::BulkString(Some(data)) => data,
            _ => {
                return Err(Error::Protocol {
                    message: "CLUSTER NODES response must be a bulk string".to_string(),
                })
            }
        };

        let text = String::from_utf8_lossy(&data);
        let mut topology = Self::new();

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Ok(node) = Self::parse_node_from_line(line) {
                topology.nodes.insert(node.id.clone(), node);
            }
        }

        // Build slot ranges from node information
        topology.build_slot_ranges();

        Ok(topology)
    }

    /// Parses a single node from a CLUSTER NODES line.
    ///
    /// Format: <id> <ip:port@cport> <flags> <master> <ping-sent> <pong-recv>
    ///         <config-epoch> <link-state> <slot> <slot> ... <slot>
    fn parse_node_from_line(line: &str) -> Result<NodeInfo> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 8 {
            return Err(Error::Protocol {
                message: format!("Invalid CLUSTER NODES line: {}", line),
            });
        }

        // Parse node ID
        let id = NodeId::new(parts[0]);

        // Parse address (format: ip:port@cport or ip:port)
        let address = parts[1].split('@').next().unwrap_or(parts[1]).to_string();

        // Parse flags
        let flags = NodeFlags::parse(parts[2]);

        // Parse master ID (for replicas, "-" means no master)
        let master_id = if parts[3] == "-" {
            None
        } else {
            Some(NodeId::new(parts[3]))
        };

        // Parse ping-sent timestamp
        let ping_sent = parts[4].parse::<u64>().unwrap_or(0);

        // Parse pong-recv timestamp
        let pong_recv = parts[5].parse::<u64>().unwrap_or(0);

        // Parse config epoch
        let config_epoch = parts[6].parse::<u64>().unwrap_or(0);

        // Parse link state
        let link_state = parts[7].to_string();

        // Parse slot ranges (from index 8 onwards)
        let mut slots = Vec::new();
        for slot_spec in parts.iter().skip(8) {
            // Slot can be: "5461" or "5461-10922" (range)
            if let Some((start_str, end_str)) = slot_spec.split_once('-') {
                // Range format
                if let (Ok(start), Ok(end)) = (start_str.parse::<u16>(), end_str.parse::<u16>()) {
                    slots.push((start, end));
                }
            } else {
                // Single slot
                if let Ok(slot) = slot_spec.parse::<u16>() {
                    slots.push((slot, slot));
                }
            }
        }

        Ok(NodeInfo {
            id,
            address,
            flags,
            master_id,
            ping_sent,
            pong_recv,
            config_epoch,
            link_state,
            slots,
        })
    }

    /// Builds slot ranges from node information.
    ///
    /// This is called after parsing CLUSTER NODES to create SlotRange
    /// objects from the slot assignments in NodeInfo.
    fn build_slot_ranges(&mut self) {
        // Collect all master nodes with their slots
        let masters: Vec<NodeInfo> = self
            .nodes
            .values()
            .filter(|node| node.is_master())
            .cloned()
            .collect();

        for master in masters {
            // Find replicas for this master
            let replicas: Vec<NodeInfo> = self
                .nodes
                .values()
                .filter(|node| {
                    node.is_replica()
                        && node
                            .master_id
                            .as_ref()
                            .map_or(false, |mid| mid == &master.id)
                })
                .cloned()
                .collect();

            // Create a SlotRange for each slot range in the master
            for (start, end) in &master.slots {
                self.slot_ranges.push(SlotRange {
                    start: *start,
                    end: *end,
                    master: master.clone(),
                    replicas: replicas.clone(),
                });
            }
        }

        // Sort slot ranges by start slot for easier lookups
        self.slot_ranges.sort_by_key(|range| range.start);
    }
}

impl Default for ClusterTopology {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_node_id_creation() {
        let id = NodeId::new("abc123");
        assert_eq!(id.as_str(), "abc123");
        assert_eq!(id.to_string(), "abc123");
    }

    #[test]
    fn test_node_id_from_string() {
        let id: NodeId = "test123".into();
        assert_eq!(id.as_str(), "test123");
    }

    #[test]
    fn test_node_flags_parse_master() {
        let flags = NodeFlags::parse("master,myself");
        assert!(flags.master);
        assert!(flags.myself);
        assert!(!flags.slave);
        assert!(!flags.fail);
    }

    #[test]
    fn test_node_flags_parse_replica() {
        let flags = NodeFlags::parse("slave");
        assert!(flags.slave);
        assert!(!flags.master);
    }

    #[test]
    fn test_node_flags_parse_failed() {
        let flags = NodeFlags::parse("master,fail");
        assert!(flags.master);
        assert!(flags.fail);
        assert!(!flags.is_available_master());
    }

    #[test]
    fn test_node_flags_is_available_master() {
        let flags = NodeFlags::parse("master");
        assert!(flags.is_available_master());

        let flags = NodeFlags::parse("master,fail");
        assert!(!flags.is_available_master());
    }

    #[test]
    fn test_node_flags_is_available_replica() {
        let flags = NodeFlags::parse("slave");
        assert!(flags.is_available_replica());

        let flags = NodeFlags::parse("slave,pfail");
        assert!(!flags.is_available_replica());
    }

    #[test]
    fn test_slot_range_contains() {
        let range = SlotRange {
            start: 0,
            end: 5460,
            master: NodeInfo {
                id: NodeId::new("node1"),
                address: "127.0.0.1:7000".to_string(),
                flags: NodeFlags::parse("master"),
                master_id: None,
                ping_sent: 0,
                pong_recv: 0,
                config_epoch: 0,
                link_state: "connected".to_string(),
                slots: Vec::new(),
            },
            replicas: Vec::new(),
        };

        assert!(range.contains(0));
        assert!(range.contains(5460));
        assert!(!range.contains(5461));
    }

    #[test]
    fn test_slot_range_len() {
        let range = SlotRange {
            start: 0,
            end: 100,
            master: NodeInfo {
                id: NodeId::new("node1"),
                address: "127.0.0.1:7000".to_string(),
                flags: NodeFlags::parse("master"),
                master_id: None,
                ping_sent: 0,
                pong_recv: 0,
                config_epoch: 0,
                link_state: "connected".to_string(),
                slots: Vec::new(),
            },
            replicas: Vec::new(),
        };

        assert_eq!(range.len(), 101);
    }

    #[test]
    fn test_cluster_topology_from_slots_simple() {
        // Simulate CLUSTER SLOTS response with one range
        let frame = Frame::Array(vec![Frame::Array(vec![
            Frame::Integer(0),
            Frame::Integer(5460),
            Frame::Array(vec![
                Frame::BulkString(Some(Bytes::from("127.0.0.1"))),
                Frame::Integer(7000),
                Frame::BulkString(Some(Bytes::from("node1"))),
            ]),
        ])]);

        let topology = ClusterTopology::from_cluster_slots(frame).unwrap();

        assert_eq!(topology.slot_ranges.len(), 1);
        assert_eq!(topology.slot_ranges[0].start, 0);
        assert_eq!(topology.slot_ranges[0].end, 5460);
        assert_eq!(topology.slot_ranges[0].master.address, "127.0.0.1:7000");
    }

    #[test]
    fn test_cluster_topology_from_slots_with_replicas() {
        let frame = Frame::Array(vec![Frame::Array(vec![
            Frame::Integer(0),
            Frame::Integer(5460),
            Frame::Array(vec![
                Frame::BulkString(Some(Bytes::from("127.0.0.1"))),
                Frame::Integer(7000),
                Frame::BulkString(Some(Bytes::from("master1"))),
            ]),
            Frame::Array(vec![
                Frame::BulkString(Some(Bytes::from("127.0.0.1"))),
                Frame::Integer(7001),
                Frame::BulkString(Some(Bytes::from("replica1"))),
            ]),
        ])]);

        let topology = ClusterTopology::from_cluster_slots(frame).unwrap();

        assert_eq!(topology.slot_ranges.len(), 1);
        assert_eq!(topology.slot_ranges[0].replicas.len(), 1);
        assert_eq!(
            topology.slot_ranges[0].replicas[0].address,
            "127.0.0.1:7001"
        );
    }

    #[test]
    fn test_cluster_topology_get_master_for_slot() {
        let frame = Frame::Array(vec![
            Frame::Array(vec![
                Frame::Integer(0),
                Frame::Integer(5460),
                Frame::Array(vec![
                    Frame::BulkString(Some(Bytes::from("127.0.0.1"))),
                    Frame::Integer(7000),
                    Frame::BulkString(Some(Bytes::from("master1"))),
                ]),
            ]),
            Frame::Array(vec![
                Frame::Integer(5461),
                Frame::Integer(10922),
                Frame::Array(vec![
                    Frame::BulkString(Some(Bytes::from("127.0.0.1"))),
                    Frame::Integer(7001),
                    Frame::BulkString(Some(Bytes::from("master2"))),
                ]),
            ]),
        ]);

        let topology = ClusterTopology::from_cluster_slots(frame).unwrap();

        let master1 = topology.get_master_for_slot(100).unwrap();
        assert_eq!(master1.address, "127.0.0.1:7000");

        let master2 = topology.get_master_for_slot(6000).unwrap();
        assert_eq!(master2.address, "127.0.0.1:7001");

        assert!(topology.get_master_for_slot(16000).is_none());
    }

    #[test]
    fn test_cluster_topology_invalid_frame() {
        let frame = Frame::SimpleString(b"invalid".to_vec());
        let result = ClusterTopology::from_cluster_slots(frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_node_info_is_master() {
        let node = NodeInfo {
            id: NodeId::new("node1"),
            address: "127.0.0.1:7000".to_string(),
            flags: NodeFlags::parse("master"),
            master_id: None,
            ping_sent: 0,
            pong_recv: 0,
            config_epoch: 0,
            link_state: "connected".to_string(),
            slots: Vec::new(),
        };

        assert!(node.is_master());
        assert!(!node.is_replica());
    }

    #[test]
    fn test_node_info_is_replica() {
        let node = NodeInfo {
            id: NodeId::new("node2"),
            address: "127.0.0.1:7001".to_string(),
            flags: NodeFlags::parse("slave"),
            master_id: Some(NodeId::new("node1")),
            ping_sent: 0,
            pong_recv: 0,
            config_epoch: 0,
            link_state: "connected".to_string(),
            slots: Vec::new(),
        };

        assert!(!node.is_master());
        assert!(node.is_replica());
    }

    #[test]
    fn test_cluster_nodes_parse_simple() {
        // Simple CLUSTER NODES response with one master
        let response = "07c37dfeb235213a872192d90877d0cd55635b91 127.0.0.1:7000@17000 \
                        myself,master - 0 1426238317239 0 connected 0-5460\n";

        let frame = Frame::BulkString(Some(Bytes::from(response)));
        let topology = ClusterTopology::from_cluster_nodes(frame).unwrap();

        assert_eq!(topology.nodes.len(), 1);
        let node = topology.nodes.values().next().unwrap();
        assert_eq!(node.id.as_str(), "07c37dfeb235213a872192d90877d0cd55635b91");
        assert_eq!(node.address, "127.0.0.1:7000");
        assert!(node.flags.master);
        assert!(node.flags.myself);
        assert_eq!(node.slots.len(), 1);
        assert_eq!(node.slots[0], (0, 5460));
    }

    #[test]
    fn test_cluster_nodes_parse_multiple() {
        let response = "\
07c37dfeb235213a872192d90877d0cd55635b91 127.0.0.1:7000@17000 master - 0 1426238317239 0 connected 0-5460
67ed2db8d677e59ec4a4cefb06858cf2a1a89fa1 127.0.0.1:7001@17001 master - 0 1426238316232 1 connected 5461-10922
292f8b365bb7edb5e285caf0b7e6ddc7265d2f4f 127.0.0.1:7002@17002 master - 0 1426238318243 2 connected 10923-16383
";

        let frame = Frame::BulkString(Some(Bytes::from(response)));
        let topology = ClusterTopology::from_cluster_nodes(frame).unwrap();

        assert_eq!(topology.nodes.len(), 3);
        assert_eq!(topology.slot_ranges.len(), 3);

        // Check first node
        let node1_id = NodeId::new("07c37dfeb235213a872192d90877d0cd55635b91");
        let node1 = topology.get_node(&node1_id).unwrap();
        assert_eq!(node1.address, "127.0.0.1:7000");
        assert!(node1.is_master());
    }

    #[test]
    fn test_cluster_nodes_parse_with_replicas() {
        let response = "\
07c37dfeb235213a872192d90877d0cd55635b91 127.0.0.1:7000@17000 master - 0 1426238317239 0 connected 0-5460
67ed2db8d677e59ec4a4cefb06858cf2a1a89fa1 127.0.0.1:7001@17001 slave 07c37dfeb235213a872192d90877d0cd55635b91 0 1426238316232 1 connected
";

        let frame = Frame::BulkString(Some(Bytes::from(response)));
        let topology = ClusterTopology::from_cluster_nodes(frame).unwrap();

        assert_eq!(topology.nodes.len(), 2);

        // Check replica
        let replica_id = NodeId::new("67ed2db8d677e59ec4a4cefb06858cf2a1a89fa1");
        let replica = topology.get_node(&replica_id).unwrap();
        assert!(replica.is_replica());
        assert_eq!(
            replica.master_id.as_ref().unwrap().as_str(),
            "07c37dfeb235213a872192d90877d0cd55635b91"
        );

        // Check slot ranges include replica
        assert_eq!(topology.slot_ranges.len(), 1);
        assert_eq!(topology.slot_ranges[0].replicas.len(), 1);
    }

    #[test]
    fn test_cluster_nodes_parse_flags() {
        let response = "abc123 127.0.0.1:7000@17000 master,fail - 0 0 0 disconnected 0-5460\n";

        let frame = Frame::BulkString(Some(Bytes::from(response)));
        let topology = ClusterTopology::from_cluster_nodes(frame).unwrap();

        let node = topology.nodes.values().next().unwrap();
        assert!(node.flags.master);
        assert!(node.flags.fail);
        assert!(!node.is_available());
        assert_eq!(node.link_state, "disconnected");
    }

    #[test]
    fn test_cluster_nodes_parse_slot_ranges() {
        let response = "abc123 127.0.0.1:7000@17000 master - 0 0 0 connected 0-100 200-300 500\n";

        let frame = Frame::BulkString(Some(Bytes::from(response)));
        let topology = ClusterTopology::from_cluster_nodes(frame).unwrap();

        let node = topology.nodes.values().next().unwrap();
        assert_eq!(node.slots.len(), 3);
        assert_eq!(node.slots[0], (0, 100));
        assert_eq!(node.slots[1], (200, 300));
        assert_eq!(node.slots[2], (500, 500));
    }

    #[test]
    fn test_cluster_nodes_invalid_frame() {
        let frame = Frame::SimpleString(b"invalid".to_vec());
        let result = ClusterTopology::from_cluster_nodes(frame);
        assert!(result.is_err());
    }

    #[test]
    fn test_cluster_nodes_empty_response() {
        let frame = Frame::BulkString(Some(Bytes::from("")));
        let topology = ClusterTopology::from_cluster_nodes(frame).unwrap();
        assert_eq!(topology.nodes.len(), 0);
        assert_eq!(topology.slot_ranges.len(), 0);
    }

    #[test]
    fn test_parse_node_from_line_full() {
        let line = "07c37dfeb235213a872192d90877d0cd55635b91 127.0.0.1:7000@17000 \
                    myself,master - 0 1426238317239 0 connected 0-5460";

        let node = ClusterTopology::parse_node_from_line(line).unwrap();

        assert_eq!(node.id.as_str(), "07c37dfeb235213a872192d90877d0cd55635b91");
        assert_eq!(node.address, "127.0.0.1:7000");
        assert!(node.flags.master);
        assert!(node.flags.myself);
        assert!(node.master_id.is_none());
        assert_eq!(node.ping_sent, 0);
        assert_eq!(node.pong_recv, 1426238317239);
        assert_eq!(node.config_epoch, 0);
        assert_eq!(node.link_state, "connected");
        assert_eq!(node.slots, vec![(0, 5460)]);
    }

    #[test]
    fn test_build_slot_ranges_sorts() {
        let mut topology = ClusterTopology::new();

        // Add master with out-of-order slots
        let master = NodeInfo {
            id: NodeId::new("master1"),
            address: "127.0.0.1:7000".to_string(),
            flags: NodeFlags::parse("master"),
            master_id: None,
            ping_sent: 0,
            pong_recv: 0,
            config_epoch: 0,
            link_state: "connected".to_string(),
            slots: vec![(5000, 6000), (0, 1000), (2000, 3000)],
        };

        topology.nodes.insert(master.id.clone(), master);
        topology.build_slot_ranges();

        // Check that slot ranges are sorted
        assert_eq!(topology.slot_ranges.len(), 3);
        assert_eq!(topology.slot_ranges[0].start, 0);
        assert_eq!(topology.slot_ranges[1].start, 2000);
        assert_eq!(topology.slot_ranges[2].start, 5000);
    }
}
