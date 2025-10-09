// src/api/handlers/encounter.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    AppState,
    domain::Encounter,
    service::ResourceService,
    api::{responses::{SuccessResponse, PaginatedResponse}, OptionalAuthUser},
};
use super::common::{SearchQuery, extract_optional_security_context};

/// Create a new encounter
pub async fn create_encounter(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Json(encounter): Json<Encounter>,
) -> Result<(StatusCode, Json<SuccessResponse<Encounter>>), crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let created = state.encounter_service.create(&context, encounter).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::new(created))))
}

/// Get an encounter by ID
pub async fn get_encounter(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SuccessResponse<Encounter>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let encounter = state.encounter_service.get(&context, &id).await?;
    Ok(Json(SuccessResponse::new(encounter)))
}

/// Update an encounter
pub async fn update_encounter(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(encounter): Json<Encounter>,
) -> Result<Json<SuccessResponse<Encounter>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let updated = state.encounter_service.update(&context, &id, encounter).await?;
    Ok(Json(SuccessResponse::new(updated)))
}

/// Delete an encounter
pub async fn delete_encounter(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    state.encounter_service.delete(&context, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Search encounters
#[derive(Debug, Deserialize)]
pub struct EncounterSearchQuery {
    #[serde(flatten)]
    pub common: SearchQuery,
    pub patient: Option<String>,
    pub status: Option<String>,
    #[serde(rename = "class")]
    pub class_: Option<String>,
}

pub async fn search_encounters(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Query(query): Query<EncounterSearchQuery>,
) -> Result<Json<PaginatedResponse<Encounter>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);

    // If searching for active encounters by patient
    if let Some(patient_id) = &query.patient {
        if query.status.as_deref() == Some("in-progress") {
            let encounters = state.encounter_service.get_active_encounters(&context, patient_id).await?;
            let count = encounters.len() as u32;
            return Ok(Json(PaginatedResponse::new(
                encounters,
                Some(count),
                0,
                count,
            )));
        }
    }

    // If searching by patient only
    if let Some(patient_id) = query.patient {
        let encounters = state.encounter_service.search_by_patient(&context, &patient_id).await?;
        let count = encounters.len() as u32;
        return Ok(Json(PaginatedResponse::new(
            encounters,
            Some(count),
            0,
            count,
        )));
    }

    // Otherwise use general search
    let params = query.common.into_search_params();
    let result = state.encounter_service.search(&context, params).await?;

    Ok(Json(PaginatedResponse::new(
        result.resources,
        result.total,
        result.offset,
        result.count,
    )))
}

/// Get encounter history
pub async fn get_encounter_history(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<SuccessResponse<Vec<Encounter>>>, crate::domain::errors::FhirError> {
    // TODO: Implement history tracking
    Ok(Json(SuccessResponse::new(vec![])))
}
