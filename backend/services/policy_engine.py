"""Custom Policy Engine using OPA-style policies"""
from typing import List, Dict, Any
from datetime import datetime, timezone

# Pre-built policy templates
POLICY_TEMPLATES = {
    "no_critical_vulns": {
        "name": "No Critical Vulnerabilities",
        "description": "Block builds with any critical vulnerabilities",
        "type": "vulnerability",
        "enforcement": "block",
        "rule": {
            "condition": "critical_count",
            "operator": "equals",
            "value": 0
        }
    },
    "max_high_vulns": {
        "name": "Maximum High Vulnerabilities",
        "description": "Warn if more than 5 high vulnerabilities",
        "type": "vulnerability",
        "enforcement": "warn",
        "rule": {
            "condition": "high_count",
            "operator": "less_than_or_equal",
            "value": 5
        }
    },
    "min_compliance_score": {
        "name": "Minimum Compliance Score",
        "description": "Require at least 80% compliance score",
        "type": "compliance",
        "enforcement": "block",
        "rule": {
            "condition": "compliance_score",
            "operator": "greater_than_or_equal",
            "value": 80
        }
    },
    "required_profiles": {
        "name": "Required Compliance Profiles",
        "description": "Enforce CIS benchmark compliance",
        "type": "compliance",
        "enforcement": "block",
        "rule": {
            "condition": "profiles_include",
            "operator": "contains",
            "value": ["cis"]
        }
    },
    "non_root_user": {
        "name": "Non-Root User Required",
        "description": "Ensure application runs as non-root",
        "type": "configuration",
        "enforcement": "block",
        "rule": {
            "condition": "user_id",
            "operator": "not_equals",
            "value": 0
        }
    },
    "approved_base_images": {
        "name": "Approved Base Images Only",
        "description": "Only allow alpine or distroless base images",
        "type": "configuration",
        "enforcement": "warn",
        "rule": {
            "condition": "base_image",
            "operator": "in",
            "value": ["alpine", "distroless"]
        }
    },
    "fresh_image": {
        "name": "Fresh Image Requirement",
        "description": "Images must be less than 30 days old",
        "type": "freshness",
        "enforcement": "warn",
        "rule": {
            "condition": "age_days",
            "operator": "less_than",
            "value": 30
        }
    },
    "sbom_required": {
        "name": "SBOM Required",
        "description": "All images must have SBOM",
        "type": "configuration",
        "enforcement": "block",
        "rule": {
            "condition": "has_sbom",
            "operator": "equals",
            "value": True
        }
    },
    "signed_images": {
        "name": "Signed Images Only",
        "description": "All images must be cryptographically signed",
        "type": "security",
        "enforcement": "block",
        "rule": {
            "condition": "is_signed",
            "operator": "equals",
            "value": True
        }
    }
}

def evaluate_policy(policy: Dict[str, Any], build_data: Dict[str, Any], config_data: Dict[str, Any] = None) -> Dict[str, Any]:
    """Evaluate a single policy against build data"""
    rule = policy['rule']
    condition = rule['condition']
    operator = rule['operator']
    expected_value = rule['value']
    
    # Get actual value based on condition
    actual_value = get_condition_value(condition, build_data, config_data)
    
    # Evaluate based on operator
    passed = evaluate_operator(operator, actual_value, expected_value)
    
    return {
        "policy_name": policy['name'],
        "policy_type": policy['type'],
        "enforcement": policy['enforcement'],
        "passed": passed,
        "actual_value": actual_value,
        "expected_value": expected_value,
        "message": generate_message(policy, passed, actual_value, expected_value)
    }

def get_condition_value(condition: str, build_data: Dict[str, Any], config_data: Dict[str, Any] = None) -> Any:
    """Extract the actual value for a condition"""
    if condition == "critical_count":
        return build_data.get('vulnerability_count', {}).get('CRITICAL', 0)
    elif condition == "high_count":
        return build_data.get('vulnerability_count', {}).get('HIGH', 0)
    elif condition == "compliance_score":
        return build_data.get('compliance_score', 0)
    elif condition == "profiles_include":
        return config_data.get('compliance_profiles', []) if config_data else []
    elif condition == "user_id":
        # Would check Dockerfile in real implementation
        return 1000  # Simulated non-root
    elif condition == "base_image":
        return config_data.get('base_image', '') if config_data else ''
    elif condition == "age_days":
        if build_data.get('completed_at'):
            try:
                completed = datetime.fromisoformat(build_data['completed_at']) if isinstance(build_data['completed_at'], str) else build_data['completed_at']
                age = (datetime.now(timezone.utc) - completed).days
                return age
            except:
                return 0
        return 0
    elif condition == "has_sbom":
        return bool(build_data.get('sbom_path'))
    elif condition == "is_signed":
        return build_data.get('is_signed', False)
    else:
        return None

def evaluate_operator(operator: str, actual: Any, expected: Any) -> bool:
    """Evaluate comparison operators"""
    if operator == "equals":
        return actual == expected
    elif operator == "not_equals":
        return actual != expected
    elif operator == "greater_than":
        return actual > expected
    elif operator == "greater_than_or_equal":
        return actual >= expected
    elif operator == "less_than":
        return actual < expected
    elif operator == "less_than_or_equal":
        return actual <= expected
    elif operator == "in":
        return actual in expected
    elif operator == "contains":
        if isinstance(expected, list) and isinstance(actual, list):
            return all(item in actual for item in expected)
        return expected in actual
    else:
        return False

def generate_message(policy: Dict[str, Any], passed: bool, actual: Any, expected: Any) -> str:
    """Generate human-readable message for policy evaluation"""
    if passed:
        return f"✓ {policy['name']}: Policy satisfied"
    else:
        enforcement = policy['enforcement'].upper()
        return f"✗ [{enforcement}] {policy['name']}: Expected {expected}, got {actual}"

def evaluate_all_policies(policies: List[Dict[str, Any]], build_data: Dict[str, Any], config_data: Dict[str, Any] = None) -> Dict[str, Any]:
    """Evaluate all policies against a build"""
    results = []
    blocks = []
    warnings = []
    infos = []
    
    for policy in policies:
        result = evaluate_policy(policy, build_data, config_data)
        results.append(result)
        
        if not result['passed']:
            if result['enforcement'] == 'block':
                blocks.append(result)
            elif result['enforcement'] == 'warn':
                warnings.append(result)
            else:
                infos.append(result)
    
    total_policies = len(policies)
    passed_policies = sum(1 for r in results if r['passed'])
    
    return {
        "total_policies": total_policies,
        "passed": passed_policies,
        "failed": total_policies - passed_policies,
        "blocked": len(blocks) == 0,
        "blocks": blocks,
        "warnings": warnings,
        "infos": infos,
        "results": results,
        "overall_status": "passed" if len(blocks) == 0 else "blocked"
    }

def get_policy_recommendation(build_data: Dict[str, Any]) -> List[str]:
    """Recommend policies based on build characteristics"""
    recommendations = []
    
    vulns = build_data.get('vulnerability_count', {})
    if vulns.get('CRITICAL', 0) > 0:
        recommendations.append("no_critical_vulns")
    if vulns.get('HIGH', 0) > 5:
        recommendations.append("max_high_vulns")
    
    if build_data.get('compliance_score', 100) < 80:
        recommendations.append("min_compliance_score")
    
    if not build_data.get('sbom_path'):
        recommendations.append("sbom_required")
    
    return recommendations