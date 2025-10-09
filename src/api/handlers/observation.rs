// src/api/handlers/observation.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    AppState,
    domain::Observation,
    service::ResourceService,
    api::{responses::{SuccessResponse, PaginatedResponse}, OptionalAuthUser},
};
use super::common::{SearchQuery, extract_optional_security_context};

/// Create a new observation
pub async fn create_observation(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Json(observation): Json<Observation>,
) -> Result<(StatusCode, Json<SuccessResponse<Observation>>), crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let created = state.observation_service.create(&context, observation).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::new(created))))
}

/// Get an observation by ID
pub async fn get_observation(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SuccessResponse<Observation>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let observation = state.observation_service.get(&context, &id).await?;
    Ok(Json(SuccessResponse::new(observation)))
}

/// Update an observation
pub async fn update_observation(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(observation): Json<Observation>,
) -> Result<Json<SuccessResponse<Observation>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let updated = state.observation_service.update(&context, &id, observation).await?;
    Ok(Json(SuccessResponse::new(updated)))
}

/// Delete an observation
pub async fn delete_observation(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    state.observation_service.delete(&context, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Search observations
#[derive(Debug, Deserialize)]
pub struct ObservationSearchQuery {
    #[serde(flatten)]
    pub common: SearchQuery,
    pub patient: Option<String>,
    pub code: Option<String>,
    pub category: Option<String>,
}

pub async fn search_observations(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Query(query): Query<ObservationSearchQuery>,
) -> Result<Json<PaginatedResponse<Observation>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);

    // If searching by patient, use specific method
    if let Some(patient_id) = query.patient {
        let observations = state.observation_service.search_by_patient(&context, &patient_id).await?;
        let count = observations.len() as u32;
        return Ok(Json(PaginatedResponse::new(
            observations,
            Some(count),
            0,
            count,
        )));
    }

    // If searching by code, use specific method
    if let Some(code) = query.code {
        let observations = state.observation_service.search_by_code(&context, &code).await?;
        let count = observations.len() as u32;
        return Ok(Json(PaginatedResponse::new(
            observations,
            Some(count),
            0,
            count,
        )));
    }

    // Otherwise use general search
    let params = query.common.into_search_params();
    let result = state.observation_service.search(&context, params).await?;

    Ok(Json(PaginatedResponse::new(
        result.resources,
        result.total,
        result.offset,
        result.count,
    )))
}

/// Get observation history
pub async fn get_observation_history(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<SuccessResponse<Vec<Observation>>>, crate::domain::errors::FhirError> {
    // TODO: Implement history tracking
    Ok(Json(SuccessResponse::new(vec![])))
}
