//! Registers a node against a running ember-server and fetches its CLN
//! `Getinfo`. Run with a live server, e.g.:
//!
//! ```sh
//! EMBER_ADDR=http://localhost:50051 EMBER_USER_ID=demo-user cargo run --example basic
//! ```

use ember_client::pb::cln::GetinfoRequest;
use ember_client::EmberClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("EMBER_ADDR").unwrap_or_else(|_| "http://localhost:50051".into());
    let user_id = std::env::var("EMBER_USER_ID").unwrap_or_else(|_| "demo-user".into());

    let client = EmberClient::connect(addr, Some(user_id)).await?;
    let scheduler = client.scheduler();

    let registered = scheduler.register("regtest", "").await?;
    println!(
        "registered node {}: {}",
        registered.node_id, registered.mnemonic
    );

    let mut node = scheduler.node().await?;
    let info = node.getinfo(GetinfoRequest {}).await?.into_inner();
    println!("{info:#?}");

    Ok(())
}
