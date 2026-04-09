"""Runtime Drift Detection Service"""
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone
import hashlib

class DriftDetector:
    """Detect configuration drift between templates and runtime"""
    
    def __init__(self):
        self.runtime_images: Dict[str, Dict[str, Any]] = {}
        self.template_digests: Dict[str, str] = {}
    
    def register_template(self, template_id: str, config: Dict[str, Any]):
        """Register a hardened template configuration"""
        digest = self._compute_config_digest(config)
        self.template_digests[template_id] = digest
    
    def register_runtime_image(self, image_id: str, image_info: Dict[str, Any]):
        """Register a running container image"""
        self.runtime_images[image_id] = {
            "info": image_info,
            "registered_at": datetime.now(timezone.utc).isoformat(),
            "digest": image_info.get("digest", "")
        }
    
    def detect_drift(self, image_id: str, template_id: str) -> Dict[str, Any]:
        """Detect drift between runtime image and template"""
        if image_id not in self.runtime_images:
            return {"error": "Runtime image not found", "has_drift": False}
        
        if template_id not in self.template_digests:
            return {"error": "Template not found", "has_drift": False}
        
        runtime_image = self.runtime_images[image_id]
        runtime_digest = runtime_image["digest"]
        template_digest = self.template_digests[template_id]
        
        has_drift = runtime_digest != template_digest
        
        drift_details = []
        if has_drift:
            # Simulate drift detection
            drift_details.append({
                "type": "digest_mismatch",
                "severity": "high",
                "message": "Runtime image digest does not match hardened template",
                "expected": template_digest,
                "actual": runtime_digest
            })
            
            # Check for common drift scenarios
            if runtime_image["info"].get("has_shell"):
                drift_details.append({
                    "type": "unauthorized_shell",
                    "severity": "critical",
                    "message": "Shell binaries detected in runtime (should be removed per template)"
                })
            
            if runtime_image["info"].get("running_as_root"):
                drift_details.append({
                    "type": "root_user",
                    "severity": "critical",
                    "message": "Container running as root user (template requires non-root)"
                })
        
        return {
            "image_id": image_id,
            "template_id": template_id,
            "has_drift": has_drift,
            "drift_count": len(drift_details),
            "drift_details": drift_details,
            "checked_at": datetime.now(timezone.utc).isoformat(),
            "risk_level": self._calculate_risk_level(drift_details)
        }
    
    def scan_all_runtime_images(self) -> List[Dict[str, Any]]:
        """Scan all registered runtime images for drift"""
        results = []
        for image_id in self.runtime_images:
            # Assume template_id from image metadata
            template_id = self.runtime_images[image_id]["info"].get("template_id")
            if template_id:
                drift_result = self.detect_drift(image_id, template_id)
                if drift_result["has_drift"]:
                    results.append(drift_result)
        return results
    
    def _compute_config_digest(self, config: Dict[str, Any]) -> str:
        """Compute digest of configuration"""
        import json
        config_str = json.dumps(config, sort_keys=True)
        return hashlib.sha256(config_str.encode()).hexdigest()
    
    def _calculate_risk_level(self, drift_details: List[Dict[str, Any]]) -> str:
        """Calculate overall risk level from drift details"""
        if not drift_details:
            return "none"
        
        severities = [d["severity"] for d in drift_details]
        if "critical" in severities:
            return "critical"
        elif "high" in severities:
            return "high"
        elif "medium" in severities:
            return "medium"
        else:
            return "low"

# Simulated Kubernetes runtime data
def simulate_k8s_runtime_images() -> List[Dict[str, Any]]:
    """Simulate runtime images from Kubernetes cluster"""
    return [
        {
            "image_id": "k8s-prod-java-app-001",
            "namespace": "production",
            "pod_name": "java-service-7d9f8b-xk2m9",
            "image_tag": "secureforge/java-app:v1.2.3",
            "digest": "sha256:abc123...",
            "template_id": "java-openjdk21-alpine",
            "has_shell": False,
            "running_as_root": False,
            "last_updated": datetime.now(timezone.utc).isoformat()
        },
        {
            "image_id": "k8s-staging-nodejs-002",
            "namespace": "staging",
            "pod_name": "nodejs-api-5c8d7-p9k4s",
            "image_tag": "nodejs-legacy:latest",
            "digest": "sha256:def456...",
            "template_id": "nodejs-20-alpine",
            "has_shell": True,  # Drift!
            "running_as_root": True,  # Drift!
            "last_updated": datetime.now(timezone.utc).isoformat()
        }
    ]