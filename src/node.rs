use crate::client::Transport;
use crate::pb::cln::node_client::NodeClient;

/// gRPC client for ember's CLN proxy (`cln.Node`), routed by ember-server to
/// the caller's own node. Named and placed to match gl-client's
/// `gl_client::node::ClnClient`, so code written against it needs only a
/// crate-name change in the `use` line, not a type rename.
pub type ClnClient = NodeClient<Transport>;
