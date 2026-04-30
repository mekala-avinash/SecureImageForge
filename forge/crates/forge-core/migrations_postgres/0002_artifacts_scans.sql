CREATE TABLE IF NOT EXISTS artifacts (
    id           BIGSERIAL PRIMARY KEY,
    build_id     TEXT NOT NULL REFERENCES builds(id) ON DELETE CASCADE,
    digest       TEXT NOT NULL,
    registry_ref TEXT,
    bytes        BIGINT NOT NULL DEFAULT 0,
    architecture TEXT NOT NULL,
    UNIQUE(build_id, digest, architecture)
);

CREATE TABLE IF NOT EXISTS scans (
    build_id    TEXT PRIMARY KEY REFERENCES builds(id) ON DELETE CASCADE,
    scanner     TEXT NOT NULL,
    scanned_at  TIMESTAMPTZ NOT NULL,
    findings    JSONB NOT NULL
);

CREATE TABLE IF NOT EXISTS sboms (
    build_id    TEXT PRIMARY KEY REFERENCES builds(id) ON DELETE CASCADE,
    format      TEXT NOT NULL,
    document    JSONB NOT NULL
);
