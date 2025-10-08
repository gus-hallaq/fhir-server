// src/service/condition_service.rs

use crate::domain::{Condition, FhirError, FhirResult};
use crate::repository::{ConditionRepository, Repository, SearchParams};
use crate::service::{
    ResourceService, SearchParameters, SearchResult, Validator, ConditionValidator,
    SecurityContext, ConditionAuthorizationRules,
};

pub struct ConditionService {
    repository: ConditionRepository,
    validator: ConditionValidator,
    auth_rules: ConditionAuthorizationRules,
}

impl ConditionService {
    pub fn new(repository: ConditionRepository) -> Self {
        Self {
            repository,
            validator: ConditionValidator,
            auth_rules: ConditionAuthorizationRules::new(),
        }
    }

    /// Validate and create a new condition
    async fn validate_and_create(&self, context: &SecurityContext, condition: Condition) -> FhirResult<Condition> {
        // Check authorization
        self.auth_rules.can_create(context, &condition)?;
        // Validate the condition
        self.validator.validate(&condition)?;
        
        // Validate subject reference
        if let Some(reference) = &condition.subject.reference {
            self.validate_reference(&reference.0).await?;
        }
        
        // Create the condition
        self.repository.create(&condition).await
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
    
    /// Search conditions by patient
    pub async fn search_by_patient(&self, context: &SecurityContext, patient_id: &str) -> FhirResult<Vec<Condition>> {
        if patient_id.trim().is_empty() {
            return Err(FhirError::Validation("Patient ID cannot be empty".to_string()));
        }

        // Check authorization
        self.auth_rules.can_search(context, Some(patient_id))?;

        self.repository.search_by_patient(patient_id).await
    }

    /// Search conditions by clinical status
    pub async fn search_by_clinical_status(&self, context: &SecurityContext, status: &str) -> FhirResult<Vec<Condition>> {
        if status.trim().is_empty() {
            return Err(FhirError::Validation("Status cannot be empty".to_string()));
        }

        // Check authorization
        self.auth_rules.can_search(context, None)?;

        let valid_statuses = ["active", "recurrence", "relapse", "inactive", "remission", "resolved"];
        if !valid_statuses.contains(&status) {
            return Err(FhirError::Validation(
                format!("Invalid clinical status: {}", status)
            ));
        }

        self.repository.search_by_clinical_status(status).await
    }

    /// Get active conditions for a patient
    pub async fn get_active_conditions(&self, context: &SecurityContext, patient_id: &str) -> FhirResult<Vec<Condition>> {
        let all_conditions = self.search_by_patient(context, patient_id).await?;
        
        // Filter for active conditions
        let active = all_conditions.into_iter()
            .filter(|c| {
                c.clinical_status.as_ref()
                    .and_then(|cs| cs.coding.as_ref())
                    .and_then(|codings| codings.first())
                    .and_then(|coding| coding.code.as_ref())
                    .map(|code| code.0 == "active")
                    .unwrap_or(false)
            })
            .collect();
        
        Ok(active)
    }
}

#[async_trait::async_trait]
impl ResourceService<Condition> for ConditionService {
    async fn create(&self, context: &SecurityContext, condition: Condition) -> FhirResult<Condition> {
        self.validate_and_create(context, condition).await
    }

    async fn get(&self, context: &SecurityContext, id: &str) -> FhirResult<Condition> {
        // Fetch the condition
        let condition = self.repository.read(id)
            .await?
            .ok_or_else(|| FhirError::NotFound {
                resource_type: "Condition".to_string(),
                id: id.to_string(),
            })?;

        // Check authorization
        self.auth_rules.can_read(context, id, Some(&condition))?;

        Ok(condition)
    }

    async fn update(&self, context: &SecurityContext, id: &str, condition: Condition) -> FhirResult<Condition> {
        // Check if condition exists
        let existing = self.repository.read(id).await?;
        if existing.is_none() {
            return Err(FhirError::NotFound {
                resource_type: "Condition".to_string(),
                id: id.to_string(),
            });
        }

        // Check authorization
        self.auth_rules.can_update(context, id, &condition)?;

        // Validate the condition
        self.validator.validate(&condition)?;

        // Update the condition
        self.repository.update(id, &condition).await
    }

    async fn delete(&self, context: &SecurityContext, id: &str) -> FhirResult<()> {
        // Check if condition exists
        let existing = self.repository.read(id).await?;
        let condition = existing.as_ref().ok_or_else(|| FhirError::NotFound {
            resource_type: "Condition".to_string(),
            id: id.to_string(),
        })?;

        // Check authorization
        self.auth_rules.can_delete(context, id, Some(condition))?;

        // Soft delete the condition
        self.repository.delete(id).await
    }

    async fn search(&self, context: &SecurityContext, params: SearchParameters) -> FhirResult<SearchResult<Condition>> {
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