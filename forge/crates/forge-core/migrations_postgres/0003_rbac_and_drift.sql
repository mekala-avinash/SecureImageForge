CREATE TABLE IF NOT EXISTS principals (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL UNIQUE,
    role         TEXT NOT NULL,
    token_hash   TEXT NOT NULL UNIQUE,
    created_at   TIMESTAMPTZ NOT NULL,
    revoked_at   TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_principals_token_hash ON principals(token_hash);

CREATE TABLE IF NOT EXISTS audit_events (
    id          BIGSERIAL PRIMARY KEY,
    actor       TEXT NOT NULL,
    action      TEXT NOT NULL,
    target      TEXT,
    outcome     TEXT NOT NULL,
    details     TEXT,
    created_at  TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_events_actor      ON audit_events(actor);
CREATE INDEX IF NOT EXISTS idx_audit_events_created_at ON audit_events(created_at);

CREATE TABLE IF NOT EXISTS drift_snapshots (
    id              BIGSERIAL PRIMARY KEY,
    build_id        TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    scanned_at      TIMESTAMPTZ NOT NULL,
    scanner         TEXT NOT NULL,
    findings        JSONB NOT NULL,
    new_critical    BIGINT NOT NULL DEFAULT 0,
    new_high        BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_drift_snapshots_build  ON drift_snapshots(build_id);

CREATE TABLE IF NOT EXISTS provenance (
    build_id     TEXT PRIMARY KEY REFERENCES builds(id) ON DELETE CASCADE,
    predicate    JSONB NOT NULL,
    attested_at  TIMESTAMPTZ NOT NULL,
    bundle_path  TEXT
);
