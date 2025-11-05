fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile proto files
    // Generated code will be placed in OUT_DIR and included via tonic::include_proto!
    // Also generate file descriptor set for gRPC reflection
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path("src/grpc/proto_descriptor.bin")
        .compile(&["proto/fhir.proto"], &["proto"])?;

    Ok(())
}
