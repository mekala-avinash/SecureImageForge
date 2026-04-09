"""
Phase 4 Feature Tests: Exception Management, Drift Detection, Remediation Policies
Tests for SecureImage Forge Phase 4 features
"""
import pytest
import requests
import os
import uuid

BASE_URL = os.environ.get('REACT_APP_BACKEND_URL', '').rstrip('/')

class TestExceptionManagement:
    """Exception Management endpoint tests"""
    
    def test_get_exception_templates(self):
        """Test GET /api/exceptions/templates returns templates"""
        response = requests.get(f"{BASE_URL}/api/exceptions/templates")
        assert response.status_code == 200
        
        data = response.json()
        assert "templates" in data
        templates = data["templates"]
        assert isinstance(templates, dict)
        print(f"PASS: Got {len(templates)} exception templates")
    
    def test_list_exceptions_empty_or_existing(self):
        """Test GET /api/exceptions returns list with counts"""
        response = requests.get(f"{BASE_URL}/api/exceptions")
        assert response.status_code == 200
        
        data = response.json()
        assert "exceptions" in data
        assert "counts" in data
        assert "pending" in data["counts"]
        assert "approved" in data["counts"]
        assert "rejected" in data["counts"]
        assert "total" in data["counts"]
        print(f"PASS: Exceptions list returned with counts: {data['counts']}")
    
    def test_create_exception_request(self):
        """Test POST /api/exceptions creates new exception"""
        exception_data = {
            "build_id": f"TEST_build_{uuid.uuid4().hex[:8]}",
            "policy_id": "cis-benchmark-level1",
            "requestor": "test_user@example.com",
            "justification": "Testing exception workflow for Phase 4",
            "duration_days": 30
        }
        
        response = requests.post(f"{BASE_URL}/api/exceptions", json=exception_data)
        assert response.status_code == 200
        
        data = response.json()
        assert "id" in data
        assert data["status"] == "pending"
        assert "message" in data
        
        # Store for later tests
        self.__class__.created_exception_id = data["id"]
        print(f"PASS: Created exception with ID: {data['id']}")
        return data["id"]
    
    def test_get_exception_by_id(self):
        """Test GET /api/exceptions/{id} returns exception details"""
        # First create an exception
        exception_data = {
            "build_id": f"TEST_build_{uuid.uuid4().hex[:8]}",
            "policy_id": "vulnerability-threshold",
            "requestor": "test_user@example.com",
            "justification": "Testing get exception by ID"
        }
        
        create_response = requests.post(f"{BASE_URL}/api/exceptions", json=exception_data)
        assert create_response.status_code == 200
        exception_id = create_response.json()["id"]
        
        # Now get it
        response = requests.get(f"{BASE_URL}/api/exceptions/{exception_id}")
        assert response.status_code == 200
        
        data = response.json()
        assert data["id"] == exception_id
        assert data["status"] == "pending"
        assert data["requestor"] == "test_user@example.com"
        print(f"PASS: Retrieved exception {exception_id}")
    
    def test_approve_exception(self):
        """Test POST /api/exceptions/{id}/approve approves exception"""
        # Create exception first
        exception_data = {
            "build_id": f"TEST_build_{uuid.uuid4().hex[:8]}",
            "policy_id": "shell-removal",
            "requestor": "test_user@example.com",
            "justification": "Testing approval workflow",
            "duration_days": 14
        }
        
        create_response = requests.post(f"{BASE_URL}/api/exceptions", json=exception_data)
        assert create_response.status_code == 200
        exception_id = create_response.json()["id"]
        
        # Approve it
        approval_data = {
            "approver": "security_admin",
            "notes": "Approved for testing purposes"
        }
        
        response = requests.post(f"{BASE_URL}/api/exceptions/{exception_id}/approve", json=approval_data)
        assert response.status_code == 200
        
        data = response.json()
        assert data["status"] == "approved"
        assert "expires_at" in data
        print(f"PASS: Exception {exception_id} approved, expires at {data['expires_at']}")
    
    def test_reject_exception(self):
        """Test POST /api/exceptions/{id}/reject rejects exception"""
        # Create exception first
        exception_data = {
            "build_id": f"TEST_build_{uuid.uuid4().hex[:8]}",
            "policy_id": "root-user-check",
            "requestor": "test_user@example.com",
            "justification": "Testing rejection workflow"
        }
        
        create_response = requests.post(f"{BASE_URL}/api/exceptions", json=exception_data)
        assert create_response.status_code == 200
        exception_id = create_response.json()["id"]
        
        # Reject it
        rejection_data = {
            "approver": "security_admin",
            "reason": "Does not meet security requirements"
        }
        
        response = requests.post(f"{BASE_URL}/api/exceptions/{exception_id}/reject", json=rejection_data)
        assert response.status_code == 200
        
        data = response.json()
        assert data["status"] == "rejected"
        assert "reason" in data
        print(f"PASS: Exception {exception_id} rejected with reason: {data['reason']}")
    
    def test_filter_exceptions_by_status(self):
        """Test GET /api/exceptions?status=pending filters correctly"""
        response = requests.get(f"{BASE_URL}/api/exceptions?status=pending")
        assert response.status_code == 200
        
        data = response.json()
        # All returned exceptions should be pending
        for exc in data["exceptions"]:
            assert exc["status"] == "pending"
        print(f"PASS: Filtered {len(data['exceptions'])} pending exceptions")


