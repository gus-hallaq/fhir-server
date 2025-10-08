// src/repository/encounter_repository.rs

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

use crate::domain::{Encounter, Id, Meta, Instant, FhirError, FhirResult};
use super::{Repository, SearchParams};
use crate::domain::resources::Resource;

pub struct EncounterRepository {
    pool: PgPool,
}

impl EncounterRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    fn extract_search_fields(&self, encounter: &Encounter) -> EncounterSearchFields {
        EncounterSearchFields {
            status: encounter.status.0.clone(),
            class_code: encounter.class.code.as_ref().map(|c| c.0.clone()),
            subject_id: encounter.subject.as_ref()
                .and_then(|r| r.reference.as_ref())
                .and_then(|ref_str| {
                    ref_str.0.split('/').last().and_then(|id| Uuid::parse_str(id).ok())
                }),
            period_start: encounter.period.as_ref()
                .and_then(|p| p.start.as_ref())
                .map(|s| s.0),
            period_end: encounter.period.as_ref()
                .and_then(|p| p.end.as_ref())
                .map(|e| e.0),
        }
    }
    
    pub async fn search_by_patient(&self, patient_id: &str) -> FhirResult<Vec<Encounter>> {
        let uuid = Uuid::parse_str(patient_id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", patient_id)))?;
        
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM encounters
            WHERE subject_id = $1 AND deleted_at IS NULL
            ORDER BY period_start DESC
            LIMIT 100
            "#
        )
        .bind(uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut encounters = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let encounter: Encounter = serde_json::from_value(resource_json)?;
            encounters.push(encounter);
        }
        
        Ok(encounters)
    }
    
    pub async fn search_by_status(&self, status: &str) -> FhirResult<Vec<Encounter>> {
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM encounters
            WHERE status = $1 AND deleted_at IS NULL
            ORDER BY period_start DESC
            LIMIT 100
            "#
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut encounters = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let encounter: Encounter = serde_json::from_value(resource_json)?;
            encounters.push(encounter);
        }
        
        Ok(encounters)
    }
}

#[async_trait::async_trait]
impl Repository<Encounter> for EncounterRepository {
    async fn create(&self, encounter: &Encounter) -> FhirResult<Encounter> {
        let mut enc = encounter.clone();
        
        let id = Uuid::new_v4().to_string();
        enc.set_id(Id(id.clone()));
        
        let meta = Meta {
            version_id: Some(Id("1".to_string())),
            last_updated: Some(Instant(Utc::now())),
            source: None,
            profile: None,
            security: None,
            tag: None,
        };
        enc.set_meta(meta);
        
        let search_fields = self.extract_search_fields(&enc);
        let resource_json = serde_json::to_value(&enc)?;
        
        let uuid = Uuid::parse_str(&id)
            .map_err(|_| FhirError::Database("Failed to parse UUID".to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO encounters (
                id, resource, status, class_code, subject_id,
                period_start, period_end
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .bind(&search_fields.status)
        .bind(search_fields.class_code)
        .bind(search_fields.subject_id)
        .bind(search_fields.period_start)
        .bind(search_fields.period_end)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        // Insert into history
        sqlx::query(
            r#"
            INSERT INTO encounters_history (id, version_id, resource, last_updated, operation)
            VALUES ($1, 1, $2, NOW(), 'CREATE')
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        Ok(enc)
    }
    
    async fn read(&self, id: &str) -> FhirResult<Option<Encounter>> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let row = sqlx::query(
            r#"
            SELECT resource
            FROM encounters
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
            let encounter: Encounter = serde_json::from_value(resource_json)?;
            Ok(Some(encounter))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&self, id: &str, encounter: &Encounter) -> FhirResult<Encounter> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let current = self.read(id).await?
            .ok_or_else(|| FhirError::NotFound {
                resource_type: "Encounter".to_string(),
                id: id.to_string(),
            })?;
        
        let current_version = current.meta
            .and_then(|m| m.version_id)
            .and_then(|v| v.0.parse::<i32>().ok())
            .unwrap_or(1);
        
        let new_version = current_version + 1;
        
        let mut updated_enc = encounter.clone();
        updated_enc.set_id(Id(id.to_string()));
        
        let meta = Meta {
            version_id: Some(Id(new_version.to_string())),
            last_updated: Some(Instant(Utc::now())),
            source: None,
            profile: None,
            security: None,
            tag: None,
        };
        updated_enc.set_meta(meta);
        
        let search_fields = self.extract_search_fields(&updated_enc);
        let resource_json = serde_json::to_value(&updated_enc)?;
        
        sqlx::query(
            r#"
            UPDATE encounters
            SET resource = $2,
                version_id = $3,
                last_updated = NOW(),
                status = $4,
                class_code = $5,
                subject_id = $6,
                period_start = $7,
                period_end = $8
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .bind(new_version)
        .bind(&search_fields.status)
        .bind(search_fields.class_code)
        .bind(search_fields.subject_id)
        .bind(search_fields.period_start)
        .bind(search_fields.period_end)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        // Insert into history
        sqlx::query(
            r#"
            INSERT INTO encounters_history (id, version_id, resource, last_updated, operation)
            VALUES ($1, $2, $3, NOW(), 'UPDATE')
            "#
        )
        .bind(uuid)
        .bind(new_version)
        .bind(&resource_json)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        Ok(updated_enc)
    }
    
    async fn delete(&self, id: &str) -> FhirResult<()> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let result = sqlx::query(
            r#"
            UPDATE encounters
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
                resource_type: "Encounter".to_string(),
                id: id.to_string(),
            });
        }
        
        Ok(())
    }
    
    async fn search(&self, params: SearchParams) -> FhirResult<Vec<Encounter>> {
        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);
        
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM encounters
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
        
        let mut encounters = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let encounter: Encounter = serde_json::from_value(resource_json)?;
            encounters.push(encounter);
        }
        
        Ok(encounters)
    }
}

struct EncounterSearchFields {
    status: String,
    class_code: Option<String>,
    subject_id: Option<Uuid>,
    period_start: Option<chrono::DateTime<Utc>>,
    period_end: Option<chrono::DateTime<Utc>>,
}