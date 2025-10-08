// src/service/authorization_rules.rs

use crate::domain::errors::FhirResult;
use crate::domain::{Observation, Patient, Condition, Encounter, Reference};
use super::authorization::{SecurityContext, Permission, Authorizer, DefaultAuthorizer};

/// Extract patient ID from a Reference
fn extract_patient_id_from_reference(reference: &Option<Reference>) -> Option<String> {
    reference.as_ref()
        .and_then(|r| r.reference.as_ref())
        .map(|ref_str| {
            // Parse "Patient/123" format
            if let Some(id) = ref_str.0.strip_prefix("Patient/") {
                id.to_string()
            } else {
                ref_str.0.clone()
            }
        })
}

/// Authorization rules for Patient resources
pub struct PatientAuthorizationRules {
    authorizer: DefaultAuthorizer,
}

impl PatientAuthorizationRules {
    pub fn new() -> Self {
        Self {
            authorizer: DefaultAuthorizer::new(),
        }
    }

    /// Check if the user can create a patient
    pub fn can_create(&self, context: &SecurityContext, _patient: &Patient) -> FhirResult<()> {
        // Only admin and clinician can create patients
        self.authorizer.check_permission(context, "Patient", Permission::Create)
    }

    /// Check if the user can read a patient
    pub fn can_read(&self, context: &SecurityContext, patient_id: &str) -> FhirResult<()> {
        self.authorizer.check_resource_access(context, "Patient", patient_id, Permission::Read)
    }

    /// Check if the user can update a patient
    pub fn can_update(&self, context: &SecurityContext, patient_id: &str, _patient: &Patient) -> FhirResult<()> {
        self.authorizer.check_resource_access(context, "Patient", patient_id, Permission::Update)
    }

    /// Check if the user can delete a patient
    pub fn can_delete(&self, context: &SecurityContext, patient_id: &str) -> FhirResult<()> {
        self.authorizer.check_resource_access(context, "Patient", patient_id, Permission::Delete)
    }

    /// Check if the user can search patients
    pub fn can_search(&self, context: &SecurityContext) -> FhirResult<()> {
        self.authorizer.check_permission(context, "Patient", Permission::Search)
    }

    /// Check if the user can read patient history
    pub fn can_read_history(&self, context: &SecurityContext, patient_id: &str) -> FhirResult<()> {
        self.authorizer.check_resource_access(context, "Patient", patient_id, Permission::ReadHistory)
    }
}

impl Default for PatientAuthorizationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Authorization rules for Observation resources
pub struct ObservationAuthorizationRules {
    authorizer: DefaultAuthorizer,
}

impl ObservationAuthorizationRules {
    pub fn new() -> Self {
        Self {
            authorizer: DefaultAuthorizer::new(),
        }
    }

    /// Check if the user can create an observation
    pub fn can_create(&self, context: &SecurityContext, observation: &Observation) -> FhirResult<()> {
        // First check if user has create permission
        self.authorizer.check_permission(context, "Observation", Permission::Create)?;

        // Check patient compartment access
        if let Some(patient_id) = extract_patient_id_from_reference(&observation.subject) {
            self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Create)?;
        }

        Ok(())
    }

    /// Check if the user can read an observation
    pub fn can_read(&self, context: &SecurityContext, observation_id: &str, observation: Option<&Observation>) -> FhirResult<()> {
        // First check base permission
        self.authorizer.check_resource_access(context, "Observation", observation_id, Permission::Read)?;

        // If we have the observation data, check patient compartment
        if let Some(obs) = observation {
            if let Some(patient_id) = extract_patient_id_from_reference(&obs.subject) {
                self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Read)?;
            }
        }

        Ok(())
    }

    /// Check if the user can update an observation
    pub fn can_update(&self, context: &SecurityContext, observation_id: &str, observation: &Observation) -> FhirResult<()> {
        // Check update permission
        self.authorizer.check_resource_access(context, "Observation", observation_id, Permission::Update)?;

        // Check patient compartment access
        if let Some(patient_id) = extract_patient_id_from_reference(&observation.subject) {
            self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Update)?;
        }

        Ok(())
    }

    /// Check if the user can delete an observation
    pub fn can_delete(&self, context: &SecurityContext, observation_id: &str, observation: Option<&Observation>) -> FhirResult<()> {
        // Check delete permission
        self.authorizer.check_resource_access(context, "Observation", observation_id, Permission::Delete)?;

        // If we have the observation data, check patient compartment
        if let Some(obs) = observation {
            if let Some(patient_id) = extract_patient_id_from_reference(&obs.subject) {
                self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Delete)?;
            }
        }

        Ok(())
    }

    /// Check if the user can search observations
    pub fn can_search(&self, context: &SecurityContext, patient_id: Option<&str>) -> FhirResult<()> {
        // Check search permission
        self.authorizer.check_permission(context, "Observation", Permission::Search)?;

        // If searching for a specific patient, check patient compartment access
        if let Some(pid) = patient_id {
            self.authorizer.check_patient_compartment_access(context, pid, Permission::Search)?;
        }

        Ok(())
    }
}

