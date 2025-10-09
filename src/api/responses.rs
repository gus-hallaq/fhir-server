// src/api/responses.rs

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use crate::domain::errors::FhirError;

/// Standard error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Convert FhirError to HTTP response
impl IntoResponse for FhirError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            FhirError::NotFound { .. } => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            FhirError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR"),
            FhirError::Forbidden { .. } => (StatusCode::FORBIDDEN, "FORBIDDEN"),
            FhirError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            FhirError::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, "SERIALIZATION_ERROR"),
            FhirError::InvalidResourceType(_) => (StatusCode::BAD_REQUEST, "INVALID_RESOURCE_TYPE"),
            FhirError::MissingRequiredField(_) => (StatusCode::BAD_REQUEST, "MISSING_REQUIRED_FIELD"),
            FhirError::InvalidReference(_) => (StatusCode::BAD_REQUEST, "INVALID_REFERENCE"),
            FhirError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT"),
            FhirError::PreconditionFailed(_) => (StatusCode::PRECONDITION_FAILED, "PRECONDITION_FAILED"),
            FhirError::UnprocessableEntity(_) => (StatusCode::UNPROCESSABLE_ENTITY, "UNPROCESSABLE_ENTITY"),
        };

        let error_response = ErrorResponse::new(error_type, self.to_string());

        (status, Json(error_response)).into_response()
    }
}

/// Success response wrapper
#[derive(Debug, Serialize)]
pub struct SuccessResponse<T: Serialize> {
    pub data: T,
}

impl<T: Serialize> SuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

/// Paginated response
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: Option<u32>,
    pub offset: u32,
    pub count: u32,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, total: Option<u32>, offset: u32, count: u32) -> Self {
        Self {
            data,
            total,
            offset,
            count,
        }
    }
}
