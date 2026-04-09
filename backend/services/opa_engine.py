"""OPA (Open Policy Agent) Policy Engine with Rego Support"""
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone
import json

# Simulated Rego policy evaluation
# In production, this would use actual OPA server or py-rego

class RegoPolicyEngine:
    """Evaluate Rego policies against build configurations"""
    
    def __init__(self):
        self.policies = {}
    
    def add_policy(self, policy_id: str, rego_code: str, metadata: Dict[str, Any]):
        """Register a new Rego policy"""
        self.policies[policy_id] = {
            "code": rego_code,
            "metadata": metadata,
            "compiled_at": datetime.now(timezone.utc).isoformat()
        }
    
    def evaluate(self, policy_id: str, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Evaluate a policy against input data"""
        if policy_id not in self.policies:
            return {"error": "Policy not found", "allowed": False}
        
        # Simulate Rego evaluation
        policy = self.policies[policy_id]
        result = self._simulate_rego_eval(policy["code"], input_data)
        
        return {
            "policy_id": policy_id,
            "allowed": result["allowed"],
            "violations": result.get("violations", []),
            "warnings": result.get("warnings", []),
            "metadata": policy["metadata"],
            "evaluated_at": datetime.now(timezone.utc).isoformat()
        }
    
    def _simulate_rego_eval(self, rego_code: str, input_data: Dict[str, Any]) -> Dict[str, Any]:
        """Simulate Rego policy evaluation"""
        violations = []
        warnings = []
        allowed = True
        
        # Parse simple rules from Rego-like code
        if "deny[msg]" in rego_code:
            # Check for common enterprise rules
            if "runtime == \"java\"" in rego_code:
                if input_data.get("runtime") == "java":
                    if "openjdk_version < 21" in rego_code:
                        # Check if using old Java version
                        violations.append({
                            "rule": "java_version_policy",
                            "message": "Java images must use OpenJDK 21 or higher",
                            "severity": "high"
                        })
                        allowed = False
            
            if "has_datadog_agent" in rego_code:
                if not input_data.get("has_monitoring_agent"):
                    violations.append({
                        "rule": "monitoring_required",
                        "message": "All production images must include Datadog agent",
                        "severity": "high"
                    })
                    allowed = False
        
        if "warn[msg]" in rego_code:
            if "base_image" in rego_code and input_data.get("base_image") == "debian":
                warnings.append({
                    "rule": "preferred_base_image",
                    "message": "Alpine is preferred over Debian for smaller attack surface",
                    "severity": "medium"
                })
        
        return {
            "allowed": allowed,
            "violations": violations,
            "warnings": warnings
        }

# Pre-built enterprise Rego policies
ENTERPRISE_REGO_POLICIES = {
    "java_openjdk21_policy": {
        "name": "Java OpenJDK 21 Enforcement",
        "description": "All Java images must use OpenJDK 21 with Datadog agent",
        "severity": "high",
        "rego_code": """
package secureforge.java

deny[msg] {
    input.runtime == "java"
    input.openjdk_version < 21
    msg := "Java images must use OpenJDK 21 or higher"
}

deny[msg] {
    input.runtime == "java"
    not input.has_datadog_agent
    msg := "All Java production images must include Datadog agent"
}
"""
    },
    "no_root_user_policy": {
        "name": "Non-Root User Mandatory",
        "description": "All images must run as non-root user (UID >= 1000)",
        "severity": "critical",
        "rego_code": """
package secureforge.security

deny[msg] {
    input.user_id == 0
    msg := "Images must not run as root user (UID 0)"
}

deny[msg] {
    input.user_id < 1000
    msg := "User ID must be >= 1000 for non-privileged operation"
}
"""
    },
    "production_base_image_policy": {
        "name": "Production Base Image Restrictions",
        "description": "Production images must use approved base images",
        "severity": "high",
        "rego_code": """
package secureforge.baseimage

approved_bases := ["alpine", "distroless"]

deny[msg] {
    not input.base_image in approved_bases
    input.environment == "production"
    msg := sprintf("Base image '%s' not approved for production. Use: %v", [input.base_image, approved_bases])
}

warn[msg] {
    input.base_image == "debian"
    msg := "Alpine or Distroless preferred over Debian for smaller attack surface"
}
"""
    },
    "cve_threshold_policy": {
        "name": "CVE Threshold Enforcement",
        "description": "Block images exceeding CVE thresholds",
        "severity": "critical",
        "rego_code": """
package secureforge.vulnerabilities

deny[msg] {
    input.critical_cves > 0
    msg := sprintf("Image has %d CRITICAL CVEs. Must be 0 for deployment.", [input.critical_cves])
}

deny[msg] {
    input.high_cves > 5
    msg := sprintf("Image has %d HIGH CVEs. Maximum allowed: 5.", [input.high_cves])
}

warn[msg] {
    input.medium_cves > 20
    msg := sprintf("Image has %d MEDIUM CVEs. Consider remediation.", [input.medium_cves])
}
"""
    },
    "slsa_level3_policy": {
        "name": "SLSA Level 3 Required",
        "description": "All production images must have SLSA L3 provenance",
        "severity": "critical",
        "rego_code": """
package secureforge.supplychain

deny[msg] {
    input.environment == "production"
    input.slsa_level < 3
    msg := "Production images require SLSA Level 3 build provenance"
}

deny[msg] {
    input.environment == "production"
    not input.is_signed
    msg := "Production images must be cryptographically signed"
}

deny[msg] {
    input.environment == "production"
    not input.has_sbom
    msg := "Production images must include SBOM"
}
"""
    }
}

def get_enterprise_policies() -> Dict[str, Dict[str, Any]]:
    """Get all pre-built enterprise Rego policies"""
    return ENTERPRISE_REGO_POLICIES

def evaluate_rego_policy(policy_code: str, input_data: Dict[str, Any]) -> Dict[str, Any]:
    """Evaluate Rego policy code against input"""
    engine = RegoPolicyEngine()
    policy_id = "custom_policy"
    engine.add_policy(policy_id, policy_code, {"type": "custom"})
    return engine.evaluate(policy_id, input_data)