# Hardened Runtime Image — Waiver Process

> When a service genuinely cannot use a hardened image *yet*, a time-boxed waiver is granted. Waivers are tracked in Backstage and reviewed in SteerCo.

## Valid waiver reasons (non-exhaustive)

1. **Required native binary not in Wolfi/distroless** (and not feasible to vendor).
2. **3rd-party SDK requires root** for an initialization step that cannot be moved to an init-container.
3. **Service writes to FS paths outside `/tmp`** and refactor is non-trivial (must be ticketed).
4. **Legacy framework cannot start as non-root** (rare; usually Java EE on ancient runtimes).

## NOT valid waiver reasons

- "Migration is hard." — pair migration is offered free.
- "I will do it later." — open a ticket with a date instead.
- "We don't have tests." — Phase-1 doesn't require new tests; it requires no regressions.

## Workflow

1. Service owner opens `WAIVER-<svc>` issue in `acme/platform` repo using the waiver template.
2. Waiver justification reviewed by Platform Lead + Staff DevSecOps.
3. If approved:
   - **Max duration:** 90 days.
   - Logged in Backstage scorecard (visible to leadership).
   - Service still runs with as many Pod Security Standard controls as possible.
   - Renewal requires CISO sign-off.
4. If rejected: pair migration scheduled within 5 working days.

## Waiver template (paste into the issue)

```yaml
service: <repo-name>
owner: <team>
reason: <one of {missing-native-binary, root-required, fs-writes, legacy-framework}>
detail: |
  <explain in detail; include error logs / refactor cost estimate>
mitigation:
  - what we will do in the meantime (e.g., baseline image with restricted SCC)
  - target date to revisit
requested_duration_days: 30
risk_acceptance: <Platform Lead + CISO sign here>
```

## SLA

- Waiver request → first review: 2 working days.
- Approved waiver → tracked in Backstage scorecard: same day.
- Expired waiver → service blocked from new deploys until renewed or resolved.
