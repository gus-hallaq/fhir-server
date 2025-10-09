// src/grpc/converters.rs
// Converters between domain models and protobuf models

use crate::domain::{self, primitives::*, datatypes::*, resources::*};
use super::proto;

// Helper functions for common conversions
fn to_proto_meta(meta: &Option<Meta>) -> Option<proto::Meta> {
    meta.as_ref().map(|m| proto::Meta {
        version_id: m.version_id.as_ref().map(|v| v.0.clone()),
        last_updated: m.last_updated.as_ref().map(|i| i.0.to_rfc3339()),
        source: m.source.as_ref().map(|s| s.0.clone()),
    })
}

fn from_proto_meta(meta: &Option<proto::Meta>) -> Option<Meta> {
    meta.as_ref().map(|m| Meta {
        version_id: m.version_id.as_ref().map(|v| Id(v.clone())),
        last_updated: m.last_updated.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| Instant(dt.with_timezone(&chrono::Utc)))
        }),
        source: m.source.as_ref().map(|s| Uri(s.clone())),
        profile: None,
        security: None,
        tag: None,
    })
}

fn to_proto_human_name(name: &HumanName) -> proto::HumanName {
    proto::HumanName {
        r#use: name.use_.as_ref().map(|c| c.0.clone()),
        text: name.text.as_ref().map(|t| t.0.clone()),
        family: name.family.as_ref().map(|f| f.0.clone()),
        given: name.given.as_ref().map(|g| g.iter().map(|s| s.0.clone()).collect()).unwrap_or_default(),
        prefix: name.prefix.as_ref().map(|p| p.iter().map(|s| s.0.clone()).collect()).unwrap_or_default(),
        suffix: name.suffix.as_ref().map(|s| s.iter().map(|s| s.0.clone()).collect()).unwrap_or_default(),
    }
}

fn from_proto_human_name(name: &proto::HumanName) -> HumanName {
    HumanName {
        use_: name.r#use.as_ref().map(|s| Code(s.clone())),
        text: name.text.as_ref().map(|t| FhirString(t.clone())),
        family: name.family.as_ref().map(|f| FhirString(f.clone())),
        given: if name.given.is_empty() {
            None
        } else {
            Some(name.given.iter().map(|s| FhirString(s.clone())).collect())
        },
        prefix: if name.prefix.is_empty() {
            None
        } else {
            Some(name.prefix.iter().map(|s| FhirString(s.clone())).collect())
        },
        suffix: if name.suffix.is_empty() {
            None
        } else {
            Some(name.suffix.iter().map(|s| FhirString(s.clone())).collect())
        },
        period: None,
    }
}

fn to_proto_codeable_concept(cc: &CodeableConcept) -> proto::CodeableConcept {
    proto::CodeableConcept {
        coding: cc.coding.as_ref().map(|codes| {
            codes.iter().map(to_proto_coding).collect()
        }).unwrap_or_default(),
        text: cc.text.as_ref().map(|t| t.0.clone()),
    }
}

fn from_proto_codeable_concept(cc: &proto::CodeableConcept) -> CodeableConcept {
    CodeableConcept {
        coding: if cc.coding.is_empty() {
            None
        } else {
            Some(cc.coding.iter().map(from_proto_coding).collect())
        },
        text: cc.text.as_ref().map(|t| FhirString(t.clone())),
    }
}

fn to_proto_coding(coding: &Coding) -> proto::Coding {
    proto::Coding {
        system: coding.system.as_ref().map(|s| s.0.clone()),
        code: coding.code.as_ref().map(|c| c.0.clone()),
        display: coding.display.as_ref().map(|d| d.0.clone()),
        version: coding.version.as_ref().map(|v| v.0.clone()),
    }
}

fn from_proto_coding(coding: &proto::Coding) -> Coding {
    Coding {
        system: coding.system.as_ref().map(|s| Uri(s.clone())),
        code: coding.code.as_ref().map(|c| Code(c.clone())),
        display: coding.display.as_ref().map(|d| FhirString(d.clone())),
        version: coding.version.as_ref().map(|v| FhirString(v.clone())),
        user_selected: None,
    }
}

fn to_proto_reference(reference: &Reference) -> proto::Reference {
    proto::Reference {
        reference: reference.reference.as_ref().map(|r| r.0.clone()),
        r#type: reference.type_.as_ref().map(|t| t.0.clone()),
        display: reference.display.as_ref().map(|d| d.0.clone()),
    }
}

fn from_proto_reference(reference: &proto::Reference) -> Reference {
    Reference {
        reference: reference.reference.as_ref().map(|r| FhirString(r.clone())),
        type_: reference.r#type.as_ref().map(|t| Uri(t.clone())),
        identifier: None,
        display: reference.display.as_ref().map(|d| FhirString(d.clone())),
    }
}

