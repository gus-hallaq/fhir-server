// src/service/validation.rs
// FHIR resource validation logic

use crate::domain::{
    Patient, Observation, Condition, Encounter,
    FhirError, FhirResult,
};

/// Validator trait for FHIR resources
pub trait Validator<T> {
    fn validate(&self, resource: &T) -> FhirResult<()>;
}

/// Patient validator
pub struct PatientValidator;

impl Validator<Patient> for PatientValidator {
    fn validate(&self, patient: &Patient) -> FhirResult<()> {
        // Validate resource type
        if patient.resource_type != "Patient" {
            return Err(FhirError::Validation(
                format!("Invalid resourceType: expected 'Patient', got '{}'", patient.resource_type)
            ));
        }
        
        // Validate name if present
        if let Some(names) = &patient.name {
            if names.is_empty() {
                return Err(FhirError::Validation(
                    "Name array cannot be empty if present".to_string()
                ));
            }
            
            for name in names {
                if name.family.is_none() && name.given.is_none() && name.text.is_none() {
                    return Err(FhirError::Validation(
                        "HumanName must have at least family, given, or text".to_string()
                    ));
                }
            }
        }
        
        // Validate gender if present
        if let Some(gender) = &patient.gender {
            let valid_genders = ["male", "female", "other", "unknown"];
            if !valid_genders.contains(&gender.0.as_str()) {
                return Err(FhirError::Validation(
                    format!("Invalid gender value: '{}'. Must be one of: male, female, other, unknown", gender.0)
                ));
            }
        }
        
        // Validate identifiers if present
        if let Some(identifiers) = &patient.identifier {
            for identifier in identifiers {
                if identifier.value.is_none() && identifier.system.is_none() {
                    return Err(FhirError::Validation(
                        "Identifier must have at least a value or system".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
}

/// Observation validator
pub struct ObservationValidator;

impl Validator<Observation> for ObservationValidator {
    fn validate(&self, observation: &Observation) -> FhirResult<()> {
        // Validate resource type
        if observation.resource_type != "Observation" {
            return Err(FhirError::Validation(
                format!("Invalid resourceType: expected 'Observation', got '{}'", observation.resource_type)
            ));
        }
        
        // Validate required fields
        if observation.status.0.is_empty() {
            return Err(FhirError::MissingRequiredField("status".to_string()));
        }
        
        // Validate status values
        let valid_statuses = [
            "registered", "preliminary", "final", "amended",
            "corrected", "cancelled", "entered-in-error", "unknown"
        ];
        if !valid_statuses.contains(&observation.status.0.as_str()) {
            return Err(FhirError::Validation(
                format!("Invalid status value: '{}'", observation.status.0)
            ));
        }
        
        // Validate code (required)
        if observation.code.coding.is_none() && observation.code.text.is_none() {
            return Err(FhirError::Validation(
                "Observation.code must have at least coding or text".to_string()
            ));
        }
        
        // Validate that either value or dataAbsentReason is present, but not both
        let has_value = observation.value.is_some();
        let has_absent_reason = observation.data_absent_reason.is_some();
        
        if has_value && has_absent_reason {
            return Err(FhirError::Validation(
                "Cannot have both value and dataAbsentReason".to_string()
            ));
        }
        
        // Validate components if present
        if let Some(components) = &observation.component {
            for component in components {
                if component.code.coding.is_none() && component.code.text.is_none() {
                    return Err(FhirError::Validation(
                        "Component.code must have at least coding or text".to_string()
                    ));
                }
                
                // Each component should have value or dataAbsentReason, but not both
                let comp_has_value = component.value.is_some();
                let comp_has_absent = component.data_absent_reason.is_some();
                
                if comp_has_value && comp_has_absent {
                    return Err(FhirError::Validation(
                        "Component cannot have both value and dataAbsentReason".to_string()
                    ));
                }
            }
        }
        
        Ok(())
    }
}

/// Condition validator
pub struct ConditionValidator;

impl Validator<Condition> for ConditionValidator {
    fn validate(&self, condition: &Condition) -> FhirResult<()> {
        // Validate resource type
        if condition.resource_type != "Condition" {
            return Err(FhirError::Validation(
                format!("Invalid resourceType: expected 'Condition', got '{}'", condition.resource_type)
            ));
        }
        
        // Validate subject (required)
        if condition.subject.reference.is_none() && condition.subject.identifier.is_none() {
            return Err(FhirError::MissingRequiredField(
                "subject (must have reference or identifier)".to_string()
            ));
        }
        
        // Validate clinical status if present
        if let Some(clinical_status) = &condition.clinical_status {
            if clinical_status.coding.is_none() {
                return Err(FhirError::Validation(
                    "clinicalStatus must have coding".to_string()
                ));
            }
        }
        
        // Validate verification status if present
        if let Some(verification_status) = &condition.verification_status {
            if verification_status.coding.is_none() {
                return Err(FhirError::Validation(
                    "verificationStatus must have coding".to_string()
                ));
            }
        }
        
        // If clinicalStatus is absent, verificationStatus should be 'entered-in-error'
        if condition.clinical_status.is_none() && condition.verification_status.is_some() {
            let verification_code = condition.verification_status.as_ref()
                .and_then(|vs| vs.coding.as_ref())
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.code.as_ref())
                .map(|code| code.0.as_str());
            
            if verification_code != Some("entered-in-error") {
                return Err(FhirError::Validation(
                    "If clinicalStatus is absent, verificationStatus must be 'entered-in-error'".to_string()
                ));
            }
        }
        
        Ok(())
    }
}

/// Encounter validator
pub struct EncounterValidator;

impl Validator<Encounter> for EncounterValidator {
    fn validate(&self, encounter: &Encounter) -> FhirResult<()> {
        // Validate resource type
        if encounter.resource_type != "Encounter" {
            return Err(FhirError::Validation(
                format!("Invalid resourceType: expected 'Encounter', got '{}'", encounter.resource_type)
            ));
        }
        
        // Validate required fields
        if encounter.status.0.is_empty() {
            return Err(FhirError::MissingRequiredField("status".to_string()));
        }
        
        // Validate status values
        let valid_statuses = [
            "planned", "arrived", "triaged", "in-progress",
            "onleave", "finished", "cancelled", "entered-in-error", "unknown"
        ];
        if !valid_statuses.contains(&encounter.status.0.as_str()) {
            return Err(FhirError::Validation(
                format!("Invalid status value: '{}'", encounter.status.0)
            ));
        }
        
        // Validate class (required)
        if encounter.class.code.is_none() && encounter.class.display.is_none() {
            return Err(FhirError::Validation(
                "Encounter.class must have at least code or display".to_string()
            ));
        }
        
        // Validate period if present
        if let Some(period) = &encounter.period {
            if let (Some(start), Some(end)) = (&period.start, &period.end) {
                if end.0 < start.0 {
                    return Err(FhirError::Validation(
                        "Period.end must be after or equal to period.start".to_string()
                    ));
                }
            }
        }
        
        // Validate status history if present
        if let Some(history) = &encounter.status_history {
            for item in history {
                if !valid_statuses.contains(&item.status.0.as_str()) {
                    return Err(FhirError::Validation(
                        format!("Invalid status in history: '{}'", item.status.0)
                    ));
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Code, CodeableConcept, Coding, FhirString, Uri, Reference};
    
    #[test]
    fn test_patient_validation_success() {
        let mut patient = Patient::new();
        patient.resource_type = "Patient".to_string();
        
        let validator = PatientValidator;
        assert!(validator.validate(&patient).is_ok());
    }
    
    #[test]
    fn test_patient_validation_invalid_gender() {
        let mut patient = Patient::new();
        patient.gender = Some(Code("invalid".to_string()));
        
        let validator = PatientValidator;
        assert!(validator.validate(&patient).is_err());
    }
    
    #[test]
    fn test_observation_validation_requires_status() {
        let observation = Observation::new(
            Code("final".to_string()),
            CodeableConcept {
                coding: Some(vec![Coding {
                    system: Some(Uri("http://loinc.org".to_string())),
                    code: Some(Code("1234-5".to_string())),
                    display: None,
                    version: None,
                    user_selected: None,
                }]),
                text: None,
            }
        );
        
        let validator = ObservationValidator;
        assert!(validator.validate(&observation).is_ok());
    }
    
    #[test]
    fn test_condition_requires_subject() {
        let subject = Reference {
            reference: Some(FhirString("Patient/123".to_string())),
            type_: None,
            identifier: None,
            display: None,
        };
        
        let condition = Condition::new(subject);
        
        let validator = ConditionValidator;
        assert!(validator.validate(&condition).is_ok());
    }
}