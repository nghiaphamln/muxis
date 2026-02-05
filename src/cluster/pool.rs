//! Connection pooling for Redis Cluster nodes.
//!
//! This module provides connection management for cluster nodes,
//! including connection reuse, health checking, and automatic reconnection.

use crate::core::multiplexed::MultiplexedConnection;
use crate::core::Error;
use crate::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::topology::NodeId;

/// Configuration for the connection pool.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of connections per node
    pub max_connections_per_node: usize,
    /// Minimum number of idle connections to maintain per node
    pub min_idle_per_node: usize,
    /// Maximum idle time before a connection is closed
    pub max_idle_time: Duration,
    /// Health check interval
    pub health_check_interval: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_node: 10,
            min_idle_per_node: 1,
            max_idle_time: Duration::from_secs(300), // 5 minutes
            health_check_interval: Duration::from_secs(30),
        }
    }
}

/// A connection to a Redis node in the cluster.
///
/// Wraps a MultiplexedConnection with additional metadata for tracking
/// health and usage.
#[derive(Debug)]
pub struct NodeConnection {
    /// The underlying multiplexed connection
    connection: MultiplexedConnection,
    /// Node address (host:port)
    address: String,
    /// Time when this connection was established
    created_at: Instant,
    /// Time of last successful operation
    last_used_at: Instant,
    /// Number of times this connection has been used
    use_count: u64,
    /// Whether this connection is currently healthy
    is_healthy: bool,
}

impl NodeConnection {
    /// Creates a new node connection.
    ///
    /// # Arguments
    ///
    /// * `connection` - The underlying multiplexed connection
    /// * `address` - Node address (host:port)
    pub fn new(connection: MultiplexedConnection, address: String) -> Self {
        let now = Instant::now();
        Self {
            connection,
            address,
            created_at: now,
            last_used_at: now,
            use_count: 0,
            is_healthy: true,
        }
    }

    /// Returns a reference to the underlying connection.
    pub fn connection(&self) -> &MultiplexedConnection {
        &self.connection
    }

    /// Returns a mutable reference to the underlying connection.
    pub fn connection_mut(&mut self) -> &mut MultiplexedConnection {
        self.last_used_at = Instant::now();
        self.use_count += 1;
        &mut self.connection
    }

    /// Returns the node address.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns the time elapsed since this connection was created.
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Returns the time elapsed since this connection was last used.
    pub fn idle_time(&self) -> Duration {
        self.last_used_at.elapsed()
    }

    /// Returns the number of times this connection has been used.
    pub fn use_count(&self) -> u64 {
        self.use_count
    }

    /// Returns whether this connection is healthy.
    pub fn is_healthy(&self) -> bool {
        self.is_healthy
    }

    /// Marks this connection as unhealthy.
    pub fn mark_unhealthy(&mut self) {
        self.is_healthy = false;
    }

    /// Marks this connection as healthy.
    pub fn mark_healthy(&mut self) {
        self.is_healthy = true;
        self.last_used_at = Instant::now();
    }

    /// Checks if this connection should be closed due to idle time.
    ///
    /// # Arguments
    ///
    /// * `max_idle_time` - Maximum allowed idle time
    pub fn should_close(&self, max_idle_time: Duration) -> bool {
        !self.is_healthy || self.idle_time() > max_idle_time
    }
}

/// Connection pool for Redis Cluster nodes.
///
/// Manages connections to multiple Redis nodes, providing connection reuse,
/// health checking, and automatic cleanup.
pub struct ConnectionPool {
    /// Pool configuration
    config: PoolConfig,
    /// Active connections per node
    connections: Arc<RwLock<HashMap<NodeId, Vec<NodeConnection>>>>,
}

impl ConnectionPool {
    /// Creates a new connection pool.
    ///
    /// # Arguments
    ///
    /// * `config` - Pool configuration
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds a connection to the pool.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID
    /// * `address` - The node address (host:port)
    /// * `connection` - The multiplexed connection to add
    ///
    /// # Errors
    ///
    /// Returns an error if the maximum connections per node has been reached.
    pub async fn add_connection(
        &self,
        node_id: NodeId,
        address: String,
        connection: MultiplexedConnection,
    ) -> Result<()> {
        let mut conns = self.connections.write().await;
        let node_conns = conns.entry(node_id).or_insert_with(Vec::new);

        if node_conns.len() >= self.config.max_connections_per_node {
            return Err(Error::Protocol {
                message: format!(
                    "Maximum connections ({}) reached for node",
                    self.config.max_connections_per_node
                ),
            });
        }

        let node_conn = NodeConnection::new(connection, address);
        node_conns.push(node_conn);
        Ok(())
    }

