use chess_core::NetResult;
use futures_rustls::pki_types::CertificateDer;
use futures_rustls::rustls::ServerConfig;
use futures_rustls::TlsAcceptor;
pub use futures_rustls::TlsStream;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::PrivateKeyDer;
use smol::io::{AsyncRead, AsyncWrite};
use std::io;
use std::sync::Arc;

pub const SERVER_CERT: &[u8] = include_bytes!("server.crt");
pub const SERVER_KEY: &[u8] = include_bytes!("server.key");

pub struct TlsServer {
    pub server_config: Arc<ServerConfig>,
}

impl TlsServer {
    pub fn new() -> NetResult<Self> {
        let cert = CertificateDer::from_pem_slice(SERVER_CERT).expect("failed to load cert");
        let key = PrivateKeyDer::from_pem_slice(SERVER_KEY).expect("failed to load key");

        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Self {
            server_config: Arc::new(server_config),
        })
    }

    pub async fn to_tls<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: S,
    ) -> NetResult<TlsStream<S>> {
        let acceptor = TlsAcceptor::from(self.server_config.clone());
        let stream = acceptor.accept(stream).await?;

        Ok(TlsStream::Server(stream))
    }
}
