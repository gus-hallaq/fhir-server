// src/domain/resources/encounter.rs

use serde::{Deserialize, Serialize};
use crate::domain::{datatypes::*, primitives::*};
use super::Resource;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<Vec<Identifier>>,
    
    pub status: Code, // planned | arrived | triaged | in-progress | onleave | finished | cancelled +
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_history: Option<Vec<EncounterStatusHistory>>,
    
    pub class: Coding, // inpatient | outpatient | ambulatory | emergency +
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_history: Option<Vec<EncounterClassHistory>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<Vec<CodeableConcept>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_type: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>, // Usually Patient
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub episode_of_care: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub based_on: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant: Option<Vec<EncounterParticipant>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appointment: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<Quantity>, // Duration
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_code: Option<Vec<CodeableConcept>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason_reference: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnosis: Option<Vec<EncounterDiagnosis>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hospitalization: Option<EncounterHospitalization>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Vec<EncounterLocation>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_provider: Option<Reference>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of: Option<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncounterStatusHistory {
    pub status: Code,
    pub period: Period,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncounterClassHistory {
    pub class: Coding,
    pub period: Period,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncounterParticipant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<Vec<CodeableConcept>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub individual: Option<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDiagnosis {
    pub condition: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<PositiveInt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncounterHospitalization {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_admission_identifier: Option<Identifier>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admit_source: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub re_admission: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diet_preference: Option<Vec<CodeableConcept>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_courtesy: Option<Vec<CodeableConcept>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_arrangement: Option<Vec<CodeableConcept>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<Reference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_disposition: Option<CodeableConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncounterLocation {
    pub location: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Code>, // planned | active | reserved | completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_type: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
}

impl Resource for Encounter {
    fn resource_type() -> &'static str {
        "Encounter"
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

impl Encounter {
    pub fn new(status: Code, class: Coding) -> Self {
        Self {
            resource_type: "Encounter".to_string(),
            id: None,
            meta: None,
            identifier: None,
            status,
            status_history: None,
            class,
            class_history: None,
            type_: None,
            service_type: None,
            priority: None,
            subject: None,
            episode_of_care: None,
            based_on: None,
            participant: None,
            appointment: None,
            period: None,
            length: None,
            reason_code: None,
            reason_reference: None,
            diagnosis: None,
            account: None,
            hospitalization: None,
            location: None,
            service_provider: None,
            part_of: None,
        }
    }
}