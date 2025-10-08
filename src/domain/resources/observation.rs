// src/domain/resources/observation.rs

use serde::{Deserialize, Serialize};
use crate::domain::{datatypes::*, primitives::*};
use super::Resource;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Observation {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<Vec<Identifier>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub based_on: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_of: Option<Vec<Reference>>,
    
    pub status: Code, // registered | preliminary | final | amended +
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Vec<CodeableConcept>>,
    
    pub code: CodeableConcept, // Type of observation
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<Reference>, // Usually Patient
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encounter: Option<Reference>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective: Option<ObservationEffective>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued: Option<Instant>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performer: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<ObservationValue>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_absent_reason: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<Vec<CodeableConcept>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Vec<Annotation>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_site: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<CodeableConcept>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub specimen: Option<Reference>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<Reference>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_range: Option<Vec<ObservationReferenceRange>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_member: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<Vec<Reference>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<Vec<ObservationComponent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ObservationEffective {
    DateTime(FhirDateTime),
    Period(Period),
    Instant(Instant),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ObservationValue {
    Quantity(Quantity),
    CodeableConcept(CodeableConcept),
    String(FhirString),
    Boolean(FhirBoolean),
    Integer(FhirInteger),
    Range(Range),
    Period(Period),
    DateTime(FhirDateTime),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ObservationReferenceRange {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low: Option<Quantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<Quantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applies_to: Option<Vec<CodeableConcept>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<FhirString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ObservationComponent {
    pub code: CodeableConcept,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<ObservationValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_absent_reason: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<Vec<CodeableConcept>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_range: Option<Vec<ObservationReferenceRange>>,
}

impl Resource for Observation {
    fn resource_type() -> &'static str {
        "Observation"
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

impl Observation {
    pub fn new(status: Code, code: CodeableConcept) -> Self {
        Self {
            resource_type: "Observation".to_string(),
            id: None,
            meta: None,
            identifier: None,
            based_on: None,
            part_of: None,
            status,
            category: None,
            code,
            subject: None,
            focus: None,
            encounter: None,
            effective: None,
            issued: None,
            performer: None,
            value: None,
            data_absent_reason: None,
            interpretation: None,
            note: None,
            body_site: None,
            method: None,
            specimen: None,
            device: None,
            reference_range: None,
            has_member: None,
            derived_from: None,
            component: None,
        }
    }
}