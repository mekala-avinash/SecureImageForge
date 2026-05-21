# Hardened Runtime Adoption Playbook

> Phase-1 Workstream 1 · Lead: Platform Lead · Window: Weeks 1–12

## Goal

Migrate the **top 10 critical services** plus all new services to hardened distroless / Wolfi runtime images by end of Phase 1, with **zero regressions** in availability or latency SLOs.

## Adoption Tiers

| Tier | Selection | Approach | Phase-1 target |
|---|---|---|---|
| **Tier A — Lighthouse (3 services)** | Pre-selected: `orders-api`, `users-svc`, `payment-svc-shadow` | Hand-walked migration by Platform Lead + service owner | Weeks 2–4 |
| **Tier B — Top 10 (7 services)** | Top 7 critical services by revenue / pageviews | Self-service with Platform Office Hours support | Weeks 5–10 |
| **Tier C — New services** | All new repos from Backstage scaffold | Hardened by default; no migration needed | Continuous |
| **Tier D — Legacy long-tail** | All remaining services | Phase-2 (post SteerCo decision) | Out of scope P1 |

## Per-Service Migration Runbook

Each service migration follows an **identical 9-step runbook** (~1–3 days per service). The Platform team has built a Backstage *technote* card that links to this runbook.

### Step 0 — Pre-flight (≤ 30 min)

```bash
backstage:open <service>                 # service in catalog
backstage:scorecard <service>            # show readiness score
git checkout -b feat/hardened-runtime
```

Readiness criteria:
- [ ] Service has a passing CI pipeline.
- [ ] Service has a `Dockerfile` at repo root.
- [ ] Service binds to `0.0.0.0:8080` (or is fixable).
- [ ] Service writes only to `/tmp` or explicit volumes (no FS writes elsewhere).
- [ ] Service does not require root.
- [ ] Service has at least a `/healthz` endpoint.

If any check fails: open a follow-up issue, fix in a separate PR, then return.

### Step 1 — Pick the runtime template

Copy from `/app/docs/runtime-images/runtimes/`:

| Language | Template |
|---|---|
| Node.js | `Dockerfile.nodejs.template` |
| Python | `Dockerfile.python.template` |
| Go | `Dockerfile.go.template` |
| Rust | `Dockerfile.rust.template` |
| Java 21 | `Dockerfile.java.template` |
| Ruby | `Dockerfile.ruby.template` |
| PHP | `Dockerfile.php.template` |
| .NET | `Dockerfile.dotnet.template` |

Customize `ARG SERVICE_NAME` and any service-specific build steps. **Do not** add a package manager or shell to the runtime stage.

### Step 2 — Local build & smoke test

```bash
SOURCE_DATE_EPOCH=$(git log -1 --pretty=%ct) \
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  --build-arg SERVICE_NAME=$(basename $PWD) \
  --build-arg VERSION=$(git describe --tags --always) \
  -t localhost:5000/myservice:dev \
  --output type=oci,rewrite-timestamp=true,dest=/tmp/img.tar .

docker run --rm --read-only --tmpfs /tmp -p 8080:8080 localhost:5000/myservice:dev
curl -fsS http://localhost:8080/healthz
```

### Step 3 — Wire CI to the reusable workflow

```yaml
# .github/workflows/build.yml
jobs:
  build:
    uses: acme/gitops/.github/workflows/_reusable-build.yml@main
    with:
      service-name: my-service
      runtime: python   # node, go, rust, java, ruby, php, dotnet
    secrets: inherit
```

This automatically applies: BuildKit hermetic build → Trivy + Grype scan → Syft SBOM → Cosign sign (Fulcio keyless) → SLSA L3 provenance → push.

### Step 4 — Update Helm values to hardened defaults

```yaml
# helm/values.yaml additions
image:
  digest: ""              # set by GitOps bump PR
  pullPolicy: IfNotPresent
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 65532
  fsGroup: 65532
  seccompProfile: { type: RuntimeDefault }
containerSecurityContext:
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities: { drop: ["ALL"] }
volumes:
  - { name: tmp, emptyDir: { sizeLimit: 64Mi, medium: Memory } }
volumeMounts:
  - { name: tmp, mountPath: /tmp }
```

### Step 5 — Wire SBOM + Cosign verify into Kyverno (nonprod)

The cluster-wide policy at `/app/docs/security/admission/kyverno-verify-images.yaml` already enforces this in nonprod from Week 1. No service-level work.

### Step 6 — Deploy to `dev` via GitOps

```bash
# In acme/gitops:
yq -i '.spec.values.image.digest = "sha256:<digest>"' apps/<svc>/overlays/dev/values.yaml
git commit -S -am "feat(<svc>): adopt hardened runtime"
gh pr create ...
```

Argo CD reconciles automatically after merge.

### Step 7 — Validate in `dev` (≥ 24h soak)

Required checks:
- [ ] All readiness/liveness probes green.
- [ ] OTel traces visible in Tempo.
- [ ] No new errors in Loki vs baseline.
- [ ] p99 latency within ±5% of baseline.
- [ ] Kyverno admission did not block (or blocked for the right reason).

### Step 8 — Promote to `staging` with progressive delivery

GitOps PR for `overlays/staging/values.yaml`. Argo Rollouts (when enabled in Phase 2) runs canary. In Phase 1 this is a standard rolling update with PDB.

### Step 9 — Document & retro

- Update Backstage TechDocs with image digest, SBOM link, SLSA provenance link.
- Capture migration time in the adoption tracker spreadsheet.
- Optional: 15-min retro with Platform Lead for first 5 migrations.

## Adoption Tracker

| # | Service | Owner | Tier | Status | PR | Soak start | Soak end | Notes |
|---|---|---|---|---|---|---|---|---|
| 1 | orders-api | <team> | A | ☐ Started ☐ Soaking ☐ Promoted | | | | |
| 2 | users-svc | <team> | A | | | | | |
| 3 | payment-svc-shadow | <team> | A | | | | | |
| 4 | (top-10 #4) | | B | | | | | |
| 5 | … | | B | | | | | |

(Live tracker in Backstage; this table is the source of truth in Git as a backup.)

## Waiver Process (when hardened image is not feasible)

See [`waiver-process.md`](./waiver-process.md).

## Support

- `#platform-runtimes` Slack channel.
- Platform Office Hours: Tuesdays 14:00, Thursdays 09:00.
- Pair migration sessions can be requested at any time.

## Phase-1 success criteria for this workstream

- ≥ 60% of new + top 10 critical services on hardened runtime images by Week 12.
- Mean image size for production runtime images ≤ 100 MB.
- Zero CVE-related admission blocks in production attributable to base-image lag (auto-rebuild + dep agent in flight from Phase 0).
- Zero in-service regressions in p99 latency or error rate attributable to the migration.
