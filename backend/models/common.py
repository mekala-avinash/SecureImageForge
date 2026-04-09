"""
Common/shared Pydantic models
"""
from datetime import datetime, timezone
import uuid

from pydantic import BaseModel, ConfigDict, Field


class Registry(BaseModel):
    model_config = ConfigDict(extra="ignore")
    id: str = Field(default_factory=lambda: str(uuid.uuid4()))
    name: str
    type: str  # jfrog, acr, dockerhub
    url: str
    username: str
    password: str  # In production, encrypt this
    created_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))


class RegistryCreate(BaseModel):
    name: str
    type: str
    url: str
    username: str
    password: str