class TestDriftDetection:
    """Drift Detection endpoint tests"""
    
    def test_get_runtime_images(self):
        """Test GET /api/drift/runtime-images returns simulated K8s images"""
        response = requests.get(f"{BASE_URL}/api/drift/runtime-images")
        assert response.status_code == 200
        
        data = response.json()
        assert "images" in data
        assert "total_count" in data
        assert "clusters" in data
        assert isinstance(data["images"], list)
        
        # Check image structure
        if data["images"]:
            image = data["images"][0]
            assert "image_id" in image or "image_tag" in image
        
        print(f"PASS: Got {data['total_count']} runtime images from {len(data['clusters'])} clusters")
    
    def test_run_drift_scan(self):
        """Test GET /api/drift/scan runs drift detection"""
        response = requests.get(f"{BASE_URL}/api/drift/scan")
        assert response.status_code == 200
        
        data = response.json()
        assert "scan_id" in data
        assert "summary" in data
        assert "results" in data
        assert "scanned_at" in data
        
        summary = data["summary"]
        assert "total_images" in summary
        assert "compliant" in summary
        assert "drifted" in summary
        assert "critical" in summary
        
        print(f"PASS: Drift scan completed - {summary['total_images']} images, {summary['drifted']} drifted, {summary['critical']} critical")
        
        # Store scan_id for history test
        self.__class__.last_scan_id = data["scan_id"]
        return data
    
    def test_drift_scan_results_structure(self):
        """Test drift scan results have proper structure"""
        response = requests.get(f"{BASE_URL}/api/drift/scan")
        assert response.status_code == 200
        
        data = response.json()
        
        # Check results structure
        for result in data["results"]:
            assert "has_drift" in result
            if result["has_drift"]:
                assert "risk_level" in result
                assert "drift_details" in result or "drift_count" in result
        
        print(f"PASS: All {len(data['results'])} drift results have proper structure")
    
    def test_get_drift_history(self):
        """Test GET /api/drift/history returns scan history"""
        # First run a scan to ensure history exists
        requests.get(f"{BASE_URL}/api/drift/scan")
        
        response = requests.get(f"{BASE_URL}/api/drift/history")
        assert response.status_code == 200
        
        data = response.json()
        assert "scans" in data
        assert "total_scans" in data
        assert isinstance(data["scans"], list)
        
        if data["scans"]:
            scan = data["scans"][0]
            assert "id" in scan
            assert "scanned_at" in scan
            assert "total_images" in scan
            assert "drifted_count" in scan
        
        print(f"PASS: Got {data['total_scans']} scans in history")
    
    def test_get_drift_stats(self):
        """Test GET /api/drift/stats returns statistics"""
        response = requests.get(f"{BASE_URL}/api/drift/stats")
        assert response.status_code == 200
        
        data = response.json()
        assert "total_scans" in data
        assert "monitored_clusters" in data
        assert "monitored_namespaces" in data
        
        print(f"PASS: Drift stats - {data['total_scans']} total scans, {data['monitored_clusters']} clusters")


