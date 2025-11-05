# gRPC Reflection Guide

This guide explains how to use gRPC reflection with the FHIR Server and demonstrates common tools and workflows.

## What is gRPC Reflection?

gRPC reflection is a protocol that allows clients to discover services, methods, and message types at runtime without needing access to `.proto` files. This is extremely useful for:

- **Development and debugging** with tools like `grpcurl` and `grpcui`
- **Service discovery** in dynamic environments
- **API exploration** without documentation
- **Integration testing** without proto dependencies

## Reflection in FHIR Server

The FHIR Server automatically enables gRPC reflection for all services. This includes:

- `fhir.PatientService`
- `fhir.ObservationService`
- `fhir.ConditionService`
- `fhir.EncounterService`
- Standard reflection service (`grpc.reflection.v1alpha.ServerReflection`)

## Using grpcurl

[grpcurl](https://github.com/fullstorydev/grpcurl) is a command-line tool for interacting with gRPC services (like curl for HTTP).

### Installation

```bash
# macOS
brew install grpcurl

# Linux
wget https://github.com/fullstorydev/grpcurl/releases/download/v1.8.9/grpcurl_1.8.9_linux_x86_64.tar.gz
tar -xvf grpcurl_1.8.9_linux_x86_64.tar.gz
sudo mv grpcurl /usr/local/bin/

# Or using Go
go install github.com/fullstorydev/grpcurl/cmd/grpcurl@latest
```

### List All Services

```bash
# Without TLS
grpcurl -plaintext localhost:50051 list

# With TLS (self-signed cert - skip verification)
grpcurl -insecure localhost:50051 list
```

Output:
```
fhir.ConditionService
fhir.EncounterService
fhir.ObservationService
fhir.PatientService
grpc.reflection.v1alpha.ServerReflection
```

### Describe a Service

```bash
# Get details about PatientService
grpcurl -plaintext localhost:50051 describe fhir.PatientService
```

Output shows all methods:
```
fhir.PatientService is a service:
service PatientService {
  rpc CreatePatient ( .fhir.CreatePatientRequest ) returns ( .fhir.CreatePatientResponse );
  rpc DeletePatient ( .fhir.DeletePatientRequest ) returns ( .fhir.DeletePatientResponse );
  rpc GetPatient ( .fhir.GetPatientRequest ) returns ( .fhir.GetPatientResponse );
  rpc GetPatientHistory ( .fhir.GetPatientHistoryRequest ) returns ( .fhir.GetPatientHistoryResponse );
  rpc SearchPatients ( .fhir.SearchPatientsRequest ) returns ( .fhir.SearchPatientsResponse );
  rpc UpdatePatient ( .fhir.UpdatePatientRequest ) returns ( .fhir.UpdatePatientResponse );
}
```

### Describe a Message Type

```bash
# Get the structure of a message
grpcurl -plaintext localhost:50051 describe fhir.Patient
```

### Call a Method

```bash
# Search for patients (without authentication)
grpcurl -plaintext \
  -d '{"family": "Doe"}' \
  localhost:50051 \
  fhir.PatientService/SearchPatients

# With authentication
grpcurl -plaintext \
  -H "authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"family": "Doe"}' \
  localhost:50051 \
  fhir.PatientService/SearchPatients

# Get a specific patient
grpcurl -plaintext \
  -H "authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"id": "patient-123"}' \
  localhost:50051 \
  fhir.PatientService/GetPatient
```

### Create a Patient

```bash
grpcurl -plaintext \
  -H "authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{
    "patient": {
      "name": [{
        "family": "Smith",
        "given": ["John"],
        "text": "John Smith"
      }],
      "gender": "male",
      "active": true
    }
  }' \
  localhost:50051 \
  fhir.PatientService/CreatePatient
```

## Using grpcui

[grpcui](https://github.com/fullstorydev/grpcui) provides a web-based GUI for interacting with gRPC services.

### Installation

```bash
# macOS
brew install grpcui

# Using Go
go install github.com/fullstorydev/grpcui/cmd/grpcui@latest
```

### Launch the UI

```bash
# Without TLS
grpcui -plaintext localhost:50051

# With TLS
grpcui -insecure localhost:50051
```

This opens a browser with an interactive interface where you can:
- Browse all services and methods
- Build and send requests with form inputs
- View formatted responses
- Explore message schemas

## Using Postman

Postman supports gRPC with reflection:

1. Create a new gRPC request
2. Enter server URL: `localhost:50051`
3. Enable "Use server reflection"
4. Select service and method from dropdown
5. Fill in the request message
6. Add metadata for authentication:
   - Key: `authorization`
   - Value: `Bearer YOUR_JWT_TOKEN`

## Using BloomRPC

[BloomRPC](https://github.com/bloomrpc/bloomrpc) is a GUI client for gRPC:

1. Download and install BloomRPC
2. Click on the server reflection icon
3. Enter: `localhost:50051`
4. Click "Connect"
5. Services will appear in the sidebar

## Programmatic Access

### Python Client with Reflection

```python
import grpc
from grpc_reflection.v1alpha import reflection_pb2
from grpc_reflection.v1alpha import reflection_pb2_grpc

# Connect to server
channel = grpc.insecure_channel('localhost:50051')

# Use reflection to list services
stub = reflection_pb2_grpc.ServerReflectionStub(channel)
request = reflection_pb2.ServerReflectionRequest(
    list_services=""
)
responses = stub.ServerReflectionInfo(iter([request]))
for response in responses:
    print(response)
```

### Go Client with Reflection

```go
package main

import (
    "context"
    "fmt"
    "google.golang.org/grpc"
    "google.golang.org/grpc/reflection/grpc_reflection_v1alpha"
)

func main() {
    conn, _ := grpc.Dial("localhost:50051", grpc.WithInsecure())
    defer conn.Close()

    client := grpc_reflection_v1alpha.NewServerReflectionClient(conn)
    stream, _ := client.ServerReflectionInfo(context.Background())

    // List services
    stream.Send(&grpc_reflection_v1alpha.ServerReflectionRequest{
        MessageRequest: &grpc_reflection_v1alpha.ServerReflectionRequest_ListServices{
            ListServices: "",
        },
    })

    resp, _ := stream.Recv()
    fmt.Println(resp)
}
```

## Authentication with Reflection

The reflection service itself doesn't require authentication, but the actual service methods do. When testing authenticated endpoints:

### Get a JWT Token First

```bash
# Login to get a token (using REST API)
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password"}' \
  | jq -r '.token')

# Use the token with grpcurl
grpcurl -plaintext \
  -H "authorization: Bearer $TOKEN" \
  -d '{"id": "patient-123"}' \
  localhost:50051 \
  fhir.PatientService/GetPatient
```

## Troubleshooting

### "failed to list services: server does not support the reflection API"

- Ensure the server is running with reflection enabled
- Check that you're connecting to the correct port (50051 by default)
- Verify network connectivity

### Connection Errors

```bash
# Check if server is running
grpcurl -plaintext localhost:50051 list

# If using TLS, don't use -plaintext
grpcurl -insecure localhost:50051 list
```

### Permission Denied Errors

- Ensure you're including the authentication token
- Verify the token is valid and not expired
- Check that your user has the necessary permissions

### "Message type not found"

- Rebuild the project to regenerate the descriptor file
- Ensure `proto_descriptor.bin` exists in `src/grpc/`

## Performance Considerations

- Reflection adds minimal overhead (a few KB per connection)
- The reflection service is stateless and lightweight
- Descriptor data is cached on the client side
- No performance impact on actual RPC calls

## Security Notes

1. **Production environments**: Some organizations disable reflection in production for security reasons
2. **Service discovery**: Reflection reveals your API structure
3. **Authentication**: Reflection service itself is unauthenticated, but actual services require auth
4. **Rate limiting**: Consider rate limiting the reflection service if exposed publicly

## Disabling Reflection

If you need to disable reflection (not recommended for development), comment out the reflection service registration in `src/grpc/server.rs`:

```rust
// Comment out these lines:
// .add_service(reflection_service)
```

## Additional Resources

- [gRPC Reflection Protocol](https://github.com/grpc/grpc/blob/master/doc/server-reflection.md)
- [grpcurl Documentation](https://github.com/fullstorydev/grpcurl)
- [grpcui Documentation](https://github.com/fullstorydev/grpcui)
- [Tonic Reflection Docs](https://docs.rs/tonic-reflection/)
