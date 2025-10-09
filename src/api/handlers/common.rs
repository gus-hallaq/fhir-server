// src/api/handlers/common.rs

use serde::Deserialize;
use crate::service::{SearchParameters, SecurityContext};
use crate::api::{OptionalAuthUser, AuthUser};

/// Common query parameters for search endpoints
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    #[serde(rename = "_count")]
    pub count: Option<u32>,
    #[serde(rename = "_offset")]
    pub offset: Option<u32>,
    #[serde(rename = "_sort")]
    pub sort: Option<String>,
}

impl SearchQuery {
    pub fn into_search_params(self) -> SearchParameters {
        SearchParameters {
            count: self.count,
            offset: self.offset,
            sort: self.sort,
            filters: Vec::new(),
        }
    }
}

/// Extract security context from authenticated user
pub fn extract_security_context(auth_user: &AuthUser) -> SecurityContext {
    auth_user.0.to_security_context()
}

/// Extract optional security context (returns system context if not authenticated)
pub fn extract_optional_security_context(optional_auth: &OptionalAuthUser) -> SecurityContext {
    match &optional_auth.0 {
        Some(claims) => claims.to_security_context(),
        None => SecurityContext::system(),
    }
}
