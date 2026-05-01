-- Enterprise team-mode foundations:
--   * auth/rbac scope bindings
--   * org/project/environment tenancy
--   * durable build job queue

CREATE TABLE IF NOT EXISTS organizations (
    id          TEXT PRIMARY KEY NOT NULL,
    name        TEXT NOT NULL UNIQUE,
    created_at  TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
    id               TEXT PRIMARY KEY NOT NULL,
    organization_id  TEXT NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name             TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL,
    UNIQUE(organization_id, name)
);

CREATE TABLE IF NOT EXISTS environments (
    id          TEXT PRIMARY KEY NOT NULL,
    project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL,
    UNIQUE(project_id, name)
);

CREATE TABLE IF NOT EXISTS group_role_bindings (
    id          BIGSERIAL PRIMARY KEY,
    group_name  TEXT NOT NULL,
    role        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL,
    UNIQUE(group_name)
);

CREATE TABLE IF NOT EXISTS principal_scopes (
    id           BIGSERIAL PRIMARY KEY,
    principal_id TEXT NOT NULL,
    scope_type   TEXT NOT NULL, -- org | project | environment
    scope_id     TEXT NOT NULL,
    role         TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL,
    UNIQUE(principal_id, scope_type, scope_id)
);

CREATE INDEX IF NOT EXISTS idx_principal_scopes_lookup
    ON principal_scopes(principal_id, scope_type, scope_id);

CREATE TABLE IF NOT EXISTS build_jobs (
    id              TEXT PRIMARY KEY NOT NULL,
    build_id        TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    project_id      TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    status          TEXT NOT NULL, -- queued|leased|running|succeeded|failed|canceled|deadletter
    attempts        INTEGER NOT NULL DEFAULT 0,
    max_retries     INTEGER NOT NULL DEFAULT 3,
    leased_until    TIMESTAMPTZ,
    worker_id       TEXT,
    next_attempt_at TIMESTAMPTZ NOT NULL,
    last_error      TEXT,
    created_at      TIMESTAMPTZ NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_build_jobs_ready
    ON build_jobs(status, next_attempt_at);

CREATE TABLE IF NOT EXISTS job_attempts (
    id            BIGSERIAL PRIMARY KEY,
    job_id         TEXT NOT NULL REFERENCES build_jobs(id) ON DELETE CASCADE,
    attempt_number INTEGER NOT NULL,
    started_at     TIMESTAMPTZ NOT NULL,
    finished_at    TIMESTAMPTZ,
    outcome        TEXT, -- success|error|canceled
    error          TEXT
);

CREATE INDEX IF NOT EXISTS idx_job_attempts_job ON job_attempts(job_id);

CREATE TABLE IF NOT EXISTS job_leases (
    id           BIGSERIAL PRIMARY KEY,
    job_id        TEXT NOT NULL REFERENCES build_jobs(id) ON DELETE CASCADE,
    worker_id     TEXT NOT NULL,
    leased_at     TIMESTAMPTZ NOT NULL,
    lease_expires TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_job_leases_job ON job_leases(job_id);

ALTER TABLE builds ADD COLUMN project_id TEXT NOT NULL DEFAULT 'default-project';

INSERT INTO organizations (id, name, created_at)
VALUES ('default-org', 'Default Organization', CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;

INSERT INTO projects (id, organization_id, name, created_at)
VALUES ('default-project', 'default-org', 'Default Project', CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;

INSERT INTO environments (id, project_id, name, created_at)
VALUES ('default-env', 'default-project', 'default', CURRENT_TIMESTAMP)
ON CONFLICT DO NOTHING;
