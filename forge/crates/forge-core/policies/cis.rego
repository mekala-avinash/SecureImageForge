package forge.cis

# CIS Docker Benchmark — image-level controls (subset).
default allow := false

deny[msg] {
    not input.spec.hardening.non_root_user
    msg := "CIS 4.1: image must not run as root (USER directive required)"
}

deny[msg] {
    input.spec.base_image != "distroless"
    not input.spec.hardening.remove_shells
    msg := "CIS 4.6: shells must be stripped from non-distroless images"
}

deny[msg] {
    input.spec.base_image != "distroless"
    not input.spec.hardening.remove_pkg_managers
    msg := "CIS 4.7: package managers must be removed from non-distroless images"
}

deny[msg] {
    some i
    finding := input.scan.findings[i]
    finding.severity == "CRITICAL"
    msg := sprintf("CIS 4.x: critical CVE %s in %s", [finding.id, finding.package])
}

allow {
    count(deny) == 0
}
