"""
VEX (Vulnerability Exploitability eXchange) Generator

Generates VEX documents following OpenVEX and CSAF specifications to communicate
vulnerability exploitability status. VEX reduces false positives by explaining
which vulnerabilities are actually exploitable in a specific context.

Formats Supported:
- OpenVEX (default): https://openvex.dev/
- CSAF VEX: https://docs.oasis-open.org/csaf/csaf/v2.0/csaf-v2.0.html

References:
- https://www.cisa.gov/sites/default/files/publications/VEX_Use_Cases_Aprill2022.pdf
- https://github.com/openvex/spec
"""
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone, timedelta
import uuid
import hashlib


class VEXStatus:
    """VEX vulnerability status values"""
    NOT_AFFECTED = "not_affected"
    AFFECTED = "affected"
    FIXED = "fixed"
    UNDER_INVESTIGATION = "under_investigation"


class VEXJustification:
    """VEX justification codes for NOT_AFFECTED status"""
    COMPONENT_NOT_PRESENT = "component_not_present"
    VULNERABLE_CODE_NOT_PRESENT = "vulnerable_code_not_present"
    VULNERABLE_CODE_NOT_IN_EXECUTE_PATH = "vulnerable_code_not_in_execute_path"
    VULNERABLE_CODE_CANNOT_BE_CONTROLLED = "vulnerable_code_cannot_be_controlled_by_adversary"
    INLINE_MITIGATIONS = "inline_mitigations_already_exist"


class VEXImpactStatement:
    """Standard impact statement templates"""
    NO_IMPACT = "No impact to this product. The vulnerable component is not present."
    MITIGATED = "Vulnerability is mitigated by existing security controls."
    LIMITED_EXPOSURE = "Limited exposure due to runtime configuration."
    REQUIRES_UPDATE = "Update to patched version required."


# Known vulnerability patterns and their exploitability contexts
CVE_EXPLOITABILITY_RULES = {
    # Log4j patterns
    "CVE-2021-44228": {
        "affected_components": ["log4j-core", "log4j-api"],
        "not_affected_if": {
            "java_version_gte": 17,  # JDK 17+ has mitigations
            "jndi_disabled": True,
            "log4j_version_gte": "2.17.0"
        },
        "mitigation_available": True
    },
    # OpenSSL patterns
    "CVE-2023-5678": {
        "affected_components": ["openssl", "libssl"],
        "not_affected_if": {
            "runtime": ["go"],  # Go uses its own crypto
            "no_tls_endpoints": True
        },
        "mitigation_available": True
    },
    # Shell-based vulnerabilities
    "CVE-SHELL-*": {
        "pattern": True,
        "affected_components": ["bash", "sh", "dash", "ash"],
        "not_affected_if": {
            "shell_removed": True
        },
        "mitigation_available": False  # Just remove the shell
    },
    # curl/wget network tools
    "CVE-2023-38545": {
        "affected_components": ["curl", "libcurl"],
        "not_affected_if": {
            "no_outbound_connections": True,
            "proxy_disabled": True
        },
        "mitigation_available": True
    }
}