fn to_proto_period(period: &Period) -> proto::Period {
    proto::Period {
        start: period.start.as_ref().map(|dt| dt.0.to_rfc3339()),
        end: period.end.as_ref().map(|dt| dt.0.to_rfc3339()),
    }
}

fn from_proto_period(period: &proto::Period) -> Period {
    Period {
        start: period.start.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| FhirDateTime(dt.with_timezone(&chrono::Utc)))
        }),
        end: period.end.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| FhirDateTime(dt.with_timezone(&chrono::Utc)))
        }),
    }
}

fn to_proto_quantity(quantity: &domain::Quantity) -> proto::Quantity {
    proto::Quantity {
        value: quantity.value.as_ref().map(|v| v.0),
        unit: quantity.unit.as_ref().map(|u| u.0.clone()),
        system: quantity.system.as_ref().map(|s| s.0.clone()),
        code: quantity.code.as_ref().map(|c| c.0.clone()),
    }
}

fn from_proto_quantity(quantity: &proto::Quantity) -> domain::Quantity {
    domain::Quantity {
        value: quantity.value.map(FhirDecimal),
        comparator: None,
        unit: quantity.unit.as_ref().map(|u| FhirString(u.clone())),
        system: quantity.system.as_ref().map(|s| Uri(s.clone())),
        code: quantity.code.as_ref().map(|c| Code(c.clone())),
    }
}

// Patient conversions
pub fn to_proto_patient(patient: &domain::Patient) -> proto::Patient {
    proto::Patient {
        id: patient.id.as_ref().map(|id| id.0.clone()),
        meta: to_proto_meta(&patient.meta),
        identifier: vec![], // Simplified - implement if needed
        active: patient.active.as_ref().map(|a| a.0),
        name: patient.name.as_ref().map(|names| {
            names.iter().map(to_proto_human_name).collect()
        }).unwrap_or_default(),
        telecom: vec![], // Simplified - implement if needed
        gender: patient.gender.as_ref().map(|g| g.0.clone()),
        birth_date: patient.birth_date.as_ref().map(|d| d.0.to_string()),
        address: vec![], // Simplified - implement if needed
        marital_status: patient.marital_status.as_ref().map(to_proto_codeable_concept),
    }
}

pub fn from_proto_patient(proto: &proto::Patient) -> domain::Patient {
    use chrono::NaiveDate;

    domain::Patient {
        resource_type: "Patient".to_string(),
        id: proto.id.as_ref().map(|id| Id(id.clone())),
        meta: from_proto_meta(&proto.meta),
        identifier: None,
        active: proto.active.map(FhirBoolean),
        name: if proto.name.is_empty() {
            None
        } else {
            Some(proto.name.iter().map(from_proto_human_name).collect())
        },
        telecom: None,
        gender: proto.gender.as_ref().map(|g| Code(g.clone())),
        birth_date: proto.birth_date.as_ref().and_then(|d| {
            NaiveDate::parse_from_str(d, "%Y-%m-%d").ok().map(FhirDate)
        }),
        deceased: None,
        address: None,
        marital_status: proto.marital_status.as_ref().map(from_proto_codeable_concept),
        multiple_birth: None,
        contact: None,
        communication: None,
        general_practitioner: None,
        managing_organization: None,
    }
}

// Observation conversions
pub fn to_proto_observation(observation: &domain::Observation) -> proto::Observation {
    let (value_quantity, value_string, value_boolean) = match &observation.value {
        Some(observation::ObservationValue::Quantity(q)) => (Some(to_proto_quantity(q)), None, None),
        Some(observation::ObservationValue::String(s)) => (None, Some(s.0.clone()), None),
        Some(observation::ObservationValue::Boolean(b)) => (None, None, Some(b.0)),
        _ => (None, None, None),
    };

    // Extract effective date time from the enum
    let effective_date_time = match &observation.effective {
        Some(observation::ObservationEffective::DateTime(dt)) => Some(dt.0.to_rfc3339()),
        _ => None,
    };

    proto::Observation {
        id: observation.id.as_ref().map(|id| id.0.clone()),
        meta: to_proto_meta(&observation.meta),
        identifier: vec![],
        status: Some(observation.status.0.clone()),
        code: Some(to_proto_codeable_concept(&observation.code)),
        subject: observation.subject.as_ref().map(to_proto_reference),
        effective_date_time,
        value: if value_quantity.is_some() {
            Some(proto::observation::Value::ValueQuantity(value_quantity.unwrap()))
        } else if value_string.is_some() {
            Some(proto::observation::Value::ValueString(value_string.unwrap()))
        } else if value_boolean.is_some() {
            Some(proto::observation::Value::ValueBoolean(value_boolean.unwrap()))
        } else {
            None
        },
    }
}

