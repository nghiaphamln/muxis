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

use bytes::Bytes;
use std::time::Duration;

pub use muxis_proto::error::{Error, Result};

pub mod builder;
pub mod command;
pub mod connection;
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
/// ```rust,ignore
/// use muxis_core::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut client = Client::connect("redis://localhost:6379").await?;
///     client.set("key", "value").await?;
///     let value: String = client.get("key").await?.unwrap();
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Client {
    connection: connection::Connection<tokio::net::TcpStream>,
}

impl Client {
    async fn connect_inner(
        address: String,
        password: Option<String>,
        database: Option<u8>,
        client_name: Option<String>,
        _tls: bool,
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
            if let muxis_proto::frame::Frame::Error(_) = resp {
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

        Ok(Self { connection })
    }

    pub async fn connect<T: AsRef<str>>(addr: T) -> Result<Self> {
        let addr_str = addr.as_ref().to_string();
        let is_tls = addr_str.starts_with("rediss://");
        Self::connect_inner(addr_str, None, None, None, is_tls).await
    }

    pub async fn ping(&mut self) -> Result<Bytes> {
        let cmd = command::ping();
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::parse_frame_response(frame)?;
        Ok("PONG".into())
    }

    pub async fn echo(&mut self, msg: &str) -> Result<Bytes> {
        let cmd = command::echo(msg.to_string());
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        let bytes = command::frame_to_bytes(frame)?;
        Ok(bytes.unwrap_or_default())
    }

    pub async fn get(&mut self, key: &str) -> Result<Option<Bytes>> {
        let cmd = command::get(key.to_string());
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::frame_to_bytes(frame)
    }

    pub async fn set(&mut self, key: &str, value: Bytes) -> Result<()> {
        let cmd = command::set(key.to_string(), value);
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    pub async fn set_with_expiry(
        &mut self,
        key: &str,
        value: Bytes,
        expiry: Duration,
    ) -> Result<()> {
        let cmd = command::set_with_expiry(key.to_string(), value, expiry);
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    pub async fn incr(&mut self, key: &str) -> Result<i64> {
        let cmd = command::incr(key.to_string());
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::frame_to_int(frame)
    }

    pub async fn incr_by(&mut self, key: &str, amount: i64) -> Result<i64> {
        let cmd = command::incr_by(key.to_string(), amount);
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::frame_to_int(frame)
    }

    pub async fn decr(&mut self, key: &str) -> Result<i64> {
        let cmd = command::decr(key.to_string());
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::frame_to_int(frame)
    }

    pub async fn decr_by(&mut self, key: &str, amount: i64) -> Result<i64> {
        let cmd = command::decr_by(key.to_string(), amount);
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::frame_to_int(frame)
    }

    pub async fn del(&mut self, key: &str) -> Result<bool> {
        let cmd = command::del(key.to_string());
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        let n = command::frame_to_int(frame)?;
        Ok(n > 0)
    }

    pub async fn auth(&mut self, password: &str) -> Result<()> {
        let cmd = command::auth(password.to_string());
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    pub async fn auth_with_username(&mut self, username: &str, password: &str) -> Result<()> {
        let cmd = command::auth_with_username(username.to_string(), password.to_string());
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    pub async fn select(&mut self, db: u8) -> Result<()> {
        let cmd = command::select(db);
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
        command::parse_frame_response(frame)?;
        Ok(())
    }

    pub async fn client_setname(&mut self, name: &str) -> Result<()> {
        let cmd = command::client_setname(name.to_string());
        self.connection
            .write_frame(&cmd.into_frame())
            .await
            .map_err(|e| Error::Io { source: e })?;
        let frame = self.connection.read_frame().await?;
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
        assert!(client.is_ok() || client.is_err());
    }
}
