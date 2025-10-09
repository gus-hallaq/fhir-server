// src/api/handlers/mod.rs

pub mod auth_handlers;
pub mod patient;
pub mod observation;
pub mod condition;
pub mod encounter;
pub mod common;

pub use auth_handlers::*;
pub use patient::*;
pub use observation::*;
pub use condition::*;
pub use encounter::*;
