// src/repository/observation_repository.rs

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

use crate::domain::{Observation, Id, Meta, Instant, FhirError, FhirResult};
use super::{Repository, SearchParams};
use crate::domain::resources::observation::ObservationEffective;
use crate::domain::resources::Resource;

pub struct ObservationRepository {
    pool: PgPool,
}

impl ObservationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    fn extract_search_fields(&self, obs: &Observation) -> ObservationSearchFields {
        ObservationSearchFields {
            status: obs.status.0.clone(),
            subject_id: obs.subject.as_ref()
                .and_then(|r| r.reference.as_ref())
                .and_then(|ref_str| {
                    ref_str.0.split('/').last().and_then(|id| Uuid::parse_str(id).ok())
                }),
            category_code: obs.category.as_ref()
                .and_then(|cats| cats.first())
                .and_then(|cat| cat.coding.as_ref())
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.code.as_ref())
                .map(|code| code.0.clone()),
            code_code: obs.code.coding.as_ref()
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.code.as_ref())
                .map(|code| code.0.clone()),
            code_system: obs.code.coding.as_ref()
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.system.as_ref())
                .map(|sys| sys.0.clone()),
            effective_datetime: match &obs.effective {
                Some(ObservationEffective::DateTime(dt)) => Some(dt.0),
                _ => None,
            },
            issued: obs.issued.as_ref().map(|i| i.0),
        }
    }
    
    pub async fn search_by_patient(&self, patient_id: &str) -> FhirResult<Vec<Observation>> {
        let uuid = Uuid::parse_str(patient_id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", patient_id)))?;
        
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM observations
            WHERE subject_id = $1 AND deleted_at IS NULL
            ORDER BY effective_datetime DESC
            LIMIT 100
            "#
        )
        .bind(uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut observations = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let obs: Observation = serde_json::from_value(resource_json)?;
            observations.push(obs);
        }
        
        Ok(observations)
    }
    
    pub async fn search_by_code(&self, code: &str) -> FhirResult<Vec<Observation>> {
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM observations
            WHERE code_code = $1 AND deleted_at IS NULL
            ORDER BY effective_datetime DESC
            LIMIT 100
            "#
        )
        .bind(code)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut observations = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let obs: Observation = serde_json::from_value(resource_json)?;
            observations.push(obs);
        }
        
        Ok(observations)
    }
}

#[async_trait::async_trait]
impl Repository<Observation> for ObservationRepository {
    async fn create(&self, observation: &Observation) -> FhirResult<Observation> {
        let mut obs = observation.clone();
        
        let id = Uuid::new_v4().to_string();
        obs.set_id(Id(id.clone()));
        
        let meta = Meta {
            version_id: Some(Id("1".to_string())),
            last_updated: Some(Instant(Utc::now())),
            source: None,
            profile: None,
            security: None,
            tag: None,
        };
        obs.set_meta(meta);
        
        let search_fields = self.extract_search_fields(&obs);
        let resource_json = serde_json::to_value(&obs)?;
        
        let uuid = Uuid::parse_str(&id)
            .map_err(|_| FhirError::Database("Failed to parse UUID".to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO observations (
                id, resource, status, subject_id, category_code,
                code_code, code_system, effective_datetime, issued
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .bind(&search_fields.status)
        .bind(search_fields.subject_id)
        .bind(search_fields.category_code)
        .bind(search_fields.code_code)
        .bind(search_fields.code_system)
        .bind(search_fields.effective_datetime)
        .bind(search_fields.issued)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        // Insert into history
        sqlx::query(
            r#"
            INSERT INTO observations_history (id, version_id, resource, last_updated, operation)
            VALUES ($1, 1, $2, NOW(), 'CREATE')
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        Ok(obs)
    }
    
    async fn read(&self, id: &str) -> FhirResult<Option<Observation>> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let row = sqlx::query(
            r#"
            SELECT resource
            FROM observations
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
            let obs: Observation = serde_json::from_value(resource_json)?;
            Ok(Some(obs))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&self, id: &str, observation: &Observation) -> FhirResult<Observation> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let current = self.read(id).await?
            .ok_or_else(|| FhirError::NotFound {
                resource_type: "Observation".to_string(),
                id: id.to_string(),
            })?;
        
        let current_version = current.meta
            .and_then(|m| m.version_id)
            .and_then(|v| v.0.parse::<i32>().ok())
            .unwrap_or(1);
        
        let new_version = current_version + 1;
        
        let mut updated_obs = observation.clone();
        updated_obs.set_id(Id(id.to_string()));
        
        let meta = Meta {
            version_id: Some(Id(new_version.to_string())),
            last_updated: Some(Instant(Utc::now())),
            source: None,
            profile: None,
            security: None,
            tag: None,
        };
        updated_obs.set_meta(meta);
        
        let search_fields = self.extract_search_fields(&updated_obs);
        let resource_json = serde_json::to_value(&updated_obs)?;
        
        sqlx::query(
            r#"
            UPDATE observations
            SET resource = $2,
                version_id = $3,
                last_updated = NOW(),
                status = $4,
                subject_id = $5,
                category_code = $6,
                code_code = $7,
                code_system = $8,
                effective_datetime = $9,
                issued = $10
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .bind(new_version)
        .bind(&search_fields.status)
        .bind(search_fields.subject_id)
        .bind(search_fields.category_code)
        .bind(search_fields.code_code)
        .bind(search_fields.code_system)
        .bind(search_fields.effective_datetime)
        .bind(search_fields.issued)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        // Insert into history
        sqlx::query(
            r#"
            INSERT INTO observations_history (id, version_id, resource, last_updated, operation)
            VALUES ($1, $2, $3, NOW(), 'UPDATE')
            "#
        )
        .bind(uuid)
        .bind(new_version)
        .bind(&resource_json)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        Ok(updated_obs)
    }
    
    async fn delete(&self, id: &str) -> FhirResult<()> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let result = sqlx::query(
            r#"
            UPDATE observations
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
                resource_type: "Observation".to_string(),
                id: id.to_string(),
            });
        }
        
        Ok(())
    }
    
    async fn search(&self, params: SearchParams) -> FhirResult<Vec<Observation>> {
        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);
        
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM observations
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
        
        let mut observations = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let obs: Observation = serde_json::from_value(resource_json)?;
            observations.push(obs);
        }
        
        Ok(observations)
    }
}

struct ObservationSearchFields {
    status: String,
    subject_id: Option<Uuid>,
    category_code: Option<String>,
    code_code: Option<String>,
    code_system: Option<String>,
    effective_datetime: Option<chrono::DateTime<Utc>>,
    issued: Option<chrono::DateTime<Utc>>,
}