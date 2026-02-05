use std::io;

use native_tls::TlsConnector;

/// Inner wrapper for TLS connector.
pub struct TlsConnectorInner {
    connector: TlsConnector,
}

impl TlsConnectorInner {
    /// Creates a new TLS connector.
    pub fn new() -> Result<Self, io::Error> {
        let connector = TlsConnector::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Self { connector })
    }

    /// Consumes the wrapper and returns the inner connector.
    pub fn into_inner(self) -> TlsConnector {
        self.connector
    }
}
