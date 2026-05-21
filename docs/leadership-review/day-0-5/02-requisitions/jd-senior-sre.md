# Senior Site Reliability Engineer

> Req ID: SRE-2026-001 · Team: Site Reliability Engineering · Reports to: Head of SRE
> Location: Remote (with on-call rotation participation) · Level: IC4 / Senior · Start: Q2

## About the role
Help us evolve from reactive operations to **SLO-driven, error-budgeted, partly-autonomous** reliability engineering. You'll define what "good" looks like for our services, build the auto-remediation playbooks, and oversee the autonomous agents that operate alongside us.

## What you'll work on
- **SLOs as code**: Sloth-style PrometheusServiceLevel rules; multi-window multi-burn-rate alerts; error-budget policy enforcement.
- **Auto-remediation**: Argo Events → Argo Workflows runbooks; agent-driven first-responder actions with HITL fallbacks.
- **Incident response**: PagerDuty → Incident Analysis Agent integration; blameless RCA workflow; postmortem rigor.
- **Disaster recovery**: tiered RTO/RPO; quarterly DR drills (multi-region failover, region evacuation, data-restore validation).
- **Chaos engineering**: LitmusChaos / Chaos Mesh game days; baked-in resilience patterns.
- **On-call**: 24×7 follow-the-sun rotation; primary + secondary; aim for ≤ 2 pages/shift.
- **Reliability advocacy**: pair with product teams to set SLOs that match actual user expectations.

## Must-have
- 6+ years engineering, with 3+ years in SRE roles (Google-style or equivalent).
- Production K8s ops (multi-cluster, multi-AZ at minimum).
- Strong with Prometheus, Grafana, Loki/Tempo (or Datadog/New Relic if migrating).
- Practiced with progressive delivery (canary / blue-green) and SLO-driven gating.
- Writes runbooks people actually use; pages people respect.
- Code-first SRE — primarily Go or Python.

## Nice-to-have
- OpenTelemetry instrumentation experience.
- Multi-region / active-active operations.
- Chaos engineering practitioner (LitmusChaos, Gremlin).
- Public talks/blogs on reliability practice.

## What success looks like in 12 months
- Every Tier-1 service has SLOs as code + burn-rate alerts + linked runbooks.
- MTTR drops materially (target: 4h → 25 min).
- ≥ 50% of common incidents are auto-remediated before paging a human.
- Quarterly DR drills pass within RTO/RPO targets across all tiers.

## What we offer
Same as platform engineering JD — plus the chance to define SRE practice at a company committed to doing it right.

## Application
Internal: `#platform-careers`. External: <link>.
