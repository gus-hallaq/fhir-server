# gRPC TLS Configuration Guide

This guide explains how to configure TLS (Transport Layer Security) for the gRPC server in the FHIR Server application.

## Overview

TLS provides encrypted communication between the gRPC client and server, ensuring that:
- Data transmitted is encrypted and cannot be intercepted
- The server identity is verified through certificates
- Communication integrity is maintained

## Configuration

### Environment Variables

The following environment variables control TLS configuration:

```bash
# Enable or disable TLS for gRPC
GRPC_TLS_ENABLED=false

# Path to TLS certificate (required if TLS is enabled)
GRPC_TLS_CERT_PATH=./certs/server.crt

# Path to TLS private key (required if TLS is enabled)
GRPC_TLS_KEY_PATH=./certs/server.key

# gRPC server host and port
GRPC_HOST=0.0.0.0
GRPC_PORT=50051
```

### Certificate Requirements

The server requires:
1. **Certificate file** (`.crt` or `.pem`): Contains the public certificate
2. **Private key file** (`.key` or `.pem`): Contains the private key

Both files must be in PEM format.

## Development/Testing Setup

### Generate Self-Signed Certificates

For development and testing purposes, you can generate self-signed certificates:

```bash
# Run the certificate generation script
./scripts/generate_certs.sh
```

This will create:
- `certs/server.crt` - Self-signed certificate
- `certs/server.key` - Private key

**⚠️ WARNING**: Self-signed certificates should NEVER be used in production!

### Enable TLS in Development

1. Generate certificates (if not already done):
   ```bash
   ./scripts/generate_certs.sh
   ```

2. Update your `.env` file:
   ```bash
   GRPC_TLS_ENABLED=true
   GRPC_TLS_CERT_PATH=./certs/server.crt
   GRPC_TLS_KEY_PATH=./certs/server.key
   ```

3. Start the server:
   ```bash
   cargo run
   ```

You should see:
```
🔒 TLS enabled for gRPC server
✅ TLS configured successfully
📡 Starting secure gRPC server on 0.0.0.0:50051
```

## Production Setup

### Obtaining Production Certificates

For production, you should obtain certificates from a trusted Certificate Authority (CA):

1. **Let's Encrypt** (free, automated):
   ```bash
   # Example using certbot
   certbot certonly --standalone -d your-domain.com
   ```

2. **Commercial CA** (DigiCert, GlobalSign, etc.):
   - Purchase a certificate
   - Follow CA-specific instructions for generation

3. **Internal CA** (for private networks):
   - Use your organization's PKI infrastructure

### Production Configuration

1. Place your certificates in a secure location:
   ```bash
   /etc/fhir-server/certs/
   ├── server.crt
   └── server.key
   ```

2. Set strict file permissions:
   ```bash
   chmod 600 /etc/fhir-server/certs/server.key
   chmod 644 /etc/fhir-server/certs/server.crt
   ```

3. Configure environment variables:
   ```bash
   GRPC_TLS_ENABLED=true
   GRPC_TLS_CERT_PATH=/etc/fhir-server/certs/server.crt
   GRPC_TLS_KEY_PATH=/etc/fhir-server/certs/server.key
   ```

## Client Configuration

### Connecting with TLS

When TLS is enabled on the server, clients must connect using the `https` scheme:

#### grpcurl Example
```bash
# Without TLS
grpcurl -plaintext localhost:50051 list

# With TLS (self-signed - skip verification)
grpcurl -insecure localhost:50051 list

# With TLS (production - with CA cert)
grpcurl -cacert certs/server.crt localhost:50051 list
```

#### Rust Client Example
```rust
use tonic::transport::{Channel, ClientTlsConfig};

// Connect with TLS
let tls_config = ClientTlsConfig::new()
    .domain_name("localhost");

let channel = Channel::from_static("https://localhost:50051")
    .tls_config(tls_config)?
    .connect()
    .await?;
```

#### Python Client Example
```python
import grpc

# Load credentials
credentials = grpc.ssl_channel_credentials(
    root_certificates=open('certs/server.crt', 'rb').read()
)

# Create secure channel
channel = grpc.secure_channel('localhost:50051', credentials)
```

## Disabling TLS

To run without TLS (not recommended for production):

```bash
GRPC_TLS_ENABLED=false
```

The server will start without encryption:
```
⚠️  TLS disabled - gRPC server running without encryption
📡 Starting gRPC server on 0.0.0.0:50051
```

## Troubleshooting

### Certificate Errors

**Error**: "Failed to read TLS certificate"
- Verify the certificate path is correct
- Check file permissions
- Ensure the file exists and is readable

**Error**: "Invalid certificate format"
- Ensure the certificate is in PEM format
- Check for correct begin/end markers: `-----BEGIN CERTIFICATE-----`

### Connection Errors

**Error**: "Connection refused" (client)
- Verify the server is running
- Check firewall rules
- Ensure client is using correct scheme (`https://` for TLS)

**Error**: "Certificate verification failed" (client)
- For self-signed certs, use `-insecure` flag or skip verification
- For production, ensure CA certificate is trusted by the client

## Security Best Practices

1. **Always use TLS in production**
2. **Never commit certificates or private keys to version control**
3. **Use strong key sizes** (minimum 2048-bit RSA)
4. **Rotate certificates regularly** (before expiration)
5. **Set restrictive file permissions** on private keys
6. **Use certificates from trusted CAs** in production
7. **Enable certificate revocation checking** when possible
8. **Monitor certificate expiration dates**

## Additional Resources

- [Tonic TLS Documentation](https://docs.rs/tonic/latest/tonic/transport/server/struct.ServerTlsConfig.html)
- [gRPC Authentication Guide](https://grpc.io/docs/guides/auth/)
- [Let's Encrypt](https://letsencrypt.org/)
- [OpenSSL Documentation](https://www.openssl.org/docs/)
