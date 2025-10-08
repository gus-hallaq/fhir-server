// src/service/observation_service.rs

use crate::domain::{Observation, FhirError, FhirResult};
use crate::repository::{ObservationRepository, Repository, SearchParams};
use crate::service::{
    ResourceService, SearchParameters, SearchResult, Validator, ObservationValidator,
    SecurityContext, ObservationAuthorizationRules,
};

pub struct ObservationService {
    repository: ObservationRepository,
    validator: ObservationValidator,
    auth_rules: ObservationAuthorizationRules,
}

impl ObservationService {
    pub fn new(repository: ObservationRepository) -> Self {
        Self {
            repository,
            validator: ObservationValidator,
            auth_rules: ObservationAuthorizationRules::new(),
        }
    }
    
    /// Validate and create a new observation
    async fn validate_and_create(
        &self,
        context: &SecurityContext,
        observation: Observation,
    ) -> FhirResult<Observation> {
        // Check authorization
        self.auth_rules.can_create(context, &observation)?;

        // Validate the observation
        self.validator.validate(&observation)?;

        // Validate subject reference if present
        if let Some(subject) = &observation.subject {
            if let Some(reference) = &subject.reference {
                self.validate_reference(&reference.0).await?;
            }
        }

        // Create the observation
        self.repository.create(&observation).await
    }
    
    /// Validate that a reference exists
    async fn validate_reference(&self, reference: &str) -> FhirResult<()> {
        // Parse reference (e.g., "Patient/123")
        let parts: Vec<&str> = reference.split('/').collect();
        if parts.len() != 2 {
            return Err(FhirError::InvalidReference(
                format!("Invalid reference format: {}", reference)
            ));
        }
        
        // For now, just validate format
        // In a full implementation, you would check if the referenced resource exists
        Ok(())
    }
    
    /// Search observations by patient
    pub async fn search_by_patient(
        &self,
        context: &SecurityContext,
        patient_id: &str,
    ) -> FhirResult<Vec<Observation>> {
        if patient_id.trim().is_empty() {
            return Err(FhirError::Validation("Patient ID cannot be empty".to_string()));
        }

        // Check authorization
        self.auth_rules.can_search(context, Some(patient_id))?;

        self.repository.search_by_patient(patient_id).await
    }

    /// Search observations by code
    pub async fn search_by_code(
        &self,
        context: &SecurityContext,
        code: &str,
    ) -> FhirResult<Vec<Observation>> {
        if code.trim().is_empty() {
            return Err(FhirError::Validation("Code cannot be empty".to_string()));
        }

        // Check authorization
        self.auth_rules.can_search(context, None)?;

        self.repository.search_by_code(code).await
    }

    /// Search observations by patient and code
    pub async fn search_by_patient_and_code(
        &self,
        context: &SecurityContext,
        patient_id: &str,
        code: &str,
    ) -> FhirResult<Vec<Observation>> {
        let mut all_observations = self.search_by_patient(context, patient_id).await?;

        // Filter by code
        all_observations.retain(|obs| {
            obs.code.coding.as_ref()
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.code.as_ref())
                .map(|c| c.0 == code)
                .unwrap_or(false)
        });

        Ok(all_observations)
    }
}

#[async_trait::async_trait]
impl ResourceService<Observation> for ObservationService {
    async fn create(&self, context: &SecurityContext, observation: Observation) -> FhirResult<Observation> {
        self.validate_and_create(context, observation).await
    }

    async fn get(&self, context: &SecurityContext, id: &str) -> FhirResult<Observation> {
        // Fetch the observation
        let observation = self.repository.read(id)
            .await?
            .ok_or_else(|| FhirError::NotFound {
                resource_type: "Observation".to_string(),
                id: id.to_string(),
            })?;

        // Check authorization
        self.auth_rules.can_read(context, id, Some(&observation))?;

        Ok(observation)
    }

    async fn update(&self, context: &SecurityContext, id: &str, observation: Observation) -> FhirResult<Observation> {
        // Check if observation exists
        let existing = self.repository.read(id).await?;
        if existing.is_none() {
            return Err(FhirError::NotFound {
                resource_type: "Observation".to_string(),
                id: id.to_string(),
            });
        }

        // Check authorization
        self.auth_rules.can_update(context, id, &observation)?;

        // Validate the observation
        self.validator.validate(&observation)?;

        // Update the observation
        self.repository.update(id, &observation).await
    }

    async fn delete(&self, context: &SecurityContext, id: &str) -> FhirResult<()> {
        // Check if observation exists
        let existing = self.repository.read(id).await?;
        let observation = existing.as_ref().ok_or_else(|| FhirError::NotFound {
            resource_type: "Observation".to_string(),
            id: id.to_string(),
        })?;

        // Check authorization
        self.auth_rules.can_delete(context, id, Some(observation))?;

        // Soft delete the observation
        self.repository.delete(id).await
    }

    async fn search(&self, context: &SecurityContext, params: SearchParameters) -> FhirResult<SearchResult<Observation>> {
        // Check authorization
        self.auth_rules.can_search(context, None)?;

        let limit = params.count.unwrap_or(100) as i64;
        let offset = params.offset.unwrap_or(0) as i64;

        let search_params = SearchParams::new()
            .with_limit(limit)
            .with_offset(offset);

        let resources = self.repository.search(search_params).await?;
        let count = resources.len() as u32;

        Ok(SearchResult::new(
            resources,
            None,
            params.offset.unwrap_or(0),
            count,
        ))
    }
}