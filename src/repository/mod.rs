// src/repository/mod.rs

pub mod patient_repository;
pub mod observation_repository;
pub mod condition_repository;
pub mod encounter_repository;

pub use patient_repository::PatientRepository;
pub use observation_repository::ObservationRepository;
pub use condition_repository::ConditionRepository;
pub use encounter_repository::EncounterRepository;

use crate::domain::errors::FhirResult;

/// Base trait for all resource repositories
#[async_trait::async_trait]
pub trait Repository<T> {
    async fn create(&self, resource: &T) -> FhirResult<T>;
    async fn read(&self, id: &str) -> FhirResult<Option<T>>;
    async fn update(&self, id: &str, resource: &T) -> FhirResult<T>;
    async fn delete(&self, id: &str) -> FhirResult<()>;
    async fn search(&self, params: SearchParams) -> FhirResult<Vec<T>>;
}

/// Search parameters for FHIR queries
#[derive(Debug, Clone)]
pub struct SearchParams {
    pub filters: Vec<SearchFilter>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SearchFilter {
    pub field: String,
    pub operator: SearchOperator,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum SearchOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
}

impl SearchParams {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            limit: Some(100),
            offset: None,
        }
    }
    
    pub fn add_filter(mut self, field: String, operator: SearchOperator, value: String) -> Self {
        self.filters.push(SearchFilter {
            field,
            operator,
            value,
        });
        self
    }
    
    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }
    
    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }
}

impl Default for SearchParams {
    fn default() -> Self {
        Self::new()
    }
}