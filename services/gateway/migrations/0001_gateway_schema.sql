-- =============================================================================
-- Gateway Service Database Schema
-- =============================================================================
-- This migration adds tables for backend origins, domains, protection settings,
-- and filter rule statistics required by the gateway service.
-- =============================================================================

-- =============================================================================
-- Backend Tables
-- =============================================================================

-- Backends (main configuration)
CREATE TABLE IF NOT EXISTS backends (
    id VARCHAR(36) PRIMARY KEY,
    organization_id VARCHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    type INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_backends_organization ON backends(organization_id);
CREATE INDEX IF NOT EXISTS idx_backends_name ON backends(name);

-- Backend Origins (origin servers)
CREATE TABLE IF NOT EXISTS backend_origins (
    id VARCHAR(36) PRIMARY KEY,
    backend_id VARCHAR(36) NOT NULL REFERENCES backends(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    address JSONB,
    port INTEGER NOT NULL,
    hostname VARCHAR(255),
    weight INTEGER DEFAULT 100,
    priority INTEGER DEFAULT 0,
    settings JSONB DEFAULT '{}',
    enabled BOOLEAN DEFAULT TRUE,
    health_status INTEGER DEFAULT 0,
    last_health_check TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_backend_origins_backend ON backend_origins(backend_id);
CREATE INDEX IF NOT EXISTS idx_backend_origins_enabled ON backend_origins(backend_id, enabled);

-- Backend Domains
CREATE TABLE IF NOT EXISTS backend_domains (
    backend_id VARCHAR(36) NOT NULL REFERENCES backends(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL,
    verification_token VARCHAR(255) NOT NULL,
    verification_method VARCHAR(50) DEFAULT 'DNS_TXT',
    verified BOOLEAN DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (backend_id, domain)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_backend_domains_domain ON backend_domains(domain);
CREATE INDEX IF NOT EXISTS idx_backend_domains_verified ON backend_domains(backend_id, verified);

-- Backend Protection Settings
CREATE TABLE IF NOT EXISTS backend_protection (
    backend_id VARCHAR(36) PRIMARY KEY REFERENCES backends(id) ON DELETE CASCADE,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- Filter Rules Tables
-- =============================================================================

-- Filter Rules
CREATE TABLE IF NOT EXISTS filter_rules (
    id VARCHAR(36) PRIMARY KEY,
    backend_id VARCHAR(36) NOT NULL REFERENCES backends(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    priority INTEGER DEFAULT 0,
    match_criteria JSONB NOT NULL DEFAULT '{}',
    action INTEGER NOT NULL DEFAULT 1,
    rate_limit JSONB,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_filter_rules_backend ON filter_rules(backend_id);
CREATE INDEX IF NOT EXISTS idx_filter_rules_priority ON filter_rules(backend_id, priority);
CREATE INDEX IF NOT EXISTS idx_filter_rules_enabled ON filter_rules(backend_id, enabled);

-- Filter Rule Statistics (time-series data)
CREATE TABLE IF NOT EXISTS filter_rule_stats (
    id BIGSERIAL PRIMARY KEY,
    rule_id VARCHAR(36) NOT NULL REFERENCES filter_rules(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    packets_matched BIGINT DEFAULT 0,
    bytes_matched BIGINT DEFAULT 0,
    packets_allowed BIGINT DEFAULT 0,
    packets_dropped BIGINT DEFAULT 0,
    packets_rate_limited BIGINT DEFAULT 0,
    packets_challenged BIGINT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_filter_rule_stats_rule_time ON filter_rule_stats(rule_id, timestamp DESC);

-- =============================================================================
-- Metrics Tables
-- =============================================================================

-- Traffic Metrics
CREATE TABLE IF NOT EXISTS traffic_metrics (
    id BIGSERIAL PRIMARY KEY,
    backend_id VARCHAR(36) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    requests_total BIGINT DEFAULT 0,
    requests_per_second BIGINT DEFAULT 0,
    bytes_in BIGINT DEFAULT 0,
    bytes_out BIGINT DEFAULT 0,
    packets_in BIGINT DEFAULT 0,
    packets_out BIGINT DEFAULT 0,
    active_connections BIGINT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_traffic_metrics_backend_time ON traffic_metrics(backend_id, timestamp DESC);

-- Geo Traffic Distribution
CREATE TABLE IF NOT EXISTS traffic_geo (
    id BIGSERIAL PRIMARY KEY,
    backend_id VARCHAR(36) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    country_code VARCHAR(2) NOT NULL,
    country_name VARCHAR(100),
    requests BIGINT DEFAULT 0,
    bytes BIGINT DEFAULT 0,
    source_ip INET
);

CREATE INDEX IF NOT EXISTS idx_traffic_geo_backend_time ON traffic_geo(backend_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_traffic_geo_country ON traffic_geo(country_code);

-- Metrics Time Series (generic)
CREATE TABLE IF NOT EXISTS metrics_timeseries (
    id BIGSERIAL PRIMARY KEY,
    backend_id VARCHAR(36) NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    value DOUBLE PRECISION NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_metrics_timeseries_lookup
    ON metrics_timeseries(backend_id, metric_name, timestamp DESC);

-- Attack Events
CREATE TABLE IF NOT EXISTS attack_events (
    id VARCHAR(36) PRIMARY KEY,
    backend_id VARCHAR(36) NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ,
    duration_seconds INTEGER DEFAULT 0,
    attack_type VARCHAR(100) NOT NULL,
    severity INTEGER DEFAULT 1,
    peak_pps BIGINT DEFAULT 0,
    peak_bps BIGINT DEFAULT 0,
    total_packets BIGINT DEFAULT 0,
    total_bytes BIGINT DEFAULT 0,
    packets_mitigated BIGINT DEFAULT 0,
    mitigation_rate DOUBLE PRECISION DEFAULT 0,
    unique_sources INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_attack_events_backend_time ON attack_events(backend_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_attack_events_type ON attack_events(attack_type);

-- =============================================================================
-- Functions and Triggers
-- =============================================================================

-- Update timestamp function (if not exists)
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply triggers for updated_at
DO $$
DECLARE
    tables TEXT[] := ARRAY[
        'backends',
        'backend_origins',
        'backend_domains',
        'backend_protection',
        'filter_rules'
    ];
    t TEXT;
BEGIN
    FOREACH t IN ARRAY tables
    LOOP
        EXECUTE format('DROP TRIGGER IF EXISTS update_%I_updated_at ON %I', t, t);
        EXECUTE format(
            'CREATE TRIGGER update_%I_updated_at
             BEFORE UPDATE ON %I
             FOR EACH ROW EXECUTE FUNCTION update_updated_at()',
            t, t
        );
    END LOOP;
END;
$$;

-- =============================================================================
-- Complete
-- =============================================================================

DO $$
BEGIN
    RAISE NOTICE 'Gateway service database schema initialized successfully';
END $$;
