"""
Analytics routes
"""
from fastapi import APIRouter
from datetime import datetime, timezone, timedelta
from typing import Dict, Any

from database import db

router = APIRouter(prefix="/analytics", tags=["analytics"])


@router.get("/trends")
async def get_analytics_trends(days: int = 30):
    """Get build trends over time"""
    end_date = datetime.now(timezone.utc)
    start_date = end_date - timedelta(days=days)
    
    builds = await db.build_history.find({
        "started_at": {"$gte": start_date.isoformat()}
    }, {"_id": 0}).to_list(1000)
    
    # Group by date
    daily_data = {}
    for build in builds:
        started_at = build.get('started_at')
        if isinstance(started_at, str):
            date_key = started_at[:10]
        else:
            date_key = started_at.strftime('%Y-%m-%d')
        
        if date_key not in daily_data:
            daily_data[date_key] = {
                "date": date_key,
                "total": 0,
                "completed": 0,
                "failed": 0,
                "avg_compliance": 0,
                "compliance_scores": []
            }
        
        daily_data[date_key]["total"] += 1
        if build.get("status") == "completed":
            daily_data[date_key]["completed"] += 1
            if build.get("compliance_score"):
                daily_data[date_key]["compliance_scores"].append(build["compliance_score"])
        elif build.get("status") == "failed":
            daily_data[date_key]["failed"] += 1
    
    # Calculate averages
    for date_key in daily_data:
        scores = daily_data[date_key]["compliance_scores"]
        if scores:
            daily_data[date_key]["avg_compliance"] = sum(scores) // len(scores)
        del daily_data[date_key]["compliance_scores"]
    
    trend_data = sorted(daily_data.values(), key=lambda x: x["date"])
    
    return {
        "period": f"Last {days} days",
        "trend_data": trend_data,
        "summary": {
            "total_builds": sum(d["total"] for d in trend_data),
            "total_completed": sum(d["completed"] for d in trend_data),
            "total_failed": sum(d["failed"] for d in trend_data)
        }
    }


@router.get("/vulnerabilities")
async def get_vulnerability_analytics():
    """Get vulnerability analytics across all builds"""
    scans = await db.scan_results.find({}, {"_id": 0}).to_list(100)
    
    severity_totals = {"critical": 0, "high": 0, "medium": 0, "low": 0}
    top_cves = {}
    
    for scan in scans:
        total_count = scan.get("total_count", {})
        for severity, count in total_count.items():
            if severity in severity_totals:
                severity_totals[severity] += count
        
        vulnerabilities = scan.get("vulnerabilities", {})
        for severity, vulns in vulnerabilities.items():
            for vuln in vulns:
                cve_id = vuln.get("id", "unknown")
                if cve_id not in top_cves:
                    top_cves[cve_id] = {"id": cve_id, "count": 0, "severity": severity}
                top_cves[cve_id]["count"] += 1
    
    return {
        "severity_distribution": severity_totals,
        "top_vulnerabilities": sorted(top_cves.values(), key=lambda x: x["count"], reverse=True)[:10],
        "total_scans": len(scans)
    }


@router.get("/health-scores")
async def get_health_score_analytics():
    """Get health score distribution"""
    scores = await db.health_scores.find({}, {"_id": 0}).to_list(1000)
    
    distribution = {"A": 0, "B": 0, "C": 0, "D": 0, "F": 0}
    total_score = 0
    
    for score in scores:
        grade = score.get("grade", "F")
        if grade in distribution:
            distribution[grade] += 1
        total_score += score.get("score", 0)
    
    return {
        "grade_distribution": distribution,
        "average_score": total_score // len(scores) if scores else 0,
        "total_assessments": len(scores)
    }


@router.get("/success-rate")
async def get_success_rate_analytics(days: int = 30):
    """Get build success rate over time"""
    end_date = datetime.now(timezone.utc)
    start_date = end_date - timedelta(days=days)
    
    total = await db.build_history.count_documents({
        "started_at": {"$gte": start_date.isoformat()}
    })
    
    completed = await db.build_history.count_documents({
        "started_at": {"$gte": start_date.isoformat()},
        "status": "completed"
    })
    
    failed = await db.build_history.count_documents({
        "started_at": {"$gte": start_date.isoformat()},
        "status": "failed"
    })
    
    return {
        "period": f"Last {days} days",
        "total_builds": total,
        "completed": completed,
        "failed": failed,
        "success_rate": round((completed / total * 100) if total > 0 else 0, 2),
        "failure_rate": round((failed / total * 100) if total > 0 else 0, 2)
    }
