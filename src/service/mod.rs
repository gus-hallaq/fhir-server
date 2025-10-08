// src/service/mod.rs

pub mod patient_service;
pub mod observation_service;
pub mod condition_service;
pub mod encounter_service;
pub mod validation;
pub mod authorization;
pub mod authorization_rules;

pub use patient_service::PatientService;
pub use observation_service::ObservationService;
pub use condition_service::ConditionService;
pub use encounter_service::EncounterService;
pub use validation::*;
pub use authorization::*;
pub use authorization_rules::*;

use crate::domain::errors::FhirResult;

/// Base trait for all resource services
#[async_trait::async_trait]
pub trait ResourceService<T> {
    async fn create(&self, context: &SecurityContext, resource: T) -> FhirResult<T>;
    async fn get(&self, context: &SecurityContext, id: &str) -> FhirResult<T>;
    async fn update(&self, context: &SecurityContext, id: &str, resource: T) -> FhirResult<T>;
    async fn delete(&self, context: &SecurityContext, id: &str) -> FhirResult<()>;
    async fn search(&self, context: &SecurityContext, params: SearchParameters) -> FhirResult<SearchResult<T>>;
}

/// FHIR search parameters
#[derive(Debug, Clone, Default)]
pub struct SearchParameters {
    pub count: Option<u32>,      // _count parameter
    pub offset: Option<u32>,     // Pagination offset
    pub sort: Option<String>,    // _sort parameter
    pub filters: Vec<(String, String)>, // Key-value pairs for search
}

/// Search result with pagination info
#[derive(Debug, Clone)]
pub struct SearchResult<T> {
    pub resources: Vec<T>,
    pub total: Option<u32>,
    pub offset: u32,
    pub count: u32,
}

impl<T> SearchResult<T> {
    pub fn new(resources: Vec<T>, total: Option<u32>, offset: u32, count: u32) -> Self {
        Self {
            resources,
            total,
            offset,
            count,
        }
    }
}