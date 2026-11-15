//! Rust client for [ember](https://github.com/liquid-lightning/ember): a
//! gRPC service that provisions and manages per-user Core Lightning (CLN)
//! nodes.
//!
//! This crate wraps both of ember-server's gRPC services:
//!
//! - `ember.scheduler.v1.Scheduler` - ember's own schema for node lifecycle
//!   (register/recover/status/list/delete). See [`Scheduler`] for an
//!   ergonomic wrapper, or [`SchedulerClient`] for the raw generated client.
//! - `cln.Node` - CLN's own `node.proto`, proxied by ember-server to the
//!   caller's own node. Exposed directly as [`node::ClnClient`], sharing
//!   the mTLS connection every call authenticates with.
//!
//! Named and shaped after Blockstream's `gl-client` (`Scheduler`,
//! `node::ClnClient`, `register`/`recover`/`node()`), so porting code
//! written against it needs little more than a crate-name change. ember's
//! node itself is custodial — the real CLN signing key is derived
//! server-side from the mnemonic you register with, there's no commitment-
//! signing HSM here — but authentication to ember-server itself mirrors
//! Greenlight's shape: [`auth::AuthKey`] is a seed-derived identity you
//! sign a server challenge with, in exchange for a CA-issued device
//! certificate that authenticates every call after (replacing a bearer
//! token with mTLS, same as Greenlight's device credentials).
//!
//! ```no_run
//! # async fn run() -> Result<(), ember_client::Error> {
//! use ember_client::auth::AuthKey;
//! use ember_client::EmberClient;
//!
//! // The baked-in pre-auth "app" identity - see `.resources/tls/VENDORED.md`.
//! let app_identity = ember_client::default_app_identity();
//! let app_client = EmberClient::connect("https://localhost:50051", app_identity.clone()).await?;
//!
//! let seed = [0u8; 32]; // Mnemonic::to_seed(passphrase), in practice.
//! let auth_key = AuthKey::from_seed(&seed)?;
//! let registered = app_client
//!     .scheduler()
//!     .register(&auth_key, "regtest", "my twenty four word mnemonic ...", "", app_identity.ca_cert_pem)
//!     .await?;
//!
//! // Reconnect with the device identity just issued - no seed needed again.
//! let node_client = EmberClient::connect("https://localhost:50051", registered.device_identity).await?;
//! let node = node_client.scheduler().node().await?;
//! # let _ = node;
//! # Ok(())
//! # }
//! ```

pub mod auth;
mod client;
mod error;
pub mod node;
mod scheduler;

pub mod pb;

pub use client::{ClientTlsIdentity, EmberClient, SchedulerClient, Transport};
pub use error::Error;
pub use scheduler::{RecoverOutcome, RegisterOutcome, Scheduler};

/// The baked-in pre-auth "app" identity's certificate, key, and ember's CA
/// cert — see `.resources/tls/VENDORED.md`. Deployment-specific; override
/// at build time with `EMBER_APP_CERT`/`EMBER_APP_KEY`/`EMBER_CA_CERT`
/// rather than editing these files directly (see `build.rs`).
pub const DEFAULT_APP_CERT_PEM: &str = include_str!(env!("EMBER_APP_CERT_PATH"));
pub const DEFAULT_APP_KEY_PEM: &str = include_str!(env!("EMBER_APP_KEY_PATH"));
pub const DEFAULT_CA_CERT_PEM: &str = include_str!(env!("EMBER_CA_CERT_PATH"));

/// Convenience constructor for [`ClientTlsIdentity`] from the three
/// constants above — the identity every `register`/`recover` call starts
/// from, before a per-node device identity exists.
pub fn default_app_identity() -> ClientTlsIdentity {
    ClientTlsIdentity {
        cert_pem: DEFAULT_APP_CERT_PEM.to_string(),
        key_pem: DEFAULT_APP_KEY_PEM.to_string(),
        ca_cert_pem: DEFAULT_CA_CERT_PEM.to_string(),
    }
}