class TestRemediationPolicies:
    """Remediation Policies endpoint tests"""
    
    def test_get_remediation_policies(self):
        """Test GET /api/remediation/policies returns policy list"""
        response = requests.get(f"{BASE_URL}/api/remediation/policies")
        assert response.status_code == 200
        
        data = response.json()
        assert "policies" in data
        assert "active_policy" in data
        assert isinstance(data["policies"], list)
        
        # Check default policies exist
        policy_names = [p["name"] for p in data["policies"]]
        print(f"PASS: Got {len(data['policies'])} policies: {policy_names}")
        
        # Check policy structure
        for policy in data["policies"]:
            assert "id" in policy
            assert "name" in policy
            assert "mode" in policy
            assert "auto_remediate_critical" in policy
            assert "auto_remediate_high" in policy
    
    def test_remediation_policy_modes(self):
        """Test that policies have valid modes"""
        response = requests.get(f"{BASE_URL}/api/remediation/policies")
        assert response.status_code == 200
        
        data = response.json()
        valid_modes = ["strict", "graceful", "notify_only"]
        
        for policy in data["policies"]:
            assert policy["mode"] in valid_modes, f"Invalid mode: {policy['mode']}"
        
        print(f"PASS: All policies have valid modes")
    
    def test_create_remediation_policy(self):
        """Test POST /api/remediation/policies creates new policy"""
        policy_data = {
            "name": f"TEST_Policy_{uuid.uuid4().hex[:8]}",
            "description": "Test policy for Phase 4 testing",
            "mode": "graceful",
            "auto_remediate_critical": True,
            "auto_remediate_high": True,
            "auto_remediate_medium": False,
            "fail_on_unfixable_critical": False,
            "notify_on_remediation": True,
            "enabled": False
        }
        
        response = requests.post(f"{BASE_URL}/api/remediation/policies", json=policy_data)
        assert response.status_code == 200
        
        data = response.json()
        assert "id" in data
        assert "message" in data
        
        self.__class__.created_policy_id = data["id"]
        print(f"PASS: Created policy with ID: {data['id']}")
    
    def test_activate_remediation_policy(self):
        """Test POST /api/remediation/policies/{id}/activate activates policy"""
        # First create a policy
        policy_data = {
            "name": f"TEST_Activate_{uuid.uuid4().hex[:8]}",
            "description": "Test policy for activation",
            "mode": "strict",
            "auto_remediate_critical": True,
            "auto_remediate_high": False,
            "enabled": False
        }
        
        create_response = requests.post(f"{BASE_URL}/api/remediation/policies", json=policy_data)
        assert create_response.status_code == 200
        policy_id = create_response.json()["id"]
        
        # Activate it
        response = requests.post(f"{BASE_URL}/api/remediation/policies/{policy_id}/activate")
        assert response.status_code == 200
        
        data = response.json()
        assert "message" in data
        print(f"PASS: Policy {policy_id} activated")
        
        # Verify it's now active
        policies_response = requests.get(f"{BASE_URL}/api/remediation/policies")
        policies_data = policies_response.json()
        
        # Find our policy and check it's enabled
        our_policy = next((p for p in policies_data["policies"] if p["id"] == policy_id), None)
        if our_policy:
            assert our_policy["enabled"] == True
            print(f"PASS: Verified policy is now enabled")
    
    def test_remediation_stats(self):
        """Test GET /api/remediation/stats returns statistics"""
        response = requests.get(f"{BASE_URL}/api/remediation/stats")
        assert response.status_code == 200
        
        data = response.json()
        assert "total_remediations_performed" in data
        assert "total_fixes_applied" in data
        assert "cve_database_size" in data
        assert "auto_fixable_cves" in data
        
        print(f"PASS: Remediation stats - {data['cve_database_size']} CVEs in database, {data['auto_fixable_cves']} auto-fixable")


class TestPolicyBasedRemediation:
    """Test policy-based auto-remediation endpoint"""
    
    def test_auto_remediate_with_policy(self):
        """Test POST /api/builds/{id}/auto-remediate-with-policy"""
        # First get a build ID from existing builds
        builds_response = requests.get(f"{BASE_URL}/api/builds")
        assert builds_response.status_code == 200
        
        builds = builds_response.json()
        if not builds:
            pytest.skip("No builds available for testing")
        
        # Find a completed build
        completed_build = next((b for b in builds if b.get("status") == "completed"), None)
        if not completed_build:
            pytest.skip("No completed builds available")
        
        build_id = completed_build["id"]
        
        # Test auto-remediate with policy
        response = requests.post(f"{BASE_URL}/api/builds/{build_id}/auto-remediate-with-policy")
        
        # Should return 200 or 404 if no scan results
        assert response.status_code in [200, 404]
        
        if response.status_code == 200:
            data = response.json()
            # Check response structure
            assert "policy_applied" in data or "status" in data or "message" in data
            print(f"PASS: Auto-remediate with policy executed for build {build_id}")
        else:
            print(f"INFO: Build {build_id} has no scan results (expected for some builds)")


class TestIntegration:
    """Integration tests for Phase 4 features"""
    
    def test_exception_workflow_complete(self):
        """Test complete exception workflow: create -> approve -> verify"""
        # Create
        exception_data = {
            "build_id": f"TEST_workflow_{uuid.uuid4().hex[:8]}",
            "policy_id": "cis-benchmark",
            "requestor": "integration_test@example.com",
            "justification": "Complete workflow integration test",
            "duration_days": 7
        }
        
        create_response = requests.post(f"{BASE_URL}/api/exceptions", json=exception_data)
        assert create_response.status_code == 200
        exception_id = create_response.json()["id"]
        
        # Verify pending
        get_response = requests.get(f"{BASE_URL}/api/exceptions/{exception_id}")
        assert get_response.status_code == 200
        assert get_response.json()["status"] == "pending"
        
        # Approve
        approve_response = requests.post(
            f"{BASE_URL}/api/exceptions/{exception_id}/approve",
            json={"approver": "admin", "notes": "Integration test approval"}
        )
        assert approve_response.status_code == 200
        
        # Verify approved
        final_response = requests.get(f"{BASE_URL}/api/exceptions/{exception_id}")
        assert final_response.status_code == 200
        assert final_response.json()["status"] == "approved"
        
        print(f"PASS: Complete exception workflow tested successfully")
    
    def test_drift_scan_and_history(self):
        """Test drift scan creates history entry"""
        # Run scan
        scan_response = requests.get(f"{BASE_URL}/api/drift/scan")
        assert scan_response.status_code == 200
        scan_id = scan_response.json()["scan_id"]
        
        # Check history
        history_response = requests.get(f"{BASE_URL}/api/drift/history")
        assert history_response.status_code == 200
        
        history = history_response.json()
        scan_ids = [s["id"] for s in history["scans"]]
        assert scan_id in scan_ids
        
        print(f"PASS: Drift scan {scan_id} found in history")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