pub fn from_proto_observation(proto: &proto::Observation) -> domain::Observation {
    let value = match &proto.value {
        Some(proto::observation::Value::ValueQuantity(q)) => {
            Some(observation::ObservationValue::Quantity(from_proto_quantity(q)))
        }
        Some(proto::observation::Value::ValueString(s)) => {
            Some(observation::ObservationValue::String(FhirString(s.clone())))
        }
        Some(proto::observation::Value::ValueBoolean(b)) => {
            Some(observation::ObservationValue::Boolean(FhirBoolean(*b)))
        }
        None => None,
    };

    let effective = proto.effective_date_time.as_ref().and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| observation::ObservationEffective::DateTime(FhirDateTime(dt.with_timezone(&chrono::Utc))))
    });

    domain::Observation {
        resource_type: "Observation".to_string(),
        id: proto.id.as_ref().map(|id| Id(id.clone())),
        meta: from_proto_meta(&proto.meta),
        identifier: None,
        based_on: None,
        part_of: None,
        status: proto.status.as_ref().map(|s| Code(s.clone())).unwrap_or(Code("final".to_string())),
        category: None,
        code: proto.code.as_ref().map(from_proto_codeable_concept).unwrap_or_else(|| CodeableConcept {
            coding: None,
            text: None,
        }),
        subject: proto.subject.as_ref().map(from_proto_reference),
        focus: None,
        encounter: None,
        effective,
        issued: None,
        performer: None,
        value,
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

// Condition conversions
pub fn to_proto_condition(condition: &domain::Condition) -> proto::Condition {
    // Extract onset datetime from the enum
    let onset_date_time = match &condition.onset {
        Some(condition::ConditionOnset::DateTime(dt)) => Some(dt.0.to_rfc3339()),
        _ => None,
    };

    proto::Condition {
        id: condition.id.as_ref().map(|id| id.0.clone()),
        meta: to_proto_meta(&condition.meta),
        identifier: vec![],
        clinical_status: condition.clinical_status.as_ref().map(to_proto_codeable_concept),
        verification_status: condition.verification_status.as_ref().map(to_proto_codeable_concept),
        code: condition.code.as_ref().map(to_proto_codeable_concept),
        subject: Some(to_proto_reference(&condition.subject)),
        onset_date_time,
        recorded_date: condition.recorded_date.as_ref().map(|dt| dt.0.to_rfc3339()),
    }
}

pub fn from_proto_condition(proto: &proto::Condition) -> domain::Condition {
    let onset = proto.onset_date_time.as_ref().and_then(|s| {
        chrono::DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| condition::ConditionOnset::DateTime(FhirDateTime(dt.with_timezone(&chrono::Utc))))
    });

    domain::Condition {
        resource_type: "Condition".to_string(),
        id: proto.id.as_ref().map(|id| Id(id.clone())),
        meta: from_proto_meta(&proto.meta),
        identifier: None,
        clinical_status: proto.clinical_status.as_ref().map(from_proto_codeable_concept),
        verification_status: proto.verification_status.as_ref().map(from_proto_codeable_concept),
        category: None,
        severity: None,
        code: proto.code.as_ref().map(from_proto_codeable_concept),
        body_site: None,
        subject: proto.subject.as_ref().map(from_proto_reference).unwrap_or_else(|| Reference {
            reference: None,
            type_: None,
            identifier: None,
            display: None,
        }),
        encounter: None,
        onset,
        abatement: None,
        recorded_date: proto.recorded_date.as_ref().and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| FhirDateTime(dt.with_timezone(&chrono::Utc)))
        }),
        recorder: None,
        asserter: None,
        stage: None,
        evidence: None,
        note: None,
    }
}

// Encounter conversions
pub fn to_proto_encounter(encounter: &domain::Encounter) -> proto::Encounter {
    proto::Encounter {
        id: encounter.id.as_ref().map(|id| id.0.clone()),
        meta: to_proto_meta(&encounter.meta),
        identifier: vec![],
        status: Some(encounter.status.0.clone()),
        class: Some(to_proto_coding(&encounter.class)),
        subject: encounter.subject.as_ref().map(to_proto_reference),
        period: encounter.period.as_ref().map(to_proto_period),
    }
}

pub fn from_proto_encounter(proto: &proto::Encounter) -> domain::Encounter {
    domain::Encounter {
        resource_type: "Encounter".to_string(),
        id: proto.id.as_ref().map(|id| Id(id.clone())),
        meta: from_proto_meta(&proto.meta),
        identifier: None,
        status: proto.status.as_ref().map(|s| Code(s.clone())).unwrap_or(Code("planned".to_string())),
        status_history: None,
        class: proto.class.as_ref().map(from_proto_coding).unwrap_or_else(|| Coding {
            system: None,
            version: None,
            code: None,
            display: None,
            user_selected: None,
        }),
        class_history: None,
        type_: None,
        service_type: None,
        priority: None,
        subject: proto.subject.as_ref().map(from_proto_reference),
        episode_of_care: None,
        based_on: None,
        participant: None,
        appointment: None,
        period: proto.period.as_ref().map(from_proto_period),
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
