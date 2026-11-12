//! Rust client for [ember](https://github.com/liquid-lightning/ember): a
//! gRPC service that provisions and manages per-user Core Lightning (CLN)
//! nodes.
//!
//! This crate wraps `ember.scheduler.v1.Scheduler`, ember's own schema for
//! node lifecycle (register/recover/status/list/delete). See [`Scheduler`]
//! for an ergonomic wrapper, or [`SchedulerClient`] for the raw generated
//! client.
//!
//! ```no_run
//! # async fn run() -> Result<(), ember_client::Error> {
//! let client = ember_client::EmberClient::connect(
//!     "http://localhost:50051",
//!     Some("user-123".to_string()),
//! )
//! .await?;
//!
//! let registered = client.scheduler().register("regtest", "").await?;
//! # let _ = registered;
//! # Ok(())
//! # }
//! ```

mod auth;
mod client;
mod error;
mod scheduler;

pub mod pb;

pub use client::{EmberClient, SchedulerClient, Transport};
pub use error::Error;
pub use scheduler::Scheduler;
