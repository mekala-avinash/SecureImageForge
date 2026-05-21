# Enterprise Platform Transformation — FAQ

> Maintained in `#platform-transformation`. Submit additions via PR to this file.

### Program

**Q: Is this a re-platform / rewrite?**
A: No. We use the strangler pattern. The existing monolith keeps running. We carve out bounded contexts incrementally and migrate routes behind a feature flag with instant rollback.

**Q: Will my service deploys break?**
A: No. Existing services keep their current pipelines through Phase 1. We migrate services to the new GitOps + signed-build flow on a per-service schedule with the owning team.

**Q: When do I have to do something?**
A: Phase 0 (now): enable gitsign signed commits (5-min setup, doc + office hours). That's it. Other changes land on a per-team basis with at least 2 weeks of notice.

**Q: What if the new scanners block my PRs?**
A: We start in warn-only mode for the first 2 weeks. After that, only HIGH/CRITICAL findings block. Every finding has an auto-fix suggestion or an explicit waiver path with security review.

### Hiring

**Q: Can internal engineers apply?**
A: Yes — strongly encouraged. See `#platform-careers` and the JDs in `docs/leadership-review/day-0-5/02-requisitions/`.

**Q: Are we slowing product to fund this?**
A: No. Existing teams keep their headcount. The 4 net-new hires are platform/security/compliance roles that don't currently exist or are under-staffed.

### Tools & tech

**Q: Why Wolfi/distroless and not Alpine?**
A: Wolfi/distroless ships zero shells, zero package managers, and signed bases — drastically smaller CVE surface. Alpine is fine for many cases but loses on supply-chain controls.

**Q: Why Istio and not Linkerd?**
A: Istio's multi-primary topology + AuthorizationPolicy + Envoy mature ecosystem fit our multi-region / mesh-federation needs. We're evaluating managed Istio (Tetrate / Anthos / Solo) to reduce operational burden.

**Q: Why Argo CD and not Flux?**
A: Either would work. Argo's UI + ApplicationSet + Argo Rollouts integration is a stronger one-stop fit for our progressive delivery needs.

**Q: Why Backstage?**
A: De-facto IDP standard in 2025–26, CNCF-incubating, extensible, and gives us a single pane of glass for service catalog, scaffolders, TechDocs, cost, SLOs.

**Q: Why Temporal for agents?**
A: Durable, replayable workflows with idempotent activities — essential for safe autonomous actions with rollback.

**Q: Will agents make changes without humans?**
A: Only for LOW blast radius (auto). MEDIUM = human-in-the-loop in Slack. HIGH/CRITICAL = CAB + 2-person rule. Every action is sandboxed, signed, audited to WORM storage, and reversible.

### Security & compliance

**Q: Will I lose root access to production?**
A: Most engineers never had it. Break-glass remains for emergencies via Vault dynamic creds with 2-person rule + session recording. This is a SOC 2/ISO requirement.

**Q: Does SOC 2 mean more red tape?**
A: Some. Most controls are evidenced automatically (Drata pulling from K8s, IdP, Vault, GitHub). Engineers will see ≤ 1 quarterly access review form.

### Cost

**Q: Won't multi-region cost more?**
A: Briefly, yes — Phase 2/3 has a double-spend transition. By Phase 3 close, we project net 25–35% YoY reduction via Karpenter, spot, right-sizing agent, and idle reaper.

### Anything else

Submit a PR to this file or drop in `#platform-transformation`.
