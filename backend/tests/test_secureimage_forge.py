"""
SecureImage Forge API Tests
Tests for Phase 1-4 features plus Granular Runtime & OS Controls (Phase 4.5)
"""
import pytest
import requests
import os
import time

BASE_URL = os.environ.get('REACT_APP_BACKEND_URL', '').rstrip('/')

class TestBasicEndpoints:
    """Test basic API endpoints"""
    
    def test_api_root(self):
        """Test API root endpoint"""
        response = requests.get(f"{BASE_URL}/api/")
        assert response.status_code == 200
        data = response.json()
        assert "message" in data
        assert "SecureImage Forge API" in data["message"]
        print(f"✓ API root returns: {data}")
    
    def test_stats_endpoint(self):
        """Test dashboard stats endpoint"""
        response = requests.get(f"{BASE_URL}/api/stats")
        assert response.status_code == 200
        data = response.json()
        assert "total_builds" in data
        assert "completed_builds" in data
        assert "failed_builds" in data
        assert "in_progress" in data
        assert "avg_compliance_score" in data
        print(f"✓ Stats endpoint returns: {data}")


class TestRuntimeVersionsEndpoints:
    """Test Phase 4.5 Granular Controls - Runtime Versions"""
    
    def test_runtime_versions_endpoint(self):
        """Test /api/runtime-versions returns version matrix"""
        response = requests.get(f"{BASE_URL}/api/runtime-versions")
        assert response.status_code == 200
        data = response.json()
        assert "runtimes" in data
        
        # Check all runtimes are present
        runtimes = data["runtimes"]
        assert "java" in runtimes
        assert "dotnet" in runtimes
        assert "go" in runtimes
        assert "nodejs" in runtimes
        
        # Check Java has versions and distributions
        java = runtimes["java"]
        assert "versions" in java
        assert "default_version" in java
        assert "default_distribution" in java
        assert "17" in java["versions"]
        assert "distributions" in java["versions"]["17"]
        print(f"✓ Runtime versions endpoint returns {len(runtimes)} runtimes")
    
    def test_specific_runtime_versions(self):
        """Test /api/runtime-versions/{runtime} endpoint"""
        response = requests.get(f"{BASE_URL}/api/runtime-versions/java")
        assert response.status_code == 200
        data = response.json()
        assert "versions" in data
        assert "default_version" in data
        print(f"✓ Java runtime versions: {list(data['versions'].keys())}")
    
    def test_invalid_runtime_returns_404(self):
        """Test invalid runtime returns 404"""
        response = requests.get(f"{BASE_URL}/api/runtime-versions/invalid_runtime")
        assert response.status_code == 404
        print("✓ Invalid runtime returns 404")


class TestBaseImageTagsEndpoints:
    """Test Phase 4.5 Granular Controls - Base Image Tags"""
    
    def test_base_image_tags_endpoint(self):
        """Test /api/base-image-tags returns tag catalog"""
        response = requests.get(f"{BASE_URL}/api/base-image-tags")
        assert response.status_code == 200
        data = response.json()
        assert "base_images" in data
        
        base_images = data["base_images"]
        assert "alpine" in base_images
        assert "debian" in base_images
        assert "distroless" in base_images
        
        # Check alpine has tags
        alpine = base_images["alpine"]
        assert "tags" in alpine
        assert "recommended_tag" in alpine
        print(f"✓ Base image tags endpoint returns {len(base_images)} base images")
    
    def test_specific_base_image_tags(self):
        """Test /api/base-image-tags/{base_image} endpoint"""
        response = requests.get(f"{BASE_URL}/api/base-image-tags/alpine")
        assert response.status_code == 200
        data = response.json()
        assert "tags" in data
        assert "recommended_tag" in data
        print(f"✓ Alpine tags: {list(data['tags'].keys())}")


class TestCISLevelsEndpoint:
    """Test Phase 4.5 Granular Controls - CIS Levels"""
    
    def test_cis_levels_endpoint(self):
        """Test /api/cis-levels returns CIS level configurations"""
        response = requests.get(f"{BASE_URL}/api/cis-levels")
        assert response.status_code == 200
        data = response.json()
        assert "levels" in data
        
        levels = data["levels"]
        assert "level1" in levels
        assert "level2" in levels
        
        # Check level1 structure
        level1 = levels["level1"]
        assert "name" in level1
        assert "description" in level1
        assert "controls" in level1
        assert "strictness" in level1
        print(f"✓ CIS levels endpoint returns {len(levels)} levels")


class TestSBOMFormatsEndpoint:
    """Test Phase 4.5 Granular Controls - SBOM Formats"""
    
    def test_sbom_formats_endpoint(self):
        """Test /api/sbom-formats returns SBOM format options"""
        response = requests.get(f"{BASE_URL}/api/sbom-formats")
        assert response.status_code == 200
        data = response.json()
        assert "formats" in data
        assert "scan_depths" in data
        
        # Check formats
        formats = data["formats"]
        assert "cyclonedx" in formats
        assert "spdx" in formats
        
        # Check scan depths
        scan_depths = data["scan_depths"]
        assert "os_only" in scan_depths
        assert "os_and_runtime" in scan_depths
        assert "full" in scan_depths
        print(f"✓ SBOM formats endpoint returns {len(formats)} formats and {len(scan_depths)} scan depths")


