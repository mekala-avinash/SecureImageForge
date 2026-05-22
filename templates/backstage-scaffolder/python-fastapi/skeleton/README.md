# {{service_name}}

{{description}}

Paved-road FastAPI service. Owned by **{{team}}**.

## Quickstart

```bash
make bootstrap
make dev        # docker compose: postgres + redis + otel + jaeger
make run        # uvicorn auto-reload on :8080
make test
```

## Architecture

- **Runtime**: Python 3.12 on hardened Wolfi base (`/app/images/runtimes/python/Dockerfile`).
- **Framework**: FastAPI + Uvicorn.
- **Persistence**: Postgres (managed) — via SQLAlchemy + alembic.
- **Cache**: Redis.
- **Observability**: OTel auto-instrumentation (`OTEL_*` env vars set by Helm library).
- **Deploy**: Helm chart imports `acme-platform-lib`; GitOps via Argo CD.
- **Build/sign**: Reusable workflow in `.github/workflows/build.yml`.

## Endpoints

- `GET /healthz` — liveness.
- `GET /ready`   — readiness (DB + Redis reachable).
- `GET /metrics` — Prometheus metrics.
- `GET /api/v1/items` — example.

## Operations

- Dashboard: https://grafana.acme.io/d/svc-{{service_name}}
- Runbooks:  `docs/runbooks/`
- Backstage: https://backstage.acme.io/catalog/default/component/{{service_name}}
- On-call:   {{team}} (rotation in PagerDuty)
