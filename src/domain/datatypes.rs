// src/domain/datatypes.rs
// FHIR Complex Datatypes

use serde::{Deserialize, Serialize};
use crate::domain::primitives::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<Instant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<Vec<Canonical>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<Coding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Vec<Coding>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Identifier {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<CodeableConcept>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigner: Option<Box<Reference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CodeableConcept {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coding: Option<Vec<Coding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<FhirString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Coding {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_selected: Option<FhirBoolean>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<Box<Identifier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<FhirString>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<FhirDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<FhirDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HumanName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given: Option<Vec<FhirString>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<Vec<FhirString>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<Vec<FhirString>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContactPoint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rank: Option<FhirInteger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<Vec<FhirString>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub district: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub period: Option<Period>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Quantity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<FhirDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparator: Option<Code>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<FhirString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Uri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<Code>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low: Option<Quantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub high: Option<Quantity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<AnnotationAuthor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<FhirDateTime>,
    pub text: FhirString,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AnnotationAuthor {
    Reference(Reference),
    String(FhirString),
}