# acme/gitops — Argo CD Source of Truth

This repository is the **single source of truth** for everything Argo CD reconciles into every cluster. Nothing is deployed by hand. Nothing is deployed from any other repo.

## Layout

```
gitops/
├── apps/              # Per-application manifests (or Helm chart refs)
├── platform/          # Platform components (Argo, Vault, Harbor, Backstage, Kyverno, observability)
├── clusters/          # Per-cluster App-of-Apps entrypoints
└── envs/              # Environment-specific value overrides (dev, staging, prod-<region>)
```

## Conventions

- **No imperative changes** — everything is declarative YAML.
- **Helm-first**, with Kustomize overlays only where necessary.
- **No secrets in this repo** — use `ExternalSecret` / `SecretProviderClass` (Vault CSI).
- **Immutable image digests** in production overlays (not tags).
- **CODEOWNERS** enforces ≥ 2 reviewers; security-sensitive paths require AppSec review.

## How to add an app

1. Open a PR adding `apps/<app-name>/base/` (Helm values or Kustomize base).
2. Add overlays in `apps/<app-name>/overlays/<env>/` as needed.
3. Add an `ApplicationSet` (or Application) in `clusters/<cluster>/`.
4. Wait for required checks: lint, kustomize-build, helm-template, kubeconform, policy.
5. Get 2 reviews including a CODEOWNER.
6. Merge → Argo CD reconciles automatically.

## Bootstrap

The cluster bootstrap is `app-of-apps`. Apply the root App once per cluster:

```bash
kubectl apply -n argocd -f platform/argocd/root-app.yaml
```

From there, Argo CD discovers and applies everything else in this repo.

## CI checks

- `helm template` + `kustomize build` succeed.
- `kubeconform` passes against pinned K8s schema.
- `conftest test` against the OPA policy bundle.
- `kyverno test` against the Kyverno policy bundle.
- `yamllint` clean.

See `.github/workflows/policy-and-build.yml`.

## Notice

- Force-push is disabled.
- `main` requires signed commits + 2 reviews + status checks green.
- The Argo CD bot identity has write access to `main` *only* via automation PRs (image-bump pattern).
