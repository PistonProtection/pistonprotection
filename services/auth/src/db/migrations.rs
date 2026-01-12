//! Database migrations for the auth service

use sqlx::PgPool;
use tracing::info;

/// Run database migrations
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running auth service database migrations");

    // Create custom types
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE user_role AS ENUM ('user', 'admin');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE organization_role AS ENUM ('owner', 'admin', 'member', 'viewer');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE subscription_status AS ENUM ('active', 'trialing', 'past_due', 'canceled', 'unpaid');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'expired', 'revoked');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Create users table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id VARCHAR(36) PRIMARY KEY,
            email VARCHAR(255) NOT NULL UNIQUE,
            username VARCHAR(50) NOT NULL UNIQUE,
            name VARCHAR(100) NOT NULL,
            avatar_url TEXT,
            password_hash TEXT,
            email_verified BOOLEAN NOT NULL DEFAULT FALSE,
            two_factor_enabled BOOLEAN NOT NULL DEFAULT FALSE,
            two_factor_secret TEXT,
            role user_role NOT NULL DEFAULT 'user',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            last_login_at TIMESTAMPTZ,
            deleted_at TIMESTAMPTZ
        );
        CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
        CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
        CREATE INDEX IF NOT EXISTS idx_users_deleted_at ON users(deleted_at);
        "#,
    )
    .execute(pool)
    .await?;

    // Create user_oauth_providers table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS user_oauth_providers (
            id VARCHAR(36) PRIMARY KEY,
            user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            provider VARCHAR(50) NOT NULL,
            provider_user_id VARCHAR(255) NOT NULL,
            access_token TEXT,
            refresh_token TEXT,
            expires_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(provider, provider_user_id)
        );
        CREATE INDEX IF NOT EXISTS idx_oauth_user_id ON user_oauth_providers(user_id);
        CREATE INDEX IF NOT EXISTS idx_oauth_provider ON user_oauth_providers(provider, provider_user_id);
        "#,
    )
    .execute(pool)
    .await?;

    // Create organizations table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS organizations (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            slug VARCHAR(50) NOT NULL UNIQUE,
            logo_url TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        );
        CREATE INDEX IF NOT EXISTS idx_organizations_slug ON organizations(slug);
        CREATE INDEX IF NOT EXISTS idx_organizations_deleted_at ON organizations(deleted_at);
        "#,
    )
    .execute(pool)
    .await?;

    // Create subscriptions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS subscriptions (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            plan_id VARCHAR(50) NOT NULL,
            plan_name VARCHAR(100) NOT NULL,
            status subscription_status NOT NULL DEFAULT 'trialing',
            stripe_customer_id VARCHAR(255),
            stripe_subscription_id VARCHAR(255),
            current_period_start TIMESTAMPTZ NOT NULL,
            current_period_end TIMESTAMPTZ NOT NULL,
            in_trial BOOLEAN NOT NULL DEFAULT TRUE,
            trial_ends_at TIMESTAMPTZ,
            cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,
            canceled_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_subscriptions_org ON subscriptions(organization_id);
        CREATE INDEX IF NOT EXISTS idx_subscriptions_stripe ON subscriptions(stripe_customer_id);
        "#,
    )
    .execute(pool)
    .await?;

    // Create organization_limits table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS organization_limits (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            max_backends INTEGER NOT NULL DEFAULT 3,
            max_origins_per_backend INTEGER NOT NULL DEFAULT 2,
            max_domains INTEGER NOT NULL DEFAULT 5,
            max_filter_rules INTEGER NOT NULL DEFAULT 10,
            max_bandwidth_bytes BIGINT NOT NULL DEFAULT 10737418240,
            max_requests BIGINT NOT NULL DEFAULT 1000000,
            advanced_protection BOOLEAN NOT NULL DEFAULT FALSE,
            priority_support BOOLEAN NOT NULL DEFAULT FALSE,
            custom_ssl BOOLEAN NOT NULL DEFAULT FALSE,
            api_access BOOLEAN NOT NULL DEFAULT TRUE,
            data_retention_days INTEGER NOT NULL DEFAULT 7,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(organization_id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create organization_usage table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS organization_usage (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            backends_count INTEGER NOT NULL DEFAULT 0,
            domains_count INTEGER NOT NULL DEFAULT 0,
            filter_rules_count INTEGER NOT NULL DEFAULT 0,
            bandwidth_used BIGINT NOT NULL DEFAULT 0,
            requests_used BIGINT NOT NULL DEFAULT 0,
            usage_reset_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(organization_id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create organization_members table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS organization_members (
            id VARCHAR(36) PRIMARY KEY,
            user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            role organization_role NOT NULL DEFAULT 'member',
            joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(user_id, organization_id)
        );
        CREATE INDEX IF NOT EXISTS idx_org_members_user ON organization_members(user_id);
        CREATE INDEX IF NOT EXISTS idx_org_members_org ON organization_members(organization_id);
        "#,
    )
    .execute(pool)
    .await?;

    // Create roles table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS roles (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) REFERENCES organizations(id) ON DELETE CASCADE,
            name VARCHAR(50) NOT NULL,
            display_name VARCHAR(100) NOT NULL,
            description TEXT,
            is_system BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(organization_id, name)
        );
        CREATE INDEX IF NOT EXISTS idx_roles_org ON roles(organization_id);
        "#,
    )
    .execute(pool)
    .await?;

    // Create permissions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS permissions (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) NOT NULL UNIQUE,
            display_name VARCHAR(150) NOT NULL,
            description TEXT,
            resource_type VARCHAR(50) NOT NULL,
            action VARCHAR(50) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_permissions_resource ON permissions(resource_type);
        "#,
    )
    .execute(pool)
    .await?;

    // Create role_permissions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS role_permissions (
            id VARCHAR(36) PRIMARY KEY,
            role_id VARCHAR(36) NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            permission_id VARCHAR(36) NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(role_id, permission_id)
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Create role_assignments table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS role_assignments (
            id VARCHAR(36) PRIMARY KEY,
            user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            role_id VARCHAR(36) NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(user_id, role_id, organization_id)
        );
        CREATE INDEX IF NOT EXISTS idx_role_assignments_user ON role_assignments(user_id);
        "#,
    )
    .execute(pool)
    .await?;

    // Create sessions table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id VARCHAR(36) PRIMARY KEY,
            user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            token_hash VARCHAR(64) NOT NULL UNIQUE,
            ip_address VARCHAR(45),
            user_agent TEXT,
            device_type VARCHAR(20),
            active BOOLEAN NOT NULL DEFAULT TRUE,
            expires_at TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_token ON sessions(token_hash);
        CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at);
        "#,
    )
    .execute(pool)
    .await?;

    // Create refresh_tokens table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS refresh_tokens (
            id VARCHAR(36) PRIMARY KEY,
            user_id VARCHAR(36) NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            session_id VARCHAR(36) NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            token_hash VARCHAR(64) NOT NULL UNIQUE,
            expires_at TIMESTAMPTZ NOT NULL,
            revoked BOOLEAN NOT NULL DEFAULT FALSE,
            revoked_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_refresh_tokens_user ON refresh_tokens(user_id);
        CREATE INDEX IF NOT EXISTS idx_refresh_tokens_token ON refresh_tokens(token_hash);
        "#,
    )
    .execute(pool)
    .await?;

    // Create api_keys table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS api_keys (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            created_by_user_id VARCHAR(36) NOT NULL REFERENCES users(id),
            name VARCHAR(100) NOT NULL,
            prefix VARCHAR(20) NOT NULL,
            key_hash VARCHAR(64) NOT NULL UNIQUE,
            permissions JSONB NOT NULL DEFAULT '[]'::JSONB,
            allowed_ips JSONB NOT NULL DEFAULT '[]'::JSONB,
            enabled BOOLEAN NOT NULL DEFAULT TRUE,
            expires_at TIMESTAMPTZ,
            last_used_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_api_keys_org ON api_keys(organization_id);
        CREATE INDEX IF NOT EXISTS idx_api_keys_hash ON api_keys(key_hash);
        CREATE INDEX IF NOT EXISTS idx_api_keys_prefix ON api_keys(prefix);
        "#,
    )
    .execute(pool)
    .await?;

    // Create invitations table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS invitations (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            email VARCHAR(255) NOT NULL,
            role organization_role NOT NULL DEFAULT 'member',
            invited_by_user_id VARCHAR(36) NOT NULL REFERENCES users(id),
            status invitation_status NOT NULL DEFAULT 'pending',
            token_hash VARCHAR(64) NOT NULL UNIQUE,
            expires_at TIMESTAMPTZ NOT NULL,
            accepted_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_invitations_org ON invitations(organization_id);
        CREATE INDEX IF NOT EXISTS idx_invitations_email ON invitations(email);
        CREATE INDEX IF NOT EXISTS idx_invitations_token ON invitations(token_hash);
        "#,
    )
    .execute(pool)
    .await?;

    // Create audit_logs table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            user_id VARCHAR(36),
            user_email VARCHAR(255),
            action VARCHAR(100) NOT NULL,
            resource_type VARCHAR(50) NOT NULL,
            resource_id VARCHAR(36),
            description TEXT NOT NULL,
            metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
            ip_address VARCHAR(45),
            user_agent TEXT,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_audit_logs_org ON audit_logs(organization_id);
        CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
        CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp);
        CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
        "#,
    )
    .execute(pool)
    .await?;

    // Run billing-related migrations
    run_billing_migrations(pool).await?;

    info!("Auth service database migrations completed");
    Ok(())
}

