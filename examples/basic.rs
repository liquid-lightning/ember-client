//! Registers a node against a running ember-server. Run with a live server,
//! e.g.:
//!
//! ```sh
//! EMBER_ADDR=http://localhost:50051 EMBER_USER_ID=demo-user cargo run --example basic
//! ```

use ember_client::EmberClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("EMBER_ADDR").unwrap_or_else(|_| "http://localhost:50051".into());
    let user_id = std::env::var("EMBER_USER_ID").unwrap_or_else(|_| "demo-user".into());

    let client = EmberClient::connect(addr, Some(user_id)).await?;

    let registered = client.scheduler().register("regtest", "").await?;
    println!(
        "registered node {}: {}",
        registered.node_id, registered.mnemonic
    );

    Ok(())
}
