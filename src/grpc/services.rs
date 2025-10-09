// src/grpc/services.rs
// gRPC service implementations

use tonic::{Request, Response, Status};
use std::sync::Arc;

use crate::AppState;
use crate::service::{ResourceService, SecurityContext};
use super::proto;
use super::converters;

// Patient Service Implementation
pub struct GrpcPatientService {
    app_state: Arc<AppState>,
}

impl GrpcPatientService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl proto::patient_service_server::PatientService for GrpcPatientService {
    async fn create_patient(
        &self,
        request: Request<proto::CreatePatientRequest>,
    ) -> Result<Response<proto::CreatePatientResponse>, Status> {
        let proto_patient = request.into_inner().patient
            .ok_or_else(|| Status::invalid_argument("Patient is required"))?;

        let patient = converters::from_proto_patient(&proto_patient);

        // Create a system security context for gRPC requests
        // TODO: Extract actual security context from request metadata
        let security_context = SecurityContext::system();

        let created_patient = self.app_state.patient_service
            .create(&security_context, patient)
            .await
            .map_err(|e| Status::internal(format!("Failed to create patient: {}", e)))?;

        let response = proto::CreatePatientResponse {
            patient: Some(converters::to_proto_patient(&created_patient)),
        };

        Ok(Response::new(response))
    }

    async fn get_patient(
        &self,
        request: Request<proto::GetPatientRequest>,
    ) -> Result<Response<proto::GetPatientResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        let patient = self.app_state.patient_service
            .get(&security_context, id)
            .await
            .map_err(|e| Status::not_found(format!("Patient not found: {}", e)))?;

        let response = proto::GetPatientResponse {
            patient: Some(converters::to_proto_patient(&patient)),
        };

        Ok(Response::new(response))
    }

    async fn update_patient(
        &self,
        request: Request<proto::UpdatePatientRequest>,
    ) -> Result<Response<proto::UpdatePatientResponse>, Status> {
        let req = request.into_inner();
        let proto_patient = req.patient
            .ok_or_else(|| Status::invalid_argument("Patient is required"))?;

        let patient = converters::from_proto_patient(&proto_patient);
        let security_context = SecurityContext::system();

        let updated_patient = self.app_state.patient_service
            .update(&security_context, &req.id, patient)
            .await
            .map_err(|e| Status::internal(format!("Failed to update patient: {}", e)))?;

        let response = proto::UpdatePatientResponse {
            patient: Some(converters::to_proto_patient(&updated_patient)),
        };

        Ok(Response::new(response))
    }

    async fn delete_patient(
        &self,
        request: Request<proto::DeletePatientRequest>,
    ) -> Result<Response<proto::DeletePatientResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        self.app_state.patient_service
            .delete(&security_context, id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete patient: {}", e)))?;

        let response = proto::DeletePatientResponse {
            success: true,
        };

        Ok(Response::new(response))
    }

    async fn search_patients(
        &self,
        request: Request<proto::SearchPatientsRequest>,
    ) -> Result<Response<proto::SearchPatientsResponse>, Status> {
        let req = request.into_inner();
        let security_context = SecurityContext::system();

        // For now, only implement family name search
        // TODO: Implement other search parameters
        let patients = if let Some(family) = req.family {
            self.app_state.patient_service
                .search_by_family(&security_context, &family)
                .await
                .map_err(|e| Status::internal(format!("Search failed: {}", e)))?
        } else {
            vec![]
        };

        let response = proto::SearchPatientsResponse {
            patients: patients.iter().map(converters::to_proto_patient).collect(),
        };

        Ok(Response::new(response))
    }

    async fn get_patient_history(
        &self,
        request: Request<proto::GetPatientHistoryRequest>,
    ) -> Result<Response<proto::GetPatientHistoryResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        let history = self.app_state.patient_service
            .get_history(&security_context, id)
            .await
            .map_err(|e| Status::internal(format!("Failed to get history: {}", e)))?;

        let response = proto::GetPatientHistoryResponse {
            versions: history.iter().map(converters::to_proto_patient).collect(),
        };

        Ok(Response::new(response))
    }
}

// Observation Service Implementation
pub struct GrpcObservationService {
    app_state: Arc<AppState>,
}

impl GrpcObservationService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl proto::observation_service_server::ObservationService for GrpcObservationService {
    async fn create_observation(
        &self,
        request: Request<proto::CreateObservationRequest>,
    ) -> Result<Response<proto::CreateObservationResponse>, Status> {
        let proto_observation = request.into_inner().observation
            .ok_or_else(|| Status::invalid_argument("Observation is required"))?;

        let observation = converters::from_proto_observation(&proto_observation);
        let security_context = SecurityContext::system();

        let created_observation = self.app_state.observation_service
            .create(&security_context, observation)
            .await
            .map_err(|e| Status::internal(format!("Failed to create observation: {}", e)))?;

        let response = proto::CreateObservationResponse {
            observation: Some(converters::to_proto_observation(&created_observation)),
        };

        Ok(Response::new(response))
    }

