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
//!   caller's own node. Exposed directly as [`node::ClnClient`] (wrapped
//!   only with the auth interceptor every call needs) rather than
//!   hand-wrapping each RPC it defines.
//!
//! Named and shaped after Blockstream's `gl-client` (`Scheduler`,
//! `node::ClnClient`, `register`/`recover`/`node()`), so porting code
//! written against it needs little more than a crate-name change -
//! ember's trust model is custodial rather than client-signed, though, so
//! there's no signer here.
//!
//! ```no_run
//! # async fn run() -> Result<(), ember_client::Error> {
//! let client = ember_client::EmberClient::connect(
//!     "http://localhost:50051",
//!     Some("user-123".to_string()),
//! )
//! .await?;
//!
//! let scheduler = client.scheduler();
//! let registered = scheduler.register("regtest", "").await?;
//! let node = scheduler.node().await?;
//! # let _ = (registered, node);
//! # Ok(())
//! # }
//! ```

mod auth;
mod client;
mod error;
pub mod node;
mod scheduler;

pub mod pb;

pub use client::{EmberClient, SchedulerClient, Transport};
pub use error::Error;
pub use scheduler::Scheduler;
