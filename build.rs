fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile proto files
    // Generated code will be placed in OUT_DIR and included via tonic::include_proto!
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile(&["proto/fhir.proto"], &["proto"])?;

    Ok(())
}
