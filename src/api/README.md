# FHIR Server REST API

This directory contains the REST API implementation for the FHIR server using Axum with JWT-based authentication.

## Authentication

The API uses JWT (JSON Web Tokens) for authentication. See [AUTHENTICATION.md](../../AUTHENTICATION.md) for detailed documentation.

**Quick Start:**
```bash
# Login to get a token
TOKEN=$(curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}' \
  | jq -r '.token')

# Use the token in requests
curl -X GET http://localhost:8080/fhir/Patient/123 \
  -H "Authorization: Bearer $TOKEN"
```

## Demo Users

- **admin/admin123** - Admin role (full access)
- **doctor/doctor123** - Clinician role (read/write)
- **patient/patient123** - Patient role (read own data only)

## Structure

```
src/api/
├── mod.rs              # Module exports
├── auth.rs             # JWT authentication & extractors
├── router.rs           # API router configuration
├── responses.rs        # Response types and error handling
├── README.md           # This file
└── handlers/           # Request handlers
    ├── mod.rs
    ├── common.rs       # Shared utilities
    ├── auth_handlers.rs # Authentication endpoints
    ├── patient.rs      # Patient resource endpoints
    ├── observation.rs  # Observation resource endpoints
    ├── condition.rs    # Condition resource endpoints
    └── encounter.rs    # Encounter resource endpoints
```

## Endpoints

All FHIR endpoints are prefixed with `/fhir/`.

### Health Check
- `GET /health` - Health check endpoint

### Authentication
- `POST /auth/login` - User login (returns JWT token)
- `POST /auth/register` - User registration (demo only)
- `GET /auth/me` - Get current user info (requires authentication)

### Patient Resource

- `POST /fhir/Patient` - Create a new patient
- `GET /fhir/Patient` - Search patients
  - Query params: `family`, `given`, `identifier`, `_count`, `_offset`, `_sort`
- `GET /fhir/Patient/:id` - Get patient by ID
- `PUT /fhir/Patient/:id` - Update a patient
- `DELETE /fhir/Patient/:id` - Delete a patient
- `GET /fhir/Patient/:id/_history` - Get patient history

### Observation Resource

- `POST /fhir/Observation` - Create a new observation
- `GET /fhir/Observation` - Search observations
  - Query params: `patient`, `code`, `category`, `_count`, `_offset`, `_sort`
- `GET /fhir/Observation/:id` - Get observation by ID
- `PUT /fhir/Observation/:id` - Update an observation
- `DELETE /fhir/Observation/:id` - Delete an observation
- `GET /fhir/Observation/:id/_history` - Get observation history

### Condition Resource

- `POST /fhir/Condition` - Create a new condition
- `GET /fhir/Condition` - Search conditions
  - Query params: `patient`, `code`, `clinical-status`, `_count`, `_offset`, `_sort`
- `GET /fhir/Condition/:id` - Get condition by ID
- `PUT /fhir/Condition/:id` - Update a condition
- `DELETE /fhir/Condition/:id` - Delete a condition
- `GET /fhir/Condition/:id/_history` - Get condition history

### Encounter Resource

- `POST /fhir/Encounter` - Create a new encounter
- `GET /fhir/Encounter` - Search encounters
  - Query params: `patient`, `status`, `class`, `_count`, `_offset`, `_sort`
- `GET /fhir/Encounter/:id` - Get encounter by ID
- `PUT /fhir/Encounter/:id` - Update an encounter
- `DELETE /fhir/Encounter/:id` - Delete an encounter
- `GET /fhir/Encounter/:id/_history` - Get encounter history

## Response Formats

### Success Response

```json
{
  "data": {
    // Resource data
  }
}
```

### Paginated Response

```json
{
  "data": [
    // Array of resources
  ],
  "total": 100,
  "offset": 0,
  "count": 20
}
```

### Error Response

```json
{
  "error": "ERROR_TYPE",
  "message": "Human-readable error message",
  "details": "Optional additional details"
}
```

## Error Types

- `NOT_FOUND` (404) - Resource not found
- `VALIDATION_ERROR` (400) - Invalid request data
- `FORBIDDEN` (403) - Authorization failed
- `DATABASE_ERROR` (500) - Database operation failed
- `SERIALIZATION_ERROR` (500) - JSON serialization failed
- `INVALID_RESOURCE_TYPE` (400) - Invalid resource type
- `MISSING_REQUIRED_FIELD` (400) - Required field missing
- `INVALID_REFERENCE` (400) - Invalid resource reference
- `CONFLICT` (409) - Resource conflict
- `PRECONDITION_FAILED` (412) - Precondition failed
- `UNPROCESSABLE_ENTITY` (422) - Unprocessable entity

## Example Usage

### Create a Patient

```bash
curl -X POST http://localhost:8080/fhir/Patient \
  -H "Content-Type: application/json" \
  -d '{
    "name": [{
      "use": "official",
      "family": "Doe",
      "given": ["John"]
    }],
    "gender": "male",
    "active": true
  }'
```

### Search Patients by Family Name

```bash
curl -X GET "http://localhost:8080/fhir/Patient?family=Doe"
```

### Get a Patient

```bash
curl -X GET http://localhost:8080/fhir/Patient/123
```

### Update a Patient

```bash
curl -X PUT http://localhost:8080/fhir/Patient/123 \
  -H "Content-Type: application/json" \
  -d '{
    "id": "123",
    "name": [{
      "use": "official",
      "family": "Doe",
      "given": ["John", "David"]
    }],
    "gender": "male",
    "active": false
  }'
```

### Delete a Patient

```bash
curl -X DELETE http://localhost:8080/fhir/Patient/123
```

## Features

- **CORS Support** - Configured to allow all origins (configurable in production)
- **Request Tracing** - HTTP request/response logging via `tower-http`
- **Error Handling** - Automatic conversion of FHIR errors to HTTP responses
- **Authorization** - Security context support (currently using system context)
- **Validation** - Input validation for all resources

## TODO

- [ ] Implement JWT authentication and extract security context from headers
- [ ] Implement resource history tracking
- [ ] Add support for additional search parameters
- [ ] Implement FHIR Bundle resources for batch operations
- [ ] Add rate limiting
- [ ] Add API versioning
- [ ] Implement conditional read/update/delete operations
- [ ] Add support for FHIR `_include` and `_revinclude` search parameters
