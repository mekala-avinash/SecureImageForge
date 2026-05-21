# GitOps Repository Bootstrap

This action creates three repositories required for the entire platform program. Concrete starter files are bootstrapped under `/app/gitops-bootstrap/` and ready to push.

## Repositories to create

| Repo | Purpose | Visibility | Primary owners |
|---|---|---|---|
| `acme/gitops` | Argo CD source of truth — every cluster, every app | private | Platform team |
| `acme/platform` | Reusable Helm charts, Crossplane compositions, Terraform/IaC modules | private | Platform team |
| `acme/tenants` | Per-tenant overlays, quotas, network policies, RBAC | private | Platform team + tenant onboarding |

## Branch protection (all three)

- `main` is the only deployable branch.
- Required reviews: ≥ 2 from CODEOWNERS.
- Require signed commits (gitsign / commit signing).
- Require status checks: `lint`, `policy`, `kustomize-build`, `helm-template`, `kubeconform`.
- Linear history; no force-push.
- Require branches to be up to date before merging.
- Restrict who can push to `main` (Argo CD bot + platform admins only).

## CODEOWNERS strategy

- Root: `* @acme/platform-team`
- Security-sensitive paths (`/policy/`, `/security/`, `/admission/`): also `@acme/security-team` (≥ 2 reviewers across owners).
- Tenant-specific overlays in `tenants/`: tenant onboarding squad as co-owner.
- Compliance-evidence paths: `@acme/compliance-team`.

## Concrete starter files

See `/app/gitops-bootstrap/`. Top-level layout:

```
/app/gitops-bootstrap/
├── gitops/                       # → push to acme/gitops
│   ├── README.md
│   ├── CODEOWNERS
│   ├── .github/
│   │   ├── pull_request_template.md
│   │   └── workflows/policy-and-build.yml
│   ├── apps/                     # one folder per app (Helm chart refs / Kustomize)
│   ├── platform/                 # platform components (Argo, Vault, Harbor, Backstage, Kyverno, observability)
│   │   ├── argocd/
│   │   │   └── root-app.yaml     # App-of-Apps bootstrap
│   │   ├── kyverno/
│   │   ├── vault/
│   │   ├── harbor/
│   │   ├── backstage/
│   │   └── observability/
│   ├── clusters/                 # per-cluster bootstrap (Argo CD ApplicationSet)
│   │   └── mgmt-use1/
│   │       └── app-of-apps.yaml
│   └── envs/                     # environment-scoped overrides (dev/staging/prod-<region>)
├── platform-repo/                # → push to acme/platform
│   ├── README.md
│   ├── CODEOWNERS
│   ├── modules/                  # Terraform / OpenTofu modules
│   ├── crossplane/               # XRDs + Compositions
│   └── helm-charts/              # Reusable Helm charts (microservice, worker, job)
└── tenants-repo/                 # → push to acme/tenants
    ├── README.md
    ├── CODEOWNERS
    └── _template/                # Tenant scaffolder source
```

## Push procedure (Day 5)

```bash
# 1. Create repos in GitHub (Org admin)
gh repo create acme/gitops    --private --confirm
gh repo create acme/platform  --private --confirm
gh repo create acme/tenants   --private --confirm

# 2. Initialize from bootstrap
cd /app/gitops-bootstrap/gitops    && git init && git add . && git commit -S -m "feat: initial bootstrap" && git remote add origin git@github.com:acme/gitops.git    && git push -u origin main
cd /app/gitops-bootstrap/platform-repo && git init && git add . && git commit -S -m "feat: initial bootstrap" && git remote add origin git@github.com:acme/platform.git  && git push -u origin main
cd /app/gitops-bootstrap/tenants-repo  && git init && git add . && git commit -S -m "feat: initial bootstrap" && git remote add origin git@github.com:acme/tenants.git   && git push -u origin main

# 3. Apply branch protection + CODEOWNERS via gh + Terraform
#    (terraform module included under platform-repo/modules/github-repo-protection)
```

## Acceptance criteria (Day 5)

- ✅ All three repos exist, private, with CODEOWNERS enforced.
- ✅ Signed-commit + required-review branch protection active on `main`.
- ✅ ArgoCD root App syncs successfully (even if empty children).
- ✅ A throwaway "hello-platform" app deploys to `dev` via Argo CD end-to-end.
- ✅ README + CONTRIBUTING merged on all three repos.
