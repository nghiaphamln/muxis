use std::io;

use native_tls::TlsConnector;

pub struct TlsConnectorInner {
    connector: TlsConnector,
}

impl TlsConnectorInner {
    pub fn new() -> Result<Self, io::Error> {
        let connector = TlsConnector::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Self { connector })
    }

    pub fn into_inner(self) -> TlsConnector {
        self.connector
    }
}
