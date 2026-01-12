-- =============================================================================
-- PistonProtection Database Initialization
-- =============================================================================
-- This script initializes the PostgreSQL database with required extensions,
-- schemas, and base tables for the PistonProtection platform.
-- =============================================================================

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Create schemas
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS config;
CREATE SCHEMA IF NOT EXISTS metrics;
CREATE SCHEMA IF NOT EXISTS audit;

-- Grant usage on schemas
GRANT USAGE ON SCHEMA auth TO pistonprotection;
GRANT USAGE ON SCHEMA config TO pistonprotection;
GRANT USAGE ON SCHEMA metrics TO pistonprotection;
GRANT USAGE ON SCHEMA audit TO pistonprotection;

-- =============================================================================
-- Auth Schema Tables
-- =============================================================================

-- Organizations
CREATE TABLE IF NOT EXISTS auth.organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    plan VARCHAR(50) DEFAULT 'free',
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Users
CREATE TABLE IF NOT EXISTS auth.users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES auth.organizations(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255),
    name VARCHAR(255),
    role VARCHAR(50) DEFAULT 'member',
    email_verified BOOLEAN DEFAULT FALSE,
    mfa_enabled BOOLEAN DEFAULT FALSE,
    mfa_secret VARCHAR(255),
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_users_email ON auth.users(email);
CREATE INDEX IF NOT EXISTS idx_users_organization ON auth.users(organization_id);

-- Sessions
CREATE TABLE IF NOT EXISTS auth.sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON auth.sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON auth.sessions(expires_at);

-- API Keys
CREATE TABLE IF NOT EXISTS auth.api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES auth.organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    prefix VARCHAR(10) NOT NULL,
    scopes TEXT[] DEFAULT '{}',
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_api_keys_org ON auth.api_keys(organization_id);

-- =============================================================================
-- Config Schema Tables
-- =============================================================================

-- Backends (origin servers)
CREATE TABLE IF NOT EXISTS config.backends (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES auth.organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    address VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL,
    protocol VARCHAR(50) DEFAULT 'tcp',
    weight INTEGER DEFAULT 100,
    health_check_path VARCHAR(255) DEFAULT '/health',
    health_check_interval INTEGER DEFAULT 30,
    enabled BOOLEAN DEFAULT TRUE,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_backends_org ON config.backends(organization_id);

-- Filter Rules
CREATE TABLE IF NOT EXISTS config.filter_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES auth.organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    priority INTEGER DEFAULT 0,
    action VARCHAR(50) NOT NULL,  -- allow, deny, rate_limit, challenge
    conditions JSONB NOT NULL,
    rate_limit_config JSONB,
    enabled BOOLEAN DEFAULT TRUE,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_filter_rules_org ON config.filter_rules(organization_id);
CREATE INDEX IF NOT EXISTS idx_filter_rules_priority ON config.filter_rules(priority);

-- IP Lists (allowlist/blocklist)
CREATE TABLE IF NOT EXISTS config.ip_lists (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES auth.organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    list_type VARCHAR(50) NOT NULL,  -- allowlist, blocklist
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS config.ip_list_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    list_id UUID REFERENCES config.ip_lists(id) ON DELETE CASCADE,
    ip_address INET NOT NULL,
    comment TEXT,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ip_entries_list ON config.ip_list_entries(list_id);
CREATE INDEX IF NOT EXISTS idx_ip_entries_ip ON config.ip_list_entries(ip_address);

-- Protection Profiles
CREATE TABLE IF NOT EXISTS config.protection_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID REFERENCES auth.organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    settings JSONB NOT NULL DEFAULT '{}',
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- Metrics Schema Tables
-- =============================================================================

-- Traffic Stats (time-series style)
CREATE TABLE IF NOT EXISTS metrics.traffic_stats (
    id BIGSERIAL PRIMARY KEY,
    organization_id UUID,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    packets_total BIGINT DEFAULT 0,
    packets_dropped BIGINT DEFAULT 0,
    bytes_total BIGINT DEFAULT 0,
    connections_total BIGINT DEFAULT 0,
    requests_total BIGINT DEFAULT 0,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_traffic_stats_org_time ON metrics.traffic_stats(organization_id, timestamp DESC);

-- Attack Events
CREATE TABLE IF NOT EXISTS metrics.attack_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID,
    attack_type VARCHAR(100) NOT NULL,
    severity VARCHAR(50) NOT NULL,
    source_ips TEXT[],
    target_port INTEGER,
    packets_per_second BIGINT,
    bytes_per_second BIGINT,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    mitigated BOOLEAN DEFAULT FALSE,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_attack_events_org ON metrics.attack_events(organization_id);
CREATE INDEX IF NOT EXISTS idx_attack_events_time ON metrics.attack_events(started_at DESC);

-- =============================================================================
-- Audit Schema Tables
-- =============================================================================

-- Audit Log
CREATE TABLE IF NOT EXISTS audit.logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID,
    user_id UUID,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id UUID,
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_org ON audit.logs(organization_id);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit.logs(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_time ON audit.logs(created_at DESC);

-- =============================================================================
-- Functions and Triggers
-- =============================================================================

-- Update timestamp function
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply updated_at triggers
DO $$
DECLARE
    t record;
BEGIN
    FOR t IN
        SELECT table_schema, table_name
        FROM information_schema.columns
        WHERE column_name = 'updated_at'
        AND table_schema IN ('auth', 'config')
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS update_%I_%I_updated_at ON %I.%I',
            t.table_schema, t.table_name, t.table_schema, t.table_name);
        EXECUTE format('CREATE TRIGGER update_%I_%I_updated_at
            BEFORE UPDATE ON %I.%I
            FOR EACH ROW EXECUTE FUNCTION update_updated_at()',
            t.table_schema, t.table_name, t.table_schema, t.table_name);
    END LOOP;
END;
$$;

-- =============================================================================
-- Initial Data
-- =============================================================================

-- Insert default organization for development
INSERT INTO auth.organizations (id, name, slug, plan)
VALUES ('00000000-0000-0000-0000-000000000001', 'Development', 'dev', 'enterprise')
ON CONFLICT (slug) DO NOTHING;

-- Insert default admin user (password: admin123)
INSERT INTO auth.users (id, organization_id, email, password_hash, name, role, email_verified)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000001',
    'admin@pistonprotection.local',
    '$argon2id$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$hash', -- placeholder, update with real hash
    'Admin User',
    'admin',
    TRUE
)
ON CONFLICT (email) DO NOTHING;

-- Insert default protection profile
INSERT INTO config.protection_profiles (organization_id, name, description, settings, is_default)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Default Protection',
    'Default DDoS protection profile',
    '{
        "rate_limit": {
            "enabled": true,
            "requests_per_second": 1000,
            "burst_size": 5000
        },
        "geo_blocking": {
            "enabled": false,
            "blocked_countries": []
        },
        "challenge": {
            "enabled": true,
            "threshold": 100
        },
        "protocol_validation": {
            "enabled": true,
            "strict_mode": false
        }
    }',
    TRUE
)
ON CONFLICT DO NOTHING;

-- =============================================================================
-- Complete
-- =============================================================================

DO $$
BEGIN
    RAISE NOTICE 'PistonProtection database initialized successfully';
END $$;
