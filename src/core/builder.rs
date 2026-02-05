use std::time::Duration;

use crate::{Client, Error};

/// Builder for configuring and creating a [`Client`] connection.
///
/// # Example
///
/// ```no_run
/// use muxis::core::builder::ClientBuilder;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = ClientBuilder::new()
///     .address("redis://localhost:6379")
///     .password("secret")
///     .database(0)
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct ClientBuilder {
    address: Option<String>,
    password: Option<String>,
    username: Option<String>,
    database: Option<u8>,
    client_name: Option<String>,
    connection_timeout: Option<Duration>,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
    tls: bool,
    queue_size: Option<usize>,
}

impl ClientBuilder {
    /// Creates a new [`ClientBuilder`] instance.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Redis server address.
    ///
    /// # Arguments
    ///
    /// * `address` - Redis address in format `redis://host:port` or `rediss://host:port` for TLS
    #[inline]
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Sets the password for authentication.
    ///
    /// # Arguments
    ///
    /// * `password` - Password string
    #[inline]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Sets the username for ACL authentication.
    ///
    /// # Arguments
    ///
    /// * `username` - Username string
    #[inline]
    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    /// Sets the Redis database number to select after connection.
    ///
    /// # Arguments
    ///
    /// * `database` - Database number (0-15)
    #[inline]
    pub fn database(mut self, database: u8) -> Self {
        self.database = Some(database);
        self
    }

    /// Sets the client connection name.
    ///
    /// # Arguments
    ///
    /// * `name` - Client name displayed in `CLIENT LIST`
    #[inline]
    pub fn client_name(mut self, name: impl Into<String>) -> Self {
        self.client_name = Some(name.into());
        self
    }

    /// Sets the connection timeout.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for connection establishment
    #[inline]
    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    /// Sets the read timeout for commands.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for a response. `None` means no timeout.
    #[inline]
    pub fn read_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Sets the write timeout for commands.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait for writes. `None` means no timeout.
    #[inline]
    pub fn write_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.write_timeout = timeout;
        self
    }

    /// Enables or disables TLS encryption.
    ///
    /// # Arguments
    ///
    /// * `enabled` - `true` to use TLS, `false` for plaintext
    #[inline]
    pub fn tls(mut self, enabled: bool) -> Self {
        self.tls = enabled;
        self
    }

    /// Sets the maximum number of pending requests in the queue.
    ///
    /// # Arguments
    ///
    /// * `size` - Maximum number of requests (default: 1024)
    #[inline]
    pub fn queue_size(mut self, size: usize) -> Self {
        self.queue_size = Some(size);
        self
    }

    /// Builds the [`Client`] connection.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidArgument`] if address is not set.
    /// Returns [`Error::Io`] if connection fails.
    #[inline]
    pub async fn build(self) -> Result<Client, Error> {
        let address = self.address.ok_or_else(|| Error::InvalidArgument {
            message: "address is required".to_string(),
        })?;

        let client = Client::connect_inner(
            address,
            self.password,
            self.database,
            self.client_name,
            self.tls,
            self.queue_size.unwrap_or(1024),
        )
        .await?;

        Ok(client)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_new() {
        let builder = ClientBuilder::new();
        assert!(builder.address.is_none());
        assert!(builder.password.is_none());
    }

    #[test]
    fn test_builder_set_address() {
        let builder = ClientBuilder::new().address("redis://localhost:6379");
        assert_eq!(builder.address, Some("redis://localhost:6379".to_string()));
    }

    #[test]
    fn test_builder_set_password() {
        let builder = ClientBuilder::new().password("secret");
        assert_eq!(builder.password, Some("secret".to_string()));
    }

    #[test]
    fn test_builder_set_database() {
        let builder = ClientBuilder::new().database(5);
        assert_eq!(builder.database, Some(5));
    }

    #[test]
    fn test_builder_set_client_name() {
        let builder = ClientBuilder::new().client_name("myapp");
        assert_eq!(builder.client_name, Some("myapp".to_string()));
    }

    #[test]
    fn test_builder_set_tls() {
        let builder = ClientBuilder::new().tls(true);
        assert!(builder.tls);
    }

    #[test]
    fn test_builder_chaining() {
        let builder = ClientBuilder::new()
            .address("redis://localhost:6379")
            .password("secret")
            .database(0)
            .client_name("test")
            .tls(false);

        assert_eq!(builder.address, Some("redis://localhost:6379".to_string()));
        assert_eq!(builder.password, Some("secret".to_string()));
        assert_eq!(builder.database, Some(0));
        assert_eq!(builder.client_name, Some("test".to_string()));
        assert!(!builder.tls);
    }

    #[tokio::test]
    async fn test_builder_build_without_address() {
        let builder = ClientBuilder::new();
        let result = builder.build().await;
        assert!(result.is_err());
        match result {
            Err(Error::InvalidArgument { message }) => {
                assert_eq!(message, "address is required");
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }
}
