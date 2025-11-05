# gRPC Quick Reference

Quick command reference for testing the FHIR Server gRPC API.

## Prerequisites

```bash
# Install grpcurl
brew install grpcurl  # macOS
# or
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
```

## Common Commands

### List All Services

```bash
grpcurl -plaintext localhost:50051 list
```

### Describe a Service

```bash
# Patient Service
grpcurl -plaintext localhost:50051 describe fhir.PatientService

# Observation Service
grpcurl -plaintext localhost:50051 describe fhir.ObservationService

# Condition Service
grpcurl -plaintext localhost:50051 describe fhir.ConditionService

# Encounter Service
grpcurl -plaintext localhost:50051 describe fhir.EncounterService
```

### Describe Message Types

```bash
grpcurl -plaintext localhost:50051 describe fhir.Patient
grpcurl -plaintext localhost:50051 describe fhir.Observation
grpcurl -plaintext localhost:50051 describe fhir.Condition
grpcurl -plaintext localhost:50051 describe fhir.Encounter
```

## Authentication

```bash
# Get a token via REST API first
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password"}' \
  | jq -r '.token')

# Use token in gRPC calls
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"id": "123"}' \
  localhost:50051 \
  fhir.PatientService/GetPatient
```

## Patient Service Examples

### Create Patient

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{
    "patient": {
      "name": [{
        "family": "Doe",
        "given": ["John"],
        "text": "John Doe"
      }],
      "gender": "male",
      "active": true
    }
  }' \
  localhost:50051 \
  fhir.PatientService/CreatePatient
```

### Get Patient

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"id": "patient-id"}' \
  localhost:50051 \
  fhir.PatientService/GetPatient
```

### Search Patients

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"family": "Doe"}' \
  localhost:50051 \
  fhir.PatientService/SearchPatients
```

### Update Patient

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{
    "id": "patient-id",
    "patient": {
      "name": [{
        "family": "Doe",
        "given": ["Jane"],
        "text": "Jane Doe"
      }],
      "gender": "female",
      "active": true
    }
  }' \
  localhost:50051 \
  fhir.PatientService/UpdatePatient
```

### Delete Patient

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"id": "patient-id"}' \
  localhost:50051 \
  fhir.PatientService/DeletePatient
```

### Get Patient History

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"id": "patient-id"}' \
  localhost:50051 \
  fhir.PatientService/GetPatientHistory
```

## Observation Service Examples

### Create Observation

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{
    "observation": {
      "status": "final",
      "code": {
        "coding": [{
          "system": "http://loinc.org",
          "code": "8867-4",
          "display": "Heart rate"
        }],
        "text": "Heart rate"
      },
      "subject": {
        "reference": "Patient/patient-id"
      }
    }
  }' \
  localhost:50051 \
  fhir.ObservationService/CreateObservation
```

### Search Observations by Patient

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"patient": "patient-id"}' \
  localhost:50051 \
  fhir.ObservationService/SearchObservations
```

## Condition Service Examples

### Create Condition

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{
    "condition": {
      "subject": {
        "reference": "Patient/patient-id"
      },
      "code": {
        "coding": [{
          "system": "http://snomed.info/sct",
          "code": "38341003",
          "display": "Hypertension"
        }],
        "text": "Hypertension"
      }
    }
  }' \
  localhost:50051 \
  fhir.ConditionService/CreateCondition
```

### Search Conditions by Patient

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"patient": "patient-id"}' \
  localhost:50051 \
  fhir.ConditionService/SearchConditions
```

## Encounter Service Examples

### Create Encounter

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{
    "encounter": {
      "status": "in-progress",
      "class": {
        "system": "http://terminology.hl7.org/CodeSystem/v3-ActCode",
        "code": "AMB",
        "display": "ambulatory"
      },
      "subject": {
        "reference": "Patient/patient-id"
      }
    }
  }' \
  localhost:50051 \
  fhir.EncounterService/CreateEncounter
```

### Search Encounters by Patient

```bash
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"patient": "patient-id"}' \
  localhost:50051 \
  fhir.EncounterService/SearchEncounters
```

## TLS Commands

When TLS is enabled, replace `-plaintext` with `-insecure` (for self-signed certs):

```bash
# With self-signed certificate
grpcurl -insecure localhost:50051 list

# With CA certificate
grpcurl -cacert certs/server.crt localhost:50051 list
```

## Interactive UI

Launch grpcui for a web-based interface:

```bash
# Without TLS
grpcui -plaintext localhost:50051

# With TLS
grpcui -insecure localhost:50051
```

## Tips

1. **Pretty Print JSON**: grpcurl automatically formats JSON output
2. **Save Requests**: Save common requests to files and use `-d @file.json`
3. **Verbose Mode**: Add `-v` or `-vv` for detailed request/response info
4. **Import Path**: Use `-import-path` if you need to reference local proto files
5. **Metadata**: Add multiple headers with multiple `-H` flags

## Environment Variables

Set these for easier testing:

```bash
# Export token
export GRPC_TOKEN="your-jwt-token-here"

# Use in commands
grpcurl -plaintext \
  -H "authorization: Bearer $GRPC_TOKEN" \
  -d '{"id": "123"}' \
  localhost:50051 \
  fhir.PatientService/GetPatient

# Set default endpoint
export GRPC_ENDPOINT="localhost:50051"

# Shorter commands
grpcurl -plaintext $GRPC_ENDPOINT list
```

## Troubleshooting

```bash
# Check server is running
grpcurl -plaintext localhost:50051 list

# Test without auth (will fail but confirms server is up)
grpcurl -plaintext -d '{"id": "test"}' \
  localhost:50051 \
  fhir.PatientService/GetPatient

# Verbose output for debugging
grpcurl -plaintext -vv localhost:50051 list
```
