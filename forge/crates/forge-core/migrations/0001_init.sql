-- Phase 0 schema baseline. Phase 1 will extend with vulnerability and SBOM tables.
CREATE TABLE IF NOT EXISTS builds (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL,
    runtime      TEXT NOT NULL,
    base_image   TEXT NOT NULL,
    status       TEXT NOT NULL,
    spec_json    TEXT NOT NULL,
    created_at   TEXT NOT NULL,
    started_at   TEXT,
    finished_at  TEXT,
    log_path     TEXT
);

CREATE INDEX IF NOT EXISTS idx_builds_status     ON builds(status);
CREATE INDEX IF NOT EXISTS idx_builds_created_at ON builds(created_at);

CREATE TABLE IF NOT EXISTS audit_log (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    actor      TEXT NOT NULL,
    action     TEXT NOT NULL,
    target     TEXT,
    details    TEXT,
    created_at TEXT NOT NULL
);
