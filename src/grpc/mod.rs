// src/grpc/mod.rs

// Generated protobuf code (compiled during build)
pub mod proto {
    tonic::include_proto!("fhir");
}

pub mod converters;
pub mod services;
pub mod server;

pub use server::start_grpc_server;
