# Paved-road service GitOps layout

Every service onboarded via Backstage gets the following structure in
`acme/gitops`:

```
apps/<service>/
├── base/                          # language-agnostic base
│   ├── kustomization.yaml
│   ├── namespace.yaml             # PSS=restricted, paved-road=true
│   ├── helm-release.yaml          # ArgoCD Application targeting the chart in service repo
│   └── secret-provider-class.yaml # Secrets-Store CSI binding to Vault
└── overlays/
    ├── dev/
    │   ├── kustomization.yaml
    │   └── values.yaml            # env-specific Helm values (image digest, replicas, ...)
    ├── staging/
    │   ├── kustomization.yaml
    │   └── values.yaml
    └── prod/
        ├── kustomization.yaml
        ├── values.yaml
        └── rollout.yaml           # Argo Rollouts canary strategy
```

The platform team maintains the `_template/` directory as the source of truth.
Backstage scaffolders copy from there when a service is created.

## Promotion flow

1. CI publishes a signed multi-arch image and an SBOM attestation.
2. CI opens a PR that bumps `image.digest` in `overlays/staging/values.yaml`.
3. Merge → ArgoCD auto-syncs the staging Application.
4. After SLO burn-rate clears, a manual approval PR bumps `overlays/prod/values.yaml`.
5. ArgoCD applies Argo Rollouts canary; AnalysisTemplates gate progressive promotion.
