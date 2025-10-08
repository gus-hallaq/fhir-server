// src/repository/condition_repository.rs

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::Utc;

use crate::domain::{Condition, Id, Meta, Instant, FhirError, FhirResult};
use super::{Repository, SearchParams};
use crate::domain::resources::condition::ConditionOnset;
use crate::domain::resources::Resource;

pub struct ConditionRepository {
    pool: PgPool,
}

impl ConditionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    fn extract_search_fields(&self, condition: &Condition) -> ConditionSearchFields {
        ConditionSearchFields {
            subject_id: condition.subject.reference.as_ref()
                .and_then(|ref_str| {
                    ref_str.0.split('/').last().and_then(|id| Uuid::parse_str(id).ok())
                }),
            clinical_status: condition.clinical_status.as_ref()
                .and_then(|cs| cs.coding.as_ref())
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.code.as_ref())
                .map(|code| code.0.clone()),
            verification_status: condition.verification_status.as_ref()
                .and_then(|vs| vs.coding.as_ref())
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.code.as_ref())
                .map(|code| code.0.clone()),
            category_code: condition.category.as_ref()
                .and_then(|cats| cats.first())
                .and_then(|cat| cat.coding.as_ref())
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.code.as_ref())
                .map(|code| code.0.clone()),
            code_code: condition.code.as_ref()
                .and_then(|c| c.coding.as_ref())
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.code.as_ref())
                .map(|code| code.0.clone()),
            code_system: condition.code.as_ref()
                .and_then(|c| c.coding.as_ref())
                .and_then(|codings| codings.first())
                .and_then(|coding| coding.system.as_ref())
                .map(|sys| sys.0.clone()),
            onset_datetime: match &condition.onset {
                Some(ConditionOnset::DateTime(dt)) => Some(dt.0),
                _ => None,
            },
            recorded_date: condition.recorded_date.as_ref().map(|d| d.0),
        }
    }
    
    pub async fn search_by_patient(&self, patient_id: &str) -> FhirResult<Vec<Condition>> {
        let uuid = Uuid::parse_str(patient_id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", patient_id)))?;
        
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM conditions
            WHERE subject_id = $1 AND deleted_at IS NULL
            ORDER BY onset_datetime DESC
            LIMIT 100
            "#
        )
        .bind(uuid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut conditions = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let condition: Condition = serde_json::from_value(resource_json)?;
            conditions.push(condition);
        }
        
        Ok(conditions)
    }
    
    pub async fn search_by_clinical_status(&self, status: &str) -> FhirResult<Vec<Condition>> {
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM conditions
            WHERE clinical_status = $1 AND deleted_at IS NULL
            ORDER BY onset_datetime DESC
            LIMIT 100
            "#
        )
        .bind(status)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        let mut conditions = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let condition: Condition = serde_json::from_value(resource_json)?;
            conditions.push(condition);
        }
        
        Ok(conditions)
    }
}

