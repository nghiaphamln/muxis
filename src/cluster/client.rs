//! Redis Cluster client implementation.
//!
//! This module provides a high-level client for Redis Cluster with automatic
//! slot-based routing, redirect handling, and topology management.

use crate::core::connection::Connection;
use crate::core::multiplexed::MultiplexedConnection;
use crate::core::{Error, Result};
use crate::proto::frame::Frame;
use bytes::Bytes;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

use super::commands::{asking, cluster_info, cluster_nodes, cluster_slots};
use super::errors::parse_redis_error;
use super::pool::{ConnectionPool, PoolConfig};
use super::slot::{key_slot, SLOT_COUNT};
use super::topology::ClusterTopology;

/// Default queue size for multiplexed connections.
const DEFAULT_QUEUE_SIZE: usize = 1024;

/// Maximum number of redirect retries before giving up.
const MAX_REDIRECTS: u8 = 5;

/// Maximum number of IO error retries before giving up.
const MAX_RETRIES_ON_IO: u8 = 3;

/// Base delay for exponential backoff on IO errors (milliseconds).
const RETRY_DELAY_MS: u64 = 100;

/// MOVED redirect count threshold to trigger topology refresh.
const MOVED_STORM_THRESHOLD: usize = 10;

/// Time window for counting MOVED redirects (seconds).
const MOVED_STORM_WINDOW: Duration = Duration::from_secs(1);

/// Minimum cooldown between topology refreshes (milliseconds).
const REFRESH_COOLDOWN: Duration = Duration::from_millis(500);

/// Helper function to create a connection to a Redis node.
async fn connect_to_node(address: &str) -> Result<MultiplexedConnection> {
    // Parse address to get host and port
    let addr = if address.starts_with("redis://") || address.starts_with("rediss://") {
        address
            .strip_prefix("redis://")
            .or_else(|| address.strip_prefix("rediss://"))
            .unwrap()
    } else {
        address
    };

    let stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|e| Error::Io { source: e })?;

    let connection = Connection::new(stream);
    Ok(MultiplexedConnection::new(connection, DEFAULT_QUEUE_SIZE))
}

/// Tracks MOVED redirects to detect topology change storms.
///
/// When many MOVED redirects occur in a short time window (e.g., during slot
/// migration or resharding), we want to refresh topology but not on every
/// single redirect to avoid excessive load.
#[derive(Debug)]
struct MovedStormTracker {
    /// Count of MOVED redirects in current window
    moved_count: AtomicUsize,
    /// Start of the current counting window
    window_start: Mutex<Instant>,
    /// Timestamp of last topology refresh
    last_refresh: Mutex<Instant>,
}

impl MovedStormTracker {
    fn new() -> Self {
        let now = Instant::now();
        let far_past = now - Duration::from_secs(3600); // 1 hour ago, ensures first refresh allowed
        Self {
            moved_count: AtomicUsize::new(0),
            window_start: Mutex::new(now),
            last_refresh: Mutex::new(far_past),
        }
    }

    /// Records a MOVED redirect and checks if we should refresh topology.
    ///
    /// Returns true if we should trigger a topology refresh based on:
    /// - MOVED count exceeding threshold within window
    /// - Cooldown period has elapsed since last refresh
    async fn should_refresh(&self) -> bool {
        let now = Instant::now();

        // Increment counter
        let count = self.moved_count.fetch_add(1, Ordering::Relaxed) + 1;

        // Check if window has expired
        let mut window_start = self.window_start.lock().await;
        if now.duration_since(*window_start) > MOVED_STORM_WINDOW {
            // Reset window
            self.moved_count.store(0, Ordering::Relaxed);
            *window_start = now;
            return false;
        }

        // Check if threshold exceeded
        if count >= MOVED_STORM_THRESHOLD {
            // Check cooldown
            let mut last_refresh = self.last_refresh.lock().await;
            if now.duration_since(*last_refresh) < REFRESH_COOLDOWN {
                return false;
            }

            // Update last refresh timestamp
            *last_refresh = now;
            return true;
        }

        false
    }

    /// Resets the storm tracker (useful after explicit refresh).
    async fn reset(&self) {
        self.moved_count.store(0, Ordering::Relaxed);
        *self.last_refresh.lock().await = Instant::now();
    }
}

