**Not vendored from anywhere — this is a throwaway dev placeholder, not a
real credential.**

`ca.pem` / `app-cert.pem` / `app-key.pem` were generated locally by
`cargo run -p domain --example gen_dev_app_cert` in the `ember` repo, from a
freshly-generated CA that no running `ember-server` actually trusts. They
only let `ember-client` (and anything built against it, e.g. `lightcore`)
open a TLS connection and get as far as `GetChallenge`/`Register`/`Recover`
against *some* `ember-server`, for local development against a server
whose own CA happens to be seeded from the matching dev fixtures — or,
more usefully, so the crate compiles and its examples run at all without
a live deployment to pull real material from.

**Every real `ember-server` deployment has its own CA** (generated once,
stored in its `ca` table — see `domain::registry::Registry::load_or_create_ca`).
Before pointing `ember-client` at a real deployment, replace these three
files with that deployment's actual app cert, by running, against that
deployment's database:

```sh
ember-cli gen-app-cert --database-url <that deployment's DATABASE_URL> \
    --out-cert app-cert.pem --out-key app-key.pem
```

and copying that deployment's CA cert (`ca.ca_cert_pem()` — also readable
straight out of the `ca` table) into `ca.pem` alongside them.

These three paths are also overridable at build time without touching
this directory — see `EMBER_APP_CERT`/`EMBER_APP_KEY`/`EMBER_CA_CERT` in
`build.rs`, mirroring how `gl-client` lets `GL_CUSTOM_NOBODY_CERT`/
`GL_CUSTOM_NOBODY_KEY` override its own baked-in files.
