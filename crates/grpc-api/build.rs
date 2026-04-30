fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let repo_proto_root = std::fs::canonicalize(manifest_dir.join("../../proto"))?;
    let auth_proto = std::fs::canonicalize(repo_proto_root.join("core/v1/auth.proto"))?;
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    // Build scripts run single-threaded here; setting PROTOC only scopes to this process.
    unsafe {
        std::env::set_var("PROTOC", protoc);
    }

    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            std::slice::from_ref(&auth_proto),
            std::slice::from_ref(&repo_proto_root),
        )?;

    println!("cargo:rerun-if-changed={}", auth_proto.display());
    Ok(())
}
