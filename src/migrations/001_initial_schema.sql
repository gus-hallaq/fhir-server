-- migrations/001_initial_schema.sql
-- FHIR Server Database Schema

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Patient table
CREATE TABLE IF NOT EXISTS patients (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource_type VARCHAR(50) NOT NULL DEFAULT 'Patient',
    version_id INTEGER NOT NULL DEFAULT 1,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Full FHIR resource as JSONB
    resource JSONB NOT NULL,
    
    -- Indexed search parameters for common queries
    active BOOLEAN,
    family_name TEXT,
    given_name TEXT,
    gender VARCHAR(20),
    birth_date DATE,
    deceased BOOLEAN,
    
    -- Audit fields
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT patients_resource_type_check CHECK (resource_type = 'Patient')
);

-- Indexes for Patient
CREATE INDEX idx_patients_family_name ON patients USING gin(to_tsvector('english', family_name));
CREATE INDEX idx_patients_given_name ON patients USING gin(to_tsvector('english', given_name));
CREATE INDEX idx_patients_gender ON patients(gender);
CREATE INDEX idx_patients_birth_date ON patients(birth_date);
CREATE INDEX idx_patients_active ON patients(active) WHERE active = true;
CREATE INDEX idx_patients_deleted_at ON patients(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_patients_resource_gin ON patients USING gin(resource);

-- Observation table
CREATE TABLE IF NOT EXISTS observations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource_type VARCHAR(50) NOT NULL DEFAULT 'Observation',
    version_id INTEGER NOT NULL DEFAULT 1,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Full FHIR resource as JSONB
    resource JSONB NOT NULL,
    
    -- Indexed search parameters
    status VARCHAR(20) NOT NULL,
    subject_id UUID REFERENCES patients(id),
    category_code TEXT,
    code_code TEXT,
    code_system TEXT,
    effective_datetime TIMESTAMP WITH TIME ZONE,
    issued TIMESTAMP WITH TIME ZONE,
    
    -- Audit fields
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT observations_resource_type_check CHECK (resource_type = 'Observation')
);

-- Indexes for Observation
CREATE INDEX idx_observations_status ON observations(status);
CREATE INDEX idx_observations_subject_id ON observations(subject_id);
CREATE INDEX idx_observations_category_code ON observations(category_code);
CREATE INDEX idx_observations_code_code ON observations(code_code);
CREATE INDEX idx_observations_effective_datetime ON observations(effective_datetime);
CREATE INDEX idx_observations_deleted_at ON observations(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_observations_resource_gin ON observations USING gin(resource);

-- Condition table
CREATE TABLE IF NOT EXISTS conditions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource_type VARCHAR(50) NOT NULL DEFAULT 'Condition',
    version_id INTEGER NOT NULL DEFAULT 1,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Full FHIR resource as JSONB
    resource JSONB NOT NULL,
    
    -- Indexed search parameters
    subject_id UUID REFERENCES patients(id),
    clinical_status VARCHAR(50),
    verification_status VARCHAR(50),
    category_code TEXT,
    code_code TEXT,
    code_system TEXT,
    onset_datetime TIMESTAMP WITH TIME ZONE,
    recorded_date TIMESTAMP WITH TIME ZONE,
    
    -- Audit fields
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT conditions_resource_type_check CHECK (resource_type = 'Condition')
);

-- Indexes for Condition
CREATE INDEX idx_conditions_subject_id ON conditions(subject_id);
CREATE INDEX idx_conditions_clinical_status ON conditions(clinical_status);
CREATE INDEX idx_conditions_code_code ON conditions(code_code);
CREATE INDEX idx_conditions_onset_datetime ON conditions(onset_datetime);
CREATE INDEX idx_conditions_deleted_at ON conditions(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_conditions_resource_gin ON conditions USING gin(resource);

-- Encounter table
CREATE TABLE IF NOT EXISTS encounters (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    resource_type VARCHAR(50) NOT NULL DEFAULT 'Encounter',
    version_id INTEGER NOT NULL DEFAULT 1,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Full FHIR resource as JSONB
    resource JSONB NOT NULL,
    
    -- Indexed search parameters
    status VARCHAR(20) NOT NULL,
    class_code VARCHAR(50),
    subject_id UUID REFERENCES patients(id),
    period_start TIMESTAMP WITH TIME ZONE,
    period_end TIMESTAMP WITH TIME ZONE,
    
    -- Audit fields
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT encounters_resource_type_check CHECK (resource_type = 'Encounter')
);

-- Indexes for Encounter
CREATE INDEX idx_encounters_status ON encounters(status);
CREATE INDEX idx_encounters_subject_id ON encounters(subject_id);
CREATE INDEX idx_encounters_class_code ON encounters(class_code);
CREATE INDEX idx_encounters_period_start ON encounters(period_start);
CREATE INDEX idx_encounters_period_end ON encounters(period_end);
CREATE INDEX idx_encounters_deleted_at ON encounters(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_encounters_resource_gin ON encounters USING gin(resource);

-- Resource history tables for versioning
CREATE TABLE IF NOT EXISTS patients_history (
    id UUID NOT NULL,
    version_id INTEGER NOT NULL,
    resource JSONB NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL,
    operation VARCHAR(10) NOT NULL, -- CREATE, UPDATE, DELETE
    PRIMARY KEY (id, version_id)
);

CREATE TABLE IF NOT EXISTS observations_history (
    id UUID NOT NULL,
    version_id INTEGER NOT NULL,
    resource JSONB NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL,
    operation VARCHAR(10) NOT NULL,
    PRIMARY KEY (id, version_id)
);

CREATE TABLE IF NOT EXISTS conditions_history (
    id UUID NOT NULL,
    version_id INTEGER NOT NULL,
    resource JSONB NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL,
    operation VARCHAR(10) NOT NULL,
    PRIMARY KEY (id, version_id)
);

CREATE TABLE IF NOT EXISTS encounters_history (
    id UUID NOT NULL,
    version_id INTEGER NOT NULL,
    resource JSONB NOT NULL,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL,
    operation VARCHAR(10) NOT NULL,
    PRIMARY KEY (id, version_id)
);

-- Triggers to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_patients_updated_at BEFORE UPDATE ON patients
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_observations_updated_at BEFORE UPDATE ON observations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_conditions_updated_at BEFORE UPDATE ON conditions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_encounters_updated_at BEFORE UPDATE ON encounters
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();