/// Redis Cluster client.
///
/// Provides automatic slot-based routing to cluster nodes and handles
/// MOVED and ASK redirects transparently.
#[derive(Debug, Clone)]
pub struct ClusterClient {
    /// Initial seed nodes
    seed_nodes: Arc<Vec<String>>,
    /// Current cluster topology
    topology: Arc<RwLock<ClusterTopology>>,
    /// Connection pool for cluster nodes
    pool: Arc<ConnectionPool>,
    /// MOVED storm tracker for throttling topology refreshes
    storm_tracker: Arc<MovedStormTracker>,
}

impl ClusterClient {
    /// Connects to a Redis Cluster using seed nodes.
    ///
    /// The address can be a single node or a comma-separated list of nodes.
    /// The client will discover the full cluster topology from the seed nodes.
    ///
    /// # Arguments
    ///
    /// * `addresses` - Seed node addresses (e.g., "redis://127.0.0.1:7000,127.0.0.1:7001")
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot connect to any seed node
    /// - Topology discovery fails
    pub async fn connect(addresses: &str) -> Result<Self> {
        let seed_nodes = Self::parse_addresses(addresses)?;

        let pool_config = PoolConfig::default();
        let pool = Arc::new(ConnectionPool::new(pool_config));

        let client = Self {
            seed_nodes: Arc::new(seed_nodes.clone()),
            topology: Arc::new(RwLock::new(ClusterTopology::new())),
            pool,
            storm_tracker: Arc::new(MovedStormTracker::new()),
        };

        // Discover cluster topology
        client.refresh_topology().await?;

        Ok(client)
    }

    /// Parses a comma-separated list of addresses into individual URLs.
    fn parse_addresses(addresses: &str) -> Result<Vec<String>> {
        let mut parsed = Vec::new();
        for addr in addresses.split(',') {
            let addr = addr.trim();
            if addr.is_empty() {
                continue;
            }
            // Ensure address has redis:// prefix
            if !addr.starts_with("redis://") && !addr.starts_with("rediss://") {
                parsed.push(format!("redis://{}", addr));
            } else {
                parsed.push(addr.to_string());
            }
        }

        if parsed.is_empty() {
            return Err(Error::InvalidArgument {
                message: "no valid addresses provided".to_string(),
            });
        }

        Ok(parsed)
    }

    /// Refreshes the cluster topology from seed nodes.
    ///
    /// This queries the cluster for slot distribution and node information.
    pub async fn refresh_topology(&self) -> Result<()> {
        // Try each seed node until we get a successful topology
        for seed_addr in self.seed_nodes.iter() {
            if let Ok(topology) = self.fetch_topology_from_node(seed_addr).await {
                let mut topo = self.topology.write().await;
                *topo = topology;
                // Reset storm tracker after successful refresh
                self.storm_tracker.reset().await;
                return Ok(());
            }
        }

        Err(Error::Protocol {
            message: "failed to refresh topology from any seed node".to_string(),
        })
    }

    /// Fetches topology from a specific node.
    async fn fetch_topology_from_node(&self, address: &str) -> Result<ClusterTopology> {
        // Connect to the node
        let conn = connect_to_node(address).await?;

        // Execute CLUSTER SLOTS
        let slots_cmd = cluster_slots();
        let slots_frame = slots_cmd.into_frame();
        let response = conn.send_command(slots_frame).await?;

        // Parse topology
        ClusterTopology::from_cluster_slots(response)
    }

    /// Gets or creates a connection to the node responsible for a given slot.
    async fn get_connection_for_slot(&self, slot: u16) -> Result<MultiplexedConnection> {
        let topology = self.topology.read().await;

        // Find the master node for this slot
        let master = topology
            .get_master_for_slot(slot)
            .ok_or_else(|| Error::Protocol {
                message: format!("no node found for slot {}", slot),
            })?;

        let node_id = master.id.clone();
        let address = master.address.clone();

        drop(topology);

        // Try to get existing connection from pool
        if let Some(conn) = self.pool.get_connection(&node_id).await {
            return Ok(conn);
        }

        // Create new connection
        let conn = connect_to_node(&address).await?;

        // Add to pool
        self.pool
            .add_connection(node_id, address, conn.clone())
            .await?;

        Ok(conn)
    }

