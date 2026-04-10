"""
Config routes (runtime versions, base images, CIS levels, etc.)
"""
from fastapi import APIRouter, HTTPException
from typing import Dict, Any

from services.version_matrix import (
    RUNTIME_VERSIONS,
    BASE_IMAGE_TAGS,
    CIS_LEVELS,
    SBOM_FORMATS,
    SBOM_SCAN_DEPTHS,
    get_runtime_versions,
    validate_runtime_config
)

router = APIRouter(tags=["config"])


@router.get("/architectures")
async def get_supported_architectures():
    """Get supported build architectures"""
    return {
        "supported": ["amd64", "arm64"],
        "default": "amd64",
        "multi_arch_builds": True
    }


@router.get("/runtime-versions")
async def get_runtime_version_matrix():
    """Get complete runtime version and distribution matrix"""
    return {"runtimes": RUNTIME_VERSIONS}


@router.get("/runtime-versions/{runtime}")
async def get_runtime_specific_versions(runtime: str):
    """Get versions and distributions for a specific runtime"""
    data = get_runtime_versions(runtime)
    if not data:
        raise HTTPException(status_code=404, detail=f"Runtime {runtime} not found")
    return data


@router.get("/base-image-tags")
async def get_base_image_tag_catalog():
    """Get available tags for base images"""
    return {"base_images": BASE_IMAGE_TAGS}


@router.get("/base-image-tags/{base_image}")
async def get_specific_base_tags(base_image: str):
    """Get tags for a specific base image"""
    data = BASE_IMAGE_TAGS.get(base_image)
    if not data:
        raise HTTPException(status_code=404, detail=f"Base image {base_image} not found")
    return data


@router.post("/validate-runtime-config")
async def validate_runtime_configuration(config: Dict[str, Any]):
    """Validate runtime configuration for compatibility"""
    runtime = config.get("runtime")
    version = config.get("runtime_version")
    distribution = config.get("runtime_distribution")
    fips_mode = config.get("fips_mode_enabled", False)
    
    if not all([runtime, version, distribution]):
        raise HTTPException(status_code=400, detail="Missing required fields: runtime, runtime_version, runtime_distribution")
    
    validation = validate_runtime_config(runtime, version, distribution, fips_mode)
    return validation


@router.get("/cis-levels")
async def get_cis_benchmark_levels():
    """Get CIS Benchmark level configurations"""
    return {"levels": CIS_LEVELS}


@router.get("/sbom-formats")
async def get_sbom_format_options():
    """Get SBOM format and scan depth options"""
    return {
        "formats": SBOM_FORMATS,
        "scan_depths": SBOM_SCAN_DEPTHS
    }
