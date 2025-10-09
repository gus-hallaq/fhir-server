// src/api/handlers/auth_handlers.rs

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::{
    api::auth::{generate_token, hash_password, verify_password, AuthError, Claims},
    service::Role,
    AppState,
};

/// Login request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: String,
    pub roles: Vec<String>,
}

/// Register request
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub role: String,
    pub patient_id: Option<String>,
    pub organization_id: Option<String>,
}

/// Register response
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub username: String,
    pub message: String,
}

/// Mock user database (in production, this would be a real database)
/// For demonstration purposes only
#[derive(Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub roles: Vec<Role>,
    pub patient_id: Option<String>,
    pub organization_id: Option<String>,
}

/// Login endpoint
///
/// For demonstration, this uses hardcoded credentials:
/// - admin/admin123 (Admin role)
/// - doctor/doctor123 (Clinician role)
/// - patient/patient123 (Patient role, patient_id: "patient-001")
pub async fn login(
    State(_state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    // Mock users - in production, fetch from database
    let users = get_mock_users();

    // Find user by username
    let user = users
        .iter()
        .find(|u| u.username == req.username)
        .ok_or(AuthError::InvalidCredentials)?;

    // Verify password
    if !verify_password(&req.password, &user.password_hash)? {
        return Err(AuthError::InvalidCredentials);
    }

    // Create claims
    let claims = Claims::new(
        user.id.clone(),
        user.roles.clone(),
        user.patient_id.clone(),
        user.organization_id.clone(),
    );

    // Generate token
    let token = generate_token(&claims)?;

    Ok(Json(LoginResponse {
        token,
        user_id: user.id.clone(),
        roles: claims.roles,
    }))
}

/// Register endpoint
///
/// Creates a new user account. In production, this would persist to a database.
pub async fn register(
    State(_state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), AuthError> {
    // Validate role
    let role = match req.role.as_str() {
        "Admin" => Role::Admin,
        "Clinician" => Role::Clinician,
        "Patient" => Role::Patient,
        _ => return Err(AuthError::InvalidCredentials),
    };

    // Hash password
    let password_hash = hash_password(&req.password)?;

    // Generate user ID (in production, this would be from database)
    let user_id = uuid::Uuid::new_v4().to_string();

    // In production, save to database:
    // - user_id
    // - username
    // - password_hash
    // - roles
    // - patient_id
    // - organization_id

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id,
            username: req.username,
            message: "User registered successfully. In production, this would be saved to database.".to_string(),
        }),
    ))
}

/// Get current user info
pub async fn me(
    user: crate::api::AuthUser,
) -> Result<Json<UserInfo>, AuthError> {
    let claims = user.0;

    Ok(Json(UserInfo {
        user_id: claims.sub,
        roles: claims.roles,
        patient_id: claims.patient_id,
        organization_id: claims.organization_id,
    }))
}

/// User info response
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub user_id: String,
    pub roles: Vec<String>,
    pub patient_id: Option<String>,
    pub organization_id: Option<String>,
}

/// Mock users for demonstration
fn get_mock_users() -> Vec<User> {
    vec![
        User {
            id: "user-admin-001".to_string(),
            username: "admin".to_string(),
            // Hash for "admin123"
            password_hash: hash_password("admin123").unwrap(),
            roles: vec![Role::Admin],
            patient_id: None,
            organization_id: None,
        },
        User {
            id: "user-doctor-001".to_string(),
            username: "doctor".to_string(),
            // Hash for "doctor123"
            password_hash: hash_password("doctor123").unwrap(),
            roles: vec![Role::Clinician],
            patient_id: None,
            organization_id: Some("org-001".to_string()),
        },
        User {
            id: "user-patient-001".to_string(),
            username: "patient".to_string(),
            // Hash for "patient123"
            password_hash: hash_password("patient123").unwrap(),
            roles: vec![Role::Patient],
            patient_id: Some("patient-001".to_string()),
            organization_id: None,
        },
    ]
}
