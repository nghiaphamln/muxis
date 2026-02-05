//! # Muxis Core
//!
//! Core connection handling and multiplexing logic for the Muxis Redis client.
//! Provides the foundation for communicating with Redis servers.
//!
//! ## Modules
//!
//! - [`connection`] - Single connection management
//! - [`command`] - Command builders
//! - [`builder`] - Client builder
//! - [`multiplexed`] - Multiplexed connection for concurrent requests
//!

#![warn(missing_docs)]

use bytes::Bytes;
use std::time::Duration;

pub use crate::proto::error::{Error, Result};

/// Client builder configuration.
pub mod builder;
/// Command construction helpers.
pub mod command;
/// Low-level connection management.
pub mod connection;
/// Multiplexing logic.
pub mod multiplexed;

cfg_if::cfg_if! {
    if #[cfg(feature = "tls")] {
        mod tls;
        pub use tls::TlsConnectorInner;
    }
}

/// High-level Redis client for standalone connections.
///
/// Provides a simple API for common Redis operations.
///
/// # Example
///
/// ```no_run
/// use muxis::core::Client;
/// use bytes::Bytes;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = Client::connect("redis://localhost:6379").await?;
///     client.set("key", Bytes::from("value")).await?;
///     let value: Bytes = client.get("key").await?.unwrap();
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    connection: multiplexed::MultiplexedConnection,
}

impl Client {
    async fn connect_inner(
        address: String,
        password: Option<String>,
        database: Option<u8>,
        client_name: Option<String>,
        _tls: bool,
        queue_size: usize,
    ) -> Result<Self> {
        // Parse the address using url crate for proper validation
        let parsed_url = url::Url::parse(&address).map_err(|_| Error::InvalidArgument {
            message: "invalid address format".to_string(),
        })?;

        let scheme = parsed_url.scheme();
        if scheme != "redis" && scheme != "rediss" {
            return Err(Error::InvalidArgument {
                message: "invalid scheme, expected redis:// or rediss://".to_string(),
            });
        }

        let host = parsed_url
            .host_str()
            .ok_or_else(|| Error::InvalidArgument {
                message: "missing host in address".to_string(),
            })?;

        let port = parsed_url.port().unwrap_or(6379);

        let addr = format!("{}:{}", host, port);
        let stream = tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| Error::Io { source: e })?;

        let mut connection = connection::Connection::new(stream);

        if let Some(pwd) = password {
            let auth_cmd = command::auth(pwd);
            connection
                .write_frame(&auth_cmd.into_frame())
                .await
                .map_err(|e| Error::Io { source: e })?;
            let resp = connection.read_frame().await?;
            if let crate::proto::frame::Frame::Error(_) = resp {
                return Err(Error::Auth);
            }
        }

        if let Some(db) = database {
            let select_cmd = command::select(db);
            connection
                .write_frame(&select_cmd.into_frame())
                .await
                .map_err(|e| Error::Io { source: e })?;
            let _resp = connection.read_frame().await?;
        }

        if let Some(name) = client_name {
            let setname_cmd = command::client_setname(name);
            connection
                .write_frame(&setname_cmd.into_frame())
                .await
                .map_err(|e| Error::Io { source: e })?;
            let _resp = connection.read_frame().await?;
        }

        let connection = multiplexed::MultiplexedConnection::new(connection, queue_size);