def generate_vex_document(
    build_id: str,
    vulnerabilities: Dict[str, List[Dict[str, Any]]],
    image_tag: str,
    runtime: str,
    config: Optional[Dict[str, Any]] = None,
    output_format: str = "openvex"
) -> Dict[str, Any]:
    """
    Generate VEX document for vulnerability exploitability analysis.
    
    Args:
        build_id: Unique build identifier
        vulnerabilities: Dict of severity -> list of vulnerabilities
        image_tag: Container image tag
        runtime: Runtime environment (java, nodejs, go, dotnet)
        config: Build configuration for context-aware analysis
        output_format: "openvex" or "csaf"
    
    Returns:
        VEX document in specified format
    """
    config = config or {}
    vex_statements = []
    
    # Analyze each vulnerability
    for severity, vuln_list in vulnerabilities.items():
        for vuln in vuln_list:
            analysis = analyze_vulnerability_exploitability(vuln, runtime, config)
            
            statement = {
                "vulnerability": {
                    "@id": f"https://nvd.nist.gov/vuln/detail/{vuln.get('id')}",
                    "name": vuln.get('id'),
                    "description": vuln.get('description', ''),
                    "aliases": vuln.get('aliases', [])
                },
                "products": [
                    {
                        "@id": f"pkg:docker/{image_tag.replace(':', '/')}",
                        "identifiers": {
                            "purl": f"pkg:docker/{image_tag.replace(':', '/')}",
                            "cpe": f"cpe:2.3:a:secureforge:{runtime}:*:*:*:*:*:*:*:*"
                        },
                        "subcomponents": [
                            {
                                "@id": f"pkg:generic/{vuln.get('package', 'unknown')}",
                                "name": vuln.get('package', 'unknown')
                            }
                        ]
                    }
                ],
                "status": analysis['status'],
                "justification": analysis.get('justification'),
                "impact_statement": analysis.get('impact_statement'),
                "action_statement": analysis.get('action_statement'),
                "remediation": analysis.get('remediation'),
                "timestamp": datetime.now(timezone.utc).isoformat(),
                "last_updated": datetime.now(timezone.utc).isoformat(),
                "supplier": "SecureImage Forge Security Team"
            }
            
            vex_statements.append(statement)
    
    # Generate document in requested format
    if output_format == "csaf":
        return _generate_csaf_vex(build_id, image_tag, vex_statements)
    else:
        return _generate_openvex(build_id, image_tag, vex_statements)


