CREATE TABLE IF NOT EXISTS artifacts (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    build_id     TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    digest       TEXT NOT NULL,
    registry_ref TEXT,
    bytes        INTEGER NOT NULL DEFAULT 0,
    architecture TEXT NOT NULL,
    UNIQUE(build_id, digest, architecture)
);

CREATE TABLE IF NOT EXISTS scans (
    build_id    TEXT PRIMARY KEY REFERENCES builds(id) ON DELETE CASCADE,
    scanner     TEXT NOT NULL,
    scanned_at  TEXT NOT NULL,
    findings    TEXT NOT NULL  -- JSON array of Vulnerability
);

CREATE TABLE IF NOT EXISTS sboms (
    build_id    TEXT PRIMARY KEY REFERENCES builds(id) ON DELETE CASCADE,
    format      TEXT NOT NULL,
    document    TEXT NOT NULL  -- raw JSON
);
