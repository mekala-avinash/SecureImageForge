"""
Build routes - stub file for future expansion
Core build routes remain in server.py for now due to complex dependencies
"""
from fastapi import APIRouter

router = APIRouter(tags=["builds"])

# NOTE: Build routes are currently in server.py due to complex dependencies
# This file is a placeholder for future refactoring
# Key routes to eventually move here:
# - POST /builds
# - GET /builds
# - GET /builds/{build_id}
# - GET /builds/{build_id}/scan
# - GET /builds/{build_id}/compliance
# - GET /builds/{build_id}/sbom
# - GET /builds/{build_id}/health
# - POST /builds/{build_id}/sign
# - GET /builds/{build_id}/signature
