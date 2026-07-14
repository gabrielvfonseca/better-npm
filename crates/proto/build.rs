fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = std::path::Path::new("../../proto");

    // Compile common first (messages only)
    tonic_build::configure()
        .build_server(false)
        .build_client(false)
        .compile_protos(
            &[proto_dir.join("common/v1/common.proto")],
            &[proto_dir],
        )?;

    // Then compile dpr - NO extern_path for common (we'll provide the module manually)
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &[proto_dir.join("dpr/v1/dpr.proto")],
            &[proto_dir],
        )?;

    Ok(())
}