# Staff DevSecOps Engineer

> Req ID: SEC-2026-001 · Team: Security Engineering · Reports to: CISO (dotted line to VP Platform)
> Location: Remote · Level: IC5 / Staff · Hiring manager: <CISO>

## About the role
This is the founding senior security engineer for our enterprise platform program. You will own the supply-chain security and policy-as-code program end-to-end — from signed commits to admission control to runtime detection. You will set the bar that every other engineer builds against.

## What you'll work on
- **Software supply chain**: drive SLSA L3+ attainment — Sigstore (Cosign, Fulcio, Rekor), in-toto attestations, SBOM (Syft, SPDX/CycloneDX), provenance generation in CI.
- **Policy-as-code**: Kyverno + OPA/Gatekeeper policy library; admission controllers; ValidatingAdmissionPolicy (CEL); separation-of-duties enforcement.
- **Runtime security**: Falco rules, Tetragon, KubeArmor; auto-quarantine workflows; SOC integration.
- **Identity & secrets**: SPIFFE/SPIRE rollout; Vault HA + KMS integration; CSI Secrets Store; cert-manager; mTLS via Istio.
- **CI/CD security**: hermetic builds; OIDC-based runner identity; signed commits (gitsign); SAST/DAST/SCA/IaC scan gating.
- **Compliance enablement**: implement controls that produce continuous audit evidence (SOC 2 / ISO / PCI / HIPAA / FedRAMP) via OPA bundles and Drata-style connectors.

## Must-have
- 8+ years engineering with 5+ years in cloud-native security.
- Deep production experience with Kubernetes security (PSS, NetworkPolicy, admission controllers).
- Hands-on with Sigstore (Cosign signing + verification), SBOMs, SLSA.
- Policy-as-code: at least one of Kyverno, OPA/Gatekeeper, Conftest, Cedar.
- Working knowledge of Vault (or equivalent) + KMS (AWS/Azure/GCP).
- Familiar with at least one compliance regime (SOC 2, ISO 27001, PCI-DSS, HIPAA, FedRAMP).
- Strong engineering chops in Go and/or Python.

## Nice-to-have
- Threat modeling (STRIDE, PASTA) at platform scale.
- Falco/eBPF runtime detection authorship.
- CNCF security project contributions.
- Air-gapped / regulated deployment experience.

## What success looks like in 12 months
- 100% of production images are signed + SBOM-attested + SLSA L3 provenance-attested at admission.
- A policy-as-code library that engineers refer to as "the easy path" — not a roadblock.
- SOC 2 Type II audit closed cleanly; ISO 27001 certification achieved.
- Zero supply-chain incidents that bypass our admission controls.

## What we offer
Same as platform engineering JD — plus direct sponsorship for security conference talks and CNCF contributions.

## Application
Internal: `#platform-careers`. External: <link>.
