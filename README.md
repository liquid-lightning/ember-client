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
`proto/ember/scheduler.proto`). ember's *node* itself is custodial — the
real CLN signing key is derived server-side from the mnemonic you register
with, there's no commitment-signing HSM in this crate — but authentication
to ember-server mirrors Greenlight's shape: [`auth::AuthKey`] is a
seed-derived identity you sign a server challenge with, in exchange for a
CA-issued device certificate, replacing a bearer token with mTLS the same
way Greenlight's device credentials do. `Scheduler::node()` never actually
fails - it's `async`/fallible only to match gl-client's call shape.

## Auth

ember-server's public listener requires mTLS: every call presents a
certificate chaining to ember's own CA (see ember's
`domain::ca::CertificateAuthority` and `bin/server/src/main.rs`) - there is
no bearer token or metadata header. [`EmberClient::connect`] takes a
[`ClientTlsIdentity`] (cert + key + the CA cert to verify the server
against):

- For `GetChallenge`/`Register`/`Recover` - before any node/device
  certificate exists - use the baked-in pre-auth "app" identity,
  [`default_app_identity`] (backed by `.resources/tls/`, see that
  directory's `VENDORED.md` for what it actually is and how to replace it
  for a real deployment).
- [`auth::AuthKey::from_seed`] derives a secp256k1 identity from the
  caller's BIP39 seed; [`Scheduler::register`]/[`Scheduler::recover`] use it
  to sign a server challenge and request a device certificate via CSR
  (`auth::generate_device_csr`) - ember's whole proof that the caller
  controls the seed it claims to.
- Everything after - node-lifecycle calls and the `cln.Node` proxy -
  reconnects with the returned device identity instead; no seed needed
  again.

## Example

```rust,no_run
use ember_client::auth::AuthKey;
use ember_client::EmberClient;

# async fn run() -> Result<(), ember_client::Error> {
let app_identity = ember_client::default_app_identity();
let app_client = EmberClient::connect("https://localhost:50051", app_identity.clone()).await?;

let seed = [0u8; 32]; // Mnemonic::to_seed(passphrase), in practice.
let auth_key = AuthKey::from_seed(&seed)?;
let registered = app_client
    .scheduler()
    .register(&auth_key, "regtest", "my twenty four word mnemonic ...", "", app_identity.ca_cert_pem)
    .await?;

// Reconnect with the device identity just issued - no seed needed again.
let node_client = EmberClient::connect("https://localhost:50051", registered.device_identity).await?;
let node = node_client.scheduler().node().await?;
# let _ = node;
# Ok(())
# }
```

See `examples/basic.rs` for a runnable version.

## Updating the vendored protos

`proto/cln/` and `proto/ember/` are both copied verbatim, not generated
independently - see each directory's `VENDORED.md` for the exact source
commit. Re-copy the files and update that note rather than hand-editing
anything under `proto/`.
