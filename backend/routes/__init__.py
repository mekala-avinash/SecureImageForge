"""
API Routes for SecureImage Forge

This module provides modular route organization.
Routes are progressively being moved from server.py to individual files.
"""
from fastapi import APIRouter

# Import routers from individual modules
from .analytics import router as analytics_router
from .policies import router as policies_router
from .registries import router as registries_router
from .remediation import router as remediation_router
from .exceptions import router as exceptions_router
from .drift import router as drift_router
from .slsa import router as slsa_router
from .vex import router as vex_router
from .webhooks import router as webhooks_router
from .config import router as config_router

# Note: builds_router is a stub - core build routes remain in server.py
# from .builds import router as builds_router

__all__ = [
    'analytics_router', 
    'policies_router',
    'registries_router',
    'remediation_router',
    'exceptions_router',
    'drift_router',
    'slsa_router',
    'vex_router',
    'webhooks_router',
    'config_router'
]
