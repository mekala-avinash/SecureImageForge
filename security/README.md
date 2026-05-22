# Security

Production policy-as-code + supply-chain controls. Every file here either runs in-cluster (Kyverno, Falco, Cilium) or in CI (Cosign, SBOM, SLSA).

## Layout

```
security/
├── kyverno/
│   ├── verify-images.yaml       # Cosign + SLSA + SBOM admission
│   └── pod-security.yaml        # Restricted PSS as Kyverno policies
├── opa-gatekeeper/
│   └── constraints.yaml         # Allowed registries + Separation-of-Duties
├── falco/
│   └── rules.yaml               # Shell-in-container, RO-FS write, unexpected egress
├── cilium/
│   └── values.yaml              # Cilium Helm values (kube-proxy replacement + Tetragon)
└── supply-chain/
    ├── cosign-policy.yaml       # Cosign policy-controller policy
    └── README.md                # End-to-end attest+verify flow
```

## Composition

```
                CI                              Cluster
                ──                              ───────
 PR/merge → Cosign sign  ──┐                  ┌─→ Kyverno verifyImages ─┐
            Syft SBOM    ──┤    OCI registry  │   Cosign policy-ctrl    │ admission
            SLSA provenance ┘─→ (Harbor/ECR) ─┘                          │ decision
                                                                          ▼
                                                            allowed / denied
                                                  ─────────
                                                  Runtime
                                                  ───────
                                                  Cilium L3-L7 NetworkPolicy (default-deny)
                                                  Istio STRICT mTLS + AuthorizationPolicy
                                                  Falco syscall rules
                                                  Tetragon eBPF observability
                                                  Trivy Operator continuous scans
```

## CI integration

The reusable workflow in `ci/github-actions/reusable-build.yml` produces the artifacts these policies enforce:
- Cosign signature → verified by `kyverno/verify-images.yaml`.
- SBOM attestation → presence required by the same policy.
- SLSA provenance attestation → verified by the same policy.

## Operational guidance

- **Phasing.** Roll new admission rules in `Audit` mode first; promote to `Enforce` after one week of clean audit logs.
- **Bypass procedure.** No bypass. A waiver is a chart-version pin to a known-old policy bundle (visible in audit) plus a CISO sign-off.
- **Updating Cilium / Istio.** Use canary clusters first; never upgrade prod control planes in-place.
