use std::env::var;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The rerun-if-env-changed lines below replace cargo's default (rerun on
    // any change in the package), so the protos must be named explicitly.
    println!("cargo:rerun-if-changed=proto");
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &[
                "proto/cln/node.proto",
                "proto/cln/assetswap.proto",
                "proto/ember/scheduler.proto",
                "proto/ember/assets.proto",
            ],
            &["proto/cln", "proto/ember"],
        )?;

    // The baked-in pre-auth "app" identity (see `src/auth.rs` and
    // `.resources/tls/VENDORED.md`) is deployment-specific — every real
    // `ember-server` has its own CA. `EMBER_APP_CERT`/`EMBER_APP_KEY`/
    // `EMBER_CA_CERT` let a build point at that deployment's real material
    // (from `ember-cli gen-app-cert`) instead of the checked-in dev
    // placeholder, mirroring how `gl-client` lets `GL_CUSTOM_NOBODY_CERT`/
    // `GL_CUSTOM_NOBODY_KEY` override its own baked-in files.
    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap();
    for (env_var, rustc_env, default_relpath) in [
        ("EMBER_APP_CERT", "EMBER_APP_CERT_PATH", "app-cert.pem"),
        ("EMBER_APP_KEY", "EMBER_APP_KEY_PATH", "app-key.pem"),
        ("EMBER_CA_CERT", "EMBER_CA_CERT_PATH", "ca.pem"),
    ] {
        println!("cargo:rerun-if-env-changed={env_var}");
        let path = var(env_var)
            .unwrap_or_else(|_| format!("{manifest_dir}/.resources/tls/{default_relpath}"));
        println!("cargo:rustc-env={rustc_env}={path}");
    }

    Ok(())
}
