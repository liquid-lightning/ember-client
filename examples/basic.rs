//! Registers a node against a running ember-server and fetches its CLN
//! `Getinfo`. Run with a live server, e.g.:
//!
//! ```sh
//! EMBER_ADDR=https://localhost:50051 cargo run --example basic
//! ```
//!
//! Requires the pre-auth "app" identity in `.resources/tls/` to actually
//! chain to that server's CA — see `.resources/tls/VENDORED.md` and
//! `EMBER_APP_CERT`/`EMBER_APP_KEY`/`EMBER_CA_CERT` in `build.rs` for
//! pointing this at a real deployment instead of the checked-in dev
//! placeholder.

use bip39::{Language, Mnemonic};
use ember_client::auth::AuthKey;
use ember_client::pb::cln::GetinfoRequest;
use ember_client::EmberClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("EMBER_ADDR").unwrap_or_else(|_| "https://localhost:50051".into());

    let app_identity = ember_client::default_app_identity();
    let app_client = EmberClient::connect(addr.clone(), app_identity.clone()).await?;

    let mut entropy = [0u8; 32];
    secp256k1::rand::RngCore::fill_bytes(&mut secp256k1::rand::thread_rng(), &mut entropy);
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
    let seed = mnemonic.to_seed("");
    let auth_key = AuthKey::from_seed(&seed[..32])?;

    let registered = app_client
        .scheduler()
        .register(
            &auth_key,
            "liquid-testnet",
            mnemonic.to_string(),
            "",
            app_identity.ca_cert_pem.clone(),
        )
        .await?;
    println!(
        "registered node {}: {}",
        registered.node_id, registered.mnemonic
    );

    // No seed needed again from here on - just the device identity.
    let node_client = EmberClient::connect(addr, registered.device_identity).await?;

    // Register returns as soon as the node's StatefulSet exists, not once
    // lightningd is up - the first proxied call races the pod's startup and
    // gets Unavailable ("connecting to node: transport error"). Retry until
    // CLN answers.
    const ATTEMPTS: u32 = 30;
    const DELAY: std::time::Duration = std::time::Duration::from_secs(5);
    let mut attempt = 1;
    let info = loop {
        let mut node = node_client.scheduler().node().await?;
        match node.getinfo(GetinfoRequest {}).await {
            Ok(resp) => break resp.into_inner(),
            Err(status) if status.code() == tonic::Code::Unavailable && attempt < ATTEMPTS => {
                eprintln!(
                    "node not ready yet ({}), retry {attempt}/{ATTEMPTS} in {}s",
                    status.message(),
                    DELAY.as_secs()
                );
                attempt += 1;
                tokio::time::sleep(DELAY).await;
            }
            Err(status) => return Err(status.into()),
        }
    };
    println!("{info:#?}");

    Ok(())
}
