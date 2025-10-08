// src/domain/errors.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FhirError {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Resource not found: {resource_type}/{id}")]
    NotFound {
        resource_type: String,
        id: String,
    },
    
    #[error("Invalid resource type: {0}")]
    InvalidResourceType(String),
    
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    
    #[error("Invalid reference: {0}")]
    InvalidReference(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Precondition failed: {0}")]
    PreconditionFailed(String),
    
    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Forbidden: {message}")]
    Forbidden {
        message: String,
    },
}

pub type FhirResult<T> = Result<T, FhirError>;