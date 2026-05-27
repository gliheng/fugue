CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE apps (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        TEXT NOT NULL,
    slug        TEXT UNIQUE NOT NULL,
    subdomain   TEXT UNIQUE NOT NULL,
    framework   TEXT NOT NULL CHECK (framework IN ('worker', 'nuxtjs', 'react-router')),
    status      TEXT NOT NULL DEFAULT 'created'
                CHECK (status IN ('created', 'building', 'deploying', 'running', 'stopped', 'error')),
    description TEXT,
    env_vars    JSONB NOT NULL DEFAULT '{}',
    source_path TEXT,
    build_path  TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_apps_slug ON apps(slug);
CREATE INDEX idx_apps_subdomain ON apps(subdomain);
CREATE INDEX idx_apps_status ON apps(status);

CREATE TABLE builds (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id      UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'running', 'success', 'failed')),
    log         TEXT,
    error       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ
);

CREATE INDEX idx_builds_app_id ON builds(app_id);
CREATE INDEX idx_builds_status ON builds(status);

CREATE TABLE deployments (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id      UUID NOT NULL REFERENCES apps(id) ON DELETE CASCADE,
    build_id    UUID NOT NULL REFERENCES builds(id),
    version     INTEGER NOT NULL DEFAULT 1,
    status      TEXT NOT NULL DEFAULT 'starting'
                CHECK (status IN ('starting', 'running', 'stopped')),
    started_at  TIMESTAMPTZ,
    stopped_at  TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deployments_app_id ON deployments(app_id);

CREATE TABLE ai_generations (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_id      UUID REFERENCES apps(id) ON DELETE SET NULL,
    prompt      TEXT NOT NULL,
    framework   TEXT NOT NULL,
    result_code TEXT,
    status      TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending', 'generating', 'success', 'failed')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
