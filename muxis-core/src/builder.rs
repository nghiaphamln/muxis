use std::time::Duration;

use crate::{Client, Error};

/// Builder for configuring and creating a [`Client`] connection.
///
/// # Example
///
/// ```rust,ignore
/// use muxis_core::ClientBuilder;
///
/// let client = ClientBuilder::new()
///     .address("redis://localhost:6379")
///     .password("secret")
///     .database(0)
///     .build()
///     .await?;
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
        )
        .await?;

        Ok(client)
    }
}
