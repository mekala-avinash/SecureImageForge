# Enterprise Integration Contracts

All inbound/outbound integrations follow a single hardened pattern.

## Outbound Webhook Standard

```http
POST /events HTTP/1.1
Host: partner.example.com
Content-Type: application/cloudevents+json
X-Acme-Event-Id: 01HX...ULID
X-Acme-Signature: t=1737000000,v1=8f29...hex
X-Acme-Idempotency-Key: 01HX...ULID
User-Agent: acme-webhooks/2.0

{ "specversion":"1.0", "type":"com.acme.deployment.promoted.v1", ... }
```

- `X-Acme-Signature` = HMAC-SHA-256 over `t.body` with current rotation key.
- Two keys live in parallel for 24h (rotation); receiver tries both.
- Replay window: ±5 min on `t`.
- Idempotency key cached at receiver for 24h.

## Retry Policy

```
attempts: 12
backoff:  exponential (base 2s, cap 5m, full jitter)
DLQ:      after final attempt → S3 acme-webhook-dlq/ + Jira ticket
```

## Inbound Webhook Validation Steps

1. TLS terminate at API Gateway (mTLS optional).
2. Source IP allowlist (per integration).
3. HMAC signature verify.
4. Replay window check.
5. Idempotency cache check.
6. Schema validation (CloudEvents + AsyncAPI).
7. Enqueue to NATS subject `incoming.<integration>.<event-type>`.

## Per-System Integration

### Jira
- Direction: bi.
- Auth: OAuth2 (3LO), token stored in Vault path `kv/integrations/jira/<env>`.
- Webhook: signed JWT via Atlassian Connect; verified against ACS public key.
- Mappings: agent decision → Issue (severity → Priority).
- Rate limit: 100 r/s/tenant; 429 → exponential retry.

### Slack
- Bot scopes: `chat:write`, `commands`, `incoming-webhook`, `users:read`.
- Block Kit for interactive approvals (agent action buttons).
- Signature: `X-Slack-Signature` v0 scheme + timestamp.

### Microsoft Teams
- App registration with `RSC` permissions (least privilege).
- Adaptive Cards for approvals.
- Webhook signature: HMAC + Graph subscription validation.

### GitHub Enterprise
- GitHub App, fine-grained per-repo scopes.
- Webhook secret rotated 90d.
- Required events: pull_request, push, check_run, workflow_run, secret_scanning_alert.

### GitLab
- Group access token, scopes: api, read_repository, write_repository.
- Project webhooks with secret token; SHA256 HMAC.

### ServiceNow
- OAuth2 + scoped app (Change, Incident, CMDB).
- Use Import Sets for high-volume; Table API for low-volume.
- CR records auto-created on HIGH/CRITICAL blast-radius changes.

### PagerDuty
- Events API v2 with routing keys (one per service).
- Webhook v3 for incident lifecycle events.
- Bi-di: agents create incidents; PagerDuty notifies Incident Analysis Agent.

### Datadog (optional dual-write)
- API key in Vault; rotation 90d.
- Send metrics & logs only (not traces; OTel is canonical).

### Splunk
- HEC token in Vault; HTTPS only; index per data class.
- 7y retention for security/compliance index.

## Schemas

All event payloads conform to CloudEvents 1.0 + AsyncAPI 3.0 contracts maintained in the **Buf Schema Registry** at `buf.acme.io/events`.
