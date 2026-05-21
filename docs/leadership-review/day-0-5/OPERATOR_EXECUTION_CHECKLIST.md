# Operator Execution Checklist — Final Manual Actions

> Three actions remain that **only a human operator can complete** because they require credentials, identities, or email systems outside this workspace. Each is reduced to a single command or a copy/paste.

## Action 1 — Redline meeting minutes (Program PM, ~30 min)

The minutes documents are **worked examples** with placeholder `<name>` tokens. Open each and substitute real names + dates:

| File | Action |
|---|---|
| `docs/leadership-review/day-0-5/03-steering-committee/minutes/2026-01-09-kickoff.md` | Replace all `<name>` placeholders; confirm dates match your calendar; validate decision tallies; publish to `#platform-transformation` |
| `docs/leadership-review/gate-reviews/phase-0-2026-02-06.md` | Same — these are forward-looking; will be re-redlined on the actual gate review day. Use as the live template during the meeting |
| `docs/leadership-review/gate-reviews/phase-0-retrospective.md` | Same — to be filled with real data on retrospective day |

```bash
grep -rln "<name>" docs/leadership-review/   # quick check of where placeholders remain
```

## Action 2 — Send 6 vendor cover emails (Compliance PM + Platform Lead, ~15 min)

Open each email file, replace `<name>`, `<phone>`, and your email signature; attach the PDFs noted at the bottom of each email; click send.

| Vendor | Email file | Sender |
|---|---|---|
| Drata | `docs/leadership-review/day-0-5/05-compliance-platform-rfp/vendor-cover-emails.md#email-1--drata` | Compliance PM |
| Vanta | `…/vendor-cover-emails.md#email-2--vanta` | Compliance PM |
| Secureframe | `…/vendor-cover-emails.md#email-3--secureframe` | Compliance PM |
| Tetrate | `docs/leadership-review/day-0-5/06-managed-istio-rfp/vendor-cover-emails.md#email-1--tetrate` | Platform Lead |
| Google Anthos | `…/vendor-cover-emails.md#email-2--google-cloud-anthos-service-mesh` | Platform Lead |
| Solo.io | `…/vendor-cover-emails.md#email-3--soloio-gloo-mesh` | Platform Lead |

PDF render command (if you have pandoc):
```bash
pandoc docs/leadership-review/day-0-5/05-compliance-platform-rfp/drata-vanta-secureframe-eval.md -o ACME-Compliance-RFP-v1.0.pdf
pandoc docs/leadership-review/day-0-5/06-managed-istio-rfp/rfp.md -o ACME-Istio-RFP-v1.0.pdf
```

Log each send in your GRC tool with `RFP-2026-001-COMPLIANCE` and `RFP-2026-002-ISTIO`.

## Action 3 — Push GitOps repos with branch protection (Platform Lead, ~12 min)

Pre-flight:
```bash
gh auth status                     # must be org admin
gh auth refresh -s admin:org,repo,workflow
terraform version                  # >= 1.5
export GITHUB_TOKEN="$(gh auth token)"
export ORG=acme                    # ← replace with your real org
```

Execute:
```bash
cd /app/gitops-bootstrap
./push-all.sh
```

The script will:
1. `gh repo create` for `acme/gitops`, `acme/platform`, `acme/tenants` (idempotent).
2. Push signed commits from local repos already initialized in `/app/gitops-bootstrap/`.
3. `terraform apply` the `github-repo-protection` module → branch protection, signed-commit enforcement, ≥ 2 reviewer + CODEOWNERS rule, 5 required status checks, no force-push, secret-scanning + push-protection enabled, restricted push allowlist (argocd-bot + platform-admins team).
4. Verify via `gh api repos/<org>/<repo>/branches/main/protection`.

Post-flight (Argo CD root App):
```bash
kubectl apply -n argocd -f /app/gitops-bootstrap/gitops/platform/argocd/root-app.yaml
argocd app wait root --timeout 300
```

If anything fails: the script is fully idempotent — fix the error and re-run.

---

## Status tracker

| # | Action | Owner | Done? | When | Notes |
|---|---|---|---|---|---|
| 1a | Redline `2026-01-09-kickoff.md` | Program PM | ☐ | | |
| 1b | Redline gate review template (on Week 4) | Program PM | ☐ | 2026-02-06 | |
| 2a | Send 3 compliance vendor emails | Compliance PM | ☐ | | |
| 2b | Send 3 Istio vendor emails | Platform Lead | ☐ | | |
| 3a | Run `push-all.sh` | Platform Lead | ☐ | | |
| 3b | Apply Argo CD root App | Platform Lead | ☐ | | |

Mark this file complete (commit it back) once all 6 sub-actions are done.