def _generate_openvex(build_id: str, image_tag: str, statements: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate OpenVEX format document"""
    doc_id = f"https://secureforge.enterprise/vex/{build_id}"
    
    # Calculate summary statistics
    total = len(statements)
    not_affected = sum(1 for s in statements if s['status'] == VEXStatus.NOT_AFFECTED)
    affected = sum(1 for s in statements if s['status'] == VEXStatus.AFFECTED)
    fixed = sum(1 for s in statements if s['status'] == VEXStatus.FIXED)
    under_investigation = sum(1 for s in statements if s['status'] == VEXStatus.UNDER_INVESTIGATION)
    
    return {
        "@context": "https://openvex.dev/ns/v0.2.0",
        "@id": doc_id,
        "author": "SecureImage Forge",
        "role": "security_analyst",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "version": 1,
        "tooling": "SecureImage Forge VEX Generator v2.0",
        "last_updated": datetime.now(timezone.utc).isoformat(),
        "statements": statements,
        "metadata": {
            "product": image_tag,
            "build_id": build_id,
            "generator_version": "2.0.0",
            "spec_version": "0.2.0"
        },
        "summary": {
            "total_vulnerabilities": total,
            "not_affected": not_affected,
            "affected": affected,
            "fixed": fixed,
            "under_investigation": under_investigation,
            "false_positive_rate": round((not_affected / total * 100) if total > 0 else 0, 2),
            "actual_risk_reduction": f"{not_affected}/{total} vulnerabilities determined non-exploitable",
            "risk_score_before": _calculate_risk_score(statements, include_not_affected=True),
            "risk_score_after": _calculate_risk_score(statements, include_not_affected=False)
        }
    }


def _generate_csaf_vex(build_id: str, image_tag: str, statements: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate CSAF VEX format document (OASIS standard)"""
    
    # Map VEX statements to CSAF vulnerabilities
    csaf_vulnerabilities = []
    for stmt in statements:
        vuln_id = stmt['vulnerability']['name']
        
        # Determine CSAF remediation category
        if stmt['status'] == VEXStatus.NOT_AFFECTED:
            remediation_category = "no_fix_planned"
            remediation_details = stmt.get('impact_statement', 'Not affected')
        elif stmt['status'] == VEXStatus.FIXED:
            remediation_category = "vendor_fix"
            remediation_details = stmt.get('action_statement', 'Update to latest version')
        elif stmt['status'] == VEXStatus.UNDER_INVESTIGATION:
            remediation_category = "none_available"
            remediation_details = "Under investigation"
        else:
            remediation_category = "workaround"
            remediation_details = stmt.get('action_statement', 'Apply recommended mitigation')
        
        csaf_vuln = {
            "cve": vuln_id,
            "title": stmt['vulnerability'].get('description', vuln_id),
            "product_status": {
                "known_affected" if stmt['status'] == VEXStatus.AFFECTED else "known_not_affected": [
                    f"CSAFPID-{image_tag.replace(':', '-').replace('/', '-')}"
                ]
            },
            "threats": [
                {
                    "category": "impact",
                    "details": stmt.get('impact_statement', 'See vulnerability details')
                }
            ],
            "remediations": [
                {
                    "category": remediation_category,
                    "details": remediation_details,
                    "product_ids": [f"CSAFPID-{image_tag.replace(':', '-').replace('/', '-')}"]
                }
            ],
            "flags": [
                {
                    "label": "component_not_present" if stmt.get('justification') == VEXJustification.COMPONENT_NOT_PRESENT else "vulnerable_code_not_present",
                    "product_ids": [f"CSAFPID-{image_tag.replace(':', '-').replace('/', '-')}"]
                }
            ] if stmt['status'] == VEXStatus.NOT_AFFECTED else []
        }
        
        csaf_vulnerabilities.append(csaf_vuln)
    
    return {
        "document": {
            "category": "csaf_vex",
            "csaf_version": "2.0",
            "title": f"VEX Document for {image_tag}",
            "publisher": {
                "category": "vendor",
                "name": "SecureImage Forge",
                "namespace": "https://secureforge.enterprise"
            },
            "tracking": {
                "id": f"SECUREFORGE-VEX-{build_id[:8].upper()}",
                "current_release_date": datetime.now(timezone.utc).isoformat(),
                "initial_release_date": datetime.now(timezone.utc).isoformat(),
                "revision_history": [
                    {
                        "date": datetime.now(timezone.utc).isoformat(),
                        "number": "1",
                        "summary": "Initial VEX release"
                    }
                ],
                "status": "final",
                "version": "1"
            },
            "distribution": {
                "tlp": {
                    "label": "WHITE"
                }
            }
        },
        "product_tree": {
            "full_product_names": [
                {
                    "name": image_tag,
                    "product_id": f"CSAFPID-{image_tag.replace(':', '-').replace('/', '-')}",
                    "product_identification_helper": {
                        "purl": f"pkg:docker/{image_tag.replace(':', '/')}"
                    }
                }
            ]
        },
        "vulnerabilities": csaf_vulnerabilities
    }


def analyze_vulnerability_exploitability(
    vuln: Dict[str, Any],
    runtime: str,
    config: Dict[str, Any]
) -> Dict[str, Any]:
    """
    Analyze if a vulnerability is actually exploitable in this context.
    
    Uses:
    - Runtime context (Go doesn't use OpenSSL, etc.)
    - Build configuration (shell removed, etc.)
    - Known CVE patterns and exploitability rules
    """
    vuln_id = vuln.get('id', '')
    package = vuln.get('package', '').lower()
    severity = vuln.get('severity', 'MEDIUM')
    
    # Check against known CVE rules
    for cve_pattern, rules in CVE_EXPLOITABILITY_RULES.items():
        if rules.get('pattern'):
            # Pattern-based matching (e.g., CVE-SHELL-*)
            if cve_pattern.replace('*', '') in vuln_id:
                return _apply_exploitability_rules(vuln, rules, runtime, config)
        elif vuln_id == cve_pattern:
            return _apply_exploitability_rules(vuln, rules, runtime, config)
    
    # Context-based analysis for unknown CVEs
    
    # 1. Shell-related vulnerabilities - check if shell was removed
    if any(shell in package for shell in ['bash', 'sh', 'dash', 'ash', 'zsh']):
        if config.get('remove_shell', False):
            return {
                "status": VEXStatus.NOT_AFFECTED,
                "justification": VEXJustification.COMPONENT_NOT_PRESENT,
                "impact_statement": "Shell binaries were removed during hardening. Vulnerability is not exploitable.",
                "action_statement": "No action required. Component has been removed from the image.",
                "remediation": None
            }
    
    # 2. Network tools vulnerabilities - check if used
    if any(tool in package for tool in ['curl', 'wget', 'ssh', 'telnet', 'ftp']):
        if config.get('remove_shell', False):  # Network tools often require shell
            return {
                "status": VEXStatus.NOT_AFFECTED,
                "justification": VEXJustification.VULNERABLE_CODE_NOT_IN_EXECUTE_PATH,
                "impact_statement": f"{package} CLI tools are present but not invoked by the application. No shell access available.",
                "action_statement": "Monitor for updates. Low priority for remediation.",
                "remediation": {"type": "update", "priority": "low"}
            }
    
    # 3. Crypto library vulnerabilities - check runtime
    if any(crypto in package for crypto in ['openssl', 'libssl', 'gnutls']):
        if runtime == 'go':
            return {
                "status": VEXStatus.NOT_AFFECTED,
                "justification": VEXJustification.COMPONENT_NOT_PRESENT,
                "impact_statement": f"Go runtime uses built-in cryptography. {package} is not invoked by the application.",
                "action_statement": "No action required for this runtime.",
                "remediation": None
            }
    
    # 4. Package manager vulnerabilities - check if removed
    if any(pm in package for pm in ['apt', 'apk', 'yum', 'dnf', 'npm', 'pip']):
        if config.get('remove_package_manager', False):
            return {
                "status": VEXStatus.NOT_AFFECTED,
                "justification": VEXJustification.COMPONENT_NOT_PRESENT,
                "impact_statement": "Package manager was removed during hardening. Cannot be exploited.",
                "action_statement": "No action required. Component removed.",
                "remediation": None
            }
    
    # 5. Runtime-specific analysis
    if runtime == 'java':
        if 'log4j' in package and config.get('runtime_version', '').startswith(('17', '21')):
            return {
                "status": VEXStatus.NOT_AFFECTED,
                "justification": VEXJustification.INLINE_MITIGATIONS,
                "impact_statement": f"Java {config.get('runtime_version')} includes mitigations for Log4j JNDI attacks.",
                "action_statement": "Continue monitoring. Update Log4j when convenient.",
                "remediation": {"type": "update", "priority": "medium"}
            }
    
    # Default: Affected and requires remediation
    remediation_priority = "critical" if severity == "CRITICAL" else "high" if severity == "HIGH" else "medium"
    
    return {
        "status": VEXStatus.AFFECTED,
        "justification": None,
        "impact_statement": f"Vulnerability {vuln_id} in {package} is potentially exploitable. Severity: {severity}.",
        "action_statement": "Update to patched version or apply security controls.",
        "remediation": {
            "type": "update",
            "priority": remediation_priority,
            "recommended_version": vuln.get('fixed_version'),
            "workaround": "Apply network segmentation and runtime monitoring."
        }
    }


def _apply_exploitability_rules(
    vuln: Dict[str, Any],
    rules: Dict[str, Any],
    runtime: str,
    config: Dict[str, Any]
) -> Dict[str, Any]:
    """Apply known exploitability rules for a CVE"""
    not_affected_conditions = rules.get('not_affected_if', {})
    
    # Check each condition
    for condition, value in not_affected_conditions.items():
        if condition == 'runtime' and runtime in value:
            return {
                "status": VEXStatus.NOT_AFFECTED,
                "justification": VEXJustification.COMPONENT_NOT_PRESENT,
                "impact_statement": f"Runtime {runtime} does not use the affected component.",
                "action_statement": "No action required for this runtime.",
                "remediation": None
            }
        
        if condition == 'shell_removed' and config.get('remove_shell', False):
            return {
                "status": VEXStatus.NOT_AFFECTED,
                "justification": VEXJustification.COMPONENT_NOT_PRESENT,
                "impact_statement": "Shell has been removed. Vulnerability not exploitable.",
                "action_statement": "No action required.",
                "remediation": None
            }
        
        if condition == 'java_version_gte':
            java_version = config.get('runtime_version', '0')
            try:
                if int(java_version.split('.')[0]) >= value:
                    return {
                        "status": VEXStatus.NOT_AFFECTED,
                        "justification": VEXJustification.INLINE_MITIGATIONS,
                        "impact_statement": f"Java {java_version} includes built-in mitigations.",
                        "action_statement": "Monitor for updates.",
                        "remediation": {"type": "monitor", "priority": "low"}
                    }
            except (ValueError, IndexError):
                pass
    
    # No matching condition - affected
    return {
        "status": VEXStatus.AFFECTED,
        "justification": None,
        "impact_statement": f"Vulnerability {vuln.get('id')} requires remediation.",
        "action_statement": "Apply available patch or mitigation.",
        "remediation": {
            "type": "update",
            "priority": "high" if rules.get('mitigation_available') else "critical"
        }
    }


def _calculate_risk_score(statements: List[Dict[str, Any]], include_not_affected: bool = True) -> int:
    """Calculate aggregate risk score from VEX statements"""
    severity_scores = {"CRITICAL": 10, "HIGH": 7, "MEDIUM": 4, "LOW": 1}
    total_score = 0
    
    for stmt in statements:
        if not include_not_affected and stmt['status'] == VEXStatus.NOT_AFFECTED:
            continue
        
        # Extract severity from vulnerability description or products
        severity = "MEDIUM"  # default
        vuln_desc = stmt.get('vulnerability', {}).get('description', '')
        for sev in severity_scores.keys():
            if sev.lower() in vuln_desc.lower():
                severity = sev
                break
        
        # Only count affected vulnerabilities
        if stmt['status'] == VEXStatus.AFFECTED:
            total_score += severity_scores.get(severity, 4)
        elif stmt['status'] == VEXStatus.UNDER_INVESTIGATION:
            total_score += severity_scores.get(severity, 4) * 0.5
    
    return int(total_score)


def get_vex_summary(vex_document: Dict[str, Any]) -> Dict[str, Any]:
    """Get executive summary of VEX document"""
    summary = vex_document.get('summary', {})
    statements = vex_document.get('statements', [])
    
    # Group by justification
    justification_counts = {}
    for stmt in statements:
        if stmt['status'] == VEXStatus.NOT_AFFECTED:
            just = stmt.get('justification', 'unknown')
            justification_counts[just] = justification_counts.get(just, 0) + 1
    
    # Get most common false positive reasons
    top_reasons = sorted(justification_counts.items(), key=lambda x: x[1], reverse=True)[:3]
    
    return {
        "document_id": vex_document.get('@id'),
        "product": vex_document.get('metadata', {}).get('product'),
        "total_vulnerabilities": summary.get('total_vulnerabilities', 0),
        "exploitable": summary.get('affected', 0),
        "non_exploitable": summary.get('not_affected', 0),
        "false_positive_rate": f"{summary.get('false_positive_rate', 0)}%",
        "risk_reduction": {
            "before": summary.get('risk_score_before', 0),
            "after": summary.get('risk_score_after', 0),
            "reduction_percent": round(
                (1 - summary.get('risk_score_after', 0) / max(summary.get('risk_score_before', 1), 1)) * 100, 1
            )
        },
        "top_non_exploitable_reasons": [
            {"reason": r[0], "count": r[1]} for r in top_reasons
        ],
        "recommendation": _generate_recommendation(summary)
    }


def _generate_recommendation(summary: Dict[str, Any]) -> str:
    """Generate actionable recommendation based on VEX analysis"""
    fp_rate = summary.get('false_positive_rate', 0)
    affected = summary.get('affected', 0)
    
    if affected == 0:
        return "All identified vulnerabilities are non-exploitable in this context. No immediate action required."
    elif fp_rate > 50:
        return f"VEX analysis reduced actionable vulnerabilities by {fp_rate}%. Focus remediation on the {affected} truly exploitable issues."
    else:
        return f"{affected} vulnerabilities require attention. Consider automated remediation for fixable issues."