impl Default for ObservationAuthorizationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Authorization rules for Condition resources
pub struct ConditionAuthorizationRules {
    authorizer: DefaultAuthorizer,
}

impl ConditionAuthorizationRules {
    pub fn new() -> Self {
        Self {
            authorizer: DefaultAuthorizer::new(),
        }
    }

    /// Check if the user can create a condition
    pub fn can_create(&self, context: &SecurityContext, condition: &Condition) -> FhirResult<()> {
        // First check if user has create permission
        self.authorizer.check_permission(context, "Condition", Permission::Create)?;

        // Check patient compartment access
        if let Some(patient_id) = extract_patient_id_from_reference(&Some(condition.subject.clone())) {
            self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Create)?;
        }

        Ok(())
    }

    /// Check if the user can read a condition
    pub fn can_read(&self, context: &SecurityContext, condition_id: &str, condition: Option<&Condition>) -> FhirResult<()> {
        // First check base permission
        self.authorizer.check_resource_access(context, "Condition", condition_id, Permission::Read)?;

        // If we have the condition data, check patient compartment
        if let Some(cond) = condition {
            if let Some(patient_id) = extract_patient_id_from_reference(&Some(cond.subject.clone())) {
                self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Read)?;
            }
        }

        Ok(())
    }

    /// Check if the user can update a condition
    pub fn can_update(&self, context: &SecurityContext, condition_id: &str, condition: &Condition) -> FhirResult<()> {
        // Check update permission
        self.authorizer.check_resource_access(context, "Condition", condition_id, Permission::Update)?;

        // Check patient compartment access
        if let Some(patient_id) = extract_patient_id_from_reference(&Some(condition.subject.clone())) {
            self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Update)?;
        }

        Ok(())
    }

    /// Check if the user can delete a condition
    pub fn can_delete(&self, context: &SecurityContext, condition_id: &str, condition: Option<&Condition>) -> FhirResult<()> {
        // Check delete permission
        self.authorizer.check_resource_access(context, "Condition", condition_id, Permission::Delete)?;

        // If we have the condition data, check patient compartment
        if let Some(cond) = condition {
            if let Some(patient_id) = extract_patient_id_from_reference(&Some(cond.subject.clone())) {
                self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Delete)?;
            }
        }

        Ok(())
    }

    /// Check if the user can search conditions
    pub fn can_search(&self, context: &SecurityContext, patient_id: Option<&str>) -> FhirResult<()> {
        // Check search permission
        self.authorizer.check_permission(context, "Condition", Permission::Search)?;

        // If searching for a specific patient, check patient compartment access
        if let Some(pid) = patient_id {
            self.authorizer.check_patient_compartment_access(context, pid, Permission::Search)?;
        }

        Ok(())
    }
}

impl Default for ConditionAuthorizationRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Authorization rules for Encounter resources
pub struct EncounterAuthorizationRules {
    authorizer: DefaultAuthorizer,
}

impl EncounterAuthorizationRules {
    pub fn new() -> Self {
        Self {
            authorizer: DefaultAuthorizer::new(),
        }
    }

    /// Check if the user can create an encounter
    pub fn can_create(&self, context: &SecurityContext, encounter: &Encounter) -> FhirResult<()> {
        // First check if user has create permission
        self.authorizer.check_permission(context, "Encounter", Permission::Create)?;

        // Check patient compartment access
        if let Some(patient_id) = extract_patient_id_from_reference(&encounter.subject) {
            self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Create)?;
        }

