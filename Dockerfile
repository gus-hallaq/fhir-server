# Multi-stage Dockerfile for FHIR Server
# Stage 1: Build the application
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Copy proto files
COPY proto ./proto

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Stage 2: Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r fhir && useradd -r -g fhir -u 1000 fhir

# Create app directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/fhir-server /app/fhir-server

# Create directories for runtime
RUN mkdir -p /app/.cache /tmp && \
    chown -R fhir:fhir /app /tmp

# Switch to non-root user
USER fhir

# Expose ports
EXPOSE 8080 50051

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/app/fhir-server", "--health-check"] || exit 1

# Run the application
CMD ["/app/fhir-server"]
