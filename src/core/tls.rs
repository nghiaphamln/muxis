use std::sync::Arc;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;

/// Internal TLS connector wrapper using rustls.
#[derive(Clone)]
pub struct TlsConnectorInner {
    connector: TlsConnector,
}

impl TlsConnectorInner {
    /// Creates a new TLS connector with default secure configuration.
    ///
    /// Uses `webpki-roots` for Mozilla's root certificates and `ring` as the crypto provider.
    pub fn new() -> crate::Result<Self> {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            connector: TlsConnector::from(Arc::new(config)),
        })
    }

    /// Returns the underlying `TlsConnector`.
    pub fn connector(&self) -> TlsConnector {
        self.connector.clone()
    }
}
