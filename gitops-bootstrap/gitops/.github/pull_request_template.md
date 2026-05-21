## Summary
<!-- What does this PR change and why? -->

## Type
- [ ] App manifest (add / update / remove)
- [ ] Platform component
- [ ] Cluster bootstrap
- [ ] Policy / Security
- [ ] Tenant overlay

## Checklist
- [ ] Commits are signed (gitsign / GPG / SSH).
- [ ] `helm template` and/or `kustomize build` succeed locally.
- [ ] `kubeconform` passes locally.
- [ ] No secrets in this PR (`SecretProviderClass` / `ExternalSecret` only).
- [ ] Production overlays use **image digests**, not tags.
- [ ] CODEOWNERS reviewers have been requested (≥ 2).
- [ ] Linked ticket: <JIRA-XXXX>

## Risk & rollback
<!-- Blast radius (LOW / MED / HIGH / CRITICAL) and rollback plan. -->

## Verification plan
<!-- How will we verify this in dev / staging before promotion? -->
