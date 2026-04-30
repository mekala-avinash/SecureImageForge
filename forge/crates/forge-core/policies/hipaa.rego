package forge.hipaa

default allow := false

deny[msg] {
    not input.spec.hardening.non_root_user
    msg := "HIPAA §164.312(a): least-privilege requires non-root user"
}

deny[msg] {
    some i
    finding := input.scan.findings[i]
    finding.severity == "CRITICAL"
    msg := sprintf("HIPAA §164.308: critical CVE %s blocks PHI workloads", [finding.id])
}

allow { count(deny) == 0 }
