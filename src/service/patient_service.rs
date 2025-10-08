// src/service/patient_service.rs

use crate::domain::{Patient, FhirError, FhirResult};
use crate::repository::{PatientRepository, Repository, SearchParams};
use crate::service::{
    ResourceService, SearchParameters, SearchResult, Validator, PatientValidator,
    SecurityContext, PatientAuthorizationRules,
};

pub struct PatientService {
    repository: PatientRepository,
    validator: PatientValidator,
    auth_rules: PatientAuthorizationRules,
}

impl PatientService {
    pub fn new(repository: PatientRepository) -> Self {
        Self {
            repository,
            validator: PatientValidator,
            auth_rules: PatientAuthorizationRules::new(),
        }
    }

    /// Validate and create a new patient
    async fn validate_and_create(&self, context: &SecurityContext, patient: Patient) -> FhirResult<Patient> {
        // Check authorization
        self.auth_rules.can_create(context, &patient)?;
        // Validate the patient
        self.validator.validate(&patient)?;
        
        // Check for duplicate identifiers
        if let Some(identifiers) = &patient.identifier {
            for identifier in identifiers {
                if let (Some(system), Some(value)) = (&identifier.system, &identifier.value) {
                    if let Some(_existing) = self.repository
                        .search_by_identifier(&system.0, &value.0)
                        .await? 
                    {
                        return Err(FhirError::Conflict(
                            format!("Patient with identifier {}|{} already exists", system.0, value.0)
                        ));
                    }
                }
            }
        }
        
        // Create the patient
        self.repository.create(&patient).await
    }
    
    /// Search patients by family name
    pub async fn search_by_family(&self, context: &SecurityContext, family: &str) -> FhirResult<Vec<Patient>> {
        // Check authorization
        self.auth_rules.can_search(context)?;

        if family.trim().is_empty() {
            return Err(FhirError::Validation("Family name cannot be empty".to_string()));
        }

        self.repository.search_by_family(family).await
    }

    /// Search patients by identifier
    pub async fn search_by_identifier(&self, context: &SecurityContext, system: &str, value: &str) -> FhirResult<Option<Patient>> {
        // Check authorization
        self.auth_rules.can_search(context)?;

        if system.trim().is_empty() || value.trim().is_empty() {
            return Err(FhirError::Validation("System and value cannot be empty".to_string()));
        }

        self.repository.search_by_identifier(system, value).await
    }

    /// Get patient history (all versions)
    pub async fn get_history(&self, context: &SecurityContext, id: &str) -> FhirResult<Vec<Patient>> {
        // Check authorization
        self.auth_rules.can_read_history(context, id)?;

        self.repository.get_history(id).await
    }
    
    /// Get specific version of a patient
    pub async fn get_version(&self, context: &SecurityContext, id: &str, version: u32) -> FhirResult<Option<Patient>> {
        // Check authorization
        self.auth_rules.can_read_history(context, id)?;

        let history = self.repository.get_history(id).await?;
        
        Ok(history.into_iter()
            .find(|p| {
                p.meta.as_ref()
                    .and_then(|m| m.version_id.as_ref())
                    .and_then(|v| v.0.parse::<u32>().ok())
                    == Some(version)
            }))
    }
    
    /// Conditional create - create only if no match found
    pub async fn conditional_create(
        &self,
        context: &SecurityContext,
        patient: Patient,
        search_params: SearchParameters,
    ) -> FhirResult<Patient> {
        // Search for existing patients matching the criteria
        let search_result = self.search(context, search_params).await?;

        if !search_result.resources.is_empty() {
            // Match found - return 412 Precondition Failed or existing resource
            return Err(FhirError::PreconditionFailed(
                "Patient matching search criteria already exists".to_string()
            ));
        }

        // No match - create new patient
        self.validate_and_create(context, patient).await
    }

    /// Conditional update - update if match found, otherwise create
    pub async fn conditional_update(
        &self,
        context: &SecurityContext,
        patient: Patient,
        search_params: SearchParameters,
    ) -> FhirResult<Patient> {
        let search_result = self.search(context, search_params).await?;

        match search_result.resources.len() {
            0 => {
                // No match - create new
                self.validate_and_create(context, patient).await
            }
            1 => {
                // Single match - update
                let existing = &search_result.resources[0];
                if let Some(id) = &existing.id {
                    self.update(context, &id.0, patient).await
                } else {
                    Err(FhirError::Database("Existing patient has no ID".to_string()))
                }
            }
            _ => {
                // Multiple matches - error
                Err(FhirError::PreconditionFailed(
                    "Multiple patients match search criteria".to_string()
                ))
            }
        }
    }
}

#[async_trait::async_trait]
impl ResourceService<Patient> for PatientService {
    async fn create(&self, context: &SecurityContext, patient: Patient) -> FhirResult<Patient> {
        self.validate_and_create(context, patient).await
    }

    async fn get(&self, context: &SecurityContext, id: &str) -> FhirResult<Patient> {
        // Check authorization
        self.auth_rules.can_read(context, id)?;

        self.repository.read(id)
            .await?
            .ok_or_else(|| FhirError::NotFound {
                resource_type: "Patient".to_string(),
                id: id.to_string(),
            })
    }

    async fn update(&self, context: &SecurityContext, id: &str, patient: Patient) -> FhirResult<Patient> {
        // Check if patient exists
        let existing = self.repository.read(id).await?;
        if existing.is_none() {
            return Err(FhirError::NotFound {
                resource_type: "Patient".to_string(),
                id: id.to_string(),
            });
        }

        // Check authorization
        self.auth_rules.can_update(context, id, &patient)?;

        // Validate the patient
        self.validator.validate(&patient)?;

        // Update the patient
        self.repository.update(id, &patient).await
    }

    async fn delete(&self, context: &SecurityContext, id: &str) -> FhirResult<()> {
        // Check if patient exists
        let existing = self.repository.read(id).await?;
        if existing.is_none() {
            return Err(FhirError::NotFound {
                resource_type: "Patient".to_string(),
                id: id.to_string(),
            });
        }

        // Check authorization
        self.auth_rules.can_delete(context, id)?;

        // Soft delete the patient
        self.repository.delete(id).await
    }

    async fn search(&self, context: &SecurityContext, params: SearchParameters) -> FhirResult<SearchResult<Patient>> {
        // Check authorization
        self.auth_rules.can_search(context)?;

        let limit = params.count.unwrap_or(100) as i64;
        let offset = params.offset.unwrap_or(0) as i64;

        let search_params = SearchParams::new()
            .with_limit(limit)
            .with_offset(offset);

        let resources = self.repository.search(search_params).await?;
        let count = resources.len() as u32;

        Ok(SearchResult::new(
            resources,
            None, // Total count would require a separate query
            params.offset.unwrap_or(0),
            count,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    
    async fn setup_test_service(pool: PgPool) -> PatientService {
        let repository = PatientRepository::new(pool);
        PatientService::new(repository)
    }
    
    #[tokio::test]
    async fn test_create_patient_validation() {
        // This would require a test database setup
        // For now, just test validation logic
        let mut patient = Patient::new();
        patient.resource_type = "InvalidType".to_string();
        
        let validator = PatientValidator;
        assert!(validator.validate(&patient).is_err());
    }
}