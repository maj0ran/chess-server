use chess_core::NetResult;
use futures_rustls::TlsConnector;
pub use futures_rustls::TlsStream;
use futures_rustls::pki_types::CertificateDer;
use futures_rustls::rustls::pki_types::ServerName;
use futures_rustls::rustls::{ClientConfig, RootCertStore};
use rustls::pki_types::pem::PemObject;
use smol::io::{AsyncRead, AsyncWrite};
use std::io;
use std::sync::Arc;

pub struct TlsClient {
    pub client_config: Arc<ClientConfig>,
}

impl TlsClient {
    pub fn new() -> NetResult<Self> {
        let ca_cert_bytes = include_bytes!("../cert/ca.crt");
        let ca_cert = CertificateDer::from_pem_slice(ca_cert_bytes).expect("failed to load cert");

        let mut root_store = RootCertStore::empty();
        root_store.add(ca_cert).unwrap();

        let client_config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Self {
            client_config: Arc::new(client_config),
        })
    }

    pub async fn to_tls<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: S,
        domain: &str,
    ) -> NetResult<TlsStream<S>> {
        let connector = TlsConnector::from(self.client_config.clone());
        let server_name = ServerName::try_from(domain)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
            .to_owned();
        let stream = connector.connect(server_name, stream).await?;
        Ok(TlsStream::Client(stream))
    }
}
