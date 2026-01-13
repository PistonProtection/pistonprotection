-- =============================================================================
-- Alerts Table Migration
-- =============================================================================
-- This migration adds the alerts table for configurable backend alerts.
-- =============================================================================

-- Alerts Table
CREATE TABLE IF NOT EXISTS alerts (
    id VARCHAR(36) PRIMARY KEY,
    backend_id VARCHAR(36) NOT NULL REFERENCES backends(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    condition JSONB,
    notifications JSONB DEFAULT '[]',
    enabled BOOLEAN DEFAULT TRUE,
    state INTEGER DEFAULT 1,  -- 0: unspecified, 1: ok, 2: pending, 3: firing
    last_triggered TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_alerts_backend ON alerts(backend_id);
CREATE INDEX IF NOT EXISTS idx_alerts_state ON alerts(state);
CREATE INDEX IF NOT EXISTS idx_alerts_enabled ON alerts(backend_id, enabled);

-- Apply update timestamp trigger
DROP TRIGGER IF EXISTS update_alerts_updated_at ON alerts;
CREATE TRIGGER update_alerts_updated_at
    BEFORE UPDATE ON alerts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at();

-- Alert History Table (for tracking triggered alerts)
CREATE TABLE IF NOT EXISTS alert_history (
    id BIGSERIAL PRIMARY KEY,
    alert_id VARCHAR(36) NOT NULL REFERENCES alerts(id) ON DELETE CASCADE,
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    value DOUBLE PRECISION,
    message TEXT,
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_alert_history_alert_time ON alert_history(alert_id, triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_alert_history_unresolved ON alert_history(alert_id, resolved_at)
    WHERE resolved_at IS NULL;

-- =============================================================================
-- Complete
-- =============================================================================

DO $$
BEGIN
    RAISE NOTICE 'Alerts table migration completed successfully';
END $$;
