//! Connection pooling for Redis Cluster nodes.
//!
//! This module provides connection management for cluster nodes,
//! including connection reuse, health checking, and automatic reconnection.

use crate::core::multiplexed::MultiplexedConnection;
use crate::core::Error;
use crate::Result;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::topology::NodeId;

/// Configuration for the connection pool.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PoolConfig {
    /// Maximum number of connections per node
    pub max_connections_per_node: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections_per_node: 10,
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
        Self {
            connection,
            address,
            is_healthy: true,
        }
    }

    /// Returns a mutable reference to the underlying connection.
    pub fn connection_mut(&mut self) -> &mut MultiplexedConnection {
        &mut self.connection
    }

    /// Returns the node address.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// Returns whether this connection is healthy.
    pub fn is_healthy(&self) -> bool {
        self.is_healthy
    }

    /// Marks this connection as unhealthy.
    pub fn mark_unhealthy(&mut self) {
        self.is_healthy = false;
    }
}

/// Connection pool for Redis Cluster nodes.
///
/// Manages connections to multiple Redis nodes, providing connection reuse,
/// health checking, and automatic cleanup.
#[derive(Debug)]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections_per_node, 10);
    }

    #[tokio::test]
    async fn test_connection_pool_new() {
        let pool_config = PoolConfig::default();
        let pool = ConnectionPool::new(pool_config);

        // We can't test total_connections directly anymore as it was removed
        // But we can verify it compiles and creates the pool
        let _ = pool;
    }
}