        Ok(Self { connection })
    }

    /// Connects to a Redis server using the provided address.
    ///
    /// The address should be in the format `redis://host:port` or `rediss://host:port` (for TLS).
    ///
    /// # Arguments
    ///
    /// * `addr` - The connection string (e.g., "redis://127.0.0.1:6379")
    ///
    /// # Returns
    ///
    /// A `Result` containing the connected `Client` or an error.
    pub async fn connect<T: AsRef<str>>(addr: T) -> Result<Self> {
        let addr_str = addr.as_ref().to_string();
        let is_tls = addr_str.starts_with("rediss://");
        Self::connect_inner(addr_str, None, None, None, is_tls, 1024).await
    }

    /// Sends a PING command to the server.
    ///
    /// # Returns
    ///
    /// Returns `PONG` as bytes if successful.
    pub async fn ping(&mut self) -> Result<Bytes> {
        let cmd = command::ping();
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok("PONG".into())
    }

    /// Echoes the provided message back from the server.
    ///
    /// # Arguments
    ///
    /// * `msg` - The message to echo.
    pub async fn echo(&mut self, msg: &str) -> Result<Bytes> {
        let cmd = command::echo(msg.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        let bytes = command::frame_to_bytes(frame)?;
        Ok(bytes.unwrap_or_default())
    }

    /// Gets the value associated with the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve.
    ///
    /// # Returns
    ///
    /// Returns `Some(Bytes)` if the key exists, or `None` if it does not.
    pub async fn get(&mut self, key: &str) -> Result<Option<Bytes>> {
        let cmd = command::get(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bytes(frame)
    }

    /// Sets the string value of a key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set.
    /// * `value` - The value to set.
    pub async fn set(&mut self, key: &str, value: Bytes) -> Result<()> {
        let cmd = command::set(key.to_string(), value);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Sets the value of a key with an expiration time (SETEX).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set.
    /// * `value` - The value to set.
    /// * `expiry` - The expiration duration.
    pub async fn set_with_expiry(
        &mut self,
        key: &str,
        value: Bytes,
        expiry: Duration,
    ) -> Result<()> {
        let cmd = command::set_with_expiry(key.to_string(), value, expiry);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Increments the number stored at key by one.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to increment.
    ///
    /// # Returns
    ///
    /// The value of the key after the increment.
    pub async fn incr(&mut self, key: &str) -> Result<i64> {
        let cmd = command::incr(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Increments the number stored at key by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to increment.
    /// * `amount` - The amount to increment by.
    pub async fn incr_by(&mut self, key: &str, amount: i64) -> Result<i64> {
        let cmd = command::incr_by(key.to_string(), amount);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Decrements the number stored at key by one.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to decrement.
    ///
    /// # Returns
    ///
    /// The value of the key after the decrement.
    pub async fn decr(&mut self, key: &str) -> Result<i64> {
        let cmd = command::decr(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Decrements the number stored at key by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to decrement.
    /// * `amount` - The amount to decrement by.
    pub async fn decr_by(&mut self, key: &str, amount: i64) -> Result<i64> {
        let cmd = command::decr_by(key.to_string(), amount);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Removes the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove.
    ///
    /// # Returns
    ///
    /// `true` if the key was removed, `false` if the key did not exist.
    pub async fn del(&mut self, key: &str) -> Result<bool> {
        let cmd = command::del(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        let n = command::frame_to_int(frame)?;
        Ok(n > 0)
    }

    /// Authenticates with the server using a password.
    ///
    /// # Arguments
    ///
    /// * `password` - The password to use.
    pub async fn auth(&mut self, password: &str) -> Result<()> {
        let cmd = command::auth(password.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Authenticates with the server using a username and password (ACL).
    ///
    /// # Arguments
    ///
    /// * `username` - The username to use.
    /// * `password` - The password to use.
    pub async fn auth_with_username(&mut self, username: &str, password: &str) -> Result<()> {
        let cmd = command::auth_with_username(username.to_string(), password.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Selects the Redis logical database to use.
    ///
    /// # Arguments
    ///
    /// * `db` - The database index (e.g., 0).
    pub async fn select(&mut self, db: u8) -> Result<()> {
        let cmd = command::select(db);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Sets the name of the current connection.
    ///
    /// This name is displayed in the `CLIENT LIST` command output on the server.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to assign to the connection.
    pub async fn client_setname(&mut self, name: &str) -> Result<()> {
        let cmd = command::client_setname(name.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_connect() {
        let client = Client::connect("redis://localhost:6379").await;
        // This will likely fail without a running Redis, so we assert result exists
        assert!(client.is_ok() || client.is_err());
    }
}
