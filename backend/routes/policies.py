"""
Policy routes
"""
from fastapi import APIRouter, HTTPException
from datetime import datetime
from typing import List

from database import db
from models.policies import Policy, PolicyCreate
from services.policy_engine import evaluate_all_policies, POLICY_TEMPLATES, get_policy_recommendation

router = APIRouter(prefix="/policies", tags=["policies"])


@router.post("", response_model=Policy)
async def create_policy(policy: PolicyCreate):
    """Create a new custom policy"""
    policy_obj = Policy(**policy.model_dump())
    policy_dict = policy_obj.model_dump()
    policy_dict['created_at'] = policy_dict['created_at'].isoformat()
    
    await db.policies.insert_one(policy_dict)
    return policy_obj


@router.get("", response_model=List[Policy])
async def get_policies():
    """Get all policies"""
    policies = await db.policies.find({}, {"_id": 0}).to_list(1000)
    
    for policy in policies:
        if isinstance(policy.get('created_at'), str):
            policy['created_at'] = datetime.fromisoformat(policy['created_at'])
    
    return policies


@router.get("/templates")
async def get_policy_templates():
    """Get pre-built policy templates"""
    return {"templates": POLICY_TEMPLATES}


@router.get("/{policy_id}")
async def get_policy(policy_id: str):
    """Get specific policy"""
    policy = await db.policies.find_one({"id": policy_id}, {"_id": 0})
    
    if not policy:
        raise HTTPException(status_code=404, detail="Policy not found")
    
    if isinstance(policy.get('created_at'), str):
        policy['created_at'] = datetime.fromisoformat(policy['created_at'])
    
    return policy


@router.put("/{policy_id}")
async def update_policy(policy_id: str, policy_update: PolicyCreate):
    """Update a policy"""
    result = await db.policies.update_one(
        {"id": policy_id},
        {"$set": policy_update.model_dump()}
    )
    
    if result.modified_count == 0:
        raise HTTPException(status_code=404, detail="Policy not found")
    
    return {"message": "Policy updated successfully"}


@router.delete("/{policy_id}")
async def delete_policy(policy_id: str):
    """Delete a policy"""
    result = await db.policies.delete_one({"id": policy_id})
    
    if result.deleted_count == 0:
        raise HTTPException(status_code=404, detail="Policy not found")
    
    return {"message": "Policy deleted successfully"}


@router.post("/{policy_id}/toggle")
async def toggle_policy(policy_id: str):
    """Enable/disable a policy"""
    policy = await db.policies.find_one({"id": policy_id}, {"_id": 0})
    
    if not policy:
        raise HTTPException(status_code=404, detail="Policy not found")
    
    new_state = not policy.get('enabled', True)
    await db.policies.update_one(
        {"id": policy_id},
        {"$set": {"enabled": new_state}}
    )
    
    return {"enabled": new_state}
