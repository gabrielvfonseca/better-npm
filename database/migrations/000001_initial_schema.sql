-- Migration: 000001_initial_schema
-- Description: Initial database schema for Dash registry

BEGIN;

CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(64) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(128),
    avatar_url TEXT,
    roles TEXT[] DEFAULT '{user}',
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS packages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(214) UNIQUE NOT NULL,
    scope VARCHAR(64),
    description TEXT,
    homepage TEXT,
    repository TEXT,
    license VARCHAR(64),
    author JSONB,
    readme TEXT,
    is_deprecated BOOLEAN DEFAULT FALSE,
    deprecation_message TEXT,
    download_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS package_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID NOT NULL REFERENCES packages(id) ON DELETE CASCADE,
    version VARCHAR(64) NOT NULL,
    description TEXT,
    main TEXT,
    types TEXT,
    exports JSONB,
    files TEXT[],
    scripts JSONB,
    dependencies JSONB DEFAULT '{}',
    dev_dependencies JSONB DEFAULT '{}',
    peer_dependencies JSONB DEFAULT '{}',
    optional_dependencies JSONB DEFAULT '{}',
    dist JSONB NOT NULL,
    has_install_script BOOLEAN DEFAULT FALSE,
    deprecated BOOLEAN DEFAULT FALSE,
    deprecation_reason TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(package_id, version)
);

CREATE TABLE IF NOT EXISTS downloads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    package_id UUID NOT NULL REFERENCES packages(id),
    version_id UUID REFERENCES package_versions(id),
    ip_address INET,
    user_agent TEXT,
    referer TEXT,
    country_code CHAR(2),
    downloaded_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID REFERENCES users(id),
    action VARCHAR(64) NOT NULL,
    resource_type VARCHAR(64),
    resource_id UUID,
    details JSONB,
    ip_address INET,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    scopes TEXT[],
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(512) NOT NULL,
    refresh_token VARCHAR(512),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_packages_name ON packages(name);
CREATE INDEX IF NOT EXISTS idx_packages_scope ON packages(scope);
CREATE INDEX IF NOT EXISTS idx_package_versions_package_id ON package_versions(package_id);
CREATE INDEX IF NOT EXISTS idx_downloads_package_id ON downloads(package_id);
CREATE INDEX IF NOT EXISTS idx_downloads_downloaded_at ON downloads(downloaded_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_action ON audit_log(action);
CREATE INDEX IF NOT EXISTS idx_audit_log_created_at ON audit_log(created_at);
CREATE INDEX IF NOT EXISTS idx_tokens_user_id ON tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER IF NOT EXISTS update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER IF NOT EXISTS update_packages_updated_at
    BEFORE UPDATE ON packages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMIT;