    /// Validates that all keys map to the same slot.
    ///
    /// This is required for multi-key commands in Redis Cluster to avoid CROSSSLOT errors.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys to validate
    ///
    /// # Returns
    ///
    /// Returns the slot number if all keys map to the same slot.
    ///
    /// # Errors
    ///
    /// Returns `Error::CrossSlot` if keys map to different slots.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "cluster")]
    /// # {
    /// use muxis::cluster::ClusterClient;
    ///
    /// // Keys with same hash tag will map to same slot
    /// let keys = vec!["user:{123}:profile", "user:{123}:settings"];
    /// let result = ClusterClient::validate_same_slot(&keys);
    /// assert!(result.is_ok());
    /// # }
    /// ```
    pub fn validate_same_slot(keys: &[&str]) -> Result<u16> {
        if keys.is_empty() {
            return Err(Error::InvalidArgument {
                message: "no keys provided".to_string(),
            });
        }

        let slot = key_slot(keys[0]);
        for key in keys.iter().skip(1) {
            let key_slot_val = key_slot(key);
            if key_slot_val != slot {
                return Err(Error::CrossSlot);
            }
        }

        Ok(slot)
    }

    /// Gets or creates a connection to a specific address.
    async fn get_connection_for_address(&self, address: &str) -> Result<MultiplexedConnection> {
        // Try to find node by address in topology
        let topology = self.topology.read().await;
        let node_id = topology
            .nodes
            .iter()
            .find(|(_id, info)| info.address == address)
            .map(|(id, _info)| id.clone());
        drop(topology);

        // If we found the node in topology, try to get from pool
        if let Some(node_id) = node_id {
            if let Some(conn) = self.pool.get_connection(&node_id).await {
                return Ok(conn);
            }
        }

        // Create new connection
        connect_to_node(address).await
    }

