CREATE TABLE IF NOT EXISTS webhook_deliveries (
    id            BIGSERIAL PRIMARY KEY,
    endpoint_url  TEXT NOT NULL,
    event_kind    TEXT NOT NULL,
    payload       TEXT NOT NULL,
    secret        TEXT NOT NULL,
    attempts      BIGINT NOT NULL DEFAULT 0,
    next_attempt  TIMESTAMPTZ NOT NULL,
    last_error    TEXT,
    delivered_at  TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_webhook_pending
    ON webhook_deliveries(delivered_at, next_attempt);
