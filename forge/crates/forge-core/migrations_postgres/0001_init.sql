CREATE TABLE IF NOT EXISTS builds (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,
    runtime      TEXT NOT NULL,
    base_image   TEXT NOT NULL,
    status       TEXT NOT NULL,
    spec_json    TEXT NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL,
    started_at   TIMESTAMPTZ,
    finished_at  TIMESTAMPTZ,
    log_path     TEXT
);

CREATE INDEX IF NOT EXISTS idx_builds_status     ON builds(status);
CREATE INDEX IF NOT EXISTS idx_builds_created_at ON builds(created_at);

CREATE TABLE IF NOT EXISTS audit_log_legacy (
    id         BIGSERIAL PRIMARY KEY,
    actor      TEXT NOT NULL,
    action     TEXT NOT NULL,
    target     TEXT,
    details    TEXT,
    created_at TIMESTAMPTZ NOT NULL
);
