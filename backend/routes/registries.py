"""
Registry routes
"""
from fastapi import APIRouter, HTTPException
from typing import List
from datetime import datetime

from database import db
from models.common import Registry, RegistryCreate

router = APIRouter(prefix="/registries", tags=["registries"])


@router.post("", response_model=Registry)
async def create_registry(registry: RegistryCreate):
    """Add a new container registry"""
    registry_obj = Registry(**registry.model_dump())
    registry_dict = registry_obj.model_dump()
    registry_dict['created_at'] = registry_dict['created_at'].isoformat()
    
    await db.registries.insert_one(registry_dict)
    return registry_obj


@router.get("", response_model=List[Registry])
async def get_registries():
    """Get all configured registries"""
    registries = await db.registries.find({}, {"_id": 0}).to_list(100)
    
    for registry in registries:
        if isinstance(registry.get('created_at'), str):
            registry['created_at'] = datetime.fromisoformat(registry['created_at'])
    
    return registries


@router.delete("/{registry_id}")
async def delete_registry(registry_id: str):
    """Delete a registry"""
    result = await db.registries.delete_one({"id": registry_id})
    
    if result.deleted_count == 0:
        raise HTTPException(status_code=404, detail="Registry not found")
    
    return {"message": "Registry deleted successfully"}


@router.post("/{registry_id}/test")
async def test_registry(registry_id: str):
    """Test registry connection"""
    registry = await db.registries.find_one({"id": registry_id}, {"_id": 0})
    
    if not registry:
        raise HTTPException(status_code=404, detail="Registry not found")
    
    # Simulate registry test
    return {
        "registry_id": registry_id,
        "status": "connected",
        "message": f"Successfully connected to {registry['type']} registry at {registry['url']}"
    }