#[async_trait::async_trait]
impl Repository<Condition> for ConditionRepository {
    async fn create(&self, condition: &Condition) -> FhirResult<Condition> {
        let mut cond = condition.clone();
        
        let id = Uuid::new_v4().to_string();
        cond.set_id(Id(id.clone()));
        
        let meta = Meta {
            version_id: Some(Id("1".to_string())),
            last_updated: Some(Instant(Utc::now())),
            source: None,
            profile: None,
            security: None,
            tag: None,
        };
        cond.set_meta(meta);
        
        let search_fields = self.extract_search_fields(&cond);
        let resource_json = serde_json::to_value(&cond)?;
        
        let uuid = Uuid::parse_str(&id)
            .map_err(|_| FhirError::Database("Failed to parse UUID".to_string()))?;
        
        sqlx::query(
            r#"
            INSERT INTO conditions (
                id, resource, subject_id, clinical_status, verification_status,
                category_code, code_code, code_system, onset_datetime, recorded_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .bind(search_fields.subject_id)
        .bind(search_fields.clinical_status)
        .bind(search_fields.verification_status)
        .bind(search_fields.category_code)
        .bind(search_fields.code_code)
        .bind(search_fields.code_system)
        .bind(search_fields.onset_datetime)
        .bind(search_fields.recorded_date)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        // Insert into history
        sqlx::query(
            r#"
            INSERT INTO conditions_history (id, version_id, resource, last_updated, operation)
            VALUES ($1, 1, $2, NOW(), 'CREATE')
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        Ok(cond)
    }
    
    async fn read(&self, id: &str) -> FhirResult<Option<Condition>> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let row = sqlx::query(
            r#"
            SELECT resource
            FROM conditions
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
            let condition: Condition = serde_json::from_value(resource_json)?;
            Ok(Some(condition))
        } else {
            Ok(None)
        }
    }
    
    async fn update(&self, id: &str, condition: &Condition) -> FhirResult<Condition> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let current = self.read(id).await?
            .ok_or_else(|| FhirError::NotFound {
                resource_type: "Condition".to_string(),
                id: id.to_string(),
            })?;
        
        let current_version = current.meta
            .and_then(|m| m.version_id)
            .and_then(|v| v.0.parse::<i32>().ok())
            .unwrap_or(1);
        
        let new_version = current_version + 1;
        
        let mut updated_cond = condition.clone();
        updated_cond.set_id(Id(id.to_string()));
        
        let meta = Meta {
            version_id: Some(Id(new_version.to_string())),
            last_updated: Some(Instant(Utc::now())),
            source: None,
            profile: None,
            security: None,
            tag: None,
        };
        updated_cond.set_meta(meta);
        
        let search_fields = self.extract_search_fields(&updated_cond);
        let resource_json = serde_json::to_value(&updated_cond)?;
        
        sqlx::query(
            r#"
            UPDATE conditions
            SET resource = $2,
                version_id = $3,
                last_updated = NOW(),
                subject_id = $4,
                clinical_status = $5,
                verification_status = $6,
                category_code = $7,
                code_code = $8,
                code_system = $9,
                onset_datetime = $10,
                recorded_date = $11
            WHERE id = $1 AND deleted_at IS NULL
            "#
        )
        .bind(uuid)
        .bind(&resource_json)
        .bind(new_version)
        .bind(search_fields.subject_id)
        .bind(search_fields.clinical_status)
        .bind(search_fields.verification_status)
        .bind(search_fields.category_code)
        .bind(search_fields.code_code)
        .bind(search_fields.code_system)
        .bind(search_fields.onset_datetime)
        .bind(search_fields.recorded_date)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        // Insert into history
        sqlx::query(
            r#"
            INSERT INTO conditions_history (id, version_id, resource, last_updated, operation)
            VALUES ($1, $2, $3, NOW(), 'UPDATE')
            "#
        )
        .bind(uuid)
        .bind(new_version)
        .bind(&resource_json)
        .execute(&self.pool)
        .await
        .map_err(|e| FhirError::Database(e.to_string()))?;
        
        Ok(updated_cond)
    }
    
    async fn delete(&self, id: &str) -> FhirResult<()> {
        let uuid = Uuid::parse_str(id)
            .map_err(|_| FhirError::InvalidReference(format!("Invalid UUID: {}", id)))?;
        
        let result = sqlx::query(
            r#"
            UPDATE conditions
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
                resource_type: "Condition".to_string(),
                id: id.to_string(),
            });
        }
        
        Ok(())
    }
    
    async fn search(&self, params: SearchParams) -> FhirResult<Vec<Condition>> {
        let limit = params.limit.unwrap_or(100);
        let offset = params.offset.unwrap_or(0);
        
        let rows = sqlx::query(
            r#"
            SELECT resource
            FROM conditions
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
        
        let mut conditions = Vec::new();
        for row in rows {
            let resource_json: serde_json::Value = row.try_get("resource")
                .map_err(|e| FhirError::Database(e.to_string()))?;
            let condition: Condition = serde_json::from_value(resource_json)?;
            conditions.push(condition);
        }
        
        Ok(conditions)
    }
}

struct ConditionSearchFields {
    subject_id: Option<Uuid>,
    clinical_status: Option<String>,
    verification_status: Option<String>,
    category_code: Option<String>,
    code_code: Option<String>,
    code_system: Option<String>,
    onset_datetime: Option<chrono::DateTime<Utc>>,
    recorded_date: Option<chrono::DateTime<Utc>>,
}