"""
Analytics routes
"""
from fastapi import APIRouter
from datetime import datetime, timezone, timedelta

from database import db
from services.health_score import calculate_health_score, get_health_grade

router = APIRouter(prefix="/analytics", tags=["analytics"])


@router.get("/trends")
async def get_analytics_trends(days: int = 30):
    """Get build and health score trends"""
    start_date = datetime.now(timezone.utc) - timedelta(days=days)
    
    # Get builds from the period
    builds = await db.build_history.find({
        "started_at": {"$gte": start_date.isoformat()}
    }, {"_id": 0}).to_list(1000)
    
    # Convert datetime strings
    for build in builds:
        if isinstance(build.get('started_at'), str):
            build['started_at'] = datetime.fromisoformat(build['started_at'])
    
    # Group by day
    daily_data = {}
    for build in builds:
        day = build['started_at'].date().isoformat()
        if day not in daily_data:
            daily_data[day] = {
                "date": day,
                "total": 0,
                "completed": 0,
                "failed": 0,
                "avg_compliance": 0,
                "compliance_scores": []
            }
        
        daily_data[day]["total"] += 1
        if build.get('status') == 'completed':
            daily_data[day]["completed"] += 1
        elif build.get('status') == 'failed':
            daily_data[day]["failed"] += 1
        
        if build.get('compliance_score'):
            daily_data[day]["compliance_scores"].append(build['compliance_score'])
    
    # Calculate averages
    trend_data = []
    for day_data in daily_data.values():
        if day_data["compliance_scores"]:
            day_data["avg_compliance"] = int(sum(day_data["compliance_scores"]) / len(day_data["compliance_scores"]))
        day_data.pop("compliance_scores")
        trend_data.append(day_data)
    
    trend_data.sort(key=lambda x: x["date"])
    
    return {
        "period_days": days,
        "trend_data": trend_data
    }


@router.get("/vulnerabilities")
async def get_vulnerability_analytics():
    """Get vulnerability trends across all builds"""
    completed_builds = await db.build_history.find({
        "status": "completed",
        "vulnerability_count": {"$exists": True}
    }, {"_id": 0}).to_list(1000)
    
    total_vulns = {"CRITICAL": 0, "HIGH": 0, "MEDIUM": 0, "LOW": 0}
    vuln_by_runtime = {}
    
    for build in completed_builds:
        vuln_count = build.get('vulnerability_count', {})
        for severity in total_vulns.keys():
            total_vulns[severity] += vuln_count.get(severity, 0)
        
        # Get config to find runtime
        config = await db.build_configs.find_one({"id": build['config_id']}, {"_id": 0})
        if config:
            runtime = config.get('runtime', 'unknown')
            if runtime not in vuln_by_runtime:
                vuln_by_runtime[runtime] = {"CRITICAL": 0, "HIGH": 0, "MEDIUM": 0, "LOW": 0}
            
            for severity in total_vulns.keys():
                vuln_by_runtime[runtime][severity] += vuln_count.get(severity, 0)
    
    return {
        "total_vulnerabilities": total_vulns,
        "by_runtime": vuln_by_runtime,
        "total_builds_analyzed": len(completed_builds)
    }


@router.get("/health-scores")
async def get_health_score_analytics():
    """Get health score distribution and trends"""
    completed_builds = await db.build_history.find({
        "status": "completed"
    }, {"_id": 0}).to_list(1000)
    
    scores = []
    grades = {"A": 0, "B": 0, "C": 0, "D": 0, "F": 0}
    
    for build in completed_builds:
        # Convert datetime strings
        if isinstance(build.get('started_at'), str):
            build['started_at'] = datetime.fromisoformat(build['started_at'])
        if build.get('completed_at') and isinstance(build['completed_at'], str):
            build['completed_at'] = datetime.fromisoformat(build['completed_at'])
        
        score = calculate_health_score(build)
        grade = get_health_grade(score)
        scores.append(score)
        grades[grade] = grades.get(grade, 0) + 1
    
    avg_score = int(sum(scores) / len(scores)) if scores else 0
    
    return {
        "average_health_score": avg_score,
        "grade_distribution": grades,
        "total_builds": len(completed_builds)
    }


@router.get("/success-rate")
async def get_success_rate_analytics(days: int = 30):
    """Get build success rate over time"""
    start_date = datetime.now(timezone.utc) - timedelta(days=days)
    
    builds = await db.build_history.find({
        "started_at": {"$gte": start_date.isoformat()}
    }, {"_id": 0}).to_list(1000)
    
    total = len(builds)
    completed = sum(1 for b in builds if b.get('status') == 'completed')
    failed = sum(1 for b in builds if b.get('status') == 'failed')
    
    success_rate = (completed / total * 100) if total > 0 else 0
    
    return {
        "period_days": days,
        "total_builds": total,
        "completed": completed,
        "failed": failed,
        "success_rate": round(success_rate, 2)
    }
