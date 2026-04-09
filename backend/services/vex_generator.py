"""VEX (Vulnerability Exploitability eXchange) Generator"""
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone
import uuid

class VEXStatus:
    NOT_AFFECTED = "not_affected"
    AFFECTED = "affected"
    FIXED = "fixed"
    UNDER_INVESTIGATION = "under_investigation"

class VEXJustification:
    COMPONENT_NOT_PRESENT = "component_not_present"
    VULNERABLE_CODE_NOT_PRESENT = "vulnerable_code_not_present"
    VULNERABLE_CODE_NOT_IN_EXECUTE_PATH = "vulnerable_code_not_in_execute_path"
    VULNERABLE_CODE_CANNOT_BE_CONTROLLED = "vulnerable_code_cannot_be_controlled_by_adversary"
    INLINE_MITIGATIONS = "inline_mitigations_already_exist"

def generate_vex_document(build_id: str, vulnerabilities: Dict[str, List[Dict[str, Any]]], 
                         image_tag: str, runtime: str) -> Dict[str, Any]:
    """Generate VEX document for vulnerability exploitability analysis"""
    
    vex_statements = []
    
    # Analyze each vulnerability for exploitability
    for severity, vuln_list in vulnerabilities.items():
        for vuln in vuln_list:
            analysis = analyze_vulnerability_exploitability(vuln, runtime)
            
            vex_statement = {
                "vulnerability": {
                    "id": vuln.get('id'),
                    "description": vuln.get('description'),
                    "severity": severity
                },
                "product": {
                    "id": image_tag,
                    "package": vuln.get('package')
                },
                "status": analysis['status'],
                "justification": analysis.get('justification'),
                "impact_statement": analysis.get('impact_statement'),
                "action_statement": analysis.get('action_statement'),
                "timestamp": datetime.now(timezone.utc).isoformat()
            }
            
            vex_statements.append(vex_statement)
    
    vex_document = {
        "@context": "https://openvex.dev/ns",
        "@id": f"https://secureforge.enterprise/vex/{build_id}",
        "author": "SecureImage Forge Security Team",
        "role": "Document Creator",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "version": "1",
        "tooling": "SecureImage Forge VEX Generator v1.0",
        "statements": vex_statements,
        "summary": {
            "total_vulnerabilities": sum(len(v) for v in vulnerabilities.values()),
            "not_affected": sum(1 for s in vex_statements if s['status'] == VEXStatus.NOT_AFFECTED),
            "affected": sum(1 for s in vex_statements if s['status'] == VEXStatus.AFFECTED),
            "false_positive_rate": calculate_false_positive_rate(vex_statements)
        }
    }
    
    return vex_document

def analyze_vulnerability_exploitability(vuln: Dict[str, Any], runtime: str) -> Dict[str, Any]:
    """Analyze if a vulnerability is actually exploitable in this context"""
    
    vuln_id = vuln.get('id', '')
    package = vuln.get('package', '')
    
    # Simulate exploitability analysis
    # In production, this would use actual code path analysis, SBOM, and runtime data
    
    # Example: OpenSSL vulnerability but no network code
    if 'openssl' in package.lower():
        if runtime in ['go']:  # Go has its own crypto
            return {
                "status": VEXStatus.NOT_AFFECTED,
                "justification": VEXJustification.COMPONENT_NOT_PRESENT,
                "impact_statement": f"{package} is present but {runtime} runtime uses built-in cryptography. OpenSSL is not invoked.",
                "action_statement": "No action required. Vulnerability does not affect this build."
            }
    
    # Example: CLI tool vulnerability in a web service
    if any(word in package.lower() for word in ['curl', 'wget', 'ssh']):
        return {
            "status": VEXStatus.NOT_AFFECTED,
            "justification": VEXJustification.VULNERABLE_CODE_NOT_IN_EXECUTE_PATH,
            "impact_statement": f"{package} CLI tools are not in application execution path. Service does not invoke these binaries.",
            "action_statement": "Monitor for updates but low priority for remediation."
        }
    
    # Example: Vulnerability in removed shell
    if 'bash' in package.lower() or 'sh' in package.lower():
        return {
            "status": VEXStatus.NOT_AFFECTED,
            "justification": VEXJustification.COMPONENT_NOT_PRESENT,
            "impact_statement": "Shell binaries were removed during hardening process. Component not present in final image.",
            "action_statement": "No action required. Component removed."
        }
    
    # Default: Affected and requires remediation
    return {
        "status": VEXStatus.AFFECTED,
        "justification": None,
        "impact_statement": f"Vulnerability {vuln_id} in {package} is potentially exploitable in this configuration.",
        "action_statement": "Update to patched version or apply mitigation controls."
    }

def calculate_false_positive_rate(vex_statements: List[Dict[str, Any]]) -> float:
    """Calculate percentage of vulnerabilities that are false positives"""
    if not vex_statements:
        return 0.0
    
    not_affected = sum(1 for s in vex_statements if s['status'] == VEXStatus.NOT_AFFECTED)
    return (not_affected / len(vex_statements)) * 100

def get_vex_by_severity_reduction(vex_document: Dict[str, Any]) -> Dict[str, int]:
    """Calculate actual risk reduction after VEX analysis"""
    statements = vex_document.get('statements', [])
    
    original_severity = {"CRITICAL": 0, "HIGH": 0, "MEDIUM": 0, "LOW": 0}
    actual_severity = {"CRITICAL": 0, "HIGH": 0, "MEDIUM": 0, "LOW": 0}
    
    for stmt in statements:
        severity = stmt['vulnerability']['severity']
        original_severity[severity] += 1
        
        if stmt['status'] == VEXStatus.AFFECTED:
            actual_severity[severity] += 1
    
    return {
        "original": original_severity,
        "actual_risk": actual_severity,
        "reduction": {
            sev: original_severity[sev] - actual_severity[sev]
            for sev in original_severity
        }
    }