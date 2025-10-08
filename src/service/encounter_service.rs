// src/service/encounter_service.rs

use crate::domain::{Encounter, FhirError, FhirResult};
use crate::repository::{EncounterRepository, Repository, SearchParams};
use crate::service::{
    ResourceService, SearchParameters, SearchResult, Validator, EncounterValidator,
    SecurityContext, EncounterAuthorizationRules,
};

pub struct EncounterService {
    repository: EncounterRepository,
    validator: EncounterValidator,
    auth_rules: EncounterAuthorizationRules,
}

impl EncounterService {
    pub fn new(repository: EncounterRepository) -> Self {
        Self {
            repository,
            validator: EncounterValidator,
            auth_rules: EncounterAuthorizationRules::new(),
        }
    }

    /// Validate and create a new encounter
    async fn validate_and_create(&self, context: &SecurityContext, encounter: Encounter) -> FhirResult<Encounter> {
        // Check authorization
        self.auth_rules.can_create(context, &encounter)?;
        // Validate the encounter
        self.validator.validate(&encounter)?;
        
        // Validate subject reference if present
        if let Some(subject) = &encounter.subject {
            if let Some(reference) = &subject.reference {
                self.validate_reference(&reference.0).await?;
            }
        }
        
        // Create the encounter
        self.repository.create(&encounter).await
    }
    
    /// Validate that a reference exists
    async fn validate_reference(&self, reference: &str) -> FhirResult<()> {
        let parts: Vec<&str> = reference.split('/').collect();
        if parts.len() != 2 {
            return Err(FhirError::InvalidReference(
                format!("Invalid reference format: {}", reference)
            ));
        }
        Ok(())
    }
    
    /// Search encounters by patient
    pub async fn search_by_patient(&self, context: &SecurityContext, patient_id: &str) -> FhirResult<Vec<Encounter>> {
        if patient_id.trim().is_empty() {
            return Err(FhirError::Validation("Patient ID cannot be empty".to_string()));
        }

        // Check authorization
        self.auth_rules.can_search(context, Some(patient_id))?;

        self.repository.search_by_patient(patient_id).await
    }

    /// Search encounters by status
    pub async fn search_by_status(&self, context: &SecurityContext, status: &str) -> FhirResult<Vec<Encounter>> {
        if status.trim().is_empty() {
            return Err(FhirError::Validation("Status cannot be empty".to_string()));
        }

        // Check authorization
        self.auth_rules.can_search(context, None)?;

        let valid_statuses = [
            "planned", "arrived", "triaged", "in-progress",
            "onleave", "finished", "cancelled", "entered-in-error", "unknown"
        ];

        if !valid_statuses.contains(&status) {
            return Err(FhirError::Validation(
                format!("Invalid encounter status: {}", status)
            ));
        }

        self.repository.search_by_status(status).await
    }

    /// Get active encounters for a patient
    pub async fn get_active_encounters(&self, context: &SecurityContext, patient_id: &str) -> FhirResult<Vec<Encounter>> {
        let all_encounters = self.search_by_patient(context, patient_id).await?;
        
        // Filter for in-progress encounters
        let active = all_encounters.into_iter()
            .filter(|e| e.status.0 == "in-progress" || e.status.0 == "arrived")
            .collect();
        
        Ok(active)
    }
    
    /// Update encounter status
    pub async fn update_status(&self, context: &SecurityContext, id: &str, new_status: &str) -> FhirResult<Encounter> {
        // Validate status
        let valid_statuses = [
            "planned", "arrived", "triaged", "in-progress",
            "onleave", "finished", "cancelled", "entered-in-error", "unknown"
        ];

        if !valid_statuses.contains(&new_status) {
            return Err(FhirError::Validation(
                format!("Invalid encounter status: {}", new_status)
            ));
        }

        // Get existing encounter
        let mut encounter = self.get(context, id).await?;
        
        // Update status
        encounter.status = crate::domain::Code(new_status.to_string());
        
        // Update the encounter
        self.repository.update(id, &encounter).await
    }
}

#[async_trait::async_trait]
impl ResourceService<Encounter> for EncounterService {
    async fn create(&self, context: &SecurityContext, encounter: Encounter) -> FhirResult<Encounter> {
        self.validate_and_create(context, encounter).await
    }

    async fn get(&self, context: &SecurityContext, id: &str) -> FhirResult<Encounter> {
        // Fetch the encounter
        let encounter = self.repository.read(id)
            .await?
            .ok_or_else(|| FhirError::NotFound {
                resource_type: "Encounter".to_string(),
                id: id.to_string(),
            })?;

        // Check authorization
        self.auth_rules.can_read(context, id, Some(&encounter))?;

        Ok(encounter)
    }

    async fn update(&self, context: &SecurityContext, id: &str, encounter: Encounter) -> FhirResult<Encounter> {
        // Check if encounter exists
        let existing = self.repository.read(id).await?;
        if existing.is_none() {
            return Err(FhirError::NotFound {
                resource_type: "Encounter".to_string(),
                id: id.to_string(),
            });
        }

        // Check authorization
        self.auth_rules.can_update(context, id, &encounter)?;

        // Validate the encounter
        self.validator.validate(&encounter)?;

        // Update the encounter
        self.repository.update(id, &encounter).await
    }

    async fn delete(&self, context: &SecurityContext, id: &str) -> FhirResult<()> {
        // Check if encounter exists
        let existing = self.repository.read(id).await?;
        let encounter = existing.as_ref().ok_or_else(|| FhirError::NotFound {
            resource_type: "Encounter".to_string(),
            id: id.to_string(),
        })?;

        // Check authorization
        self.auth_rules.can_delete(context, id, Some(encounter))?;

        // Soft delete the encounter
        self.repository.delete(id).await
    }

    async fn search(&self, context: &SecurityContext, params: SearchParameters) -> FhirResult<SearchResult<Encounter>> {
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