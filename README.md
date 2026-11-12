# ember-client

A Rust client for [`ember`](https://github.com/liquid-lightning/ember), the
gRPC service that provisions and manages per-user Core Lightning (CLN)
nodes. This crate wraps ember's own schema,
`ember.scheduler.v1.Scheduler` - node lifecycle
(register/recover/status/list/delete) - vendored from
`proto/ember/scheduler.proto` (see `proto/ember/VENDORED.md`). Use
[`Scheduler`] for a typed wrapper over it, or [`SchedulerClient`] for the raw
generated client.

## Server

This crate is the client counterpart to
[`liquid-lightning/ember`](https://github.com/liquid-lightning/ember)'s
gRPC server (`ember-server`, `bin/server` in that repo). It only talks to
ember's own `Scheduler` service; it does not reach into the CLN pods
ember-server proxies calls to. Everything under `proto/ember/` here is
copied verbatim from that repo at the commit pinned in
`proto/ember/VENDORED.md` - re-copy it from there, don't hand-edit it here.

## Reference

This crate's shape - a single [`EmberClient::connect`] entry point handing
out a typed [`Scheduler`] wrapper over the generated gRPC client - was
referenced from Blockstream's [`gl-client`](https://github.com/Blockstream/greenlight)
crate, the equivalent client for Greenlight. ember's own naming
(`Scheduler`, `Register`/`Recover`/`GetNodeInfo`) follows gl-client's shape
too (see `proto/ember/scheduler.proto`), though ember's trust model is
custodial rather than client-signed, so there's no signer here.

## Auth

ember-server's auth is currently a trust-me `x-user-id` gRPC metadata header
with no signature or token validation (see ember's
`bin/server/src/auth.rs` - an open item on ember's side, not this crate's).
[`EmberClient::connect`] takes an optional `user_id` and attaches it to every
call; pass `None` for calls that don't check it (`GetChallenge`,
`GetNodeInfo`, `DeleteNode` as of writing).

## Example

```rust,no_run
use ember_client::EmberClient;

# async fn run() -> Result<(), ember_client::Error> {
let client = EmberClient::connect("http://localhost:50051", Some("user-123".to_string())).await?;

let registered = client.scheduler().register("regtest", "").await?;
# let _ = registered;
# Ok(())
# }
```

See `examples/basic.rs` for a runnable version.

## TLS

ember-server's external listener has no TLS of its own as of writing - plain
h2c, matching `EmberClient::connect`. If/when it does, enable the `tls`
feature and use `EmberClient::connect_tls` instead.

## Updating the vendored proto

`proto/ember/` is copied verbatim from `ember`, not generated independently
- see `proto/ember/VENDORED.md` for the exact source commit. Re-copy the
file and update that note rather than hand-editing anything under `proto/`.