/// Run billing and subscription related migrations
async fn run_billing_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    info!("Running billing migrations");

    // Create additional enum types for billing
    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE plan_type AS ENUM ('free', 'starter', 'pro', 'enterprise');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE billing_period AS ENUM ('monthly', 'yearly');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE invoice_status AS ENUM ('draft', 'open', 'paid', 'uncollectible', 'void');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DO $$ BEGIN
            CREATE TYPE usage_metric_type AS ENUM ('requests', 'bandwidth_bytes', 'blocked_requests', 'challenges_served');
        EXCEPTION
            WHEN duplicate_object THEN null;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Create plans table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS plans (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            plan_type plan_type NOT NULL DEFAULT 'starter',
            description TEXT,
            stripe_product_id VARCHAR(255) UNIQUE,
            stripe_price_id_monthly VARCHAR(255),
            stripe_price_id_yearly VARCHAR(255),
            price_monthly_cents BIGINT NOT NULL DEFAULT 0,
            price_yearly_cents BIGINT NOT NULL DEFAULT 0,
            max_backends INTEGER NOT NULL DEFAULT 3,
            max_origins_per_backend INTEGER NOT NULL DEFAULT 2,
            max_domains INTEGER NOT NULL DEFAULT 5,
            max_filter_rules INTEGER NOT NULL DEFAULT 10,
            max_bandwidth_bytes BIGINT NOT NULL DEFAULT 10737418240,
            max_requests BIGINT NOT NULL DEFAULT 1000000,
            advanced_protection BOOLEAN NOT NULL DEFAULT FALSE,
            priority_support BOOLEAN NOT NULL DEFAULT FALSE,
            custom_ssl BOOLEAN NOT NULL DEFAULT FALSE,
            api_access BOOLEAN NOT NULL DEFAULT TRUE,
            data_retention_days INTEGER NOT NULL DEFAULT 7,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_plans_type ON plans(plan_type);
        CREATE INDEX IF NOT EXISTS idx_plans_active ON plans(is_active);
        CREATE INDEX IF NOT EXISTS idx_plans_stripe_product ON plans(stripe_product_id);
        "#,
    )
    .execute(pool)
    .await?;

    // Add billing_period to subscriptions table
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'subscriptions' AND column_name = 'billing_period'
            ) THEN
                ALTER TABLE subscriptions ADD COLUMN billing_period billing_period NOT NULL DEFAULT 'monthly';
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Add plan_type to subscriptions table
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'subscriptions' AND column_name = 'plan_type'
            ) THEN
                ALTER TABLE subscriptions ADD COLUMN plan_type plan_type NOT NULL DEFAULT 'free';
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Add stripe_payment_method_id to subscriptions table
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'subscriptions' AND column_name = 'stripe_payment_method_id'
            ) THEN
                ALTER TABLE subscriptions ADD COLUMN stripe_payment_method_id VARCHAR(255);
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Add cancellation_reason to subscriptions table
    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM information_schema.columns
                WHERE table_name = 'subscriptions' AND column_name = 'cancellation_reason'
            ) THEN
                ALTER TABLE subscriptions ADD COLUMN cancellation_reason TEXT;
            END IF;
        END $$;
        "#,
    )
    .execute(pool)
    .await?;

    // Add index on stripe_subscription_id
    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_subscriptions_stripe_sub ON subscriptions(stripe_subscription_id);
        "#,
    )
    .execute(pool)
    .await?;

    // Create invoices table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS invoices (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            subscription_id VARCHAR(36) NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
            stripe_invoice_id VARCHAR(255) UNIQUE,
            stripe_payment_intent_id VARCHAR(255),
            number VARCHAR(100),
            status invoice_status NOT NULL DEFAULT 'draft',
            currency VARCHAR(3) NOT NULL DEFAULT 'usd',
            subtotal_cents BIGINT NOT NULL DEFAULT 0,
            tax_cents BIGINT NOT NULL DEFAULT 0,
            total_cents BIGINT NOT NULL DEFAULT 0,
            amount_paid_cents BIGINT NOT NULL DEFAULT 0,
            amount_due_cents BIGINT NOT NULL DEFAULT 0,
            description TEXT,
            invoice_pdf_url TEXT,
            hosted_invoice_url TEXT,
            period_start TIMESTAMPTZ NOT NULL,
            period_end TIMESTAMPTZ NOT NULL,
            due_date TIMESTAMPTZ,
            paid_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_invoices_org ON invoices(organization_id);
        CREATE INDEX IF NOT EXISTS idx_invoices_subscription ON invoices(subscription_id);
        CREATE INDEX IF NOT EXISTS idx_invoices_stripe ON invoices(stripe_invoice_id);
        CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
        CREATE INDEX IF NOT EXISTS idx_invoices_created ON invoices(created_at);
        "#,
    )
    .execute(pool)
    .await?;

    // Create usage_records table for detailed usage tracking
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS usage_records (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            subscription_id VARCHAR(36) NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
            metric_type usage_metric_type NOT NULL,
            quantity BIGINT NOT NULL,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            stripe_usage_record_id VARCHAR(255),
            idempotency_key VARCHAR(255),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_usage_records_org ON usage_records(organization_id);
        CREATE INDEX IF NOT EXISTS idx_usage_records_subscription ON usage_records(subscription_id);
        CREATE INDEX IF NOT EXISTS idx_usage_records_timestamp ON usage_records(timestamp);
        CREATE INDEX IF NOT EXISTS idx_usage_records_metric ON usage_records(metric_type);
        CREATE INDEX IF NOT EXISTS idx_usage_records_idempotency ON usage_records(idempotency_key);
        "#,
    )
    .execute(pool)
    .await?;

    // Create usage_summaries table for monthly aggregates
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS usage_summaries (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            subscription_id VARCHAR(36) NOT NULL REFERENCES subscriptions(id) ON DELETE CASCADE,
            period_start TIMESTAMPTZ NOT NULL,
            period_end TIMESTAMPTZ NOT NULL,
            total_requests BIGINT NOT NULL DEFAULT 0,
            total_bandwidth_bytes BIGINT NOT NULL DEFAULT 0,
            total_blocked_requests BIGINT NOT NULL DEFAULT 0,
            total_challenges_served BIGINT NOT NULL DEFAULT 0,
            overage_requests BIGINT NOT NULL DEFAULT 0,
            overage_bandwidth_bytes BIGINT NOT NULL DEFAULT 0,
            overage_charges_cents BIGINT NOT NULL DEFAULT 0,
            reported_to_stripe BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE(organization_id, period_start, period_end)
        );
        CREATE INDEX IF NOT EXISTS idx_usage_summaries_org ON usage_summaries(organization_id);
        CREATE INDEX IF NOT EXISTS idx_usage_summaries_period ON usage_summaries(period_start, period_end);
        "#,
    )
    .execute(pool)
    .await?;

    // Create payment_methods table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS payment_methods (
            id VARCHAR(36) PRIMARY KEY,
            organization_id VARCHAR(36) NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
            stripe_payment_method_id VARCHAR(255) NOT NULL UNIQUE,
            payment_type VARCHAR(50) NOT NULL,
            card_brand VARCHAR(50),
            card_last4 VARCHAR(4),
            card_exp_month INTEGER,
            card_exp_year INTEGER,
            is_default BOOLEAN NOT NULL DEFAULT FALSE,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_payment_methods_org ON payment_methods(organization_id);
        CREATE INDEX IF NOT EXISTS idx_payment_methods_stripe ON payment_methods(stripe_payment_method_id);
        "#,
    )
    .execute(pool)
    .await?;

    // Create billing_events table for webhook event tracking
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS billing_events (
            id VARCHAR(36) PRIMARY KEY,
            stripe_event_id VARCHAR(255) NOT NULL UNIQUE,
            event_type VARCHAR(100) NOT NULL,
            organization_id VARCHAR(36) REFERENCES organizations(id) ON DELETE SET NULL,
            subscription_id VARCHAR(36) REFERENCES subscriptions(id) ON DELETE SET NULL,
            invoice_id VARCHAR(36) REFERENCES invoices(id) ON DELETE SET NULL,
            payload JSONB NOT NULL,
            processed BOOLEAN NOT NULL DEFAULT FALSE,
            processed_at TIMESTAMPTZ,
            error_message TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
        CREATE INDEX IF NOT EXISTS idx_billing_events_stripe ON billing_events(stripe_event_id);
        CREATE INDEX IF NOT EXISTS idx_billing_events_type ON billing_events(event_type);
        CREATE INDEX IF NOT EXISTS idx_billing_events_processed ON billing_events(processed);
        CREATE INDEX IF NOT EXISTS idx_billing_events_created ON billing_events(created_at);
        "#,
    )
    .execute(pool)
    .await?;

    // Insert default plans if they don't exist
    sqlx::query(
        r#"
        INSERT INTO plans (id, name, plan_type, description, price_monthly_cents, price_yearly_cents,
            max_backends, max_origins_per_backend, max_domains, max_filter_rules,
            max_bandwidth_bytes, max_requests, advanced_protection, priority_support,
            custom_ssl, api_access, data_retention_days, is_active)
        VALUES
            ('plan_free', 'Free', 'free', 'Get started with basic DDoS protection',
             0, 0, 1, 1, 1, 5, 1073741824, 100000, false, false, false, false, 1, true),
            ('plan_starter', 'Starter', 'starter', 'For small websites and applications',
             2900, 29000, 3, 2, 5, 20, 10737418240, 1000000, false, false, false, true, 7, true),
            ('plan_pro', 'Pro', 'pro', 'For growing businesses',
             9900, 99000, 10, 5, 20, 100, 107374182400, 10000000, true, false, true, true, 30, true),
            ('plan_enterprise', 'Enterprise', 'enterprise', 'For large organizations with custom needs',
             29900, 299000, 100, 20, 100, 1000, 1099511627776, 100000000, true, true, true, true, 365, true)
        ON CONFLICT (id) DO UPDATE SET
            name = EXCLUDED.name,
            description = EXCLUDED.description,
            price_monthly_cents = EXCLUDED.price_monthly_cents,
            price_yearly_cents = EXCLUDED.price_yearly_cents,
            updated_at = NOW();
        "#,
    )
    .execute(pool)
    .await?;

    info!("Billing migrations completed");
    Ok(())
}
