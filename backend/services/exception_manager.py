"""Exception Management System for Policy Deviations"""
from typing import Dict, List, Any, Optional
from datetime import datetime, timezone, timedelta
from enum import Enum
import uuid

class ExceptionStatus(str, Enum):
    PENDING = "pending"
    APPROVED = "approved"
    REJECTED = "rejected"
    EXPIRED = "expired"

class ExceptionRequest:
    """Request for Deviation from Security Policy"""
    
    def __init__(self, build_id: str, policy_id: str, requestor: str, justification: str, duration_days: int = 30):
        self.id = str(uuid.uuid4())
        self.build_id = build_id
        self.policy_id = policy_id
        self.requestor = requestor
        self.justification = justification
        self.duration_days = duration_days
        self.status = ExceptionStatus.PENDING
        self.created_at = datetime.now(timezone.utc)
        self.expires_at = self.created_at + timedelta(days=duration_days)
        self.approver = None
        self.approved_at = None
        self.rejection_reason = None
    
    def approve(self, approver: str, notes: Optional[str] = None):
        """Approve the exception request"""
        self.status = ExceptionStatus.APPROVED
        self.approver = approver
        self.approved_at = datetime.now(timezone.utc)
        self.approval_notes = notes
    
    def reject(self, approver: str, reason: str):
        """Reject the exception request"""
        self.status = ExceptionStatus.REJECTED
        self.approver = approver
        self.approved_at = datetime.now(timezone.utc)
        self.rejection_reason = reason
    
    def is_expired(self) -> bool:
        """Check if exception has expired"""
        if self.status != ExceptionStatus.APPROVED:
            return False
        return datetime.now(timezone.utc) > self.expires_at
    
    def to_dict(self) -> Dict[str, Any]:
        return {
            "id": self.id,
            "build_id": self.build_id,
            "policy_id": self.policy_id,
            "requestor": self.requestor,
            "justification": self.justification,
            "status": self.status.value,
            "duration_days": self.duration_days,
            "created_at": self.created_at.isoformat(),
            "expires_at": self.expires_at.isoformat() if self.status == ExceptionStatus.APPROVED else None,
            "approver": self.approver,
            "approved_at": self.approved_at.isoformat() if self.approved_at else None,
            "rejection_reason": self.rejection_reason,
            "is_expired": self.is_expired()
        }

class ExceptionManager:
    """Manage policy exception workflow"""
    
    def __init__(self):
        self.exceptions: Dict[str, ExceptionRequest] = {}
    
    def create_request(self, build_id: str, policy_id: str, requestor: str, 
                      justification: str, duration_days: int = 30) -> ExceptionRequest:
        """Create a new exception request"""
        request = ExceptionRequest(build_id, policy_id, requestor, justification, duration_days)
        self.exceptions[request.id] = request
        return request
    
    def get_pending_requests(self) -> List[ExceptionRequest]:
        """Get all pending exception requests"""
        return [req for req in self.exceptions.values() if req.status == ExceptionStatus.PENDING]
    
    def get_active_exceptions(self, build_id: Optional[str] = None) -> List[ExceptionRequest]:
        """Get all active (approved and not expired) exceptions"""
        active = []
        for req in self.exceptions.values():
            if req.status == ExceptionStatus.APPROVED and not req.is_expired():
                if build_id is None or req.build_id == build_id:
                    active.append(req)
        return active
    
    def check_exception_exists(self, build_id: str, policy_id: str) -> bool:
        """Check if an active exception exists for a build/policy combination"""
        for req in self.get_active_exceptions(build_id):
            if req.policy_id == policy_id:
                return True
        return False

# Common exception request templates
EXCEPTION_TEMPLATES = {
    "debug_shell_access": {
        "title": "Debug Shell Access Request",
        "description": "Request shell access for debugging purposes",
        "default_duration_days": 7,
        "requires_approval_from": ["security_lead", "engineering_manager"],
        "justification_required": True
    },
    "root_user_access": {
        "title": "Root User Access Request",
        "description": "Request root user privileges for specific operations",
        "default_duration_days": 3,
        "requires_approval_from": ["ciso", "security_lead"],
        "justification_required": True
    },
    "legacy_base_image": {
        "title": "Legacy Base Image Exception",
        "description": "Use non-approved base image for legacy compatibility",
        "default_duration_days": 90,
        "requires_approval_from": ["security_lead", "architecture_lead"],
        "justification_required": True
    },
    "cve_threshold_override": {
        "title": "CVE Threshold Override",
        "description": "Deploy image exceeding CVE thresholds with mitigation plan",
        "default_duration_days": 14,
        "requires_approval_from": ["security_lead"],
        "justification_required": True
    }
}

def get_exception_templates() -> Dict[str, Dict[str, Any]]:
    """Get all exception request templates"""
    return EXCEPTION_TEMPLATES