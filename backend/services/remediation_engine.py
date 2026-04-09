"""CIS Benchmark Remediation Suggestion Engine"""
from typing import List, Dict, Any

REMEDIATION_RULES = {
    "non_root_user": {
        "title": "Configure Non-Root User",
        "severity": "critical",
        "remediation": """Add the following to your Dockerfile:

RUN groupadd -g 1000 appuser && useradd -r -u 1000 -g appuser appuser
USER 1000:1000

This ensures the application runs with minimal privileges.""",
        "impact": "Prevents privilege escalation attacks",
        "effort": "Low"
    },
    "no_shell": {
        "title": "Remove Shell Binaries",
        "severity": "high",
        "remediation": """Add the following to your Dockerfile after installing dependencies:

RUN rm -rf /bin/sh /bin/bash /usr/bin/sh /usr/bin/bash /bin/dash 2>/dev/null || true

This removes shell access from the container.""",
        "impact": "Reduces attack surface by removing shell capabilities",
        "effort": "Low"
    },
    "no_package_manager": {
        "title": "Remove Package Managers",
        "severity": "high",
        "remediation": """Add the following to your Dockerfile:

For Alpine:
RUN rm -rf /sbin/apk /usr/bin/apk /etc/apk 2>/dev/null || true

For Debian/Ubuntu:
RUN rm -rf /usr/bin/apt* /usr/bin/dpkg* /var/lib/apt /var/lib/dpkg 2>/dev/null || true

This prevents runtime package installation.""",
        "impact": "Prevents unauthorized software installation",
        "effort": "Low"
    },
    "read_only_fs": {
        "title": "Enable Read-Only Root Filesystem",
        "severity": "medium",
        "remediation": """Run the container with read-only filesystem:

docker run --read-only -v /tmp:/tmp:rw myimage

Or in Kubernetes:

securityContext:
  readOnlyRootFilesystem: true

Mount only necessary paths as writable (e.g., /tmp).""",
        "impact": "Prevents malicious file modifications",
        "effort": "Medium"
    },
    "no_ssh": {
        "title": "Remove SSH Server",
        "severity": "high",
        "remediation": """Ensure SSH is not installed in your Dockerfile.

For Debian/Ubuntu:
RUN apt-get purge -y openssh-server openssh-client

For Alpine:
RUN apk del openssh

Use 'kubectl exec' or 'docker exec' for debugging instead.""",
        "impact": "Prevents unauthorized remote access",
        "effort": "Low"
    },
    "audit_logging": {
        "title": "Configure Audit Logging",
        "severity": "medium",
        "remediation": """Configure application to send logs to stdout/stderr:

# In your application
import logging
import sys

logging.basicConfig(
    stream=sys.stdout,
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

Collect logs using your orchestrator's logging system.""",
        "impact": "Enables security incident detection and compliance",
        "effort": "Medium"
    },
    "fips_crypto": {
        "title": "Enable FIPS-Compliant Cryptography",
        "severity": "high",
        "remediation": """Use FIPS-validated base images:

FROM mcr.microsoft.com/dotnet/aspnet:8.0-cbl-mariner2.0-distroless

Or configure OpenSSL in FIPS mode:

RUN openssl fipsinstall
ENV OPENSSL_FIPS=1

Ensure all cryptographic operations use FIPS-approved algorithms.""",
        "impact": "Meets federal compliance requirements for cryptography",
        "effort": "High"
    },
    "controlled_entrypoints": {
        "title": "Use Controlled Entry Points",
        "severity": "medium",
        "remediation": """Define explicit ENTRYPOINT and CMD:

ENTRYPOINT ["java", "-jar"]
CMD ["/app/app.jar"]

Avoid using shell form (e.g., ENTRYPOINT java -jar app.jar)
Use exec form with array syntax instead.""",
        "impact": "Prevents shell injection and ensures proper signal handling",
        "effort": "Low"
    }
}

def generate_remediation_suggestions(compliance_checks: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Generate remediation suggestions for failed compliance checks"""
    suggestions = []
    
    for check in compliance_checks:
        if check['status'] in ['failed', 'warning']:
            check_name = check['check']
            if check_name in REMEDIATION_RULES:
                rule = REMEDIATION_RULES[check_name]
                suggestions.append({
                    "check": check_name,
                    "profile": check['profile'],
                    "title": rule['title'],
                    "severity": rule['severity'],
                    "remediation": rule['remediation'],
                    "impact": rule['impact'],
                    "effort": rule['effort'],
                    "status": check['status']
                })
    
    return suggestions

def get_cis_benchmark_score(checks: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Calculate detailed CIS benchmark scoring"""
    total = len(checks)
    passed = sum(1 for c in checks if c['status'] == 'passed')
    failed = sum(1 for c in checks if c['status'] == 'failed')
    warnings = sum(1 for c in checks if c['status'] == 'warning')
    
    # Weight by severity
    critical_failed = sum(1 for c in checks if c['status'] == 'failed' and c.get('severity') == 'critical')
    high_failed = sum(1 for c in checks if c['status'] == 'failed' and c.get('severity') == 'high')
    
    # Calculate weighted score
    score = 100
    score -= critical_failed * 25
    score -= high_failed * 15
    score -= (failed - critical_failed - high_failed) * 10
    score -= warnings * 5
    
    return {
        "score": max(0, score),
        "total_checks": total,
        "passed": passed,
        "failed": failed,
        "warnings": warnings,
        "critical_failures": critical_failed,
        "high_failures": high_failed,
        "grade": get_grade(max(0, score))
    }

def get_grade(score: int) -> str:
    """Get letter grade for CIS score"""
    if score >= 95:
        return 'A+'
    elif score >= 90:
        return 'A'
    elif score >= 85:
        return 'B+'
    elif score >= 80:
        return 'B'
    elif score >= 75:
        return 'C+'
    elif score >= 70:
        return 'C'
    elif score >= 60:
        return 'D'
    else:
        return 'F'