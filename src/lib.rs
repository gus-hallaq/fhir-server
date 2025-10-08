// src/lib.rs
// Library exports for the FHIR server

pub mod config;
pub mod domain;
pub mod repository;
pub mod service;

// Re-export commonly used types
pub use config::DatabaseConfig;
pub use domain::{
    // Primitives
    FhirString, Id, Uri, Code, FhirBoolean, FhirInteger, 
    FhirDecimal, FhirDate, FhirDateTime, Instant,
    
    // Datatypes
    Meta, Identifier, CodeableConcept, Coding, Reference,
    HumanName, ContactPoint, Address, Period, Quantity, Annotation,
    
    // Resources
    Patient, Observation, Condition, Encounter,
    
    // Errors
    FhirError, FhirResult,
};

pub use repository::{
    PatientRepository, ObservationRepository, 
    ConditionRepository, EncounterRepository,
    Repository, SearchParams, SearchFilter, SearchOperator,
};

pub use service::{
    PatientService, ObservationService,
    ConditionService, EncounterService,
    ResourceService, SearchParameters, SearchResult,
    Validator, PatientValidator, ObservationValidator,
    ConditionValidator, EncounterValidator,
};