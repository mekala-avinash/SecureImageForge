# acme/tenants — Tenant Overlays

Per-tenant configuration: namespaces, ResourceQuotas, LimitRanges, NetworkPolicies, RBAC, data-residency labels, BYOK key references, and per-tenant feature flags.

## Layout

```
tenants/
├── _template/         # Used by Backstage scaffolder to onboard new tenants
├── <tenant-id>/
│   ├── tenant.yaml            # canonical metadata (region, tier, data class)
│   ├── namespaces.yaml        # NS + quotas + limit ranges
│   ├── rbac.yaml              # RoleBindings (mapped to IdP groups via SCIM)
│   ├── network-policies.yaml  # Cilium / Kubernetes NetworkPolicies
│   ├── secrets.yaml           # SecretProviderClass references (no secret values)
│   └── apps/                  # per-tenant app overlays (rare; most tenants are soft-tenant)
```

## Tenancy tiers

- **Soft (namespace)** — internal squads.
- **Medium (vCluster)** — external customers, regulated dev/test sandboxes.
- **Hard (dedicated cluster)** — PCI/HIPAA/gov; uses `clusters/pci-*` in `acme/gitops`.

## Onboarding

Use the Backstage scaffolder. It auto-PRs into this repo and the `acme/gitops` repo with the tenant's bootstrap resources.

## CODEOWNERS

The platform team + the tenant onboarding squad co-own this repo. Tenant-specific overlays may add tenant lead as a reviewer.
