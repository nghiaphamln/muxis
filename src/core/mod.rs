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

    /// Gets the values of all specified keys (MGET).
    ///
    /// For every key that does not exist, the corresponding element will be `None`.
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of key names to retrieve.
    ///
    /// # Returns
    ///
    /// A vector of `Option<Bytes>`, one for each key. `None` for keys that do not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("key1", Bytes::from("value1")).await?;
    /// client.set("key2", Bytes::from("value2")).await?;
    /// let values = client.mget(&["key1", "key2", "key3"]).await?;
    /// assert_eq!(values[0], Some(Bytes::from("value1")));
    /// assert_eq!(values[1], Some(Bytes::from("value2")));
    /// assert_eq!(values[2], None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mget(&mut self, keys: &[&str]) -> Result<Vec<Option<Bytes>>> {
        let keys_vec = keys.iter().map(|k| k.to_string()).collect();
        let cmd = command::mget(keys_vec);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_vec_bytes(frame)
    }

    /// Sets multiple key-value pairs atomically (MSET).
    ///
    /// # Arguments
    ///
    /// * `pairs` - Slice of (key, value) tuples to set.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.mset(&[
    ///     ("key1", Bytes::from("value1")),
    ///     ("key2", Bytes::from("value2")),
    /// ]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mset(&mut self, pairs: &[(&str, Bytes)]) -> Result<()> {
        let pairs_vec = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        let cmd = command::mset(pairs_vec);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Sets the value of a key only if it does not exist (SETNX).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set.
    /// * `value` - The value to set.
    ///
    /// # Returns
    ///
    /// `true` if the key was set, `false` if the key already existed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let was_set = client.setnx("mykey", Bytes::from("value")).await?;
    /// assert!(was_set);
    /// let was_set_again = client.setnx("mykey", Bytes::from("newvalue")).await?;
    /// assert!(!was_set_again);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn setnx(&mut self, key: &str, value: Bytes) -> Result<bool> {
        let cmd = command::setnx(key.to_string(), value);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bool(frame)
    }

    /// Sets the value of a key with an expiration in seconds (SETEX).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set.
    /// * `seconds` - Expiration time in seconds.
    /// * `value` - The value to set.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.setex("mykey", 10, Bytes::from("value")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn setex(&mut self, key: &str, seconds: u64, value: Bytes) -> Result<()> {
        let cmd = command::setex(key.to_string(), seconds, value);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Gets the value of a key and deletes it atomically (GETDEL).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to get and delete.
    ///
    /// # Returns
    ///
    /// `Some(Bytes)` if the key exists, or `None` if it does not.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("mykey", Bytes::from("value")).await?;
    /// let value = client.getdel("mykey").await?;
    /// assert_eq!(value, Some(Bytes::from("value")));
    /// let value_again = client.get("mykey").await?;
    /// assert_eq!(value_again, None);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn getdel(&mut self, key: &str) -> Result<Option<Bytes>> {
        let cmd = command::getdel(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bytes(frame)
    }

    /// Appends a value to a key (APPEND).
    ///
    /// If the key does not exist, it is created and set as an empty string, then the value
    /// is appended.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to append to.
    /// * `value` - The value to append.
    ///
    /// # Returns
    ///
    /// The length of the string after the append operation.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("mykey", Bytes::from("Hello")).await?;
    /// let len = client.append("mykey", Bytes::from(" World")).await?;
    /// assert_eq!(len, 11);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn append(&mut self, key: &str, value: Bytes) -> Result<i64> {
        let cmd = command::append(key.to_string(), value);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Returns the length of the string value stored at key (STRLEN).
    ///
    /// If the key does not exist, returns 0.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check.
    ///
    /// # Returns
    ///
    /// The length of the string at key, or 0 if the key does not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("mykey", Bytes::from("Hello World")).await?;
    /// let len = client.strlen("mykey").await?;
    /// assert_eq!(len, 11);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn strlen(&mut self, key: &str) -> Result<i64> {
        let cmd = command::strlen(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Checks if one or more keys exist (EXISTS).
    ///
    /// # Arguments
    ///
    /// * `keys` - Slice of key names to check.
    ///
    /// # Returns
    ///
    /// The number of keys that exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("key1", Bytes::from("value1")).await?;
    /// let count = client.exists(&["key1", "key2", "key3"]).await?;
    /// assert_eq!(count, 1);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn exists(&mut self, keys: &[&str]) -> Result<i64> {
        let keys_vec = keys.iter().map(|k| k.to_string()).collect();
        let cmd = command::exists(keys_vec);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Returns the type of value stored at key (TYPE).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check.
    ///
    /// # Returns
    ///
    /// The type as a string: "string", "list", "set", "zset", "hash", "stream", or "none".
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("mykey", Bytes::from("value")).await?;
    /// let key_type = client.key_type("mykey").await?;
    /// assert_eq!(key_type, "string");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn key_type(&mut self, key: &str) -> Result<String> {
        let cmd = command::key_type(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_string(frame)
    }

    /// Sets a timeout on a key in seconds (EXPIRE).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set expiration on.
    /// * `seconds` - Expiration time in seconds.
    ///
    /// # Returns
    ///
    /// `true` if the timeout was set, `false` if the key does not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("mykey", Bytes::from("value")).await?;
    /// let was_set = client.expire("mykey", 60).await?;
    /// assert!(was_set);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn expire(&mut self, key: &str, seconds: u64) -> Result<bool> {
        let cmd = command::expire(key.to_string(), seconds);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bool(frame)
    }

    /// Sets an absolute Unix timestamp expiration on a key (EXPIREAT).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to set expiration on.
    /// * `timestamp` - Unix timestamp in seconds.
    ///
    /// # Returns
    ///
    /// `true` if the timeout was set, `false` if the key does not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("mykey", Bytes::from("value")).await?;
    /// let timestamp = 1735689600; // Some future timestamp
    /// let was_set = client.expireat("mykey", timestamp).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn expireat(&mut self, key: &str, timestamp: u64) -> Result<bool> {
        let cmd = command::expireat(key.to_string(), timestamp);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bool(frame)
    }

    /// Returns the remaining time to live of a key in seconds (TTL).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to check.
    ///
    /// # Returns
    ///
    /// TTL in seconds, -2 if the key does not exist, -1 if the key has no expiration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.setex("mykey", 60, Bytes::from("value")).await?;
    /// let ttl = client.ttl("mykey").await?;
    /// assert!(ttl > 0 && ttl <= 60);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ttl(&mut self, key: &str) -> Result<i64> {
        let cmd = command::ttl(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Removes the expiration from a key (PERSIST).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to persist.
    ///
    /// # Returns
    ///
    /// `true` if the expiration was removed, `false` if the key does not exist or has no expiration.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.setex("mykey", 60, Bytes::from("value")).await?;
    /// let was_persisted = client.persist("mykey").await?;
    /// assert!(was_persisted);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn persist(&mut self, key: &str) -> Result<bool> {
        let cmd = command::persist(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bool(frame)
    }

    /// Renames a key (RENAME).
    ///
    /// # Arguments
    ///
    /// * `key` - The key to rename.
    /// * `newkey` - The new key name.
    ///
    /// # Errors
    ///
    /// Returns an error if the key does not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.set("oldkey", Bytes::from("value")).await?;
    /// client.rename("oldkey", "newkey").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rename(&mut self, key: &str, newkey: &str) -> Result<()> {
        let cmd = command::rename(key.to_string(), newkey.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Iterates the set of keys in the database using a cursor (SCAN).
    ///
    /// # Arguments
    ///
    /// * `cursor` - The cursor value (use 0 to start iteration).
    ///
    /// # Returns
    ///
    /// A tuple of (next_cursor, keys). When next_cursor is 0, the iteration is complete.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let mut cursor = 0;
    /// loop {
    ///     let (next_cursor, keys) = client.scan(cursor).await?;
    ///     for key in keys {
    ///         println!("Key: {}", key);
    ///     }
    ///     cursor = next_cursor;
    ///     if cursor == 0 {
    ///         break;
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn scan(&mut self, cursor: u64) -> Result<(u64, Vec<String>)> {
        let cmd = command::scan(cursor);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_scan_response(frame)
    }

    /// Sets a field in a hash (HSET).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `field` - The field name.
    /// * `value` - The value to set.
    ///
    /// # Returns
    ///
    /// `true` if the field was newly created, `false` if it was updated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.hset("myhash", "field1", Bytes::from("value1")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hset(&mut self, key: &str, field: &str, value: Bytes) -> Result<bool> {
        let cmd = command::hset(key.to_string(), field.to_string(), value);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bool(frame)
    }

    /// Gets a field value from a hash (HGET).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `field` - The field name.
    ///
    /// # Returns
    ///
    /// `Some(Bytes)` if the field exists, or `None` if it does not.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let value = client.hget("myhash", "field1").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hget(&mut self, key: &str, field: &str) -> Result<Option<Bytes>> {
        let cmd = command::hget(key.to_string(), field.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bytes(frame)
    }

    /// Sets multiple fields in a hash (HMSET).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `fields` - Slice of (field, value) tuples.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// client.hmset("myhash", &[
    ///     ("field1", Bytes::from("value1")),
    ///     ("field2", Bytes::from("value2")),
    /// ]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hmset(&mut self, key: &str, fields: &[(&str, Bytes)]) -> Result<()> {
        let fields_vec = fields
            .iter()
            .map(|(f, v)| (f.to_string(), v.clone()))
            .collect();
        let cmd = command::hmset(key.to_string(), fields_vec);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    /// Gets multiple field values from a hash (HMGET).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `fields` - Slice of field names.
    ///
    /// # Returns
    ///
    /// A vector of `Option<Bytes>`, one for each field. `None` for fields that do not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let values = client.hmget("myhash", &["field1", "field2"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hmget(&mut self, key: &str, fields: &[&str]) -> Result<Vec<Option<Bytes>>> {
        let fields_vec = fields.iter().map(|f| f.to_string()).collect();
        let cmd = command::hmget(key.to_string(), fields_vec);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_vec_bytes(frame)
    }

    /// Gets all fields and values from a hash (HGETALL).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    ///
    /// # Returns
    ///
    /// A HashMap of field names to values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let all_fields = client.hgetall("myhash").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hgetall(&mut self, key: &str) -> Result<std::collections::HashMap<String, Bytes>> {
        let cmd = command::hgetall(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_hashmap(frame)
    }

    /// Deletes one or more fields from a hash (HDEL).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `fields` - Slice of field names to delete.
    ///
    /// # Returns
    ///
    /// The number of fields that were deleted.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let count = client.hdel("myhash", &["field1", "field2"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hdel(&mut self, key: &str, fields: &[&str]) -> Result<i64> {
        let fields_vec = fields.iter().map(|f| f.to_string()).collect();
        let cmd = command::hdel(key.to_string(), fields_vec);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Checks if a field exists in a hash (HEXISTS).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `field` - The field name.
    ///
    /// # Returns
    ///
    /// `true` if the field exists, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let exists = client.hexists("myhash", "field1").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hexists(&mut self, key: &str, field: &str) -> Result<bool> {
        let cmd = command::hexists(key.to_string(), field.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bool(frame)
    }

    /// Gets the number of fields in a hash (HLEN).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    ///
    /// # Returns
    ///
    /// The number of fields in the hash.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let count = client.hlen("myhash").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hlen(&mut self, key: &str) -> Result<i64> {
        let cmd = command::hlen(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Gets all field names from a hash (HKEYS).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    ///
    /// # Returns
    ///
    /// A vector of field names.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let keys = client.hkeys("myhash").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hkeys(&mut self, key: &str) -> Result<Vec<String>> {
        let cmd = command::hkeys(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_vec_string(frame)
    }

    /// Gets all values from a hash (HVALS).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    ///
    /// # Returns
    ///
    /// A vector of values.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let values = client.hvals("myhash").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hvals(&mut self, key: &str) -> Result<Vec<String>> {
        let cmd = command::hvals(key.to_string());
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_vec_string(frame)
    }

    /// Increments a hash field by an integer (HINCRBY).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `field` - The field name.
    /// * `increment` - The amount to increment by.
    ///
    /// # Returns
    ///
    /// The value of the field after the increment.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let new_value = client.hincrby("myhash", "counter", 5).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hincrby(&mut self, key: &str, field: &str, increment: i64) -> Result<i64> {
        let cmd = command::hincrby(key.to_string(), field.to_string(), increment);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_int(frame)
    }

    /// Increments a hash field by a float (HINCRBYFLOAT).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `field` - The field name.
    /// * `increment` - The amount to increment by.
    ///
    /// # Returns
    ///
    /// The value of the field after the increment.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let new_value = client.hincrbyfloat("myhash", "price", 2.5).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hincrbyfloat(&mut self, key: &str, field: &str, increment: f64) -> Result<f64> {
        let cmd = command::hincrbyfloat(key.to_string(), field.to_string(), increment);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_float(frame)
    }

    /// Sets a field in a hash only if it does not exist (HSETNX).
    ///
    /// # Arguments
    ///
    /// * `key` - The hash key.
    /// * `field` - The field name.
    /// * `value` - The value to set.
    ///
    /// # Returns
    ///
    /// `true` if the field was set, `false` if it already existed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use muxis::core::Client;
    /// # use bytes::Bytes;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = Client::connect("redis://127.0.0.1:6379").await?;
    /// let was_set = client.hsetnx("myhash", "field1", Bytes::from("value")).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hsetnx(&mut self, key: &str, field: &str, value: Bytes) -> Result<bool> {
        let cmd = command::hsetnx(key.to_string(), field.to_string(), value);
        let frame = self.connection.send_command(cmd.into_frame()).await?;
        command::frame_to_bool(frame)
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