    /// Executes a command with automatic redirect handling.
    ///
    /// This method handles MOVED and ASK redirects transparently:
    /// - MOVED: Updates topology cache and retries on the new node
    /// - ASK: Sends ASKING command and retries once on the temporary node
    ///
    /// Additionally provides resilience features:
    /// - MOVED storm detection: Throttles topology refreshes during migrations
    /// - IO error retry: Automatically retries on connection failures with exponential backoff
    /// - Node failure handling: Marks unhealthy connections and refreshes topology
    ///
    /// # Arguments
    ///
    /// * `frame` - The command frame to execute
    /// * `slot` - The slot number for the command (used for routing)
    ///
    /// # Returns
    ///
    /// Returns the response frame from Redis.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Maximum redirect count exceeded
    /// - Maximum retry count exceeded
    /// - Connection fails after all retries
    /// - Command execution fails
    async fn execute_with_redirects(&self, frame: Frame, slot: u16) -> Result<Frame> {
        let mut redirects = 0;
        let mut io_retries = 0;
        let current_frame = frame;

        loop {
            // Get connection for the slot
            let conn_result = self.get_connection_for_slot(slot).await;

            let conn = match conn_result {
                Ok(conn) => conn,
                Err(Error::Io { source }) => {
                    // IO error getting connection - likely node down
                    io_retries += 1;
                    if io_retries > MAX_RETRIES_ON_IO {
                        return Err(Error::Io { source });
                    }

                    // Refresh topology and retry
                    if let Err(e) = self.refresh_topology().await {
                        tracing::warn!("Failed to refresh topology after connection error: {}", e);
                    }

                    // Exponential backoff
                    let delay_ms = RETRY_DELAY_MS * 2_u64.pow(io_retries as u32 - 1);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    continue;
                }
                Err(e) => return Err(e),
            };

            // Execute command
            let result = conn.send_command(current_frame.clone()).await;

            match result {
                Ok(response) => return Ok(response),
                Err(Error::Server { message }) => {
                    // Parse the error to check for redirects
                    let error = parse_redis_error(message.as_bytes());

                    match error {
                        Error::Moved {
                            slot: _new_slot,
                            address,
                        } => {
                            // MOVED redirect: permanent slot migration
                            redirects += 1;
                            if redirects > MAX_REDIRECTS {
                                return Err(Error::Protocol {
                                    message: format!(
                                        "exceeded maximum redirects ({})",
                                        MAX_REDIRECTS
                                    ),
                                });
                            }

                            // Check if we should refresh topology (storm detection)
                            if self.storm_tracker.should_refresh().await {
                                tracing::debug!(
                                    "MOVED storm detected, refreshing topology (threshold: {})",
                                    MOVED_STORM_THRESHOLD
                                );
                                if let Err(e) = self.refresh_topology().await {
                                    tracing::warn!("Failed to refresh topology after MOVED: {}", e);
                                }
                            } else {
                                tracing::trace!(
                                    "MOVED redirect to {} for slot {}, not refreshing yet",
                                    address,
                                    _new_slot
                                );
                            }

                            // Retry with updated topology (loop will use slot routing)
                            continue;
                        }
                        Error::Ask {
                            slot: _ask_slot,
                            address,
                        } => {
                            // ASK redirect: temporary migration, use ASKING
                            redirects += 1;
                            if redirects > MAX_REDIRECTS {
                                return Err(Error::Protocol {
                                    message: format!(
                                        "exceeded maximum redirects ({})",
                                        MAX_REDIRECTS
                                    ),
                                });
                            }

                            // Get connection to the ASK address
                            let ask_conn = self.get_connection_for_address(&address).await?;

                            // Send ASKING command
                            let asking_cmd = asking();
                            ask_conn.send_command(asking_cmd.into_frame()).await?;

                            // Retry the command on the ASK node
                            return ask_conn.send_command(current_frame).await;
                        }
                        _ => {
                            // Other errors: return as-is
                            return Err(error);
                        }
                    }
                }
                Err(Error::Io { source }) => {
                    // IO error during command execution - connection failure
                    io_retries += 1;
                    if io_retries > MAX_RETRIES_ON_IO {
                        return Err(Error::Io { source });
                    }

                    tracing::warn!(
                        "IO error on slot {}, retry {}/{}: {}",
                        slot,
                        io_retries,
                        MAX_RETRIES_ON_IO,
                        source
                    );

                    // Mark connection as unhealthy in pool
                    // (Pool will filter it out on next get_connection)
                    let topology = self.topology.read().await;
                    if let Some(master) = topology.get_master_for_slot(slot) {
                        self.pool.mark_unhealthy(&master.id, &master.address).await;
                        tracing::debug!("Marked node {} as unhealthy", master.address);
                    }
                    drop(topology);

                    // Refresh topology to discover new master
                    if let Err(e) = self.refresh_topology().await {
                        tracing::warn!("Failed to refresh topology after IO error: {}", e);
                    }

                    // Exponential backoff
                    let delay_ms = RETRY_DELAY_MS * 2_u64.pow(io_retries as u32 - 1);
                    tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Returns the number of known nodes in the cluster.
    pub async fn node_count(&self) -> usize {
        let topology = self.topology.read().await;
        topology.nodes.len()
    }

    /// Returns the total number of slot ranges in the cluster.
    pub async fn slot_range_count(&self) -> usize {
        let topology = self.topology.read().await;
        topology.slot_ranges.len()
    }

    /// Checks if the cluster covers all slots (0-16383).
    pub async fn is_fully_covered(&self) -> bool {
        let topology = self.topology.read().await;
        let mut covered = vec![false; SLOT_COUNT as usize];

        for range in &topology.slot_ranges {
            for slot in range.start..=range.end {
                covered[slot as usize] = true;
            }
        }

        covered.iter().all(|&c| c)
    }

    /// Gets a string value from Redis.
    ///
    /// This method automatically handles MOVED and ASK redirects.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve
    ///
    /// # Returns
    ///
    /// Returns the value if the key exists, or None if the key does not exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "cluster")]
    /// # {
    /// # use muxis::cluster::ClusterClient;
    /// # async fn example() -> muxis::Result<()> {
    /// let client = ClusterClient::connect("127.0.0.1:7000").await?;
    ///
    /// if let Some(value) = client.get("mykey").await? {
    ///     println!("Value: {:?}", value);
    /// }
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        let slot = key_slot(key);
        let cmd = crate::core::command::get(key.to_string());
        let frame = self.execute_with_redirects(cmd.into_frame(), slot).await?;

        match frame {
            Frame::BulkString(data) => Ok(data),
            Frame::Null => Ok(None),
            _ => Err(Error::Protocol {
                message: "unexpected response type for GET".to_string(),
            }),
        }
    }

    /// Sets a string value in Redis.
    ///
    /// This method automatically handles MOVED and ASK redirects.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set
    /// * `value` - The value to store
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "cluster")]
    /// # {
    /// # use muxis::cluster::ClusterClient;
    /// # use bytes::Bytes;
    /// # async fn example() -> muxis::Result<()> {
    /// let client = ClusterClient::connect("127.0.0.1:7000").await?;
    /// client.set("mykey", Bytes::from("value")).await?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub async fn set(&self, key: &str, value: Bytes) -> Result<()> {
        let slot = key_slot(key);
        let cmd = crate::core::command::set(key.to_string(), value);
        self.execute_with_redirects(cmd.into_frame(), slot).await?;
        Ok(())
    }

