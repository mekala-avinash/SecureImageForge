"""
Config routes (runtime versions, base images, CIS levels, etc.)
"""
from fastapi import APIRouter, HTTPException
from typing import Dict, Any

from services.version_matrix import (
    get_runtime_version_matrix,
    get_base_image_tag_catalog,
    validate_runtime_config,
    get_cis_level_configs,
    get_sbom_format_options
)

router = APIRouter(tags=["config"])


@router.get("/architectures")
async def get_supported_architectures():
    """Get supported build architectures"""
    return {
        "architectures": [
            {"id": "amd64", "name": "AMD64/x86_64", "description": "Standard x86 64-bit"},
            {"id": "arm64", "name": "ARM64/aarch64", "description": "ARM 64-bit (Apple Silicon, AWS Graviton)"}
        ]
    }


@router.get("/runtime-versions")
async def get_runtime_version_matrix_endpoint():
    """Get all runtime version matrices"""
    return get_runtime_version_matrix()


@router.get("/runtime-versions/{runtime}")
async def get_runtime_specific_versions(runtime: str):
    """Get versions for a specific runtime"""
    matrix = get_runtime_version_matrix()
    if runtime not in matrix:
        raise HTTPException(status_code=404, detail=f"Runtime '{runtime}' not found")
    return {runtime: matrix[runtime]}


@router.get("/base-image-tags")
async def get_base_image_tag_catalog_endpoint():
    """Get all base image tag options"""
    return get_base_image_tag_catalog()


@router.get("/base-image-tags/{base_image}")
async def get_specific_base_tags(base_image: str):
    """Get tags for a specific base image"""
    catalog = get_base_image_tag_catalog()
    if base_image not in catalog:
        raise HTTPException(status_code=404, detail=f"Base image '{base_image}' not found")
    return {base_image: catalog[base_image]}


@router.post("/validate-runtime-config")
async def validate_runtime_configuration(config: Dict[str, Any]):
    """Validate a runtime configuration combination"""
    result = validate_runtime_config(
        runtime=config.get("runtime"),
        version=config.get("version"),
        distribution=config.get("distribution"),
        base_image=config.get("base_image"),
        base_tag=config.get("base_tag"),
        fips_enabled=config.get("fips_enabled", False)
    )
    
    if not result["valid"]:
        raise HTTPException(status_code=400, detail=result["errors"])
    
    return result


@router.get("/cis-levels")
async def get_cis_levels():
    """Get CIS benchmark level configurations"""
    return {"levels": get_cis_level_configs()}


@router.get("/sbom-formats")
async def get_sbom_formats():
    """Get SBOM format and scan depth options"""
    return get_sbom_format_options()