class TestBuildEndpoints:
    """Test Build CRUD operations"""
    
    def test_get_builds_list(self):
        """Test GET /api/builds returns list"""
        response = requests.get(f"{BASE_URL}/api/builds")
        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        print(f"✓ Builds list returns {len(data)} builds")
    
    def test_create_build_simple(self):
        """Test POST /api/builds with simple config"""
        payload = {
            "name": "TEST_simple_build",
            "runtime": "java",
            "base_image": "alpine",
            "compliance_profiles": ["cis"],
            "remove_shell": True,
            "remove_package_manager": True,
            "enable_sbom": True,
            "enable_signing": True
        }
        response = requests.post(f"{BASE_URL}/api/builds", json=payload)
        assert response.status_code == 200
        data = response.json()
        assert "id" in data
        assert data["config_name"] == "TEST_simple_build"
        assert data["status"] in ["queued", "building"]
        print(f"✓ Simple build created with ID: {data['id']}")
        return data["id"]
    
    def test_create_build_with_advanced_config(self):
        """Test POST /api/builds with advanced configuration payload"""
        payload = {
            "name": "TEST_advanced_build",
            "runtime": "java",
            "base_image": "alpine",
            "compliance_profiles": ["cis", "hipaa"],
            "architecture": ["amd64", "arm64"],
            "remove_shell": True,
            "remove_package_manager": True,
            "enable_sbom": True,
            "enable_signing": True,
            # Advanced fields
            "runtime_version": "17",
            "runtime_distribution": "temurin",
            "base_image_tag": "3.19.1",
            "binary_whitelist": ["/usr/bin/curl", "/usr/bin/wget"],
            "env_sanitization_rules": ["AWS_SECRET_KEY", "DB_PASSWORD"],
            "cis_level": 2,
            "fips_mode_enabled": False,
            "custom_labels": {
                "owner": "platform-team",
                "cost-center": "engineering"
            },
            "sbom_format": "cyclonedx",
            "sbom_scan_depth": "os_and_runtime"
        }
        response = requests.post(f"{BASE_URL}/api/builds", json=payload)
        assert response.status_code == 200
        data = response.json()
        assert "id" in data
        assert data["config_name"] == "TEST_advanced_build"
        print(f"✓ Advanced build created with ID: {data['id']}")
        return data["id"]
    
    def test_get_build_by_id(self):
        """Test GET /api/builds/{build_id}"""
        # First create a build
        payload = {
            "name": "TEST_get_build",
            "runtime": "nodejs",
            "base_image": "debian",
            "compliance_profiles": ["soc2"]
        }
        create_response = requests.post(f"{BASE_URL}/api/builds", json=payload)
        assert create_response.status_code == 200
        build_id = create_response.json()["id"]
        
        # Then get it
        response = requests.get(f"{BASE_URL}/api/builds/{build_id}")
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == build_id
        assert data["config_name"] == "TEST_get_build"
        print(f"✓ Build retrieved: {data['config_name']} - Status: {data['status']}")
    
    def test_get_nonexistent_build_returns_404(self):
        """Test GET /api/builds/{invalid_id} returns 404"""
        response = requests.get(f"{BASE_URL}/api/builds/nonexistent-id-12345")
        assert response.status_code == 404
        print("✓ Nonexistent build returns 404")


class TestValidateRuntimeConfig:
    """Test runtime configuration validation"""
    
    def test_validate_valid_config(self):
        """Test validation of valid runtime config"""
        payload = {
            "runtime": "java",
            "runtime_version": "17",
            "runtime_distribution": "temurin",
            "fips_mode_enabled": False
        }
        response = requests.post(f"{BASE_URL}/api/validate-runtime-config", json=payload)
        assert response.status_code == 200
        data = response.json()
        assert data["valid"] == True
        print(f"✓ Valid config validation passed: {data}")
    
    def test_validate_invalid_runtime(self):
        """Test validation of invalid runtime"""
        payload = {
            "runtime": "invalid_runtime",
            "runtime_version": "1.0",
            "runtime_distribution": "official"
        }
        response = requests.post(f"{BASE_URL}/api/validate-runtime-config", json=payload)
        assert response.status_code == 200
        data = response.json()
        assert data["valid"] == False
        assert len(data["errors"]) > 0
        print(f"✓ Invalid runtime validation failed as expected: {data['errors']}")
    
    def test_validate_fips_incompatible(self):
        """Test validation of FIPS mode with incompatible distribution"""
        payload = {
            "runtime": "java",
            "runtime_version": "17",
            "runtime_distribution": "temurin",  # temurin doesn't support FIPS
            "fips_mode_enabled": True
        }
        response = requests.post(f"{BASE_URL}/api/validate-runtime-config", json=payload)
        assert response.status_code == 200
        data = response.json()
        assert data["valid"] == False
        print(f"✓ FIPS incompatibility detected: {data['errors']}")


