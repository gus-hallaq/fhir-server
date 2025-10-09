# Authentication & Authorization

This document describes the JWT-based authentication and role-based authorization system implemented in the FHIR server.

## Overview

The FHIR server uses JWT (JSON Web Tokens) for authentication and a role-based access control (RBAC) system for authorization.

## Authentication Flow

### 1. User Login

Users authenticate by sending credentials to the `/auth/login` endpoint:

```bash
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }'
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": "user-admin-001",
  "roles": ["Admin"]
}
```

### 2. Using the Token

Include the JWT token in the `Authorization` header for subsequent requests:

```bash
curl -X GET http://localhost:8080/fhir/Patient/123 \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### 3. Token Expiration

Tokens expire after 24 hours. After expiration, users must log in again to obtain a new token.

## User Roles

The system supports four roles with different permission levels:

### System
- Full access to all resources
- Used for internal operations
- Cannot be assigned to regular users

### Admin
- Full access to all FHIR resources
- Can create, read, update, and delete any resource
- Access to all patient data

### Clinician
- Read and write access to patient data
- Can create, read, update, and search resources
- Cannot delete resources
- Optionally scoped to an organization

### Patient
- Read-only access to their own data
- Can only access resources in their patient compartment
- Cannot create, update, or delete resources

## Permission Matrix

| Role       | Create | Read | Update | Delete | Search | History |
|------------|--------|------|--------|--------|--------|---------|
| System     | ✅     | ✅   | ✅     | ✅     | ✅     | ✅      |
| Admin      | ✅     | ✅   | ✅     | ✅     | ✅     | ✅      |
| Clinician  | ✅     | ✅   | ✅     | ❌     | ✅     | ✅      |
| Patient    | ❌     | ✅*  | ❌     | ❌     | ✅*    | ✅*     |

\* Patients can only access their own data

## Demo Users

For testing purposes, the system includes three demo users:

### Admin User
```json
{
  "username": "admin",
  "password": "admin123",
  "role": "Admin"
}
```

### Clinician User
```json
{
  "username": "doctor",
  "password": "doctor123",
  "role": "Clinician",
  "organization_id": "org-001"
}
```

### Patient User
```json
{
  "username": "patient",
  "password": "patient123",
  "role": "Patient",
  "patient_id": "patient-001"
}
```

## API Endpoints

### Authentication Endpoints

#### Login
```
POST /auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "admin123"
}
```

#### Register (Demo - not persisted)
```
POST /auth/register
Content-Type: application/json

{
  "username": "newuser",
  "password": "password123",
  "role": "Clinician",
  "organization_id": "org-001"
}
```

#### Get Current User Info
```
GET /auth/me
Authorization: Bearer <token>
```

**Response:**
```json
{
  "user_id": "user-admin-001",
  "roles": ["Admin"],
  "patient_id": null,
  "organization_id": null
}
```

## Authorization Rules

### Patient Compartment

Patients can only access resources that reference them:
- **Patient**: Only their own patient resource
- **Observation**: Only observations with `subject` referencing their patient ID
- **Condition**: Only conditions with `subject` referencing their patient ID
- **Encounter**: Only encounters with `subject` referencing their patient ID

### Example Authorization Checks

**Patient accessing their own data:**
```bash
# Patient user (patient-001) can access
curl -X GET http://localhost:8080/fhir/Patient/patient-001 \
  -H "Authorization: Bearer <patient_token>"
# ✅ Allowed

# Patient user (patient-001) cannot access another patient
curl -X GET http://localhost:8080/fhir/Patient/patient-002 \
  -H "Authorization: Bearer <patient_token>"
# ❌ 403 Forbidden
```

**Clinician accessing any patient:**
```bash
curl -X GET http://localhost:8080/fhir/Patient/patient-001 \
  -H "Authorization: Bearer <clinician_token>"
# ✅ Allowed
```

**Admin full access:**
```bash
curl -X DELETE http://localhost:8080/fhir/Patient/patient-001 \
  -H "Authorization: Bearer <admin_token>"
