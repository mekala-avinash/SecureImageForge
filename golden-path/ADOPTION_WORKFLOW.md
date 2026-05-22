# Paved-Road Adoption Workflow

> How an existing team migrates an existing service onto the paved road.
> Target migration time: **1–3 days** depending on service complexity.

## Phase A — Day 0 (≤ 30 min)

```bash
# 1. Install the pavedroad CLI (one-time, per dev machine)
brew install acme/tap/pavedroad      # or: pipx install pavedroad-cli

# 2. Run the readiness check on the existing repo
cd ~/code/orders-api
pavedroad doctor

# Output: checklist of what's compliant + what needs fixing.
```

`doctor` evaluates:
- Service binds to `0.0.0.0:<port>` (configurable).
- Service does not require root.
- Service writes only to `/tmp` or named volumes.
- Service exposes `/healthz` + `/ready` endpoints (or can be added).
- Service has a passing test suite + CI.

Anything red → fix in a separate PR first.

## Phase B — Day 1 (~ 4–8 hours)

```bash
pavedroad migrate --from existing --to paved-road
```

This is an **interactive** migration that:

1. Detects language + framework (`pyproject.toml`, `package.json`, `go.mod`, …).
2. Copies the appropriate hardened `Dockerfile` from `/app/images/runtimes/<lang>/Dockerfile` with service-name substitutions.
3. Copies the reusable GitHub Actions / GitLab CI workflow from `/app/ci/` and wires `secrets: inherit`.
4. Generates a Helm chart that imports `platform/helm-library` and exposes only service-specific values.
5. Adds OTel SDK to dependencies if not present; wires auto-instrumentation env vars.
6. Adds the Backstage catalog file (`catalog-info.yaml`).
7. Opens a single PR with the migration changes; auto-assigns to the platform team for co-review.

The migration **does not** delete any of your existing code. It only adds paved-road artifacts.

## Phase C — Day 1 (afternoon) — verify in `dev`

```bash
# After PR merge:
pavedroad watch --env dev --service orders-api
# Streams Argo CD sync events, OTel traces, logs from your new image.
```

Required green-lights:
- Argo CD `Synced + Healthy`.
- `/healthz` + `/ready` returning 200.
- Traces visible in Tempo.
- Logs flowing to Loki.
- No new Kyverno admission denials.

## Phase D — Day 2 — soak in `staging`

Default soak window: **24 hours**.

Promote via:

```bash
pavedroad promote --service orders-api --to staging
# Opens a GitOps PR bumping the staging overlay's image digest.
```

Watch the dashboards (auto-generated):
- `https://grafana.acme.io/d/svc-orders-api`
- `https://backstage.acme.io/catalog/default/component/orders-api`

## Phase E — Day 3 — promote to `prod` (canary)

```bash
pavedroad promote --service orders-api --to prod
# This opens a GitOps PR that engages Argo Rollouts canary.
```

The canary runs automatically (see `ci/github-actions/reusable-deploy.yml` + `platform/helm-library/templates/_rollout.tpl`):

```
  setWeight: 5  → analysis 10m
  setWeight: 25 → analysis 15m
  setWeight: 50 → analysis 30m
  setWeight: 100
```

Analysis = Prom queries against the service's success-rate + p99 latency. Auto-abort on breach.

## Phase F — Day 3+ — cleanup

Delete the legacy deployment manifests / pipelines after **7 days** of stable canary completions:

```bash
pavedroad cleanup --service orders-api --confirm
```

## Adoption support

| Need | Channel |
|---|---|
| Stuck during `pavedroad doctor` | `#platform-pavedroad` |
| Live pair migration | Office Hours: Tue 14:00, Thu 09:00 |
| Cannot use hardened image | File waiver via `/app/docs/phase-1/01-hardened-runtimes/waiver-process.md` |
| Migration broke something | Roll back: `pavedroad rollback --service <name>` (uses Argo CD history) |

## Adoption KPIs (per team)

After adoption, you can expect:
- Lead time for change: **5 days → < 1 hour**.
- Deploy frequency: weekly → per-PR.
- MTTR: **4h → < 25 min**.
- Image CVE exposure: ↓ ~95%.
- Audit-evidence work per quarter: **~0** (it collects itself).
