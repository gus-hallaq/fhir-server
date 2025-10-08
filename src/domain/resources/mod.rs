// src/domain/resources/mod.rs

pub mod patient;
pub mod observation;
pub mod condition;
pub mod encounter;

pub use patient::Patient;
pub use observation::Observation;
pub use condition::Condition;
pub use encounter::Encounter;

use crate::domain::primitives::{Id};
use crate::domain::datatypes::Meta;

/// Base trait for all FHIR resources
pub trait Resource {
    fn resource_type() -> &'static str;
    fn id(&self) -> Option<&Id>;
    fn meta(&self) -> Option<&Meta>;
    fn set_id(&mut self, id: Id);
    fn set_meta(&mut self, meta: Meta);
}