    async fn get_observation(
        &self,
        request: Request<proto::GetObservationRequest>,
    ) -> Result<Response<proto::GetObservationResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        let observation = self.app_state.observation_service
            .get(&security_context, id)
            .await
            .map_err(|e| Status::not_found(format!("Observation not found: {}", e)))?;

        let response = proto::GetObservationResponse {
            observation: Some(converters::to_proto_observation(&observation)),
        };

        Ok(Response::new(response))
    }

    async fn update_observation(
        &self,
        request: Request<proto::UpdateObservationRequest>,
    ) -> Result<Response<proto::UpdateObservationResponse>, Status> {
        let req = request.into_inner();
        let proto_observation = req.observation
            .ok_or_else(|| Status::invalid_argument("Observation is required"))?;

        let observation = converters::from_proto_observation(&proto_observation);
        let security_context = SecurityContext::system();

        let updated_observation = self.app_state.observation_service
            .update(&security_context, &req.id, observation)
            .await
            .map_err(|e| Status::internal(format!("Failed to update observation: {}", e)))?;

        let response = proto::UpdateObservationResponse {
            observation: Some(converters::to_proto_observation(&updated_observation)),
        };

        Ok(Response::new(response))
    }

    async fn delete_observation(
        &self,
        request: Request<proto::DeleteObservationRequest>,
    ) -> Result<Response<proto::DeleteObservationResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        self.app_state.observation_service
            .delete(&security_context, id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete observation: {}", e)))?;

        let response = proto::DeleteObservationResponse {
            success: true,
        };

        Ok(Response::new(response))
    }

    async fn search_observations(
        &self,
        request: Request<proto::SearchObservationsRequest>,
    ) -> Result<Response<proto::SearchObservationsResponse>, Status> {
        let req = request.into_inner();
        let security_context = SecurityContext::system();

        let observations = if let Some(patient_id) = req.patient {
            self.app_state.observation_service
                .search_by_patient(&security_context, &patient_id)
                .await
                .map_err(|e| Status::internal(format!("Search failed: {}", e)))?
        } else {
            vec![]
        };

        let response = proto::SearchObservationsResponse {
            observations: observations.iter().map(converters::to_proto_observation).collect(),
        };

        Ok(Response::new(response))
    }
}

// Condition Service Implementation
pub struct GrpcConditionService {
    app_state: Arc<AppState>,
}

impl GrpcConditionService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl proto::condition_service_server::ConditionService for GrpcConditionService {
    async fn create_condition(
        &self,
        request: Request<proto::CreateConditionRequest>,
    ) -> Result<Response<proto::CreateConditionResponse>, Status> {
        let proto_condition = request.into_inner().condition
            .ok_or_else(|| Status::invalid_argument("Condition is required"))?;

        let condition = converters::from_proto_condition(&proto_condition);
        let security_context = SecurityContext::system();

        let created_condition = self.app_state.condition_service
            .create(&security_context, condition)
            .await
            .map_err(|e| Status::internal(format!("Failed to create condition: {}", e)))?;

        let response = proto::CreateConditionResponse {
            condition: Some(converters::to_proto_condition(&created_condition)),
        };

        Ok(Response::new(response))
    }

    async fn get_condition(
        &self,
        request: Request<proto::GetConditionRequest>,
    ) -> Result<Response<proto::GetConditionResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        let condition = self.app_state.condition_service
            .get(&security_context, id)
            .await
            .map_err(|e| Status::not_found(format!("Condition not found: {}", e)))?;

        let response = proto::GetConditionResponse {
            condition: Some(converters::to_proto_condition(&condition)),
        };

