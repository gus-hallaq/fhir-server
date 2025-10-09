// src/api/handlers/patient.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    AppState,
    domain::Patient,
    service::ResourceService,
    api::{responses::{SuccessResponse, PaginatedResponse}, OptionalAuthUser},
};
use super::common::{SearchQuery, extract_optional_security_context};

/// Create a new patient
pub async fn create_patient(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Json(patient): Json<Patient>,
) -> Result<(StatusCode, Json<SuccessResponse<Patient>>), crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let created = state.patient_service.create(&context, patient).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::new(created))))
}

/// Get a patient by ID
pub async fn get_patient(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SuccessResponse<Patient>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let patient = state.patient_service.get(&context, &id).await?;
    Ok(Json(SuccessResponse::new(patient)))
}

/// Update a patient
pub async fn update_patient(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(patient): Json<Patient>,
) -> Result<Json<SuccessResponse<Patient>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let updated = state.patient_service.update(&context, &id, patient).await?;
    Ok(Json(SuccessResponse::new(updated)))
}

/// Delete a patient
pub async fn delete_patient(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    state.patient_service.delete(&context, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Search patients
#[derive(Debug, Deserialize)]
pub struct PatientSearchQuery {
    #[serde(flatten)]
    pub common: SearchQuery,
    pub family: Option<String>,
    pub given: Option<String>,
    pub identifier: Option<String>,
}

pub async fn search_patients(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Query(query): Query<PatientSearchQuery>,
) -> Result<Json<PaginatedResponse<Patient>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);

    // If searching by family name, use specific method
    if let Some(family) = query.family {
        let patients = state.patient_service.search_by_family(&context, &family).await?;
        let count = patients.len() as u32;
        return Ok(Json(PaginatedResponse::new(
            patients,
            Some(count),
            0,
            count,
        )));
    }

    // Otherwise use general search
    let params = query.common.into_search_params();
    let result = state.patient_service.search(&context, params).await?;

    Ok(Json(PaginatedResponse::new(
        result.resources,
        result.total,
        result.offset,
        result.count,
    )))
}

/// Get patient history
pub async fn get_patient_history(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SuccessResponse<Vec<Patient>>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let history = state.patient_service.get_history(&context, &id).await?;
    Ok(Json(SuccessResponse::new(history)))
}
