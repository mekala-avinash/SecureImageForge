package forge.soc2

default allow := false

deny[msg] {
    not input.spec.hardening.non_root_user
    msg := "SOC2 CC6.1: container must run as non-root user"
}

deny[msg] {
    not input.spec.sign
    msg := "SOC2 CC7.2: artifact signing required for change management"
}

deny[msg] {
    some i
    finding := input.scan.findings[i]
    finding.severity == "CRITICAL"
    msg := sprintf("SOC2 CC7.1: critical CVE %s must be remediated", [finding.id])
}

allow { count(deny) == 0 }