        Ok(())
    }

    /// Check if the user can read an encounter
    pub fn can_read(&self, context: &SecurityContext, encounter_id: &str, encounter: Option<&Encounter>) -> FhirResult<()> {
        // First check base permission
        self.authorizer.check_resource_access(context, "Encounter", encounter_id, Permission::Read)?;

        // If we have the encounter data, check patient compartment
        if let Some(enc) = encounter {
            if let Some(patient_id) = extract_patient_id_from_reference(&enc.subject) {
                self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Read)?;
            }
        }

        Ok(())
    }

    /// Check if the user can update an encounter
    pub fn can_update(&self, context: &SecurityContext, encounter_id: &str, encounter: &Encounter) -> FhirResult<()> {
        // Check update permission
        self.authorizer.check_resource_access(context, "Encounter", encounter_id, Permission::Update)?;

        // Check patient compartment access
        if let Some(patient_id) = extract_patient_id_from_reference(&encounter.subject) {
            self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Update)?;
        }

        Ok(())
    }

    /// Check if the user can delete an encounter
    pub fn can_delete(&self, context: &SecurityContext, encounter_id: &str, encounter: Option<&Encounter>) -> FhirResult<()> {
        // Check delete permission
        self.authorizer.check_resource_access(context, "Encounter", encounter_id, Permission::Delete)?;

        // If we have the encounter data, check patient compartment
        if let Some(enc) = encounter {
            if let Some(patient_id) = extract_patient_id_from_reference(&enc.subject) {
                self.authorizer.check_patient_compartment_access(context, &patient_id, Permission::Delete)?;
            }
        }

        Ok(())
    }

    /// Check if the user can search encounters
    pub fn can_search(&self, context: &SecurityContext, patient_id: Option<&str>) -> FhirResult<()> {
        // Check search permission
        self.authorizer.check_permission(context, "Encounter", Permission::Search)?;

        // If searching for a specific patient, check patient compartment access
        if let Some(pid) = patient_id {
            self.authorizer.check_patient_compartment_access(context, pid, Permission::Search)?;
        }

        Ok(())
    }
}

impl Default for EncounterAuthorizationRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Code, CodeableConcept, FhirString, Uri};
    use crate::service::authorization::{SecurityContext, Role};

    #[test]
    fn test_observation_authorization_with_patient_context() {
        let rules = ObservationAuthorizationRules::new();

        // Patient accessing their own observation
        let patient_ctx = SecurityContext::patient("user1".to_string(), "patient1".to_string());

        let mut observation = Observation::new(
            Code("final".to_string()),
            CodeableConcept {
                coding: None,
                text: Some(FhirString("Test".to_string())),
            }
        );
        observation.subject = Some(Reference {
            reference: Some(FhirString("Patient/patient1".to_string())),
            type_: Some(Uri("Patient".to_string())),
            identifier: None,
            display: None,
        });

        // Patient can search their own observations
        assert!(rules.can_search(&patient_ctx, Some("patient1")).is_ok());

        // Patient cannot search another patient's observations
        assert!(rules.can_search(&patient_ctx, Some("patient2")).is_err());

        // Patient cannot create observations
        assert!(rules.can_create(&patient_ctx, &observation).is_err());
    }

    #[test]
    fn test_clinician_authorization() {
        let rules = ObservationAuthorizationRules::new();
        let clinician_ctx = SecurityContext::clinician("doc1".to_string(), None);

        // Clinician can search any patient's observations
        assert!(rules.can_search(&clinician_ctx, Some("patient1")).is_ok());
        assert!(rules.can_search(&clinician_ctx, Some("patient2")).is_ok());
    }

    #[test]
    fn test_admin_authorization() {
        let rules = ObservationAuthorizationRules::new();
        let admin_ctx = SecurityContext::admin("admin1".to_string());

        // Admin can do everything
        assert!(rules.can_search(&admin_ctx, Some("patient1")).is_ok());

        let mut observation = Observation::new(
            Code("final".to_string()),
            CodeableConcept {
                coding: None,
                text: Some(FhirString("Test".to_string())),
            }
        );
        observation.subject = Some(Reference {
            reference: Some(FhirString("Patient/patient1".to_string())),
            type_: Some(Uri("Patient".to_string())),
            identifier: None,
            display: None,
        });

        assert!(rules.can_create(&admin_ctx, &observation).is_ok());
        assert!(rules.can_delete(&admin_ctx, "obs1", Some(&observation)).is_ok());
    }
}
