# gRPC Interface for FHIR Server

This FHIR server now includes a gRPC interface alongside the REST API, providing high-performance, type-safe communication for FHIR resources.

## Overview

The gRPC interface runs on **port 50051** (default) and provides services for:
- Patient resources
- Observation resources
- Condition resources
- Encounter resources

## Architecture

### Directory Structure

```
proto/
└── fhir.proto          # Protocol Buffer definitions

src/grpc/
├── mod.rs              # Module exports (includes generated proto code)
├── converters.rs       # Conversion between domain and proto models
├── services.rs         # gRPC service implementations
└── server.rs           # gRPC server setup
```

**Note:** The generated protobuf code is **not** stored in the source tree. During the build process, `build.rs` compiles `proto/fhir.proto` and places the generated Rust code in the build output directory (`OUT_DIR`). This code is then included at compile time using the `tonic::include_proto!("fhir")` macro in `src/grpc/mod.rs`.

### Protocol Buffers

Proto definitions are located in `proto/fhir.proto` and include:
- FHIR primitive types (Identifier, HumanName, CodeableConcept, etc.)
- FHIR resource types (Patient, Observation, Condition, Encounter)
- Request/Response messages for CRUD operations
- Service definitions for each resource type

## Running the Server

The gRPC server starts automatically when you run the FHIR server:

```bash
cargo run
```

You should see output indicating both servers are running:
```
📊 HTTP Server listening on http://0.0.0.0:8080
📡 gRPC Server listening on 0.0.0.0:50051
🎉 All servers running!
```

## Available Services

### PatientService

```protobuf
service PatientService {
    rpc CreatePatient(CreatePatientRequest) returns (CreatePatientResponse);
    rpc GetPatient(GetPatientRequest) returns (GetPatientResponse);
    rpc UpdatePatient(UpdatePatientRequest) returns (UpdatePatientResponse);
    rpc DeletePatient(DeletePatientRequest) returns (DeletePatientResponse);
    rpc SearchPatients(SearchPatientsRequest) returns (SearchPatientsResponse);
    rpc GetPatientHistory(GetPatientHistoryRequest) returns (GetPatientHistoryResponse);
}
```

### ObservationService

```protobuf
service ObservationService {
    rpc CreateObservation(CreateObservationRequest) returns (CreateObservationResponse);
    rpc GetObservation(GetObservationRequest) returns (GetObservationResponse);
    rpc UpdateObservation(UpdateObservationRequest) returns (UpdateObservationResponse);
    rpc DeleteObservation(DeleteObservationRequest) returns (DeleteObservationResponse);
    rpc SearchObservations(SearchObservationsRequest) returns (SearchObservationsResponse);
}
```

### ConditionService

```protobuf
service ConditionService {
    rpc CreateCondition(CreateConditionRequest) returns (CreateConditionResponse);
    rpc GetCondition(GetConditionRequest) returns (GetConditionResponse);
    rpc UpdateCondition(UpdateConditionRequest) returns (UpdateConditionResponse);
    rpc DeleteCondition(DeleteConditionRequest) returns (DeleteConditionResponse);
    rpc SearchConditions(SearchConditionsRequest) returns (SearchConditionsResponse);
}
```

### EncounterService

```protobuf
service EncounterService {
    rpc CreateEncounter(CreateEncounterRequest) returns (CreateEncounterResponse);
    rpc GetEncounter(GetEncounterRequest) returns (GetEncounterResponse);
    rpc UpdateEncounter(UpdateEncounterRequest) returns (UpdateEncounterResponse);
    rpc DeleteEncounter(DeleteEncounterRequest) returns (DeleteEncounterResponse);
    rpc SearchEncounters(SearchEncountersRequest) returns (SearchEncountersResponse);
}
```

## Client Example

### Using grpcurl

Install grpcurl:
```bash
brew install grpcurl
```

List available services:
```bash
grpcurl -plaintext localhost:50051 list
```

