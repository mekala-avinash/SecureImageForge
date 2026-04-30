CREATE TABLE IF NOT EXISTS principals (
    id           TEXT PRIMARY KEY NOT NULL,
    name         TEXT NOT NULL UNIQUE,
    role         TEXT NOT NULL,         -- 'admin' | 'operator' | 'viewer'
    token_hash   TEXT NOT NULL UNIQUE,  -- sha256 of bearer token
    created_at   TEXT NOT NULL,
    revoked_at   TEXT
);

CREATE INDEX IF NOT EXISTS idx_principals_token_hash ON principals(token_hash);

-- Append-only audit trail. The orchestrator and the API layer both write here.
CREATE TABLE IF NOT EXISTS audit_events (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    actor       TEXT NOT NULL,
    action      TEXT NOT NULL,
    target      TEXT,
    outcome     TEXT NOT NULL,         -- 'success' | 'denied' | 'error'
    details     TEXT,                  -- JSON envelope
    created_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audit_events_actor      ON audit_events(actor);
CREATE INDEX IF NOT EXISTS idx_audit_events_created_at ON audit_events(created_at);

-- Drift snapshots: re-scans of previously-built images expose new CVEs.
CREATE TABLE IF NOT EXISTS drift_snapshots (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    build_id        TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    scanned_at      TEXT NOT NULL,
    scanner         TEXT NOT NULL,
    findings        TEXT NOT NULL,
    new_critical    INTEGER NOT NULL DEFAULT 0,
    new_high        INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_drift_snapshots_build  ON drift_snapshots(build_id);

-- in-toto SLSA provenance records, one per build.
CREATE TABLE IF NOT EXISTS provenance (
    build_id     TEXT PRIMARY KEY REFERENCES builds(id) ON DELETE CASCADE,
    predicate    TEXT NOT NULL,         -- in-toto statement (JSON)
    attested_at  TEXT NOT NULL,
    bundle_path  TEXT
);
