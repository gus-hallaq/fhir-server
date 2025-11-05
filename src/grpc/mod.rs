// src/grpc/mod.rs

// Generated protobuf code (compiled during build)
pub mod proto {
    tonic::include_proto!("fhir");

    // File descriptor set for gRPC reflection
    pub const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("proto_descriptor.bin");
}

pub mod auth;
pub mod converters;
pub mod services;
pub mod server;

pub use server::start_grpc_server;
