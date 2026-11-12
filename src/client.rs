use std::sync::Arc;

use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
#[cfg(feature = "tls")]
use tonic::transport::ClientTlsConfig;

use crate::auth::UserIdInterceptor;
use crate::error::Error;
use crate::pb::ember::scheduler::v1::scheduler_client::SchedulerClient as GeneratedSchedulerClient;
use crate::Scheduler;

/// Transport ember-client's generated client runs over: a shared [`Channel`]
/// with the `x-user-id` interceptor attached.
pub type Transport = InterceptedService<Channel, UserIdInterceptor>;

/// gRPC client for ember's own schema (`ember.scheduler.v1.Scheduler`): node
/// lifecycle - register/recover/status/list/delete. See [`Scheduler`] for an
/// ergonomic wrapper over this.
pub type SchedulerClient = GeneratedSchedulerClient<Transport>;

/// Entry point: connects once to ember-server and hands out
/// [`SchedulerClient`]s that share the connection and caller identity.
#[derive(Clone)]
pub struct EmberClient {
    channel: Channel,
    user_id: Option<Arc<str>>,
}

impl EmberClient {
    /// Connects to `addr` (e.g. `http://localhost:50051`) over plain h2c -
    /// ember-server's external listener has no TLS of its own as of
    /// writing (see ember's README); use [`Self::connect_tls`] once/if it
    /// does. `user_id` is sent as the `x-user-id` metadata header on every
    /// call; pass `None` if the caller only needs calls that don't check it
    /// (`GetChallenge`, `GetNodeInfo`, `DeleteNode` as of writing - see
    /// ember's `bin/server/src/auth.rs`).
    pub async fn connect(
        addr: impl Into<String>,
        user_id: impl Into<Option<String>>,
    ) -> Result<Self, Error> {
        let addr = addr.into();
        let channel = Channel::from_shared(addr.clone())
            .map_err(|e| Error::invalid_uri(addr, e))?
            .connect()
            .await?;
        Ok(Self {
            channel,
            user_id: user_id.into().map(Arc::from),
        })
    }

    /// Like [`Self::connect`], but over TLS. Requires the `tls` feature.
    #[cfg(feature = "tls")]
    pub async fn connect_tls(
        addr: impl Into<String>,
        tls: ClientTlsConfig,
        user_id: impl Into<Option<String>>,
    ) -> Result<Self, Error> {
        let addr = addr.into();
        let channel = Channel::from_shared(addr.clone())
            .map_err(|e| Error::invalid_uri(addr, e))?
            .tls_config(tls)?
            .connect()
            .await?;
        Ok(Self {
            channel,
            user_id: user_id.into().map(Arc::from),
        })
    }

    /// Returns a fresh client for ember's own Scheduler service, sharing
    /// this connection - cheap to call repeatedly, since `Channel` is a
    /// cheaply cloneable handle rather than a new connection.
    pub fn scheduler_client(&self) -> SchedulerClient {
        GeneratedSchedulerClient::with_interceptor(self.channel.clone(), self.interceptor())
    }

    /// Returns an ergonomic wrapper over [`Self::scheduler_client`] - see
    /// [`Scheduler`].
    pub fn scheduler(&self) -> Scheduler {
        Scheduler::new(self.scheduler_client())
    }

    fn interceptor(&self) -> UserIdInterceptor {
        UserIdInterceptor::new(self.user_id.clone())
    }
}
