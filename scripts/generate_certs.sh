#!/bin/bash
# Generate self-signed TLS certificates for development/testing
# DO NOT use these certificates in production!

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CERTS_DIR="$SCRIPT_DIR/../certs"

# Create certs directory if it doesn't exist
mkdir -p "$CERTS_DIR"

echo "🔐 Generating self-signed TLS certificates..."

# Generate private key
openssl genrsa -out "$CERTS_DIR/server.key" 2048
echo "✅ Generated private key: $CERTS_DIR/server.key"

# Generate certificate signing request
openssl req -new -key "$CERTS_DIR/server.key" \
  -out "$CERTS_DIR/server.csr" \
  -subj "/C=US/ST=State/L=City/O=Organization/OU=Department/CN=localhost"
echo "✅ Generated CSR: $CERTS_DIR/server.csr"

# Generate self-signed certificate (valid for 365 days)
openssl x509 -req -days 365 \
  -in "$CERTS_DIR/server.csr" \
  -signkey "$CERTS_DIR/server.key" \
  -out "$CERTS_DIR/server.crt"
echo "✅ Generated certificate: $CERTS_DIR/server.crt"

# Clean up CSR
rm "$CERTS_DIR/server.csr"

echo ""
echo "🎉 Certificate generation complete!"
echo ""
echo "📁 Certificate files location:"
echo "   Certificate: $CERTS_DIR/server.crt"
echo "   Private Key: $CERTS_DIR/server.key"
echo ""
echo "⚠️  WARNING: These are self-signed certificates for development only."
echo "   Do NOT use in production environments!"
echo ""
echo "To enable TLS in your gRPC server, add to your .env file:"
echo "   GRPC_TLS_ENABLED=true"
echo "   GRPC_TLS_CERT_PATH=./certs/server.crt"
echo "   GRPC_TLS_KEY_PATH=./certs/server.key"
