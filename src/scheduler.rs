use crate::auth::{self, AuthKey};
use crate::client::{ClientTlsIdentity, EmberClient};
use crate::error::Error;
use crate::node::ClnClient;
use crate::pb::ember::scheduler::v1::{
    ChallengeRequest, DeleteNodeRequest, ListNodesRequest, ListNodesResponse, NodeInfoRequest,
    NodeInfoResponse, RecoveryRequest, RegistrationRequest,
};

/// What `Scheduler::register` hands back: the new node's id and recovery
/// mnemonic (write it down — ember never shows it again), plus the device
/// identity to reconnect with for everything else (see
/// `EmberClient::connect`) — no mnemonic/seed needed again after this.
pub struct RegisterOutcome {
    pub node_id: String,
    pub mnemonic: String,
    pub device_identity: ClientTlsIdentity,
}

/// What `Scheduler::recover` hands back: the recovered node's id and a
/// fresh device identity, same as [`RegisterOutcome`] minus the mnemonic
/// (the caller already has it — it's what they just recovered with).
pub struct RecoverOutcome {
    pub node_id: String,
    pub device_identity: ClientTlsIdentity,
}

/// Ergonomic wrapper over ember's Scheduler service (node lifecycle:
/// register/recover/status/list/delete), plus access to the caller's node
/// over ember's CLN proxy - named and shaped after gl-client's `Scheduler`
/// so porting code from it needs little more than a crate-name change.
/// Cheap to clone - it's just a handle on the shared connection.
#[derive(Clone)]
pub struct Scheduler {
    ember: EmberClient,
}

impl Scheduler {
    pub fn new(ember: EmberClient) -> Self {
        Self { ember }
    }

    /// Registers a new node: generates a fresh device identity, proves
    /// `auth_key`'s ownership via a signed challenge, and submits
    /// `mnemonic` (ember is custodial — it derives the real CLN signing
    /// key from this text server-side) along with `network` and an
    /// optional `invite_code`. Must be called on an [`EmberClient`]
    /// connected with the baked-in pre-auth "app" identity (see
    /// `auth::generate_device_csr`'s module docs) — `ca_cert_pem` is that
    /// same connection's pinned CA, carried through into the returned
    /// device identity since ember-server doesn't hand its CA cert back
    /// over the wire.
    pub async fn register(
        &self,
        auth_key: &AuthKey,
        network: impl Into<String>,
        mnemonic: impl Into<String>,
        invite_code: impl Into<String>,
        ca_cert_pem: impl Into<String>,
    ) -> Result<RegisterOutcome, Error> {
        let node_id = auth_key.node_id();
        let challenge = self
            .ember
            .scheduler_client()
            .get_challenge(ChallengeRequest {
                node_id: node_id.to_vec(),
            })
            .await?
            .into_inner()
            .challenge;
        let signature = auth_key.sign_challenge(&challenge)?;
        let device = auth::generate_device_csr()?;

        let request = RegistrationRequest {
            network: network.into(),
            challenge,
            signature: signature.to_vec(),
            csr: device.csr_pem.into_bytes(),
            invite_code: invite_code.into(),
            node_id: node_id.to_vec(),
            mnemonic: mnemonic.into(),
        };
        let response = self
            .ember
            .scheduler_client()
            .register(request)
            .await?
            .into_inner();

        // Empty `device_key` means the server counter-signed our CSR as
        // submitted (the normal path — see `domain::ca::CertificateAuthority::
        // sign_csr`'s docs): the private key never left this process, so
        // keep the one we generated above rather than whatever (nothing)
        // the response carries.
        let device_key_pem = if response.device_key.is_empty() {
            device.key_pair.serialize_pem()
        } else {
            response.device_key
        };

        Ok(RegisterOutcome {
            node_id: response.node_id,
            mnemonic: response.mnemonic,
            device_identity: ClientTlsIdentity {
                cert_pem: response.device_cert,
                key_pem: device_key_pem,
                ca_cert_pem: ca_cert_pem.into(),
            },
        })
    }

    /// Recovers a previously registered node, proving `auth_key`'s
    /// ownership the same way as [`Self::register`], and requesting a
    /// fresh device identity for it (e.g. after losing the original one).
    /// Same pre-auth-connection requirement as `register` above.
    pub async fn recover(
        &self,
        auth_key: &AuthKey,
        mnemonic: impl Into<String>,
        ca_cert_pem: impl Into<String>,
    ) -> Result<RecoverOutcome, Error> {
        let node_id = auth_key.node_id();
        let challenge = self
            .ember
            .scheduler_client()
            .get_challenge(ChallengeRequest {
                node_id: node_id.to_vec(),
            })
            .await?
            .into_inner()
            .challenge;
        let signature = auth_key.sign_challenge(&challenge)?;
        let device = auth::generate_device_csr()?;

        let request = RecoveryRequest {
            mnemonic: mnemonic.into(),
            challenge,
            signature: signature.to_vec(),
            csr: device.csr_pem.into_bytes(),
            node_id: node_id.to_vec(),
        };
        let response = self
            .ember
            .scheduler_client()
            .recover(request)
            .await?
            .into_inner();

        let device_key_pem = if response.device_key.is_empty() {
            device.key_pair.serialize_pem()
        } else {
            response.device_key
        };

        Ok(RecoverOutcome {
            node_id: response.node_id,
            device_identity: ClientTlsIdentity {
                cert_pem: response.device_cert,
                key_pem: device_key_pem,
                ca_cert_pem: ca_cert_pem.into(),
            },
        })
    }

    /// Gets this connection's own node's current status (ember-server
    /// resolves "own" from the caller's device certificate — see
    /// `bin/server/src/auth.rs`).
    pub async fn node_info(&self) -> Result<NodeInfoResponse, Error> {
        // The request-level `node_id` is accepted on the wire for
        // backward compatibility but ignored server-side; left empty here.
        let request = NodeInfoRequest {
            node_id: String::new(),
        };
        Ok(self
            .ember
            .scheduler_client()
            .get_node_info(request)
            .await?
            .into_inner())
    }

    /// Lists nodes registered to the caller — at most one, since each
    /// device identity authenticates exactly one node now.
    pub async fn list_nodes(&self) -> Result<ListNodesResponse, Error> {
        Ok(self
            .ember
            .scheduler_client()
            .list_nodes(ListNodesRequest {})
            .await?
            .into_inner())
    }

    /// Deletes this connection's own node.
    pub async fn delete_node(&self) -> Result<(), Error> {
        let request = DeleteNodeRequest {
            node_id: String::new(),
        };
        self.ember.scheduler_client().delete_node(request).await?;
        Ok(())
    }

    /// Returns a client for the caller's own node over ember's CLN proxy.
    /// Mirrors gl-client's `Scheduler::node()`; ember has nothing to
    /// schedule, so this never actually fails, but stays `async` and
    /// fallible to match that call shape.
    pub async fn node(&self) -> Result<ClnClient, Error> {
        Ok(self.ember.node_client())
    }
}
