#!/bin/bash

# Create database
createdb fhir

# Run migrations
sqlx migrate run --database-url postgres://postgres:postgres@localhost/fhir

echo "Database setup complete!"