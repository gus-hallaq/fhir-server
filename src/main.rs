// src/main.rs

mod api;
mod config;
mod domain;
mod repository;
mod service;
mod grpc;

use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use domain::resources::observation::ObservationValue;

use config::{DatabaseConfig, GrpcConfig};
use repository::{
    PatientRepository, 
    ObservationRepository, 
    ConditionRepository, 
    EncounterRepository,
};
use service::{
    PatientService, 
    ObservationService, 
    ConditionService, 
    EncounterService,
};

/// Application state that will be shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub patient_service: Arc<PatientService>,
    pub observation_service: Arc<ObservationService>,
    pub condition_service: Arc<ConditionService>,
    pub encounter_service: Arc<EncounterService>,
}

impl AppState {
    pub fn new(
        patient_service: PatientService,
        observation_service: ObservationService,
        condition_service: ConditionService,
        encounter_service: EncounterService,
    ) -> Self {
        Self {
            patient_service: Arc::new(patient_service),
            observation_service: Arc::new(observation_service),
            condition_service: Arc::new(condition_service),
            encounter_service: Arc::new(encounter_service),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fhir_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("🚀 Starting FHIR Server...");
    
    // Initialize database
    info!("📦 Connecting to database...");
    let db_config = DatabaseConfig::from_env();
    let pool = db_config.create_pool().await?;
    info!("✅ Database connection established");
    
    // Run migrations (commented out - function not yet implemented)
    // info!("🔄 Running database migrations...");
    // run_migrations(&pool).await?;
    // info!("✅ Migrations completed");
    
    // Initialize repositories
    info!("🏗️  Initializing repositories...");
    let patient_repo = PatientRepository::new(pool.clone());
    let observation_repo = ObservationRepository::new(pool.clone());
    let condition_repo = ConditionRepository::new(pool.clone());
    let encounter_repo = EncounterRepository::new(pool.clone());
    info!("✅ Repositories initialized");
    
    // Initialize services
    info!("⚙️  Initializing services...");
    let patient_service = PatientService::new(patient_repo);
    let observation_service = ObservationService::new(observation_repo);
    let condition_service = ConditionService::new(condition_repo);
    let encounter_service = EncounterService::new(encounter_repo);
    info!("✅ Services initialized");
    
    // Create application state
    let app_state = AppState::new(
        patient_service,
        observation_service,
        condition_service,
        encounter_service,
    );
    
    info!("🎉 FHIR Server initialized successfully!");
    
    // Run example operations
    if std::env::var("RUN_EXAMPLES").unwrap_or_default() == "true" {
        info!("🧪 Running example operations...");
        if let Err(e) = run_examples(app_state.clone()).await {
            error!("❌ Example operations failed: {}", e);
        } else {
            info!("✅ Example operations completed");
        }
    }
    
    // Create and start the Axum web server
    info!("🌐 Starting web server...");
    let app = api::create_router(app_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await?;

    info!("✅ FHIR Server is ready!");
    info!("📊 HTTP Server listening on http://0.0.0.0:8080");

    // Initialize gRPC configuration
    let grpc_config = GrpcConfig::from_env();
    let grpc_addr = grpc_config.address();
    let grpc_tls_enabled = grpc_config.tls_enabled;

    // Spawn gRPC server in a separate task
    let grpc_state = app_state.clone();
    let grpc_handle = tokio::spawn(async move {
        if let Err(e) = grpc::start_grpc_server(grpc_state, grpc_config).await {
            error!("❌ gRPC server error: {}", e);
        }
    });

    if grpc_tls_enabled {
        info!("📡 Secure gRPC Server (TLS) listening on {}", grpc_addr);
    } else {
        info!("📡 gRPC Server listening on {}", grpc_addr);
    }
    info!("🎉 All servers running!");

    // Run both servers concurrently
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                error!("❌ HTTP server error: {}", e);
            }
        }
        result = grpc_handle => {
            if let Err(e) = result {
                error!("❌ gRPC server task error: {}", e);
            }
        }
    }

    info!("🛑 Shutting down gracefully...");

    Ok(())
}

