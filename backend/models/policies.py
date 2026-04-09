"""
Policy-related Pydantic models
"""
from datetime import datetime, timezone
from typing import Any, Dict
import uuid

from pydantic import BaseModel, ConfigDict, Field


class Policy(BaseModel):
    model_config = ConfigDict(extra="ignore")
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str
    description: str
    type: str  # vulnerability, compliance, configuration, security, freshness
    enforcement: str  # block, warn, info
    rule: Dict[str, Any]
    enabled: bool = True
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class PolicyCreate(BaseModel):
    name: str
    description: str
    type: str
    enforcement: str
    rule: Dict[str, Any]
    enabled: bool = True