Create a patient:
```bash
grpcurl -plaintext -d '{
  "patient": {
    "name": [{
      "family": "Smith",
      "given": ["John"]
    }],
    "gender": "male",
    "active": true
  }
}' localhost:50051 fhir.PatientService/CreatePatient
```

Get a patient:
```bash
grpcurl -plaintext -d '{"id": "123"}' localhost:50051 fhir.PatientService/GetPatient
```

### Using Rust Client

Add to your `Cargo.toml`:
```toml
[dependencies]
tonic = "0.11"
prost = "0.12"
tokio = { version = "1.0", features = ["full"] }

[build-dependencies]
tonic-build = "0.11"
```

Example client code:
```rust
use tonic::Request;

// Include generated code
pub mod fhir {
    tonic::include_proto!("fhir");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = fhir::patient_service_client::PatientServiceClient::connect(
        "http://localhost:50051"
    ).await?;

    let request = Request::new(fhir::GetPatientRequest {
        id: "123".to_string(),
    });

    let response = client.get_patient(request).await?;
    println!("Patient: {:?}", response.into_inner());

    Ok(())
}
```

## Development

### Building

The protobuf files are automatically compiled during the build process via `build.rs`. Generated code is placed in `src/grpc/generated/`.

### Prerequisites

- Protocol Buffers compiler (protoc):
  ```bash
  # macOS
  brew install protobuf

  # Ubuntu/Debian
  apt-get install protobuf-compiler

  # Windows
  # Download from https://github.com/protocolbuffers/protobuf/releases
  ```

### Modifying Proto Definitions

1. Edit `proto/fhir.proto`
2. Run `cargo build` to regenerate code
3. Update converters in `src/grpc/converters.rs` if needed
4. Update service implementations in `src/grpc/services.rs` if needed

## Security

The current implementation uses a system-level security context for all gRPC requests. In a production environment, you should:

1. **Add authentication**: Extract JWT tokens or API keys from request metadata
2. **Implement authorization**: Use the extracted credentials to create proper `SecurityContext`
3. **Enable TLS**: Configure the server with TLS certificates
4. **Add rate limiting**: Protect against abuse

Example with authentication:
```rust
#[tonic::async_trait]
impl proto::patient_service_server::PatientService for GrpcPatientService {
    async fn get_patient(
        &self,
        request: Request<proto::GetPatientRequest>,
    ) -> Result<Response<proto::GetPatientResponse>, Status> {
        // Extract auth token from metadata
        let token = request.metadata()
            .get("authorization")
            .and_then(|t| t.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing authorization token"))?;

        // Create security context from token
        let security_context = SecurityContext::from_token(token)
            .map_err(|_| Status::unauthenticated("Invalid token"))?;

        // Use the security context...
    }
}
```

## Performance Considerations

gRPC provides several advantages over REST:

1. **Binary Protocol**: More efficient serialization with Protocol Buffers
2. **HTTP/2**: Multiplexing, server push, and header compression
3. **Type Safety**: Strong typing enforced by proto definitions
4. **Bidirectional Streaming**: Support for real-time data flows (not yet implemented)

## Future Enhancements

- [ ] Add gRPC reflection for better tooling support
- [ ] Implement streaming operations for large datasets
- [ ] Add comprehensive error handling with custom status codes
- [ ] Implement interceptors for logging and metrics
- [ ] Add gRPC-Web support for browser clients
- [ ] Implement bidirectional streaming for real-time updates
- [ ] Add batch operations for bulk resource management

## Troubleshooting

### Build Errors

If you encounter protobuf compilation errors:
```bash
# Verify protoc is installed
protoc --version

# Clean and rebuild
cargo clean
cargo build
```

### Connection Issues

If clients can't connect:
```bash
# Check if the server is listening
lsof -i :50051

# Test with grpcurl
grpcurl -plaintext localhost:50051 list
```

## References

- [tonic Documentation](https://docs.rs/tonic/)
- [Protocol Buffers](https://protobuf.dev/)
- [gRPC](https://grpc.io/)
- [FHIR Specification](https://www.hl7.org/fhir/)
