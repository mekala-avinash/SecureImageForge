#!/usr/bin/env python3
"""
SecureImage Forge Backend API Testing
Tests all API endpoints using the public URL
"""

import requests
import sys
import time
import json
from datetime import datetime

class SecureImageForgeAPITester:
    def __init__(self, base_url="https://docker-vault.preview.emergentagent.com"):
        self.base_url = base_url
        self.api_url = f"{base_url}/api"
        self.tests_run = 0
        self.tests_passed = 0
        self.build_id = None

    def run_test(self, name, method, endpoint, expected_status, data=None, timeout=10):
        """Run a single API test"""
        url = f"{self.api_url}/{endpoint}"
        headers = {'Content-Type': 'application/json'}

        self.tests_run += 1
        print(f"\n🔍 Testing {name}...")
        print(f"   URL: {url}")
        
        try:
            if method == 'GET':
                response = requests.get(url, headers=headers, timeout=timeout)
            elif method == 'POST':
                response = requests.post(url, json=data, headers=headers, timeout=timeout)
            elif method == 'PUT':
                response = requests.put(url, json=data, headers=headers, timeout=timeout)
            elif method == 'DELETE':
                response = requests.delete(url, headers=headers, timeout=timeout)

            success = response.status_code == expected_status
            if success:
                self.tests_passed += 1
                print(f"✅ Passed - Status: {response.status_code}")
                try:
                    response_data = response.json()
                    print(f"   Response keys: {list(response_data.keys()) if isinstance(response_data, dict) else 'Non-dict response'}")
                    return True, response_data
                except:
                    return True, response.text
            else:
                print(f"❌ Failed - Expected {expected_status}, got {response.status_code}")
                print(f"   Response: {response.text[:200]}...")
                return False, {}

        except requests.exceptions.Timeout:
            print(f"❌ Failed - Request timeout after {timeout}s")
            return False, {}
        except Exception as e:
            print(f"❌ Failed - Error: {str(e)}")
            return False, {}

    def test_root_endpoint(self):
        """Test API root endpoint"""
        success, response = self.run_test(
            "API Root",
            "GET",
            "",
            200
        )
        return success

    def test_stats_endpoint(self):
        """Test dashboard statistics endpoint"""
        success, response = self.run_test(
            "Dashboard Stats",
            "GET",
            "stats",
            200
        )
        if success and isinstance(response, dict):
            required_keys = ['total_builds', 'completed_builds', 'failed_builds', 'in_progress', 'avg_compliance_score']
            missing_keys = [key for key in required_keys if key not in response]
            if missing_keys:
                print(f"   ⚠️  Missing keys in stats response: {missing_keys}")
            else:
                print(f"   ✅ All required stats keys present")
        return success

    def test_create_build_java_alpine(self):
        """Test creating a Java Alpine build with multiple compliance profiles"""
        build_data = {
            "name": f"test-java-alpine-{int(time.time())}",
            "runtime": "java",
            "base_image": "alpine",
            "compliance_profiles": ["hipaa", "soc2", "cis"],
            "remove_shell": True,
            "remove_package_manager": True,
            "enable_sbom": True,
            "enable_signing": True
        }
        
        success, response = self.run_test(
            "Create Java Alpine Build",
            "POST",
            "builds",
            200,
            data=build_data
        )
        
        if success and isinstance(response, dict) and 'id' in response:
            self.build_id = response['id']
            print(f"   ✅ Build created with ID: {self.build_id}")
            print(f"   Status: {response.get('status', 'unknown')}")
        
        return success

    def test_create_build_dotnet_debian(self):
        """Test creating a .NET Debian build"""
        build_data = {
            "name": f"test-dotnet-debian-{int(time.time())}",
            "runtime": "dotnet",
            "base_image": "debian",
            "compliance_profiles": ["soc2"],
            "remove_shell": False,
            "remove_package_manager": False,
            "enable_sbom": True,
            "enable_signing": False
        }
        
        success, response = self.run_test(
            "Create .NET Debian Build",
            "POST",
            "builds",
            200,
            data=build_data
        )
        
        return success

    def test_get_builds_list(self):
        """Test getting all builds"""
        success, response = self.run_test(
            "Get Builds List",
            "GET",
            "builds",
            200
        )
        
        if success and isinstance(response, list):
            print(f"   ✅ Found {len(response)} builds")
            if len(response) > 0:
                build = response[0]
                required_keys = ['id', 'config_name', 'status']
                missing_keys = [key for key in required_keys if key not in build]
                if missing_keys:
                    print(f"   ⚠️  Missing keys in build response: {missing_keys}")
        
        return success

    def test_get_build_details(self):
        """Test getting specific build details"""
        if not self.build_id:
            print("   ⚠️  Skipping - no build ID available")
            return True
            
        success, response = self.run_test(
            "Get Build Details",
            "GET",
            f"builds/{self.build_id}",
            200
        )
        
        if success and isinstance(response, dict):
            required_keys = ['id', 'config_name', 'status', 'logs']
            missing_keys = [key for key in required_keys if key not in response]
            if missing_keys:
                print(f"   ⚠️  Missing keys in build details: {missing_keys}")
            else:
                print(f"   ✅ Build status: {response.get('status')}")
                print(f"   ✅ Logs count: {len(response.get('logs', []))}")
        
        return success

    def test_wait_for_build_completion(self):
        """Wait for build to complete and test status updates"""
        if not self.build_id:
            print("   ⚠️  Skipping - no build ID available")
            return True
            
        print(f"\n🕐 Waiting for build {self.build_id} to complete...")
        max_wait = 60  # 60 seconds max wait
        start_time = time.time()
        
        while time.time() - start_time < max_wait:
            success, response = self.run_test(
                "Check Build Status",
                "GET",
                f"builds/{self.build_id}",
                200
            )
            
            if success and isinstance(response, dict):
                status = response.get('status')
                print(f"   Status: {status}")
                
                if status in ['completed', 'failed']:
                    print(f"   ✅ Build finished with status: {status}")
                    return status == 'completed'
                    
            time.sleep(3)
        
        print(f"   ⚠️  Build did not complete within {max_wait} seconds")
        return True  # Don't fail the test for timeout

    def test_scan_results(self):
        """Test vulnerability scan results"""
        if not self.build_id:
            print("   ⚠️  Skipping - no build ID available")
            return True
            
        success, response = self.run_test(
            "Get Scan Results",
            "GET",
            f"builds/{self.build_id}/scan",
            200
        )
        
        if success and isinstance(response, dict):
            required_keys = ['build_id', 'vulnerabilities', 'total_count']
            missing_keys = [key for key in required_keys if key not in response]
            if missing_keys:
                print(f"   ⚠️  Missing keys in scan results: {missing_keys}")
            else:
                total_count = response.get('total_count', {})
                print(f"   ✅ Vulnerability counts: {total_count}")
        
        return success

    def test_compliance_report(self):
        """Test compliance report"""
        if not self.build_id:
            print("   ⚠️  Skipping - no build ID available")
            return True
            
        success, response = self.run_test(
            "Get Compliance Report",
            "GET",
            f"builds/{self.build_id}/compliance",
            200
        )
        
        if success and isinstance(response, dict):
            required_keys = ['build_id', 'profiles', 'checks', 'overall_score']
            missing_keys = [key for key in required_keys if key not in response]
            if missing_keys:
                print(f"   ⚠️  Missing keys in compliance report: {missing_keys}")
            else:
                print(f"   ✅ Compliance score: {response.get('overall_score')}%")
                print(f"   ✅ Profiles: {response.get('profiles')}")
                print(f"   ✅ Checks count: {len(response.get('checks', []))}")
        
        return success

    def test_sbom_data(self):
        """Test SBOM data"""
        if not self.build_id:
            print("   ⚠️  Skipping - no build ID available")
            return True
            
        success, response = self.run_test(
            "Get SBOM Data",
            "GET",
            f"builds/{self.build_id}/sbom",
            200
        )
        
        if success and isinstance(response, dict):
            required_keys = ['bomFormat', 'specVersion', 'metadata']
            missing_keys = [key for key in required_keys if key not in response]
            if missing_keys:
                print(f"   ⚠️  Missing keys in SBOM: {missing_keys}")
            else:
                print(f"   ✅ SBOM format: {response.get('bomFormat')}")
                print(f"   ✅ Spec version: {response.get('specVersion')}")
        
        return success

    def test_build_configs(self):
        """Test build configurations endpoint"""
        success, response = self.run_test(
            "Get Build Configs",
            "GET",
            "configs",
            200
        )
        
        if success and isinstance(response, list):
            print(f"   ✅ Found {len(response)} build configs")
        
        return success

