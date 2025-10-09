// src/grpc/server.rs
// gRPC server setup and startup

use tonic::transport::Server;
use std::sync::Arc;
use anyhow::Result;
use tracing::info;

use crate::AppState;
use super::proto::{
    patient_service_server::PatientServiceServer,
    observation_service_server::ObservationServiceServer,
    condition_service_server::ConditionServiceServer,
    encounter_service_server::EncounterServiceServer,
};
use super::services::{
    GrpcPatientService,
    GrpcObservationService,
    GrpcConditionService,
    GrpcEncounterService,
};

/// Start the gRPC server
pub async fn start_grpc_server(app_state: AppState, addr: &str) -> Result<()> {
    let addr = addr.parse()?;
    let app_state = Arc::new(app_state);

    info!("🔌 Initializing gRPC services...");

    // Create service instances
    let patient_service = GrpcPatientService::new(app_state.clone());
    let observation_service = GrpcObservationService::new(app_state.clone());
    let condition_service = GrpcConditionService::new(app_state.clone());
    let encounter_service = GrpcEncounterService::new(app_state.clone());

    info!("✅ gRPC services initialized");
    info!("📡 Starting gRPC server on {}", addr);

    // Build and start the server
    Server::builder()
        .add_service(PatientServiceServer::new(patient_service))
        .add_service(ObservationServiceServer::new(observation_service))
        .add_service(ConditionServiceServer::new(condition_service))
        .add_service(EncounterServiceServer::new(encounter_service))
        .serve(addr)
        .await?;

    Ok(())
}