class TestPoliciesEndpoints:
    """Test Policies CRUD operations"""
    
    def test_get_policies_list(self):
        """Test GET /api/policies returns list"""
        response = requests.get(f"{BASE_URL}/api/policies")
        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        print(f"✓ Policies list returns {len(data)} policies")
    
    def test_get_policy_templates(self):
        """Test GET /api/policies/templates returns templates"""
        response = requests.get(f"{BASE_URL}/api/policies/templates")
        assert response.status_code == 200
        data = response.json()
        assert "templates" in data
        templates = data["templates"]
        assert len(templates) > 0
        print(f"✓ Policy templates returns {len(templates)} templates")
    
    def test_create_policy(self):
        """Test POST /api/policies creates policy"""
        payload = {
            "name": "TEST_no_critical_vulns",
            "description": "Block builds with critical vulnerabilities",
            "type": "vulnerability",
            "enforcement": "block",
            "rule": {
                "condition": "critical_count",
                "operator": "<=",
                "value": 0
            },
            "enabled": True
        }
        response = requests.post(f"{BASE_URL}/api/policies", json=payload)
        assert response.status_code == 200
        data = response.json()
        assert "id" in data
        assert data["name"] == "TEST_no_critical_vulns"
        print(f"✓ Policy created with ID: {data['id']}")
        return data["id"]


class TestAnalyticsEndpoints:
    """Test Analytics endpoints"""
    
    def test_analytics_trends(self):
        """Test GET /api/analytics/trends"""
        response = requests.get(f"{BASE_URL}/api/analytics/trends?days=30")
        assert response.status_code == 200
        data = response.json()
        assert "period_days" in data
        assert "trend_data" in data
        print(f"✓ Analytics trends returns {len(data['trend_data'])} data points")
    
    def test_analytics_success_rate(self):
        """Test GET /api/analytics/success-rate"""
        response = requests.get(f"{BASE_URL}/api/analytics/success-rate?days=30")
        assert response.status_code == 200
        data = response.json()
        assert "success_rate" in data
        assert "total_builds" in data
        assert "completed" in data
        assert "failed" in data
        print(f"✓ Success rate: {data['success_rate']}%")
    
    def test_analytics_health_scores(self):
        """Test GET /api/analytics/health-scores"""
        response = requests.get(f"{BASE_URL}/api/analytics/health-scores")
        assert response.status_code == 200
        data = response.json()
        assert "average_health_score" in data
        assert "grade_distribution" in data
        print(f"✓ Average health score: {data['average_health_score']}")
    
    def test_analytics_vulnerabilities(self):
        """Test GET /api/analytics/vulnerabilities"""
        response = requests.get(f"{BASE_URL}/api/analytics/vulnerabilities")
        assert response.status_code == 200
        data = response.json()
        assert "total_vulnerabilities" in data
        assert "by_runtime" in data
        print(f"✓ Vulnerability analytics: {data['total_vulnerabilities']}")


class TestArchitecturesEndpoint:
    """Test architectures endpoint"""
    
    def test_get_supported_architectures(self):
        """Test GET /api/architectures"""
        response = requests.get(f"{BASE_URL}/api/architectures")
        assert response.status_code == 200
        data = response.json()
        assert "supported" in data
        assert "amd64" in data["supported"]
        assert "arm64" in data["supported"]
        assert "multi_arch_builds" in data
        print(f"✓ Supported architectures: {data['supported']}")


class TestBuildCompletionFlow:
    """Test build completion and related endpoints"""
    
    def test_build_completion_flow(self):
        """Test full build flow - create and wait for completion"""
        # Create build
        payload = {
            "name": "TEST_completion_flow",
            "runtime": "go",
            "base_image": "distroless",
            "compliance_profiles": ["cis"]
        }
        create_response = requests.post(f"{BASE_URL}/api/builds", json=payload)
        assert create_response.status_code == 200
        build_id = create_response.json()["id"]
        print(f"✓ Build created: {build_id}")
        
        # Wait for completion (max 30 seconds)
        max_wait = 30
        waited = 0
        while waited < max_wait:
            response = requests.get(f"{BASE_URL}/api/builds/{build_id}")
            assert response.status_code == 200
            status = response.json()["status"]
            if status in ["completed", "failed"]:
                print(f"✓ Build finished with status: {status}")
                break
            time.sleep(2)
            waited += 2
        
        # Check build details
        build_response = requests.get(f"{BASE_URL}/api/builds/{build_id}")
        build_data = build_response.json()
        
        if build_data["status"] == "completed":
            # Test scan results
            scan_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/scan")
            assert scan_response.status_code == 200
            print(f"✓ Scan results available")
            
            # Test compliance report
            compliance_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/compliance")
            assert compliance_response.status_code == 200
            print(f"✓ Compliance report available")
            
            # Test health score
            health_response = requests.get(f"{BASE_URL}/api/builds/{build_id}/health")
            assert health_response.status_code == 200
            print(f"✓ Health score available")


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short"])
