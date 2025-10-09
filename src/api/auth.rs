// src/api/auth.rs

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::service::{SecurityContext, Role};
use std::collections::HashSet;

/// JWT secret key - should be loaded from environment variable in production
/// TODO: Load from environment variable
pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-this-in-production".to_string())
}

/// Token expiration time in hours
const TOKEN_EXPIRATION_HOURS: i64 = 24;

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// User roles (serialized as strings)
    pub roles: Vec<String>,
    /// Expiration time (as UTC timestamp)
    pub exp: i64,
    /// Issued at (as UTC timestamp)
    pub iat: i64,
    /// Optional patient ID (for patient users)
    pub patient_id: Option<String>,
    /// Optional organization ID
    pub organization_id: Option<String>,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(
        user_id: String,
        roles: Vec<Role>,
        patient_id: Option<String>,
        organization_id: Option<String>,
    ) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(TOKEN_EXPIRATION_HOURS);

        let role_strings: Vec<String> = roles.iter().map(|r| format!("{:?}", r)).collect();

        Self {
            sub: user_id,
            roles: role_strings,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            patient_id,
            organization_id,
        }
    }

    /// Create admin claims
    pub fn admin(user_id: String) -> Self {
        Self::new(user_id, vec![Role::Admin], None, None)
    }

    /// Create clinician claims
    pub fn clinician(user_id: String, organization_id: Option<String>) -> Self {
        Self::new(user_id, vec![Role::Clinician], None, organization_id)
    }

    /// Create patient claims
    pub fn patient(user_id: String, patient_id: String) -> Self {
        Self::new(user_id, vec![Role::Patient], Some(patient_id), None)
    }

    /// Create system claims
    pub fn system() -> Self {
        Self::new("system".to_string(), vec![Role::System], None, None)
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }

    /// Parse roles from strings
    fn parse_roles(&self) -> HashSet<Role> {
        self.roles
            .iter()
            .filter_map(|r| match r.as_str() {
                "Admin" => Some(Role::Admin),
                "Clinician" => Some(Role::Clinician),
                "Patient" => Some(Role::Patient),
                "System" => Some(Role::System),
                _ => None,
            })
            .collect()
    }

    /// Convert claims to SecurityContext
    pub fn to_security_context(&self) -> SecurityContext {
        let roles = self.parse_roles();
        let mut context = SecurityContext {
            user_id: self.sub.clone(),
            roles,
            patient_id: self.patient_id.clone(),
            organization_id: self.organization_id.clone(),
            claims: std::collections::HashMap::new(),
        };

        // Add additional claims
        context.claims.insert("iat".to_string(), self.iat.to_string());
        context.claims.insert("exp".to_string(), self.exp.to_string());

        context
    }
}

/// Generate a JWT token from claims
pub fn generate_token(claims: &Claims) -> Result<String, AuthError> {
    let secret = get_jwt_secret();
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AuthError::TokenCreation(e.to_string()))
}

/// Validate and decode a JWT token
pub fn validate_token(token: &str) -> Result<Claims, AuthError> {
    let secret = get_jwt_secret();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AuthError::InvalidToken(e.to_string()))?;

    if token_data.claims.is_expired() {
        return Err(AuthError::TokenExpired);
    }

    Ok(token_data.claims)
}

/// Hash a password using bcrypt
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AuthError::PasswordHashError(e.to_string()))
}

/// Verify a password against a hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    bcrypt::verify(password, hash)
        .map_err(|e| AuthError::PasswordVerificationError(e.to_string()))
}

/// Authentication errors
#[derive(Debug)]
pub enum AuthError {
    InvalidToken(String),
    TokenExpired,
    TokenCreation(String),
    MissingToken,
    PasswordHashError(String),
    PasswordVerificationError(String),
    InvalidCredentials,
    Unauthorized,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::InvalidToken(msg) => (StatusCode::UNAUTHORIZED, format!("Invalid token: {}", msg)),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired".to_string()),
            AuthError::TokenCreation(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Token creation failed: {}", msg)),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing authorization token".to_string()),
            AuthError::PasswordHashError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Password hash error: {}", msg)),
            AuthError::PasswordVerificationError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Password verification error: {}", msg)),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
            AuthError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
        };

        let body = serde_json::json!({
            "error": "AUTHENTICATION_ERROR",
            "message": message,
        });

        (status, Json(body)).into_response()
    }
}

/// Extractor for authenticated user from JWT token
pub struct AuthUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the Authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::MissingToken)?;

        // Validate and decode the token
        let claims = validate_token(bearer.token())?;

        Ok(AuthUser(claims))
    }
}

/// Optional extractor for authenticated user (doesn't fail if no token)
pub struct OptionalAuthUser(pub Option<Claims>);

#[async_trait]
impl<S> FromRequestParts<S> for OptionalAuthUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await;

        let claims = match auth_header {
            Ok(TypedHeader(Authorization(bearer))) => {
                validate_token(bearer.token()).ok()
            }
            Err(_) => None,
        };

        Ok(OptionalAuthUser(claims))
    }
}