    /// Deletes a key from Redis.
    ///
    /// This method automatically handles MOVED and ASK redirects.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to delete
    ///
    /// # Returns
    ///
    /// Returns 1 if the key was deleted, 0 if the key did not exist.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "cluster")]
    /// # {
    /// # use muxis::cluster::ClusterClient;
    /// # async fn example() -> muxis::Result<()> {
    /// let client = ClusterClient::connect("127.0.0.1:7000").await?;
    /// let deleted = client.del("mykey").await?;
    /// println!("Deleted {} keys", deleted);
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub async fn del(&self, key: &str) -> Result<i64> {
        let slot = key_slot(key);
        let cmd = crate::core::command::del(key.to_string());
        let frame = self.execute_with_redirects(cmd.into_frame(), slot).await?;

        match frame {
            Frame::Integer(n) => Ok(n),
            _ => Err(Error::Protocol {
                message: "unexpected response type for DEL".to_string(),
            }),
        }
    }

    /// Checks if a key exists in Redis.
    ///
    /// This method automatically handles MOVED and ASK redirects.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check
    ///
    /// # Returns
    ///
    /// Returns true if the key exists, false otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(feature = "cluster")]
    /// # {
    /// # use muxis::cluster::ClusterClient;
    /// # async fn example() -> muxis::Result<()> {
    /// let client = ClusterClient::connect("127.0.0.1:7000").await?;
    ///
    /// if client.exists("mykey").await? {
    ///     println!("Key exists");
    /// }
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let slot = key_slot(key);
        let cmd = crate::core::command::exists(vec![key.to_string()]);
        let frame = self.execute_with_redirects(cmd.into_frame(), slot).await?;

