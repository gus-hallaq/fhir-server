// src/config.rs
// Database configuration and connection setup

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/fhir".to_string()),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            connect_timeout: Duration::from_secs(
                std::env::var("DB_CONNECT_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30)
            ),
            idle_timeout: Duration::from_secs(
                std::env::var("DB_IDLE_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(600)
            ),
        }
    }
    
    pub async fn create_pool(&self) -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(self.connect_timeout)
            .idle_timeout(Some(self.idle_timeout))
            .connect(&self.url)
            .await
    }
}

// ============================================
// .env file example
// ============================================
/*
DATABASE_URL=postgres://postgres:postgres@localhost:5432/fhir
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=2
DB_CONNECT_TIMEOUT=30
DB_IDLE_TIMEOUT=600

SERVER_HOST=0.0.0.0
SERVER_PORT=8080

RUST_LOG=info,fhir_server=debug
*/

// ============================================
// Database migration runner
// ============================================

// use sqlx::migrate::Migrator;

// pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
//     // Migrations will be embedded in the binary at compile time
//     // Place migration files in: migrations/*.sql
    
//     static MIGRATOR: Migrator = sqlx::migrate!("./migrations");
    
//     MIGRATOR.run(pool).await?;
    
//     Ok(())
// }

// ============================================
// Setup script example
// ============================================
/*
# setup_database.sh

#!/bin/bash

# Create database
createdb fhir

# Run migrations
sqlx migrate run --database-url postgres://postgres:postgres@localhost/fhir

echo "Database setup complete!"
*/