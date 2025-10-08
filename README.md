# FHIR Server (Rust)

A production-ready FHIR R4/R5 server implementation in Rust using Axum, SQLx, and PostgreSQL.

## 🏗️ Architecture

```
┌─────────────────────────────────────────┐
│         API Layer (Axum)                │
│     REST endpoints, handlers            │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      Service Layer                      │
│   Business logic, validation            │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│     Repository Layer (SQLx)             │
│   Database operations, queries          │
└─────────────────────────────────────────┘
                  ↓
┌─────────────────────────────────────────┐
│      PostgreSQL Database                │
│   Hybrid storage (JSONB + relational)   │
└─────────────────────────────────────────┘
```

## ✨ Features

### Domain Layer
- ✅ FHIR R4/R5 resource models (Patient, Observation, Condition, Encounter)
- ✅ FHIR primitive types (Id, Code, DateTime, etc.)
- ✅ FHIR complex datatypes (CodeableConcept, Reference, HumanName, etc.)
- ✅ Type-safe domain models with serde serialization

### Repository Layer
- ✅ Hybrid storage: Full FHIR resources in JSONB + indexed search parameters
- ✅ CRUD operations for all resources
- ✅ Version tracking and history
- ✅ Soft deletes
- ✅ Search by indexed parameters
- ✅ Connection pooling
- ✅ Database migrations

### Service Layer
- ✅ FHIR validation (required fields, cardinality, business rules)
- ✅ Reference validation
- ✅ Duplicate checking
- ✅ Conditional create/update
- ✅ Search with pagination
- ✅ Resource-specific queries

### API Layer (Coming Next)
- 🔲 RESTful FHIR endpoints
- 🔲 FHIR search parameters
- 🔲 Content negotiation (JSON/XML)
- 🔲 CapabilityStatement
- 🔲 Bundle support

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- PostgreSQL 14+ ([Install PostgreSQL](https://www.postgresql.org/download/))
- sqlx-cli: `cargo install sqlx-cli --no-default-features --features postgres`

### Installation

1. **Clone the repository**
```bash
git clone <your-repo-url>
cd fhir-server
```

2. **Set up environment variables**
```bash
cp .env.example .env
# Edit .env with your database credentials
```

3. **Create the database**
```bash
createdb fhir
```

4. **Run migrations**
```bash
sqlx migrate run
```

5. **Build the project**
```bash
cargo build --release
```

6. **Run the server**
```bash
cargo run
```

The server will start on `http://0.0.0.0:8080`

## 📋 Project Structure

```
fhir-server/
├── Cargo.toml
├── .env.example
├── README.md
├── migrations/
│   └── 001_initial_schema.sql
└── src/
    ├── main.rs                 # Application entry point
    ├── lib.rs                  # Library exports
    ├── config.rs               # Database configuration
    ├── domain/
    │   ├── mod.rs
    │   ├── primitives.rs       # FHIR primitive types
    │   ├── datatypes.rs        # FHIR complex datatypes
    │   ├── errors.rs           # Error types
    │   └── resources/
    │       ├── mod.rs
    │       ├── patient.rs
    │       ├── observation.rs
    │       ├── condition.rs
    │       └── encounter.rs
    ├── repository/
    │   ├── mod.rs
    │   ├── patient_repository.rs
    │   ├── observation_repository.rs
    │   ├── condition_repository.rs
    │   └── encounter_repository.rs
    └── service/
        ├── mod.rs
        ├── validation.rs
        ├── patient_service.rs
        ├── observation_service.rs
        ├── condition_service.rs
        └── encounter_service.rs
```

## 🧪 Running Examples

The server includes example operations that demonstrate creating and querying FHIR resources.

Set `RUN_EXAMPLES=true` in your `.env` file and run:

```bash
cargo run
```

You'll see output like:
```
🚀 Starting FHIR Server...
✅ Database connection established
✅ Migrations completed
✅ Repositories initialized
✅ Services initialized
🧪 Running example operations...
✅ Created patient with ID: 123e4567-e89b-12d3-a456-426614174000
✅ Created observation with ID: 234e5678-e89b-12d3-a456-426614174001
✅ Created condition with ID: 345e6789-e89b-12d3-a456-426614174002
✅ Created encounter with ID: 456e7890-e89b-12d3-a456-426614174003
🎉 All example operations completed successfully!
```

## 🔍 Development

### Run tests
```bash
cargo test
```

### Run with logging
```bash
RUST_LOG=debug cargo run
```

### Format code
```bash
cargo fmt
```

### Lint code
```bash
cargo clippy
```

### Check for compile errors without building
```bash
cargo check
```

## 📊 Database Schema

The database uses a hybrid approach:
- **JSONB column**: Stores the complete FHIR resource
- **Relational columns**: Indexed search parameters for fast queries
- **History tables**: Track all versions for audit trail
- **Soft deletes**: Resources marked deleted, not removed

Example table structure:
```sql
CREATE TABLE patients (
    id UUID PRIMARY KEY,
    resource JSONB NOT NULL,           -- Full FHIR resource
    version_id INTEGER NOT NULL,
    last_updated TIMESTAMP NOT NULL,
    
    -- Search parameters
    family_name TEXT,
    given_name TEXT,
    gender VARCHAR(20),
    birth_date DATE,
    
    -- Audit
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    deleted_at TIMESTAMP
);
```

## 🔐 Security (TODO)

Future security features:
- OAuth 2.0 / SMART on FHIR authentication
- JWT token validation
- Role-based access control (RBAC)
- Audit logging
- Rate limiting

## 🚧 Roadmap

- [x] Domain models
- [x] Repository layer
- [x] Service layer with validation
- [ ] API layer (Axum REST endpoints)
- [ ] FHIR search parameters
- [ ] Bundle support
- [ ] Transaction operations
- [ ] CapabilityStatement
- [ ] Authentication & Authorization
- [ ] More resource types (Practitioner, Organization, etc.)
- [ ] Terminology services
- [ ] Bulk data export

## 📝 License

MIT License - see LICENSE file for details

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📚 Resources

- [FHIR Specification](https://www.hl7.org/fhir/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/)