# ember-client

A Rust client for [`ember`](https://github.com/liquid-lightning/ember), the
gRPC service that provisions and manages per-user Core Lightning (CLN)
nodes. This crate wraps both of ember-server's gRPC services:

- **`ember.scheduler.v1.Scheduler`** - node lifecycle
  (register/recover/status/list/delete), ember's own schema, vendored from
  `proto/ember/scheduler.proto` (see `proto/ember/VENDORED.md`). Use
  [`Scheduler`] for a typed wrapper over it, or [`SchedulerClient`] for the
  raw generated client.
- **`cln.Node`** - Core Lightning's own `node.proto`, proxied by
  ember-server to the caller's own node, vendored verbatim in `proto/cln/`
  (see `proto/cln/VENDORED.md`). Rather than hand-wrapping each RPC it
  defines, this crate exposes the full tonic-generated client directly as
  [`node::ClnClient`] - get one from [`Scheduler::node`].

## Server

This crate is the client counterpart to
[`liquid-lightning/ember`](https://github.com/liquid-lightning/ember)'s
gRPC server (`ember-server`, `bin/server` in that repo). Everything under
`proto/` here is copied verbatim from that repo (for `proto/cln/`, from CLN
upstream by way of ember) at the commits pinned in each directory's
`VENDORED.md` - re-copy from there, don't hand-edit anything under `proto/`.

## Reference

This crate's shape was referenced from Blockstream's
[`gl-client`](https://github.com/Blockstream/greenlight) crate, the
equivalent client for Greenlight: a [`Scheduler`] with
`register`/`recover`/`node()`, where `node()` hands back a CLN client
(gl-client's `node::ClnClient`, here [`node::ClnClient`]) for the caller's
own node. ember's own naming (`Scheduler`,
`Register`/`Recover`/`GetNodeInfo`) follows gl-client's shape too (see
`proto/ember/scheduler.proto`), though ember's trust model is custodial
rather than client-signed, so there's no signer here, and `node()` never
actually fails - it's `async`/fallible only to match gl-client's call
shape.

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

let scheduler = client.scheduler();
let registered = scheduler.register("regtest", "").await?;
let node = scheduler.node().await?;
# let _ = (registered, node);
# Ok(())
# }
```

See `examples/basic.rs` for a runnable version.

## TLS

ember-server's external listener has no TLS of its own as of writing - plain
h2c, matching `EmberClient::connect`. If/when it does, enable the `tls`
feature and use `EmberClient::connect_tls` instead.

## Updating the vendored protos

`proto/cln/` and `proto/ember/` are both copied verbatim, not generated
independently - see each directory's `VENDORED.md` for the exact source
commit. Re-copy the files and update that note rather than hand-editing
anything under `proto/`.