    /// Gets a connection from the pool for the specified node.
    ///
    /// Returns None if no healthy connection exists for this node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID
    pub async fn get_connection(&self, node_id: &NodeId) -> Option<MultiplexedConnection> {
        let mut conns = self.connections.write().await;
        if let Some(node_conns) = conns.get_mut(node_id) {
            // Remove and return first healthy connection
            if let Some(pos) = node_conns.iter().position(|c| c.is_healthy()) {
                let mut conn = node_conns.remove(pos);
                let connection = conn.connection_mut().clone();
                node_conns.push(conn);
                return Some(connection);
            }
        }
        None
    }

    /// Removes a connection for the specified node.
    ///
    /// This is typically called when a connection becomes unhealthy.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID
    /// * `address` - The node address to match
    pub async fn remove_connection(&self, node_id: &NodeId, address: &str) {
        let mut conns = self.connections.write().await;
        if let Some(node_conns) = conns.get_mut(node_id) {
            node_conns.retain(|c| c.address() != address);
            if node_conns.is_empty() {
                conns.remove(node_id);
            }
        }
    }

    /// Marks a connection as unhealthy.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID
    /// * `address` - The node address
    pub async fn mark_unhealthy(&self, node_id: &NodeId, address: &str) {
        let mut conns = self.connections.write().await;
        if let Some(node_conns) = conns.get_mut(node_id) {
            for conn in node_conns.iter_mut() {
                if conn.address() == address {
                    conn.mark_unhealthy();
                }
            }
        }
    }

    /// Performs cleanup of idle and unhealthy connections.
    ///
    /// This should be called periodically to remove stale connections.
    pub async fn cleanup(&self) {
        let mut conns = self.connections.write().await;
        let max_idle = self.config.max_idle_time;

        for (node_id, node_conns) in conns.iter_mut() {
            let initial_len = node_conns.len();
            node_conns.retain(|c| !c.should_close(max_idle));
            let removed = initial_len - node_conns.len();
            if removed > 0 {
                tracing::debug!(
                    "Cleaned up {} idle/unhealthy connections for node {}",
                    removed,
                    node_id
                );
            }
        }

        // Remove nodes with no connections
        conns.retain(|_, node_conns| !node_conns.is_empty());
    }

    /// Returns the total number of connections across all nodes.
    pub async fn total_connections(&self) -> usize {
        let conns = self.connections.read().await;
        conns.values().map(|v| v.len()).sum()
    }

    /// Returns the number of connections for a specific node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID
    pub async fn node_connection_count(&self, node_id: &NodeId) -> usize {
        let conns = self.connections.read().await;
        conns.get(node_id).map_or(0, |v| v.len())
    }

    /// Returns the number of healthy connections for a specific node.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The node ID
    pub async fn healthy_connection_count(&self, node_id: &NodeId) -> usize {
        let conns = self.connections.read().await;
        conns
            .get(node_id)
            .map_or(0, |v| v.iter().filter(|c| c.is_healthy()).count())
    }

    /// Clears all connections from the pool.
    ///
    /// This is useful for testing or when forcing a full reconnection.
    pub async fn clear(&self) {
        let mut conns = self.connections.write().await;
        conns.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections_per_node, 10);
        assert_eq!(config.min_idle_per_node, 1);
        assert_eq!(config.max_idle_time, Duration::from_secs(300));
        assert_eq!(config.health_check_interval, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_connection_pool_new() {
        let pool_config = PoolConfig::default();
        let pool = ConnectionPool::new(pool_config);

        assert_eq!(pool.total_connections().await, 0);
    }

    #[tokio::test]
    async fn test_connection_pool_clear() {
        let pool_config = PoolConfig::default();
        let pool = ConnectionPool::new(pool_config);

        assert_eq!(pool.total_connections().await, 0);

        pool.clear().await;

        assert_eq!(pool.total_connections().await, 0);
    }

    // NOTE: More comprehensive tests require mocking MultiplexedConnection
    // or integration tests with a real Redis cluster
}
