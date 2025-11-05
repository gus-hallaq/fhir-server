// src/grpc/auth.rs
// gRPC authentication middleware for extracting and validating JWT tokens from metadata

use tonic::{Request, Status};
use crate::api::auth::validate_token;
use crate::service::SecurityContext;

/// Metadata key for authorization token
const AUTHORIZATION_HEADER: &str = "authorization";
const BEARER_PREFIX: &str = "Bearer ";

/// Extract and validate authentication token from gRPC request metadata
/// Returns a SecurityContext if authentication is successful
pub fn extract_security_context<T>(request: &Request<T>) -> Result<SecurityContext, Status> {
    // Get metadata from the request
    let metadata = request.metadata();

    // Extract the authorization header
    let auth_header = metadata
        .get(AUTHORIZATION_HEADER)
        .ok_or_else(|| Status::unauthenticated("Missing authorization token"))?;

    // Convert metadata value to string
    let auth_str = auth_header
        .to_str()
        .map_err(|_| Status::unauthenticated("Invalid authorization header format"))?;

    // Check if it starts with "Bearer "
    if !auth_str.starts_with(BEARER_PREFIX) {
        return Err(Status::unauthenticated(
            "Authorization header must start with 'Bearer '",
        ));
    }

    // Extract the token (remove "Bearer " prefix)
    let token = &auth_str[BEARER_PREFIX.len()..];

    // Validate the token and get claims
    let claims = validate_token(token)
        .map_err(|e| Status::unauthenticated(format!("Invalid token: {:?}", e)))?;

    // Convert claims to SecurityContext
    Ok(claims.to_security_context())
}

/// Extract security context or return a system context if authentication fails
/// This is useful for optional authentication
pub fn extract_security_context_optional<T>(request: &Request<T>) -> SecurityContext {
    extract_security_context(request).unwrap_or_else(|_| SecurityContext::system())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::metadata::MetadataValue;
    use crate::api::auth::{generate_token, Claims};
    use crate::service::Role;

    #[test]
    fn test_extract_security_context_success() {
        // Create a valid token
        let claims = Claims::admin("test_user".to_string());
        let token = generate_token(&claims).unwrap();

        // Create a request with the token in metadata
        let mut request = Request::new(());
        request.metadata_mut().insert(
            AUTHORIZATION_HEADER,
            MetadataValue::try_from(format!("Bearer {}", token)).unwrap(),
        );

        // Extract security context
        let result = extract_security_context(&request);
        assert!(result.is_ok());

        let context = result.unwrap();
        assert_eq!(context.user_id, "test_user");
        assert!(context.has_role(&Role::Admin));
    }

    #[test]
    fn test_extract_security_context_missing_token() {
        let request: Request<()> = Request::new(());
        let result = extract_security_context(&request);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn test_extract_security_context_invalid_bearer_format() {
        let mut request = Request::new(());
        request.metadata_mut().insert(
            AUTHORIZATION_HEADER,
            MetadataValue::try_from("InvalidFormat token123").unwrap(),
        );

        let result = extract_security_context(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_security_context_optional_fallback() {
        let request: Request<()> = Request::new(());
        let context = extract_security_context_optional(&request);

        // Should fall back to system context
        assert!(context.is_system());
        assert_eq!(context.user_id, "system");
    }
}