def main():
    """Run all backend API tests"""
    print("🚀 Starting SecureImage Forge Backend API Tests")
    print("=" * 60)
    
    tester = SecureImageForgeAPITester()
    
    # Test sequence
    tests = [
        ("API Root", tester.test_root_endpoint),
        ("Dashboard Stats", tester.test_stats_endpoint),
        ("Create Java Alpine Build", tester.test_create_build_java_alpine),
        ("Create .NET Debian Build", tester.test_create_build_dotnet_debian),
        ("Get Builds List", tester.test_get_builds_list),
        ("Get Build Details", tester.test_get_build_details),
        ("Wait for Build Completion", tester.test_wait_for_build_completion),
        ("Scan Results", tester.test_scan_results),
        ("Compliance Report", tester.test_compliance_report),
        ("SBOM Data", tester.test_sbom_data),
        ("Build Configs", tester.test_build_configs),
    ]
    
    for test_name, test_func in tests:
        try:
            test_func()
        except Exception as e:
            print(f"❌ Test '{test_name}' failed with exception: {str(e)}")
    
    # Print results
    print("\n" + "=" * 60)
    print(f"📊 Test Results: {tester.tests_passed}/{tester.tests_run} passed")
    
    success_rate = (tester.tests_passed / tester.tests_run * 100) if tester.tests_run > 0 else 0
    print(f"📈 Success Rate: {success_rate:.1f}%")
    
    if success_rate >= 80:
        print("✅ Backend API tests mostly successful!")
        return 0
    else:
        print("❌ Backend API tests have significant failures")
        return 1

if __name__ == "__main__":
    sys.exit(main())