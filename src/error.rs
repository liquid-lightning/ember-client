/// Errors from connecting to or calling ember-server.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid ember-server address {addr:?}: {source}")]
    InvalidUri {
        addr: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("connecting to ember-server: {}", error_chain(.0))]
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

/// `err` followed by each of its sources, `: `-separated. tonic's transport
/// error displays only as "transport error"; the actual cause (refused
/// connection, TLS failure, DNS, ...) is in its source chain.
fn error_chain(err: &dyn std::error::Error) -> String {
    let mut out = err.to_string();
    let mut source = err.source();
    while let Some(e) = source {
        out.push_str(": ");
        out.push_str(&e.to_string());
        source = e.source();
    }
    out
}
