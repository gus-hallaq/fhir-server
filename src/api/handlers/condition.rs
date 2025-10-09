// src/api/handlers/condition.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::{
    AppState,
    domain::Condition,
    service::ResourceService,
    api::{responses::{SuccessResponse, PaginatedResponse}, OptionalAuthUser},
};
use super::common::{SearchQuery, extract_optional_security_context};

/// Create a new condition
pub async fn create_condition(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Json(condition): Json<Condition>,
) -> Result<(StatusCode, Json<SuccessResponse<Condition>>), crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let created = state.condition_service.create(&context, condition).await?;
    Ok((StatusCode::CREATED, Json(SuccessResponse::new(created))))
}

/// Get a condition by ID
pub async fn get_condition(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SuccessResponse<Condition>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let condition = state.condition_service.get(&context, &id).await?;
    Ok(Json(SuccessResponse::new(condition)))
}

/// Update a condition
pub async fn update_condition(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(condition): Json<Condition>,
) -> Result<Json<SuccessResponse<Condition>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    let updated = state.condition_service.update(&context, &id, condition).await?;
    Ok(Json(SuccessResponse::new(updated)))
}

/// Delete a condition
pub async fn delete_condition(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);
    state.condition_service.delete(&context, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Search conditions
#[derive(Debug, Deserialize)]
pub struct ConditionSearchQuery {
    #[serde(flatten)]
    pub common: SearchQuery,
    pub patient: Option<String>,
    pub code: Option<String>,
    #[serde(rename = "clinical-status")]
    pub clinical_status: Option<String>,
}

pub async fn search_conditions(
    auth: OptionalAuthUser,
    State(state): State<AppState>,
    Query(query): Query<ConditionSearchQuery>,
) -> Result<Json<PaginatedResponse<Condition>>, crate::domain::errors::FhirError> {
    let context = extract_optional_security_context(&auth);

    // If searching for active conditions by patient
    if let Some(patient_id) = &query.patient {
        if query.clinical_status.as_deref() == Some("active") {
            let conditions = state.condition_service.get_active_conditions(&context, patient_id).await?;
            let count = conditions.len() as u32;
            return Ok(Json(PaginatedResponse::new(
                conditions,
                Some(count),
                0,
                count,
            )));
        }
    }

    // If searching by patient only
    if let Some(patient_id) = query.patient {
        let conditions = state.condition_service.search_by_patient(&context, &patient_id).await?;
        let count = conditions.len() as u32;
        return Ok(Json(PaginatedResponse::new(
            conditions,
            Some(count),
            0,
            count,
        )));
    }

    // Otherwise use general search
    let params = query.common.into_search_params();
    let result = state.condition_service.search(&context, params).await?;

    Ok(Json(PaginatedResponse::new(
        result.resources,
        result.total,
        result.offset,
        result.count,
    )))
}

/// Get condition history
pub async fn get_condition_history(
    State(_state): State<AppState>,
    Path(_id): Path<String>,
) -> Result<Json<SuccessResponse<Vec<Condition>>>, crate::domain::errors::FhirError> {
    // TODO: Implement history tracking
    Ok(Json(SuccessResponse::new(vec![])))
}
