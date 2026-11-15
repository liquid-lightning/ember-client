use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

use crate::error::Error;
use crate::node::ClnClient;
use crate::pb::cln::node_client::NodeClient;
use crate::pb::ember::scheduler::v1::scheduler_client::SchedulerClient as GeneratedSchedulerClient;
use crate::Scheduler;

/// Transport ember-client's generated clients run over: a plain `Channel`.
/// Caller identity comes entirely from the mTLS client certificate
/// presented at connect time (see [`EmberClient::connect`]) — ember-server's
/// public listener requires one chaining to its CA for every call (see
/// `bin/server/src/main.rs`), so there's no separate per-call metadata
/// layer the way a bearer header would need.
pub type Transport = Channel;

/// gRPC client for ember's own schema (`ember.scheduler.v1.Scheduler`): node
/// lifecycle - register/recover/status/list/delete. See [`Scheduler`] for an
/// ergonomic wrapper over this.
pub type SchedulerClient = GeneratedSchedulerClient<Transport>;

/// A client certificate + key + the CA cert to verify ember-server's own
/// certificate against — either the baked-in pre-auth "app" identity (see
/// `auth::generate_device_csr`'s module docs and `.resources/tls/
/// VENDORED.md`), used for `GetChallenge`/`Register`/`Recover` before any
/// node exists, or a per-node device identity handed back by a prior
/// `Register`/`Recover` (see [`crate::scheduler::RegisterOutcome`]/
/// [`crate::scheduler::RecoverOutcome`]), used for everything after.
#[derive(Clone)]
pub struct ClientTlsIdentity {
    pub cert_pem: String,
    pub key_pem: String,
    pub ca_cert_pem: String,
}

/// Entry point: connects once to ember-server and hands out
/// [`SchedulerClient`]s and [`ClnClient`]s that share the connection and
/// mTLS identity.
#[derive(Clone)]
pub struct EmberClient {
    channel: Channel,
}

impl EmberClient {
    /// Connects to `addr` (e.g. `https://ember.example.com:50051`) over
    /// mTLS, presenting `identity`'s certificate and pinning its CA to
    /// verify ember-server's own certificate against (ember-server's CA is
    /// deployment-specific — there's no public root to fall back to, see
    /// `ClientTlsIdentity`'s docs).
    pub async fn connect(addr: impl Into<String>, identity: ClientTlsIdentity) -> Result<Self, Error> {
        let addr = addr.into();
        let tls = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(identity.ca_cert_pem))
            .identity(Identity::from_pem(identity.cert_pem, identity.key_pem));

        let channel = Channel::from_shared(addr.clone())
            .map_err(|e| Error::invalid_uri(addr, e))?
            .tls_config(tls)?
            .connect()
            .await?;

        Ok(Self { channel })
    }

    /// Returns a fresh client for ember's own Scheduler service, sharing
    /// this connection - cheap to call repeatedly, since `Channel` is a
    /// cheaply cloneable handle rather than a new connection.
    pub fn scheduler_client(&self) -> SchedulerClient {
        GeneratedSchedulerClient::new(self.channel.clone())
    }

    /// Returns a fresh client for ember's CLN proxy (`cln.Node`), routed by
    /// ember-server to the caller's own node, sharing this connection.
    pub fn node_client(&self) -> ClnClient {
        NodeClient::new(self.channel.clone())
    }

    /// Returns an ergonomic wrapper over [`Self::scheduler_client`] and
    /// [`Self::node_client`] - see [`Scheduler`].
    pub fn scheduler(&self) -> Scheduler {
        Scheduler::new(self.clone())
    }
}
