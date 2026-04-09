"""Lifecycle Manager - Image Deprecation and Tombstoning"""
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone, timedelta
from enum import Enum

class ImageLifecycleStage(str, Enum):
    ACTIVE = "active"
    DEPRECATED = "deprecated"
    TOMBSTONED = "tombstoned"

class DeprecationPolicy:
    """Policy for deprecating images"""
    
    def __init__(self, max_age_days: int = 90, critical_cve_threshold: int = 0, high_cve_threshold: int = 5):
        self.max_age_days = max_age_days
        self.critical_cve_threshold = critical_cve_threshold
        self.high_cve_threshold = high_cve_threshold
    
    def should_deprecate(self, image_data: Dict[str, Any]) -> tuple[bool, List[str]]:
        """Determine if an image should be deprecated"""
        reasons = []
        
        # Check age
        if image_data.get('completed_at'):
            try:
                completed = datetime.fromisoformat(image_data['completed_at']) if isinstance(image_data['completed_at'], str) else image_data['completed_at']
                age_days = (datetime.now(timezone.utc) - completed).days
                
                if age_days > self.max_age_days:
                    reasons.append(f"Image is {age_days} days old (max allowed: {self.max_age_days})")
            except:
                pass
        
        # Check CVEs
        vuln_count = image_data.get('vulnerability_count', {})
        critical = vuln_count.get('CRITICAL', 0)
        high = vuln_count.get('HIGH', 0)
        
        if critical > self.critical_cve_threshold:
            reasons.append(f"Has {critical} CRITICAL CVEs (threshold: {self.critical_cve_threshold})")
        
        if high > self.high_cve_threshold:
            reasons.append(f"Has {high} HIGH CVEs (threshold: {self.high_cve_threshold})")
        
        return len(reasons) > 0, reasons

class LifecycleManager:
    """Manage image lifecycle"""
    
    def __init__(self, policy: Optional[DeprecationPolicy] = None):
        self.policy = policy or DeprecationPolicy()
        self.lifecycle_states: Dict[str, Dict[str, Any]] = {}
    
    def evaluate_image(self, build_id: str, image_data: Dict[str, Any]) -> Dict[str, Any]:
        """Evaluate image lifecycle stage"""
        should_deprecate, reasons = self.policy.should_deprecate(image_data)
        
        current_stage = ImageLifecycleStage.ACTIVE
        if should_deprecate:
            current_stage = ImageLifecycleStage.DEPRECATED
        
        # Check if should be tombstoned (critical vulnerabilities + old)
        if reasons and any('CRITICAL' in r for r in reasons):
            vuln_count = image_data.get('vulnerability_count', {})
            if vuln_count.get('CRITICAL', 0) > 2:
                current_stage = ImageLifecycleStage.TOMBSTONED
        
        lifecycle_info = {
            "build_id": build_id,
            "stage": current_stage.value,
            "should_deprecate": should_deprecate,
            "deprecation_reasons": reasons,
            "evaluated_at": datetime.now(timezone.utc).isoformat(),
            "actions_required": self._get_required_actions(current_stage)
        }
        
        self.lifecycle_states[build_id] = lifecycle_info
        return lifecycle_info
    
    def tombstone_image(self, build_id: str, reason: str) -> Dict[str, Any]:
        """Tombstone an image to prevent pulls"""
        tombstone_record = {
            "build_id": build_id,
            "stage": ImageLifecycleStage.TOMBSTONED.value,
            "tombstoned_at": datetime.now(timezone.utc).isoformat(),
            "reason": reason,
            "pull_blocked": True,
            "replacement_available": True,
            "message": f"This image has been tombstoned: {reason}. Please use the latest hardened version."
        }
        
        self.lifecycle_states[build_id] = tombstone_record
        return tombstone_record
    
    def get_deprecated_images(self) -> List[Dict[str, Any]]:
        """Get all deprecated images"""
        return [
            info for info in self.lifecycle_states.values()
            if info.get('stage') == ImageLifecycleStage.DEPRECATED.value
        ]
    
    def get_tombstoned_images(self) -> List[Dict[str, Any]]:
        """Get all tombstoned images"""
        return [
            info for info in self.lifecycle_states.values()
            if info.get('stage') == ImageLifecycleStage.TOMBSTONED.value
        ]
    
    def _get_required_actions(self, stage: ImageLifecycleStage) -> List[str]:
        """Get required actions for lifecycle stage"""
        actions = {
            ImageLifecycleStage.ACTIVE: [],
            ImageLifecycleStage.DEPRECATED: [
                "Update to latest hardened version",
                "Schedule migration during next deployment window",
                "Review CVE remediation options"
            ],
            ImageLifecycleStage.TOMBSTONED: [
                "IMMEDIATE ACTION REQUIRED: Stop using this image",
                "Deploy latest hardened replacement",
                "Critical vulnerabilities must be addressed"
            ]
        }
        return actions.get(stage, [])

def run_garbage_collection(lifecycle_manager: LifecycleManager, builds: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Run garbage collection on all builds"""
    deprecated = 0
    tombstoned = 0
    
    for build in builds:
        result = lifecycle_manager.evaluate_image(build['id'], build)
        if result['stage'] == ImageLifecycleStage.DEPRECATED.value:
            deprecated += 1
        elif result['stage'] == ImageLifecycleStage.TOMBSTONED.value:
            tombstoned += 1
    
    return {
        "total_evaluated": len(builds),
        "deprecated": deprecated,
        "tombstoned": tombstoned,
        "active": len(builds) - deprecated - tombstoned,
        "run_at": datetime.now(timezone.utc).isoformat()
    }