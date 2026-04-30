CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    endpoint_url  TEXT NOT NULL,
    event_kind    TEXT NOT NULL,
    payload       TEXT NOT NULL,        -- JSON event body
    secret        TEXT NOT NULL,        -- HMAC secret captured at enqueue time
    attempts      INTEGER NOT NULL DEFAULT 0,
    next_attempt  TEXT NOT NULL,
    last_error    TEXT,
    delivered_at  TEXT
);

CREATE INDEX IF NOT EXISTS idx_webhook_pending
    ON webhook_deliveries(delivered_at, next_attempt);