/// Example operations to demonstrate the system
async fn run_examples(state: AppState) -> Result<()> {
    use domain::{
        Patient, Observation, Condition, Encounter,
        HumanName, FhirString, Code, FhirBoolean,
        CodeableConcept, Coding, Uri, Reference, Period, FhirDateTime,
    };
    use service::{ResourceService, SecurityContext};
    use chrono::Utc;

    // Create a system security context for these example operations
    let system_context = SecurityContext::system();

    info!("Creating example patient...");
    
    // Create a patient
    let mut patient = Patient::new();
    patient.name = Some(vec![HumanName {
        use_: Some(Code("official".to_string())),
        text: Some(FhirString("John Doe".to_string())),
        family: Some(FhirString("Doe".to_string())),
        given: Some(vec![FhirString("John".to_string())]),
        prefix: None,
        suffix: None,
        period: None,
    }]);
    patient.gender = Some(Code("male".to_string()));
    patient.active = Some(FhirBoolean(true));

    let created_patient = state.patient_service.create(&system_context, patient).await?;
    let patient_id = created_patient.id.as_ref().unwrap().0.clone();
    info!("✅ Created patient with ID: {}", patient_id);

    // Create an observation for the patient
    info!("Creating example observation...");

    let mut observation = Observation::new(
        Code("final".to_string()),
        CodeableConcept {
            coding: Some(vec![Coding {
                system: Some(Uri("http://loinc.org".to_string())),
                code: Some(Code("8867-4".to_string())),
                display: Some(FhirString("Heart rate".to_string())),
                version: None,
                user_selected: None,
            }]),
            text: Some(FhirString("Heart rate".to_string())),
        }
    );
    
    observation.subject = Some(Reference {
        reference: Some(FhirString(format!("Patient/{}", patient_id))),
        type_: Some(Uri("Patient".to_string())),
        identifier: None,
        display: Some(FhirString("John Doe".to_string())),
    });
    
    observation.value = Some(ObservationValue::Quantity(domain::Quantity {
        value: Some(domain::FhirDecimal(72.0)),
        unit: Some(FhirString("beats/min".to_string())),
        system: Some(Uri("http://unitsofmeasure.org".to_string())),
        code: Some(Code("/min".to_string())),
        comparator: None,
    }));

    let created_observation = state.observation_service.create(&system_context, observation).await?;
    let observation_id = created_observation.id.as_ref().unwrap().0.clone();
    info!("✅ Created observation with ID: {}", observation_id);

    // Create a condition for the patient
    info!("Creating example condition...");
    
    let mut condition = Condition::new(Reference {
        reference: Some(FhirString(format!("Patient/{}", patient_id))),
        type_: Some(Uri("Patient".to_string())),
        identifier: None,
        display: Some(FhirString("John Doe".to_string())),
    });
    
    condition.code = Some(CodeableConcept {
        coding: Some(vec![Coding {
            system: Some(Uri("http://snomed.info/sct".to_string())),
            code: Some(Code("38341003".to_string())),
            display: Some(FhirString("Hypertension".to_string())),
            version: None,
            user_selected: None,
        }]),
        text: Some(FhirString("Hypertension".to_string())),
    });
    
    condition.clinical_status = Some(CodeableConcept {
        coding: Some(vec![Coding {
            system: Some(Uri("http://terminology.hl7.org/CodeSystem/condition-clinical".to_string())),
            code: Some(Code("active".to_string())),
            display: Some(FhirString("Active".to_string())),
            version: None,
            user_selected: None,
        }]),
        text: None,
    });

    let created_condition = state.condition_service.create(&system_context, condition).await?;
    let condition_id = created_condition.id.as_ref().unwrap().0.clone();
    info!("✅ Created condition with ID: {}", condition_id);

    // Create an encounter for the patient
    info!("Creating example encounter...");
    
    let mut encounter = Encounter::new(
        Code("in-progress".to_string()),
        Coding {
            system: Some(Uri("http://terminology.hl7.org/CodeSystem/v3-ActCode".to_string())),
            code: Some(Code("AMB".to_string())),
            display: Some(FhirString("ambulatory".to_string())),
            version: None,
            user_selected: None,
        }
    );
    
    encounter.subject = Some(Reference {
        reference: Some(FhirString(format!("Patient/{}", patient_id))),
        type_: Some(Uri("Patient".to_string())),
        identifier: None,
        display: Some(FhirString("John Doe".to_string())),
    });
    
    encounter.period = Some(Period {
        start: Some(FhirDateTime(Utc::now())),
        end: None,
    });

    let created_encounter = state.encounter_service.create(&system_context, encounter).await?;
    let encounter_id = created_encounter.id.as_ref().unwrap().0.clone();
    info!("✅ Created encounter with ID: {}", encounter_id);

    // Demonstrate search operations
    info!("Testing search operations...");

    // Search by family name
    let patients = state.patient_service.search_by_family(&system_context, "Doe").await?;
    info!("✅ Found {} patients with family name 'Doe'", patients.len());

    // Search observations by patient
    let observations = state.observation_service.search_by_patient(&system_context, &patient_id).await?;
    info!("✅ Found {} observations for patient", observations.len());

    // Get active conditions
    let active_conditions = state.condition_service.get_active_conditions(&system_context, &patient_id).await?;
    info!("✅ Found {} active conditions for patient", active_conditions.len());

    // Get active encounters
    let active_encounters = state.encounter_service.get_active_encounters(&system_context, &patient_id).await?;
    info!("✅ Found {} active encounters for patient", active_encounters.len());

    // Demonstrate update operation
    info!("Testing update operation...");
    let retrieved_patient = state.patient_service.get(&system_context, &patient_id).await?;
    let mut updated_patient = retrieved_patient.clone();
    updated_patient.active = Some(FhirBoolean(false));

    let _updated = state.patient_service.update(&system_context, &patient_id, updated_patient).await?;
    info!("✅ Updated patient status to inactive");

    // Get patient history
    let history = state.patient_service.get_history(&system_context, &patient_id).await?;
    info!("✅ Patient has {} versions in history", history.len());
    
    info!("🎉 All example operations completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_app_state_creation() {
        // This is a basic test structure
        // In real tests, you'd set up a test database
        
        // For now, just verify the test infrastructure works
        assert!(true);
    }
}