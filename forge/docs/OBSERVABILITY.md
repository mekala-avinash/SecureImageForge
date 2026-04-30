# Observability & integrations

## Prometheus metrics

`forge serve` installs a global `metrics-exporter-prometheus` recorder and
exposes the scrape endpoint at `GET /metrics`. The endpoint is open (no
bearer token required) so a sidecar Prometheus scraper can pull from
localhost or a private metrics network without provisioning a token; lock it
down with a reverse proxy if you bind to a public interface.

Stable series so far:

| Metric | Labels | Type | Source |
|---|---|---|---|
| `forge_api_requests_total` | `method`, `route`, `status` | counter | axum middleware |
| `forge_api_request_duration_seconds` | `method`, `route`, `status` | histogram | axum middleware |
| `forge_builds_started_total` | `runtime` | counter | orchestrator |
| `forge_builds_succeeded_total` | `runtime` | counter | orchestrator |
| `forge_builds_failed_total` | `runtime`, `reason` | counter | orchestrator |
| `forge_policy_denied_total` | `profile` | counter | OPA gate |
| `forge_scan_duration_seconds` | `scanner` | histogram | scan adapters |
| `forge_drift_new_critical_total` | — | counter | drift detector |
| `forge_drift_new_high_total` | — | counter | drift detector |

The series names live in [`forge_core::metrics`](../crates/forge-core/src/metrics.rs)
so callers don't hand-write strings.

## OpenTelemetry tracing

[`telemetry::init_with_endpoint`](../crates/forge-core/src/telemetry.rs) takes
the `[telemetry] otlp_endpoint` from config (or `FORGE_OTLP_ENDPOINT` env).

- Build without features: the endpoint is only logged for diagnostics; spans
  go to stdout via tracing-subscriber.
- Build with `--features otlp` (Phase 6.5): the OTLP HTTP/protobuf exporter
  is installed and a `tracing-opentelemetry` layer attaches to every
  in-process span, so traces flow to your collector alongside stdout.

```bash
cargo build -p forge-cli --features forge-core/otlp --release
FORGE_OTLP_ENDPOINT=http://otel-collector:4318/v1/traces forge serve
```

`service.name` defaults to `forge-api`; override with `[telemetry] service_name`.

## SSE log streaming

`GET /v1/builds/{id}/log/stream` returns Server-Sent Events. Each new chunk
of the persisted log file is sent as a `data:` event; once the file stops
growing for ~60 seconds the daemon emits an `event: eof` and closes the
stream. Consumers can reconnect to resume reading.

Curl example:

```bash
curl -N -H "authorization: Bearer $TOKEN" \
  http://127.0.0.1:7878/v1/builds/$ID/log/stream
```

## Outbound webhooks

Configure under `[webhooks]`:

```toml
[[webhooks.endpoints]]
url     = "https://hooks.example.com/forge"
secret  = "shared-hmac-secret"
events  = ["build.succeeded", "policy.deny", "drift.alert"]   # empty = all
```

The dispatcher signs each delivery body with HMAC-SHA256 of the JSON
payload and ships:

```
POST /forge HTTP/1.1
content-type: application/json
x-forge-event: build.succeeded
x-forge-signature: sha256=<hex>
```

Verify on the receiver with [`forge_core::webhooks::verify`](../crates/forge-core/src/webhooks.rs)
or any HMAC-SHA256 implementation.

### Persistent retry queue (Phase 6.5)

When delivery fails (5xx, 4xx, transport error) the event is kept in the
`webhook_deliveries` table with an exponential `next_attempt`. A background
worker (started by `forge serve`) polls due rows on a 5-second tick:

| Attempt | Backoff |
|---|---|
| 1 | 1 s |
| 2 | 2 s |
| 3 | 4 s |
| 4 | 8 s |
| … | doubling, capped at 30 minutes |

Successful deliveries are marked `delivered_at`. The worker prunes entries
older than 7 days once an hour. Inspect the queue with:

```sql
SELECT id, endpoint_url, event_kind, attempts, next_attempt, last_error
  FROM webhook_deliveries
 WHERE delivered_at IS NULL
 ORDER BY next_attempt;
```

Event payload shape:

```json
{
  "kind": "build.succeeded",
  "occurred_at": "2026-04-30T12:00:00Z",
  "subject": "fa3c3e3a-…",
  "payload": { "runtime": "go", "duration_seconds": 42, "policy": "allow" }
}
```

## Storage backends

`[storage].database_url` (or `DATABASE_URL` / `FORGE_DATABASE_URL`) selects
the backend:

- `sqlite://<data_dir>/forge.sqlite` (default) — single-user / desktop
- `file:./forge.sqlite` — same, local working directory
- `postgres://user:pass@host/db` — Phase 6.5 ships a Postgres connection
  pool, the `migrations_postgres/` schema, and a read-mirror
  [`PgBuildRepo`](../crates/forge-core/src/pg_storage.rs) for builds. Build
  with `--features forge-core/pg` to enable. Full write-path parity for
  scans/sboms/drift/provenance lands in Phase 7 alongside per-table Any-
  driver adapters.

`Backend::detect` is the single source of truth — see
[`forge_core::storage::Backend`](../crates/forge-core/src/storage.rs).

## forge-sdk: programmatic API client

The [`forge-sdk`](../crates/forge-sdk/src/lib.rs) crate ships a typed Rust
client suitable for CI tooling, internal scripts, or custom dashboards.

```rust
use forge_sdk::{Client, CreateBuildRequest};

#[tokio::main]
async fn main() -> forge_sdk::Result<()> {
    let client = Client::new("http://forge.internal:7878", std::env::var("FORGE_TOKEN")?)?;
    let created = client.create_build(&CreateBuildRequest {
        name: "checkout-api".into(),
        runtime: "java".into(),
        base: "distroless".into(),
        compliance: vec!["cis".into(), "soc2".into()],
        architectures: vec!["amd64".into(), "arm64".into()],
        no_sbom: false,
        no_sign: false,
    }).await?;
    client.start_build(&created.id).await?;
    Ok(())
}
```

The crate has no transitive dependency on `forge-core` so downstream
consumers don't pull in sqlx / dioxus / tray-icon.

### Retry policy (Phase 6.5)

Every request goes through `Client::send_with_retry` and retries on
transport errors plus `5xx`, `429`, and `408` responses. The default
`RetryPolicy { max_attempts: 4, initial_delay: 200ms, max_delay: 5s }`
doubles the delay each attempt up to the cap.

```rust
use forge_sdk::{Client, RetryPolicy};
use std::time::Duration;

let client = Client::new("http://forge.internal:7878", token)?
    .with_retry(RetryPolicy {
        max_attempts: 6,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(2),
    });

// Or disable retries for a one-shot integration.
let strict = client.clone().without_retry();
```