# ✅ Allowed
```

## JWT Claims Structure

The JWT token contains the following claims:

```json
{
  "sub": "user-admin-001",           // User ID
  "roles": ["Admin"],                // User roles
  "patient_id": null,                // Patient ID (for patient users)
  "organization_id": null,           // Organization ID (for clinicians)
  "iat": 1234567890,                 // Issued at
  "exp": 1234654290                  // Expiration time
}
```

## Security Configuration

### JWT Secret

The JWT secret is loaded from the `JWT_SECRET` environment variable. If not set, a default value is used (not suitable for production).

**Set the secret:**
```bash
export JWT_SECRET="your-super-secret-key-change-this"
```

### Password Hashing

Passwords are hashed using bcrypt with a cost factor of 12 (default).

## Optional vs Required Authentication

### Optional Authentication (OptionalAuthUser)

Most endpoints use `OptionalAuthUser`, which means:
- If a valid JWT token is provided, the user context is extracted
- If no token is provided, a system context is used
- This allows both authenticated and unauthenticated access

### Required Authentication (AuthUser)

The `/auth/me` endpoint uses `AuthUser`, which requires a valid JWT token.
- Returns 401 Unauthorized if no token is provided
- Returns 401 Unauthorized if the token is invalid or expired

## Error Responses

### 401 Unauthorized
Returned when authentication fails:
```json
{
  "error": "AUTHENTICATION_ERROR",
  "message": "Invalid token: ..."
}
```

Common causes:
- Missing Authorization header
- Invalid token format
- Expired token
- Invalid signature

### 403 Forbidden
Returned when authorization fails:
```json
{
  "error": "FORBIDDEN",
  "message": "User xyz does not have permission ... for resource type Patient"
}
```

Common causes:
- User lacks required role
- Patient trying to access another patient's data
- Clinician trying to delete resources

## Example Workflows

### 1. Admin Creating a Patient

```bash
# Login as admin
TOKEN=$(curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' \
  | jq -r '.token')

# Create patient
curl -X POST http://localhost:8080/fhir/Patient \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": [{
      "use": "official",
      "family": "Smith",
      "given": ["Jane"]
    }],
    "gender": "female",
    "active": true
  }'
```

### 2. Clinician Viewing Patient Data

```bash
# Login as doctor
TOKEN=$(curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"doctor","password":"doctor123"}' \
  | jq -r '.token')

# Search patients
curl -X GET "http://localhost:8080/fhir/Patient?family=Smith" \
  -H "Authorization: Bearer $TOKEN"

# Get specific patient
curl -X GET http://localhost:8080/fhir/Patient/patient-001 \
  -H "Authorization: Bearer $TOKEN"
```

### 3. Patient Viewing Their Own Data

```bash
# Login as patient
TOKEN=$(curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"patient","password":"patient123"}' \
  | jq -r '.token')

# View own patient resource
curl -X GET http://localhost:8080/fhir/Patient/patient-001 \
  -H "Authorization: Bearer $TOKEN"

# View own observations
curl -X GET "http://localhost:8080/fhir/Observation?patient=patient-001" \
  -H "Authorization: Bearer $TOKEN"
```

## Production Considerations

### 1. JWT Secret Management

**Never use the default secret in production!**

Set a strong secret:
```bash
export JWT_SECRET=$(openssl rand -base64 32)
```

### 2. Database Integration

The current implementation uses mock users. In production:
- Store user credentials in a database
- Hash passwords before storage
- Implement user management endpoints
- Add user registration workflow
- Implement password reset functionality

### 3. Token Refresh

Consider implementing refresh tokens for better UX:
- Short-lived access tokens (15 minutes)
- Long-lived refresh tokens (7 days)
- Refresh endpoint to get new access tokens

### 4. Rate Limiting

Implement rate limiting on authentication endpoints:
- Prevent brute force attacks
- Limit failed login attempts
- Implement account lockout after multiple failures

### 5. HTTPS

Always use HTTPS in production to protect tokens in transit.

### 6. Token Revocation

Implement token blacklisting for:
- User logout
- Password changes
- Admin-initiated token revocation

## Code Structure

### Authentication Module (`src/api/auth.rs`)

- `Claims` - JWT claims structure
- `generate_token()` - Create JWT tokens
- `validate_token()` - Validate and decode tokens
- `hash_password()` - Hash passwords with bcrypt
- `verify_password()` - Verify password hashes
- `AuthUser` - Required authentication extractor
- `OptionalAuthUser` - Optional authentication extractor

### Authorization Module (`src/service/authorization.rs`)

- `SecurityContext` - User context with roles and permissions
- `Role` - User role enum
- `Permission` - Permission types
- `Authorizer` - Authorization trait
- `DefaultAuthorizer` - Default authorization implementation

### Auth Handlers (`src/api/handlers/auth_handlers.rs`)

- `login()` - User login endpoint
- `register()` - User registration endpoint (demo)
- `me()` - Get current user info

## Testing Authentication

### Unit Tests

Run the authorization tests:
```bash
cargo test authorization
```

### Integration Tests

Use the demo users to test the authentication flow:

```bash
# Test admin access
./test_admin.sh

# Test clinician access
./test_clinician.sh

# Test patient access (restricted)
./test_patient.sh
```

## Future Enhancements

- [ ] Implement refresh tokens
- [ ] Add OAuth2/OIDC support
- [ ] Implement MFA (Multi-Factor Authentication)
- [ ] Add audit logging for authentication events
- [ ] Implement session management
- [ ] Add API key authentication for system-to-system calls
- [ ] Implement SMART on FHIR authorization
- [ ] Add support for external identity providers
