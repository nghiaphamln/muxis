use std::time::Duration;

use crate::{Client, Error};

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn username(mut self, username: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self
    }

    pub fn database(mut self, database: u8) -> Self {
        self.database = Some(database);
        self
    }

    pub fn client_name(mut self, name: impl Into<String>) -> Self {
        self.client_name = Some(name.into());
        self
    }

    pub fn connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = Some(timeout);
        self
    }

    pub fn read_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.read_timeout = timeout;
        self
    }

    pub fn write_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.write_timeout = timeout;
        self
    }

    pub fn tls(mut self, enabled: bool) -> Self {
        self.tls = enabled;
        self
    }

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