        Ok(Response::new(response))
    }

    async fn update_condition(
        &self,
        request: Request<proto::UpdateConditionRequest>,
    ) -> Result<Response<proto::UpdateConditionResponse>, Status> {
        let req = request.into_inner();
        let proto_condition = req.condition
            .ok_or_else(|| Status::invalid_argument("Condition is required"))?;

        let condition = converters::from_proto_condition(&proto_condition);
        let security_context = SecurityContext::system();

        let updated_condition = self.app_state.condition_service
            .update(&security_context, &req.id, condition)
            .await
            .map_err(|e| Status::internal(format!("Failed to update condition: {}", e)))?;

        let response = proto::UpdateConditionResponse {
            condition: Some(converters::to_proto_condition(&updated_condition)),
        };

        Ok(Response::new(response))
    }

    async fn delete_condition(
        &self,
        request: Request<proto::DeleteConditionRequest>,
    ) -> Result<Response<proto::DeleteConditionResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        self.app_state.condition_service
            .delete(&security_context, id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete condition: {}", e)))?;

        let response = proto::DeleteConditionResponse {
            success: true,
        };

        Ok(Response::new(response))
    }

    async fn search_conditions(
        &self,
        request: Request<proto::SearchConditionsRequest>,
    ) -> Result<Response<proto::SearchConditionsResponse>, Status> {
        let req = request.into_inner();
        let security_context = SecurityContext::system();

        let conditions = if let Some(patient_id) = req.patient {
            self.app_state.condition_service
                .get_active_conditions(&security_context, &patient_id)
                .await
                .map_err(|e| Status::internal(format!("Search failed: {}", e)))?
        } else {
            vec![]
        };

        let response = proto::SearchConditionsResponse {
            conditions: conditions.iter().map(converters::to_proto_condition).collect(),
        };

        Ok(Response::new(response))
    }
}

// Encounter Service Implementation
pub struct GrpcEncounterService {
    app_state: Arc<AppState>,
}

impl GrpcEncounterService {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self { app_state }
    }
}

#[tonic::async_trait]
impl proto::encounter_service_server::EncounterService for GrpcEncounterService {
    async fn create_encounter(
        &self,
        request: Request<proto::CreateEncounterRequest>,
    ) -> Result<Response<proto::CreateEncounterResponse>, Status> {
        let proto_encounter = request.into_inner().encounter
            .ok_or_else(|| Status::invalid_argument("Encounter is required"))?;

        let encounter = converters::from_proto_encounter(&proto_encounter);
        let security_context = SecurityContext::system();

        let created_encounter = self.app_state.encounter_service
            .create(&security_context, encounter)
            .await
            .map_err(|e| Status::internal(format!("Failed to create encounter: {}", e)))?;

        let response = proto::CreateEncounterResponse {
            encounter: Some(converters::to_proto_encounter(&created_encounter)),
        };

        Ok(Response::new(response))
    }

    async fn get_encounter(
        &self,
        request: Request<proto::GetEncounterRequest>,
    ) -> Result<Response<proto::GetEncounterResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        let encounter = self.app_state.encounter_service
            .get(&security_context, id)
            .await
            .map_err(|e| Status::not_found(format!("Encounter not found: {}", e)))?;

        let response = proto::GetEncounterResponse {
            encounter: Some(converters::to_proto_encounter(&encounter)),
        };

        Ok(Response::new(response))
    }

    async fn update_encounter(
        &self,
        request: Request<proto::UpdateEncounterRequest>,
    ) -> Result<Response<proto::UpdateEncounterResponse>, Status> {
        let req = request.into_inner();
        let proto_encounter = req.encounter
            .ok_or_else(|| Status::invalid_argument("Encounter is required"))?;

        let encounter = converters::from_proto_encounter(&proto_encounter);
        let security_context = SecurityContext::system();

        let updated_encounter = self.app_state.encounter_service
            .update(&security_context, &req.id, encounter)
            .await
            .map_err(|e| Status::internal(format!("Failed to update encounter: {}", e)))?;

        let response = proto::UpdateEncounterResponse {
            encounter: Some(converters::to_proto_encounter(&updated_encounter)),
        };

        Ok(Response::new(response))
    }

    async fn delete_encounter(
        &self,
        request: Request<proto::DeleteEncounterRequest>,
    ) -> Result<Response<proto::DeleteEncounterResponse>, Status> {
        let id = &request.into_inner().id;
        let security_context = SecurityContext::system();

        self.app_state.encounter_service
            .delete(&security_context, id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete encounter: {}", e)))?;

        let response = proto::DeleteEncounterResponse {
            success: true,
        };

        Ok(Response::new(response))
    }

    async fn search_encounters(
        &self,
        request: Request<proto::SearchEncountersRequest>,
    ) -> Result<Response<proto::SearchEncountersResponse>, Status> {
        let req = request.into_inner();
        let security_context = SecurityContext::system();

        let encounters = if let Some(patient_id) = req.patient {
            self.app_state.encounter_service
                .get_active_encounters(&security_context, &patient_id)
                .await
                .map_err(|e| Status::internal(format!("Search failed: {}", e)))?
        } else {
            vec![]
        };

        let response = proto::SearchEncountersResponse {
            encounters: encounters.iter().map(converters::to_proto_encounter).collect(),
        };

        Ok(Response::new(response))
    }
}
