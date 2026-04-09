"""Health Score Calculation Engine"""
from datetime import datetime, timezone
from typing import Dict, Any

def calculate_health_score(build_data: Dict[str, Any]) -> int:
    """
    Calculate overall health score for a container image
    
    Scoring factors:
    - Vulnerability count (0-40 points)
    - Compliance score (0-30 points)
    - Image age (0-15 points)
    - Build success (0-15 points)
    """
    score = 0
    
    # Vulnerability scoring (40 points max)
    vuln_count = build_data.get('vulnerability_count', {})
    critical = vuln_count.get('CRITICAL', 0)
    high = vuln_count.get('HIGH', 0)
    medium = vuln_count.get('MEDIUM', 0)
    low = vuln_count.get('LOW', 0)
    
    vuln_score = 40
    vuln_score -= critical * 10  # -10 per critical
    vuln_score -= high * 5       # -5 per high
    vuln_score -= medium * 2     # -2 per medium
    vuln_score -= low * 0.5      # -0.5 per low
    vuln_score = max(0, vuln_score)
    score += vuln_score
    
    # Compliance scoring (30 points max)
    compliance = build_data.get('compliance_score', 0)
    compliance_score = (compliance / 100) * 30
    score += compliance_score
    
    # Build success (15 points)
    if build_data.get('status') == 'completed':
        score += 15
    elif build_data.get('status') == 'failed':
        score += 0
    else:
        score += 7.5
    
    # Image freshness (15 points) - newer is better
    if build_data.get('completed_at'):
        try:
            if isinstance(build_data['completed_at'], str):
                completed = datetime.fromisoformat(build_data['completed_at'])
            else:
                completed = build_data['completed_at']
            
            age_days = (datetime.now(timezone.utc) - completed).days
            
            if age_days < 7:
                score += 15
            elif age_days < 30:
                score += 10
            elif age_days < 90:
                score += 5
            else:
                score += 2
        except:
            score += 5
    else:
        score += 5
    
    return min(100, int(score))

def get_health_grade(score: int) -> str:
    """Convert health score to letter grade"""
    if score >= 90:
        return 'A'
    elif score >= 80:
        return 'B'
    elif score >= 70:
        return 'C'
    elif score >= 60:
        return 'D'
    else:
        return 'F'

def get_health_status(score: int) -> str:
    """Get health status description"""
    if score >= 90:
        return 'Excellent'
    elif score >= 80:
        return 'Good'
    elif score >= 70:
        return 'Fair'
    elif score >= 60:
        return 'Poor'
    else:
        return 'Critical'