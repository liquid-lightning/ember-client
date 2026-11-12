use crate::client::SchedulerClient;
use crate::error::Error;
use crate::pb::ember::scheduler::v1::{
    ChallengeRequest, DeleteNodeRequest, ListNodesRequest, ListNodesResponse, NodeInfoRequest,
    NodeInfoResponse, RecoveryRequest, RecoveryResponse, RegistrationRequest, RegistrationResponse,
};

/// Ergonomic wrapper over [`SchedulerClient`] for ember's node-lifecycle
/// calls. Each method fills in the placeholder challenge/signature/csr
/// fields ember doesn't check yet (ember is custodial: the server holds the
/// mnemonic, rather than a client-side signer) and unwraps the response.
#[derive(Clone)]
pub struct Scheduler {
    client: SchedulerClient,
}

impl Scheduler {
    pub fn new(client: SchedulerClient) -> Self {
        Self { client }
    }

    /// Fetches a fresh challenge nonce. Nothing on ember's side currently
    /// verifies a signature over it.
    pub async fn get_challenge(&mut self) -> Result<Vec<u8>, Error> {
        let response = self.client.get_challenge(ChallengeRequest {}).await?;
        Ok(response.into_inner().challenge)
    }

    /// Registers a new node for the caller, returning its id and recovery
    /// mnemonic. `network` and `invite_code` may be empty strings.
    pub async fn register(
        &mut self,
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
        Ok(self.client.register(request).await?.into_inner())
    }

    /// Recovers a previously registered node from its mnemonic.
    pub async fn recover(
        &mut self,
        mnemonic: impl Into<String>,
    ) -> Result<RecoveryResponse, Error> {
        let request = RecoveryRequest {
            mnemonic: mnemonic.into(),
            challenge: Vec::new(),
            signature: Vec::new(),
            csr: Vec::new(),
        };
        Ok(self.client.recover(request).await?.into_inner())
    }

    /// Gets a node's current status.
    pub async fn node_info(
        &mut self,
        node_id: impl Into<String>,
    ) -> Result<NodeInfoResponse, Error> {
        let request = NodeInfoRequest {
            node_id: node_id.into(),
        };
        Ok(self.client.get_node_info(request).await?.into_inner())
    }

    /// Lists nodes registered to the caller.
    pub async fn list_nodes(&mut self) -> Result<ListNodesResponse, Error> {
        Ok(self
            .client
            .list_nodes(ListNodesRequest {})
            .await?
            .into_inner())
    }

    /// Deletes a node.
    pub async fn delete_node(&mut self, node_id: impl Into<String>) -> Result<(), Error> {
        let request = DeleteNodeRequest {
            node_id: node_id.into(),
        };
        self.client.delete_node(request).await?;
        Ok(())
    }
}