        match frame {
            Frame::Integer(n) => Ok(n > 0),
            _ => Err(Error::Protocol {
                message: "unexpected response type for EXISTS".to_string(),
            }),
        }
    }

    /// Returns information about the cluster state (CLUSTER INFO).
    ///
    /// Executes the command on a random node.
    pub async fn cluster_info(&self) -> Result<String> {
        let cmd = cluster_info();
        // Pick a random node (seed node or from topology)
        // For simplicity, use refresh_topology logic's seed node or first available
        // But we want to use the pool.
        // Let's pick slot 0.
        let frame = self.execute_with_redirects(cmd.into_frame(), 0).await?;
        match frame {
            Frame::BulkString(Some(bytes)) => {
                String::from_utf8(bytes.to_vec()).map_err(|e| Error::Protocol {
                    message: format!("invalid utf8 in cluster info: {}", e),
                })
            }
            _ => Err(Error::Protocol {
                message: "unexpected response for CLUSTER INFO".to_string(),
            }),
        }
    }

    /// Returns the cluster node configuration (CLUSTER NODES).
    ///
    /// Executes the command on a random node.
    pub async fn cluster_nodes(&self) -> Result<String> {
        let cmd = cluster_nodes();
        // Pick slot 0
        let frame = self.execute_with_redirects(cmd.into_frame(), 0).await?;
        match frame {
            Frame::BulkString(Some(bytes)) => {
                String::from_utf8(bytes.to_vec()).map_err(|e| Error::Protocol {
                    message: format!("invalid utf8 in cluster nodes: {}", e),
                })
            }
            _ => Err(Error::Protocol {
                message: "unexpected response for CLUSTER NODES".to_string(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_addresses_single() {
        let result = ClusterClient::parse_addresses("127.0.0.1:7000").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "redis://127.0.0.1:7000");
    }

    #[test]
    fn test_parse_addresses_multiple() {
        let result = ClusterClient::parse_addresses("127.0.0.1:7000,127.0.0.1:7001").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "redis://127.0.0.1:7000");
        assert_eq!(result[1], "redis://127.0.0.1:7001");
    }

    #[test]
    fn test_parse_addresses_with_scheme() {
        let result = ClusterClient::parse_addresses("redis://127.0.0.1:7000").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "redis://127.0.0.1:7000");
    }

    #[test]
    fn test_parse_addresses_empty() {
        let result = ClusterClient::parse_addresses("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_addresses_whitespace() {
        let result =
            ClusterClient::parse_addresses("  127.0.0.1:7000  ,  127.0.0.1:7001  ").unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_cluster_client_node_count() {
        let pool_config = PoolConfig::default();
        let pool = Arc::new(ConnectionPool::new(pool_config));

        let client = ClusterClient {
            seed_nodes: Arc::new(vec!["redis://127.0.0.1:7000".to_string()]),
            topology: Arc::new(RwLock::new(ClusterTopology::new())),
            pool,
            storm_tracker: Arc::new(MovedStormTracker::new()),
        };

        assert_eq!(client.node_count().await, 0);
    }

    #[tokio::test]
    async fn test_cluster_client_is_fully_covered_empty() {
        let pool_config = PoolConfig::default();
        let pool = Arc::new(ConnectionPool::new(pool_config));

        let client = ClusterClient {
            seed_nodes: Arc::new(vec!["redis://127.0.0.1:7000".to_string()]),
            topology: Arc::new(RwLock::new(ClusterTopology::new())),
            pool,
            storm_tracker: Arc::new(MovedStormTracker::new()),
        };

        assert!(!client.is_fully_covered().await);
    }

    #[tokio::test]
    async fn test_max_redirects_constant() {
        // Document expected redirect limits for reference
        // MAX_REDIRECTS = 5, which is reasonable for cluster operations
        let pool_config = PoolConfig::default();
        let pool = Arc::new(ConnectionPool::new(pool_config));

        let _client = ClusterClient {
            seed_nodes: Arc::new(vec!["redis://127.0.0.1:7000".to_string()]),
            topology: Arc::new(RwLock::new(ClusterTopology::new())),
            pool,
            storm_tracker: Arc::new(MovedStormTracker::new()),
        };

        // Test passes if we can create a client (constant is defined)
        assert_eq!(MAX_REDIRECTS, 5);
    }

    #[tokio::test]
    async fn test_get_connection_for_address_not_in_topology() {
        let pool_config = PoolConfig::default();
        let pool = Arc::new(ConnectionPool::new(pool_config));

        let client = ClusterClient {
            seed_nodes: Arc::new(vec!["redis://127.0.0.1:7000".to_string()]),
            topology: Arc::new(RwLock::new(ClusterTopology::new())),
            pool,
            storm_tracker: Arc::new(MovedStormTracker::new()),
        };

        // Should attempt to create connection even if address not in topology
        // This will fail because nothing is listening, but tests the logic
        let result = client.get_connection_for_address("127.0.0.1:9999").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_default_queue_size() {
        // Ensure DEFAULT_QUEUE_SIZE is reasonable
        assert_eq!(DEFAULT_QUEUE_SIZE, 1024);
    }

    #[test]
    fn test_validate_same_slot_single_key() {
        let keys = vec!["mykey"];
        let result = ClusterClient::validate_same_slot(&keys);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_same_slot_same_hash_tag() {
        // Keys with same hash tag should map to same slot
        let keys = vec!["user:{123}:profile", "user:{123}:settings"];
        let result = ClusterClient::validate_same_slot(&keys);
        assert!(result.is_ok());
        let slot = result.unwrap();
        // Verify both keys map to this slot
        assert_eq!(key_slot("user:{123}:profile"), slot);
        assert_eq!(key_slot("user:{123}:settings"), slot);
    }

    #[test]
    fn test_validate_same_slot_different_slots() {
        // Different keys should fail (unless they happen to map to same slot)
        let keys = vec!["key1", "key2"];
        let slot1 = key_slot("key1");
        let slot2 = key_slot("key2");

        let result = ClusterClient::validate_same_slot(&keys);
        if slot1 != slot2 {
            // Should return CrossSlot error
            assert!(matches!(result, Err(Error::CrossSlot)));
        } else {
            // By chance they map to same slot
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_validate_same_slot_empty() {
        let keys: Vec<&str> = vec![];
        let result = ClusterClient::validate_same_slot(&keys);
        assert!(matches!(result, Err(Error::InvalidArgument { .. })));
    }

    #[test]
    fn test_validate_same_slot_multiple_same_slot() {
        // Using hash tags to guarantee same slot
        let keys = vec!["{user}:1", "{user}:2", "{user}:3"];
        let result = ClusterClient::validate_same_slot(&keys);
        assert!(result.is_ok());
    }

    // Tests for resilience constants
    #[test]
    fn test_resilience_constants() {
        assert_eq!(MAX_RETRIES_ON_IO, 3);
        assert_eq!(RETRY_DELAY_MS, 100);
        assert_eq!(MOVED_STORM_THRESHOLD, 10);
        assert_eq!(MOVED_STORM_WINDOW, Duration::from_secs(1));
        assert_eq!(REFRESH_COOLDOWN, Duration::from_millis(500));
    }

    // Tests for MovedStormTracker
    #[tokio::test]
    async fn test_storm_tracker_initial_state() {
        let tracker = MovedStormTracker::new();
        // First MOVED should not trigger refresh (threshold not reached)
        let should_refresh = tracker.should_refresh().await;
        assert!(!should_refresh);
    }

    #[tokio::test]
    async fn test_storm_tracker_threshold_detection() {
        let tracker = MovedStormTracker::new();

        // Simulate MOVED redirects below threshold (9 times)
        for i in 0..9 {
            let result = tracker.should_refresh().await;
            assert!(
                !result,
                "Should not refresh at count {} (below threshold)",
                i + 1
            );
        }

        // 10th MOVED should trigger refresh (at threshold)
        let result = tracker.should_refresh().await;
        assert!(result, "Should refresh at threshold (10)");
    }

    #[tokio::test]
    async fn test_storm_tracker_cooldown() {
        let tracker = MovedStormTracker::new();

        // Manually set counter to threshold - 1, then call should_refresh to reach threshold
        tracker
            .moved_count
            .store(MOVED_STORM_THRESHOLD - 1, Ordering::Relaxed);

        let first_refresh = tracker.should_refresh().await;
        assert!(first_refresh, "First refresh should succeed at threshold");

        // Immediate second attempt should be blocked by cooldown even if counter at threshold
        tracker
            .moved_count
            .store(MOVED_STORM_THRESHOLD - 1, Ordering::Relaxed);
        let second_refresh = tracker.should_refresh().await;
        assert!(!second_refresh, "Should be blocked by cooldown");

        // Sleep for real time to allow cooldown to expire
        tokio::time::sleep(REFRESH_COOLDOWN + Duration::from_millis(100)).await;
        tracker
            .moved_count
            .store(MOVED_STORM_THRESHOLD - 1, Ordering::Relaxed);
        let third_refresh = tracker.should_refresh().await;
        assert!(third_refresh, "Should refresh after cooldown expires");
    }

    #[tokio::test]
    async fn test_storm_tracker_window_reset() {
        let tracker = MovedStormTracker::new();

        // Add some MOVED redirects (5 times)
        for _ in 0..5 {
            tracker.should_refresh().await;
        }

        // Counter should be 5 now
        let count_before = tracker.moved_count.load(Ordering::Relaxed);
        assert_eq!(count_before, 5, "Counter should be 5 after 5 calls");

        // Sleep for real time beyond window to trigger reset
        tokio::time::sleep(MOVED_STORM_WINDOW + Duration::from_millis(100)).await;

        // Next should_refresh call will detect expired window and reset counter
        // The method increments first (5 -> 6), detects expiry, resets to 0, returns false
        let result = tracker.should_refresh().await;
        assert!(!result, "Should not refresh when window resets");

        // Counter should be 0 after window reset
        let count_after = tracker.moved_count.load(Ordering::Relaxed);
        assert_eq!(
            count_after, 0,
            "Counter should be 0 after window reset, got {}",
            count_after
        );
    }

    #[tokio::test]
    async fn test_storm_tracker_reset() {
        let tracker = MovedStormTracker::new();

        // Add some redirects
        for _ in 0..5 {
            tracker.should_refresh().await;
        }

        assert_eq!(tracker.moved_count.load(Ordering::Relaxed), 5);

        // Reset tracker
        tracker.reset().await;

        // Counter should be reset
        assert_eq!(tracker.moved_count.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn test_io_retry_constants() {
        // Verify exponential backoff calculation
        let delay1 = RETRY_DELAY_MS * 2_u64.pow(0); // 100ms
        let delay2 = RETRY_DELAY_MS * 2_u64.pow(1); // 200ms
        let delay3 = RETRY_DELAY_MS * 2_u64.pow(2); // 400ms

        assert_eq!(delay1, 100);
        assert_eq!(delay2, 200);
        assert_eq!(delay3, 400);
    }
}
