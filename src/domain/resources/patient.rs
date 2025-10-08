// src/domain/resources/patient.rs

use serde::{Deserialize, Serialize};
use crate::domain::{datatypes::*, primitives::*};
use super::Resource;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<Vec<Identifier>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<FhirBoolean>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<Vec<HumanName>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telecom: Option<Vec<ContactPoint>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<Code>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<FhirDate>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deceased: Option<PatientDeceased>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Vec<Address>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marital_status: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_birth: Option<PatientMultipleBirth>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<Vec<PatientContact>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub communication: Option<Vec<PatientCommunication>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub general_practitioner: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managing_organization: Option<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PatientDeceased {
    Boolean(FhirBoolean),
    DateTime(FhirDateTime),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PatientMultipleBirth {
    Boolean(FhirBoolean),
    Integer(FhirInteger),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PatientContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<Vec<CodeableConcept>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<HumanName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telecom: Option<Vec<ContactPoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PatientCommunication {
    pub language: CodeableConcept,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<FhirBoolean>,
}

impl Resource for Patient {
    fn resource_type() -> &'static str {
        "Patient"
    }
    
    fn id(&self) -> Option<&Id> {
        self.id.as_ref()
    }
    
    fn meta(&self) -> Option<&Meta> {
        self.meta.as_ref()
    }
    
    fn set_id(&mut self, id: Id) {
        self.id = Some(id);
    }
    
    fn set_meta(&mut self, meta: Meta) {
        self.meta = Some(meta);
    }
}

impl Patient {
    pub fn new() -> Self {
        Self {
            resource_type: "Patient".to_string(),
            id: None,
            meta: None,
            identifier: None,
            active: None,
            name: None,
            telecom: None,
            gender: None,
            birth_date: None,
            deceased: None,
            address: None,
            marital_status: None,
            multiple_birth: None,
            contact: None,
            communication: None,
            general_practitioner: None,
            managing_organization: None,
        }
    }
}

impl Default for Patient {
    fn default() -> Self {
        Self::new()
    }
}