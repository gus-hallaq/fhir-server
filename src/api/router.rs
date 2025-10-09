// src/api/router.rs

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::AppState;
use super::handlers::{
    // Auth handlers
    login, register, me,

    // Patient handlers
    create_patient, get_patient, update_patient, delete_patient,
    search_patients, get_patient_history,

    // Observation handlers
    create_observation, get_observation, update_observation, delete_observation,
    search_observations, get_observation_history,

    // Condition handlers
    create_condition, get_condition, update_condition, delete_condition,
    search_conditions, get_condition_history,

    // Encounter handlers
    create_encounter, get_encounter, update_encounter, delete_encounter,
    search_encounters, get_encounter_history,
};

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health check endpoint
        .route("/health", get(health_check))

        // Auth routes (public)
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/me", get(me))

        // Patient routes
        .route("/fhir/Patient", post(create_patient))
        .route("/fhir/Patient", get(search_patients))
        .route("/fhir/Patient/:id", get(get_patient))
        .route("/fhir/Patient/:id", put(update_patient))
        .route("/fhir/Patient/:id", delete(delete_patient))
        .route("/fhir/Patient/:id/_history", get(get_patient_history))

        // Observation routes
        .route("/fhir/Observation", post(create_observation))
        .route("/fhir/Observation", get(search_observations))
        .route("/fhir/Observation/:id", get(get_observation))
        .route("/fhir/Observation/:id", put(update_observation))
        .route("/fhir/Observation/:id", delete(delete_observation))
        .route("/fhir/Observation/:id/_history", get(get_observation_history))

        // Condition routes
        .route("/fhir/Condition", post(create_condition))
        .route("/fhir/Condition", get(search_conditions))
        .route("/fhir/Condition/:id", get(get_condition))
        .route("/fhir/Condition/:id", put(update_condition))
        .route("/fhir/Condition/:id", delete(delete_condition))
        .route("/fhir/Condition/:id/_history", get(get_condition_history))

        // Encounter routes
        .route("/fhir/Encounter", post(create_encounter))
        .route("/fhir/Encounter", get(search_encounters))
        .route("/fhir/Encounter/:id", get(get_encounter))
        .route("/fhir/Encounter/:id", put(update_encounter))
        .route("/fhir/Encounter/:id", delete(delete_encounter))
        .route("/fhir/Encounter/:id/_history", get(get_encounter_history))

        // Add middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())

        // Add application state
        .with_state(state)
}

/// Health check handler
async fn health_check() -> &'static str {
    "OK"
}
