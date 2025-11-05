// src/grpc/server.rs
// gRPC server setup and startup

use tonic::transport::{Server, ServerTlsConfig, Identity};
use tonic_reflection::server::Builder as ReflectionBuilder;
use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::info;

use crate::AppState;
use crate::config::GrpcConfig;
use super::proto::{
    patient_service_server::PatientServiceServer,
    observation_service_server::ObservationServiceServer,
    condition_service_server::ConditionServiceServer,
    encounter_service_server::EncounterServiceServer,
    FILE_DESCRIPTOR_SET,
};
use super::services::{
    GrpcPatientService,
    GrpcObservationService,
    GrpcConditionService,
    GrpcEncounterService,
};

/// Start the gRPC server
pub async fn start_grpc_server(app_state: AppState, config: GrpcConfig) -> Result<()> {
    let addr = config.address().parse()?;
    let app_state = Arc::new(app_state);

    info!("🔌 Initializing gRPC services...");

    // Create service instances
    let patient_service = GrpcPatientService::new(app_state.clone());
    let observation_service = GrpcObservationService::new(app_state.clone());
    let condition_service = GrpcConditionService::new(app_state.clone());
    let encounter_service = GrpcEncounterService::new(app_state.clone());

    info!("✅ gRPC services initialized");

    // Build reflection service
    info!("🔍 Setting up gRPC reflection...");
    let reflection_service = ReflectionBuilder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()
        .context("Failed to build reflection service")?;
    info!("✅ gRPC reflection configured");

    // Build the server with optional TLS
    let mut server_builder = Server::builder();

    if config.tls_enabled {
        info!("🔒 TLS enabled for gRPC server");

        // Load TLS certificate and key
        let cert_path = config.tls_cert_path
            .as_ref()
            .context("TLS enabled but certificate path not provided")?;
        let key_path = config.tls_key_path
            .as_ref()
            .context("TLS enabled but key path not provided")?;

        let cert = tokio::fs::read(cert_path)
            .await
            .context(format!("Failed to read TLS certificate from {:?}", cert_path))?;
        let key = tokio::fs::read(key_path)
            .await
            .context(format!("Failed to read TLS key from {:?}", key_path))?;

        let identity = Identity::from_pem(cert, key);
        let tls_config = ServerTlsConfig::new().identity(identity);

        server_builder = Server::builder()
            .tls_config(tls_config)
            .context("Failed to configure TLS")?;

        info!("✅ TLS configured successfully");
        info!("📡 Starting secure gRPC server on {}", addr);
    } else {
        info!("⚠️  TLS disabled - gRPC server running without encryption");
        info!("📡 Starting gRPC server on {}", addr);
    }

    // Add services and start the server
    server_builder
        .add_service(reflection_service)
        .add_service(PatientServiceServer::new(patient_service))
        .add_service(ObservationServiceServer::new(observation_service))
        .add_service(ConditionServiceServer::new(condition_service))
        .add_service(EncounterServiceServer::new(encounter_service))
        .serve(addr)
        .await?;

    Ok(())
}

// ✅ Authentication enabled - JWT tokens extracted from gRPC metadata
// ✅ TLS enabled - Secure connections with configurable certificates
// ✅ gRPC reflection enabled - Service discovery for tools like grpcurl
// TODO: Add streaming operations for real-time updates