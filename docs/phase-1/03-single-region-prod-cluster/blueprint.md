# Single-Region Production Cluster — Blueprint

> Phase-1 Workstream 3 · Lead: Cloud Infra Lead + Platform Lead · Window: Weeks 1–11
> Target cluster: `prod-use1` (EKS, us-east-1, multi-AZ)

## Topology

```
┌────────────────────────────────────────────────────────────────────────┐
│ AWS Account: acme-prod-use1 (vended via AFT / Control Tower)            │
│                                                                         │
│ VPC: 10.20.0.0/16                                                       │
│  ├── private-a: 10.20.0.0/19   (us-east-1a)                             │
│  ├── private-b: 10.20.32.0/19  (us-east-1b)                             │
│  ├── private-c: 10.20.64.0/19  (us-east-1c)                             │
│  ├── public-a:  10.20.96.0/22  (NAT, NLB only)                          │
│  ├── public-b:  10.20.100.0/22                                          │
│  └── public-c:  10.20.104.0/22                                          │
│                                                                         │
│ EKS Cluster: prod-use1 (v1.31)                                          │
│  ├── Control plane: private endpoint only                               │
│  ├── Karpenter NodePool: general (m6i/m7i, spot 70/30)                  │
│  ├── Karpenter NodePool: memory-opt (r6i, on-demand)                    │
│  ├── Karpenter NodePool: gpu (g5, on-demand, tainted)                   │
│  ├── Karpenter NodePool: spot-batch (mixed, 100% spot)                  │
│  └── Add-ons: vpc-cni, kube-proxy, coredns, ebs-csi, efs-csi, snapshot  │
│                                                                         │
│ Networking:                                                             │
│  ├── Cilium (CNI replacement) — default-deny + L7 NetworkPolicy         │
│  ├── Istio (multi-primary ready) — STRICT mTLS                          │
│  ├── AWS Load Balancer Controller — internal NLB only                   │
│  └── ExternalDNS — Route53 private zone                                 │
│                                                                         │
│ Platform add-ons (GitOps-managed):                                      │
│  ├── ArgoCD (HA), External Secrets Operator, CSI Secrets Store          │
│  ├── Vault Agent Injector, cert-manager, Karpenter, Cluster Autoscaler  │
│  ├── Kyverno, Falco, Trivy Operator, Cilium Tetragon                    │
│  └── kube-prometheus-stack, Loki, Tempo, OTel Operator                  │
└────────────────────────────────────────────────────────────────────────┘
```

## Decisions

| Topic | Decision | Rationale |
|---|---|---|
| K8s version | 1.31 | Newest stable on EKS; supports VAP and CEL |
| CNI | Cilium (replaces aws-vpc-cni) | L7 policy, Tetragon eBPF integration, identity-aware policy |
| Mesh | Istio (managed candidate: Tetrate POC in flight) | Multi-primary topology in Phase 2 |
| Autoscaling | Karpenter (no Cluster Autoscaler for general pools) | Consolidation + spot mix + faster scale |
| Ingress | AWS NLB + Istio gateway | Anycast in Phase 2; NLB single-region in P1 |
| Storage | gp3 default; EFS for shared-writes | Cost & IOPS for prod tier |
| Identity | IRSA + Pod Identity (transitioning) | Native AWS, zero static creds |
| Secrets | Vault CSI provider + External Secrets | Dynamic creds, no env-var leakage |
| Image registry | ECR (regional cache) + Harbor (canonical) | Cosign verifies at admission |
| GitOps | ArgoCD with App-of-Apps from `acme/gitops` | Already bootstrapped in Phase 0 |

## Hardening checklist (per CIS Kubernetes Benchmark v1.9)

- [x] Control plane logging: all 5 log types to CloudWatch.
- [x] Private cluster endpoint (no public API access).
- [x] EKS auth: OIDC via Keycloak; no aws-auth ConfigMap edits by humans.
- [x] Default-deny NetworkPolicy in every namespace (Cilium).
- [x] Pod Security Standard "restricted" enforced in `prod-*` namespaces.
- [x] No service-account tokens auto-mounted unless needed.
- [x] etcd encryption at rest (EKS managed).
- [x] Secret encryption at rest (KMS CMK).
- [x] No privileged DaemonSets except curated platform set (Cilium, OTel collector).

## Bootstrap order (Argo App-of-Apps wave numbers)

| Wave | Components |
|---|---|
| -100 | ArgoCD itself (chicken-and-egg via initial helmfile) |
| -90  | cert-manager, external-secrets-operator, csi-secrets-store-driver |
| -80  | Cilium, kube-proxy replacement |
| -70  | AWS Load Balancer Controller, ExternalDNS, EBS/EFS CSI drivers |
| -60  | Vault Agent Injector |
| -50  | Kyverno + ClusterPolicies (verifyImages from Day 1, enforce on nonprod, audit on prod week 1) |
| -40  | Karpenter NodePools |
| -30  | Istio control plane (STRICT mTLS) |
| -20  | OpenTelemetry Operator, Prometheus, Loki, Tempo |
| -10  | Backstage discovery, Trivy Operator, Falco, Tetragon |
|   0  | Workloads (pilot service first, then progressive ramp) |

## Cluster lifecycle policy

- **No in-place upgrades for major K8s versions** — blue/green clusters with workload migration.
- **Add-ons upgraded weekly** via Renovate PRs against `gitops/platform/`.
- **Quarterly DR drill** — drain entire cluster to a sister cluster (multi-region preview in Phase 2).

## Cost guardrails (FinOps)

- Karpenter consolidation enabled.
- 70% target spot share on general pool.
- Idle namespace reaper (nonprod) — out-of-scope for prod cluster.
- Quota per namespace enforced (defined in tenant repo overlays).
- Kubecost installed Week 6 for showback.

## Files in this folder

- [`eks-terraform-module.tf`](./eks-terraform-module.tf) — single Terraform module that creates the VPC + EKS + IRSA roles + KMS CMK + base IAM.
- [`karpenter.yaml`](./karpenter.yaml) — NodePool + EC2NodeClass definitions.
- [`cilium-values.yaml`](./cilium-values.yaml) — Cilium Helm values with kube-proxy replacement + Hubble + Tetragon enabled.

## Acceptance criteria (Phase-1 gate)

- [ ] Cluster passes CIS Kubernetes Benchmark v1.9 ≥ 95%.
- [ ] kube-bench + kubeaudit + Polaris run clean (or documented exceptions).
- [ ] Argo CD App-of-Apps fully syncs to 0 OutOfSync.
- [ ] Pilot service migrated from staging EKS to `prod-use1` cluster with green SLOs.
- [ ] Drain-AZ chaos drill: zone us-east-1a drained, all workloads stay green.
- [ ] DR runbook tested (backup + restore of pilot stateful workload).
