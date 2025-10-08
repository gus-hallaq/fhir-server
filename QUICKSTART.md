# Quick Start Guide

Get your FHIR server running in 5 minutes!

## Step 1: Install Prerequisites

### macOS
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PostgreSQL
brew install postgresql@14
brew services start postgresql@14

# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres
```

### Linux (Ubuntu/Debian)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PostgreSQL
sudo apt update
sudo apt install postgresql postgresql-contrib
sudo systemctl start postgresql

# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres
```

### Windows
```bash
# Install Rust from https://rustup.rs/

# Install PostgreSQL from https://www.postgresql.org/download/windows/

# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres
```

## Step 2: Set Up Database

```bash
# Create database
createdb fhir

# Or if you need to use sudo (Linux)
sudo -u postgres createdb fhir
```

## Step 3: Configure Environment

```bash
# Copy environment template
cp .env.example .env

# Edit .env file with your database credentials
# The default configuration works for most local setups
```

Your `.env` should look like:
```env
DATABASE_URL=postgres://postgres:postgres@localhost:5432/fhir
RUN_EXAMPLES=true
RUST_LOG=info,fhir_server=debug
```

## Step 4: Run Migrations

```bash
sqlx migrate run
```

You should see:
```
Applied 001_initial_schema.sql
```

## Step 5: Build and Run

```bash
# Build
cargo build --release

# Run
cargo run
```

You should see output like:
```
🚀 Starting FHIR Server...
📦 Connecting to database...
✅ Database connection established
🔄 Running database migrations...
✅ Migrations completed
🏗️  Initializing repositories...
✅ Repositories initialized
⚙️  Initializing services...
✅ Services initialized
🎉 FHIR Server initialized successfully!
🧪 Running example operations...
✅ Created patient with ID: 123e4567-e89b-12d3-a456-426614174000
✅ Created observation with ID: 234e5678-e89b-12d3-a456-426614174001
✅ All example operations completed successfully!
✅ FHIR Server is ready!
📊 Server listening on http://0.0.0.0:8080
```

## Step 6: Verify Installation

### Check Database
```bash
# Connect to database
psql fhir

# List tables
\dt

# You should see:
# patients
# observations
# conditions
# encounters
# patients_history
# observations_history
# conditions_history
# encounters_history

# Query a patient
SELECT id, resource->>'resourceType' as type, resource->'name'->0->>'family' as family 
FROM patients;

# Exit psql
\q
```

### Check Logs
The server should show successful operations if `RUN_EXAMPLES=true`:
- Patient created
- Observation created
- Condition created
- Encounter created
- Search operations working

## Common Issues

### Issue: "Database connection failed"
**Solution**: Check that PostgreSQL is running:
```bash
# macOS
brew services list

# Linux
sudo systemctl status postgresql

# Windows
# Check Services app for "postgresql" service
```

### Issue: "sqlx-cli not found"
**Solution**: Make sure cargo bin directory is in your PATH:
```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell
source ~/.bashrc
```

### Issue: "Migration failed"
**Solution**: Drop and recreate the database:
```bash
dropdb fhir
createdb fhir
sqlx migrate run
```

### Issue: "Connection pool timeout"
**Solution**: Increase connection timeout in `.env`:
```env
DB_CONNECT_TIMEOUT=60
DB_MAX_CONNECTIONS=20
```

## Next Steps

Now that your server is running:

1. **Explore the code**: Start with `src/main.rs` to see how everything connects
2. **Add the API layer**: Follow the next tutorial to add REST endpoints with Axum
3. **Create your own resources**: Add more FHIR resource types
4. **Write tests**: Add integration tests for your resources

## Quick Commands Reference

```bash
# Development
cargo run                  # Run in debug mode
cargo run --release        # Run in release mode
cargo test                 # Run tests
cargo check               # Quick compile check
cargo fmt                 # Format code
cargo clippy              # Lint code

# Database
createdb fhir             # Create database
dropdb fhir              # Drop database
sqlx migrate run         # Run migrations
sqlx migrate revert      # Revert last migration
psql fhir                # Connect to database

# Logs
RUST_LOG=debug cargo run          # Debug logging
RUST_LOG=trace cargo run          # Trace logging
RUST_LOG=sqlx=debug cargo run     # SQL query logging
```

## Need Help?

- Check the [README.md](README.md) for detailed documentation
- Review example code in `src/main.rs` function `run_examples()`
- Check the [FHIR specification](https://www.hl7.org/fhir/)

Happy coding! 🚀