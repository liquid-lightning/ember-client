use std::sync::Arc;

use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tonic::{Request, Status};

/// Attaches the `x-user-id` metadata header ember-server's auth module reads
/// for calls that need caller identity. It's a placeholder trust-me header
/// on ember's side today - no signature or token validation yet (see
/// ember's `bin/server/src/auth.rs`) - so this interceptor just carries
/// whatever `user_id` [`EmberClient`](crate::EmberClient) was given. Calls
/// that don't check it (`GetChallenge`, `GetNodeInfo`, `DeleteNode` as of
/// writing) silently ignore the header.
#[derive(Clone)]
pub struct UserIdInterceptor {
    user_id: Option<Arc<str>>,
}

impl UserIdInterceptor {
    pub(crate) fn new(user_id: Option<Arc<str>>) -> Self {
        Self { user_id }
    }
}

impl Interceptor for UserIdInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        if let Some(user_id) = &self.user_id {
            let value = MetadataValue::try_from(user_id.as_ref())
                .map_err(|_| Status::invalid_argument("x-user-id must be ASCII"))?;
            request.metadata_mut().insert("x-user-id", value);
        }
        Ok(request)
    }
}
