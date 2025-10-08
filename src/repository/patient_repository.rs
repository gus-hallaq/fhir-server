// src/repository/patient_repository.rs

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

use crate::domain::{Patient, Id, Meta, Instant, FhirError, FhirResult};
use super::{Repository, SearchParams};
use crate::domain::resources::patient::PatientDeceased;
use crate::domain::resources::Resource;
pub struct PatientRepository {
    pool: PgPool,
}

impl PatientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Extract searchable fields from Patient resource
    fn extract_search_fields(&self, patient: &Patient) -> PatientSearchFields {
        PatientSearchFields {
            active: patient.active.as_ref().map(|b| b.0),
            family_name: patient.name.as_ref()
                .and_then(|names| names.first())
                .and_then(|name| name.family.as_ref())
                .map(|f| f.0.clone()),
            given_name: patient.name.as_ref()
                .and_then(|names| names.first())
                .and_then(|name| name.given.as_ref())
                .and_then(|given| given.first())
                .map(|g| g.0.clone()),
            gender: patient.gender.as_ref().map(|g| g.0.clone()),
            birth_date: patient.birth_date.as_ref().map(|d| d.0),
            deceased: match &patient.deceased {
                Some(PatientDeceased::Boolean(b)) => Some(b.0),
                Some(PatientDeceased::DateTime(_)) => Some(true),
                None => None,
            },
        }
    }
    
    /// Get patient history (all versions)
    pub async fn get_history(&self, id: &str) -> FhirResult<Vec<Patient>> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM patients_history
            WHERE id = $1
            ORDER BY version_id DESC
            "#
        )
        .bind(uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut patients = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let patient: Patient = serde_json::from_value(resource_json)?;
            patients.push(patient);
        }
        
        Ok(patients)
    }
    
    /// Search by family name
    pub async fn search_by_family(&self, family: &str) -> FhirResult<Vec<Patient>> {
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM patients
            WHERE family_name ILIKE $1
              AND deleted_at IS NULL
            ORDER BY last_updated DESC
            LIMIT 100
            "#
        )
        .bind(format!("%{}%", family))
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut patients = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let patient: Patient = serde_json::from_value(resource_json)?;
            patients.push(patient);
        }
        
        Ok(patients)
    }
    
    /// Search by identifier
    pub async fn search_by_identifier(&self, system: &str, value: &str) -> FhirResult<Option<Patient>> {
        let row = sqlx::query(
            r#"
            SELECT resource
            FROM patients
            WHERE resource @> jsonb_build_object(
                'identifier', jsonb_build_array(
                    jsonb_build_object('system', $1, 'value', $2)
                )
            )
            AND deleted_at IS NULL
            LIMIT 1
            "#
        )
        .bind(system)
        .bind(value)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        if let Some(row) = row {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let patient: Patient = serde_json::from_value(resource_json)?;
            Ok(Some(patient))
        } else {
            Ok(None)
        }
    }
}

#[async_trait::async_trait]
impl Repository<Patient> for PatientRepository {
    async fn create(&self, patient: &Patient) -> FhirResult<Patient> {
        let mut patient = patient.clone();
        
        // Generate ID if not present
        let id = Uuid::new_v4().to_string();
        patient.set_id(Id(id.clone()));
        
        // Set meta
        let meta = Meta {
            version_id: Some(Id("1".to_string())),
            last_updated: Some(Instant(Utc::now())),
            source: None,
            profile: None,
            security: None,
            tag: None,
        };
        patient.set_meta(meta);
        
        let search_fields = self.extract_search_fields(&patient);
        let resource_json = serde_json::to_value(&patient)?;
        
        let uuid = Uuid::parse_str(&id)
            .map_err(|_| FhirError::Database("Failed to parse UUID".to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO patients (
                id, resource, active, family_name, given_name, 
                gender, birth_date, deceased
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .bind(search_fields.active)
        .bind(search_fields.family_name)
        .bind(search_fields.given_name)
        .bind(search_fields.gender)
        .bind(search_fields.birth_date)
        .bind(search_fields.deceased)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        // Insert into history
        sqlx::query(
            r#"
            INSERT INTO patients_history (id, version_id, resource, last_updated, operation)
            VALUES ($1, 1, $2, NOW(), 'CREATE')
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        Ok(patient)
    }
    
    async fn read(&self, id: &str) -> FhirResult<Option<Patient>> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let row = sqlx::query(
            r#"
            SELECT resource
            FROM patients
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        if let Some(row) = row {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let patient: Patient = serde_json::from_value(resource_json)?;
            Ok(Some(patient))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&self, id: &str, patient: &Patient) -> FhirResult<Patient> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        // Get current version
        let current = self.read(id).await?
            .ok_or_else(|| FhirError::NotFound {
                resource_type: "Patient".to_string(),
                id: id.to_string(),
            })?;
        
        let current_version = current.meta
            .and_then(|m| m.version_id)
            .and_then(|v| v.0.parse::<i32>().ok())
            .unwrap_or(1);
        
        let new_version = current_version + 1;
        
        let mut updated_patient = patient.clone();
        updated_patient.set_id(Id(id.to_string()));
        
        let meta = Meta {
            version_id: Some(Id(new_version.to_string())),
            last_updated: Some(Instant(Utc::now())),
            source: None,
            profile: None,
            security: None,
            tag: None,
        };
        updated_patient.set_meta(meta);
        
        let search_fields = self.extract_search_fields(&updated_patient);
        let resource_json = serde_json::to_value(&updated_patient)?;
        
        sqlx::query(
            r#"
            UPDATE patients
            SET resource = $2,
                version_id = $3,
                last_updated = NOW(),
                active = $4,
                family_name = $5,
                given_name = $6,
                gender = $7,
                birth_date = $8,
                deceased = $9
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .bind(new_version)
        .bind(search_fields.active)
        .bind(search_fields.family_name)
        .bind(search_fields.given_name)
        .bind(search_fields.gender)
        .bind(search_fields.birth_date)
        .bind(search_fields.deceased)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        // Insert into history
        sqlx::query(
            r#"
            INSERT INTO patients_history (id, version_id, resource, last_updated, operation)
            VALUES ($1, $2, $3, NOW(), 'UPDATE')
            "#
        )
        .bind(uuid)
        .bind(new_version)
        .bind(&resource_json)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        Ok(updated_patient)
    }
    
    async fn delete(&self, id: &str) -> FhirResult<()> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        // Soft delete
        let result = sqlx::query(
            r#"
            UPDATE patients
            SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        if result.rows_affected() == 0 {
            return Err(FhirError::NotFound {
                resource_type: "Patient".to_string(),
                id: id.to_string(),
            });
        }
        
        Ok(())
    }
    
    async fn search(&self, params: SearchParams) -> FhirResult<Vec<Patient>> {
        // Basic search implementation - can be extended
        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);
        
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM patients
            WHERE deleted_at IS NULL
            ORDER BY last_updated DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut patients = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let patient: Patient = serde_json::from_value(resource_json)?;
            patients.push(patient);
        }
        
        Ok(patients)
    }
}

struct PatientSearchFields {
    active: Option<bool>,
    family_name: Option<String>,
    given_name: Option<String>,
    gender: Option<String>,
    birth_date: Option<chrono::NaiveDate>,
    deceased: Option<bool>,
}