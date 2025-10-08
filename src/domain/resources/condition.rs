// src/domain/resources/condition.rs

use serde::{Deserialize, Serialize};
use crate::domain::{datatypes::*, primitives::*};
use super::Resource;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<Vec<Identifier>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clinical_status: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_status: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Vec<CodeableConcept>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_site: Option<Vec<CodeableConcept>>,
    
    pub subject: Reference, // Patient or Group
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encounter: Option<Reference>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onset: Option<ConditionOnset>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub abatement: Option<ConditionAbatement>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorded_date: Option<FhirDateTime>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recorder: Option<Reference>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asserter: Option<Reference>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<Vec<ConditionStage>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<ConditionEvidence>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Vec<Annotation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConditionOnset {
    DateTime(FhirDateTime),
    Age(Quantity),
    Period(Period),
    Range(Range),
    String(FhirString),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConditionAbatement {
    DateTime(FhirDateTime),
    Age(Quantity),
    Period(Period),
    Range(Range),
    String(FhirString),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConditionStage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assessment: Option<Vec<Reference>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<CodeableConcept>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConditionEvidence {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<CodeableConcept>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Vec<Reference>>,
}

impl Resource for Condition {
    fn resource_type() -> &'static str {
        "Condition"
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

impl Condition {
    pub fn new(subject: Reference) -> Self {
        Self {
            resource_type: "Condition".to_string(),
            id: None,
            meta: None,
            identifier: None,
            clinical_status: None,
            verification_status: None,
            category: None,
            severity: None,
            code: None,
            body_site: None,
            subject,
            encounter: None,
            onset: None,
            abatement: None,
            recorded_date: None,
            recorder: None,
            asserter: None,
            stage: None,
            evidence: None,
            note: None,
        }
    }
}