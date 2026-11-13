use crate::client::EmberClient;
use crate::error::Error;
use crate::node::ClnClient;
use crate::pb::ember::scheduler::v1::{
    ChallengeRequest, DeleteNodeRequest, ListNodesRequest, ListNodesResponse, NodeInfoRequest,
    NodeInfoResponse, RecoveryRequest, RecoveryResponse, RegistrationRequest, RegistrationResponse,
};

/// Ergonomic wrapper over ember's Scheduler service (node lifecycle:
/// register/recover/status/list/delete), plus access to the caller's node
/// over ember's CLN proxy - named and shaped after gl-client's `Scheduler`
/// so porting code from it needs little more than a crate-name change.
/// Each method fills in the placeholder challenge/signature/csr fields
/// ember doesn't check yet (ember is custodial: the server holds the
/// mnemonic, rather than a client-side signer) and unwraps the response.
/// Cheap to clone - it's just a handle on the shared connection.
#[derive(Clone)]
pub struct Scheduler {
    ember: EmberClient,
}

impl Scheduler {
    pub fn new(ember: EmberClient) -> Self {
        Self { ember }
    }

    /// Fetches a fresh challenge nonce. Nothing on ember's side currently
    /// verifies a signature over it.
    pub async fn get_challenge(&self) -> Result<Vec<u8>, Error> {
        let response = self
            .ember
            .scheduler_client()
            .get_challenge(ChallengeRequest {})
            .await?;
        Ok(response.into_inner().challenge)
    }

    /// Registers a new node for the caller, returning its id and recovery
    /// mnemonic. `network` and `invite_code` may be empty strings.
    pub async fn register(
        &self,
        network: impl Into<String>,
        invite_code: impl Into<String>,
    ) -> Result<RegistrationResponse, Error> {
        let request = RegistrationRequest {
            network: network.into(),
            challenge: Vec::new(),
            signature: Vec::new(),
            csr: Vec::new(),
            invite_code: invite_code.into(),
        };
        Ok(self
            .ember
            .scheduler_client()
            .register(request)
            .await?
            .into_inner())
    }

    /// Recovers a previously registered node from its mnemonic.
    pub async fn recover(&self, mnemonic: impl Into<String>) -> Result<RecoveryResponse, Error> {
        let request = RecoveryRequest {
            mnemonic: mnemonic.into(),
            challenge: Vec::new(),
            signature: Vec::new(),
            csr: Vec::new(),
        };
        Ok(self
            .ember
            .scheduler_client()
            .recover(request)
            .await?
            .into_inner())
    }

    /// Gets a node's current status.
    pub async fn node_info(&self, node_id: impl Into<String>) -> Result<NodeInfoResponse, Error> {
        let request = NodeInfoRequest {
            node_id: node_id.into(),
        };
        Ok(self
            .ember
            .scheduler_client()
            .get_node_info(request)
            .await?
            .into_inner())
    }

    /// Lists nodes registered to the caller.
    pub async fn list_nodes(&self) -> Result<ListNodesResponse, Error> {
        Ok(self
            .ember
            .scheduler_client()
            .list_nodes(ListNodesRequest {})
            .await?
            .into_inner())
    }

    /// Deletes a node.
    pub async fn delete_node(&self, node_id: impl Into<String>) -> Result<(), Error> {
        let request = DeleteNodeRequest {
            node_id: node_id.into(),
        };
        self.ember.scheduler_client().delete_node(request).await?;
        Ok(())
    }

    /// Returns a client for the caller's own node over ember's CLN proxy.
    /// Mirrors gl-client's `Scheduler::node()`, which schedules the node and
    /// connects to it; ember has nothing to schedule - every call through
    /// the returned client is routed by ember-server per the caller's
    /// identity - so this never actually fails, but stays `async` and
    /// fallible to match that call shape.
    pub async fn node(&self) -> Result<ClnClient, Error> {
        Ok(self.ember.node_client())
    }
}
