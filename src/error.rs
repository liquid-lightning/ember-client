/// Errors from connecting to or calling ember-server.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid ember-server address {addr:?}: {source}")]
    InvalidUri {
        addr: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("connecting to ember-server: {0}")]
    Transport(#[from] tonic::transport::Error),
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error("auth key error: {0}")]
    Auth(#[from] secp256k1::Error),
    #[error("device certificate error: {0}")]
    Cert(#[from] rcgen::Error),
    #[error("invalid credentials blob: {0}")]
    InvalidCredentials(&'static str),
}

impl Error {
    pub(crate) fn invalid_uri(
        addr: String,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::InvalidUri {
            addr,
            source: Box::new(source),
        }
    }
}
