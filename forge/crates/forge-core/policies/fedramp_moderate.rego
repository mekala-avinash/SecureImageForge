package forge.fedramp_moderate

# FedRAMP Moderate — image-level controls (subset).
default allow := false

deny[msg] {
    not input.spec.hardening.non_root_user
    msg := "FedRAMP AC-6: Least Privilege - image must not run as root"
}

deny[msg] {
    input.spec.base_image != "distroless"
    not input.spec.hardening.remove_shells
    msg := "FedRAMP CM-7: Least Functionality - shells must be stripped from non-distroless images"
}

deny[msg] {
    input.spec.base_image != "distroless"
    not input.spec.hardening.remove_pkg_managers
    msg := "FedRAMP CM-7: Least Functionality - package managers must be removed from non-distroless images"
}

deny[msg] {
    some i
    finding := input.scan.findings[i]
    finding.severity == "CRITICAL"
    msg := sprintf("FedRAMP RA-5: Vulnerability Scanning - critical CVE %s in %s", [finding.id, finding.package])
}

deny[msg] {
    some i
    finding := input.scan.findings[i]
    finding.severity == "HIGH"
    msg := sprintf("FedRAMP RA-5: Vulnerability Scanning - high CVE %s in %s", [finding.id, finding.package])
}

allow {
    count(deny) == 0
}
