#!/usr/bin/env python3
"""SecureImage Forge CLI Tool"""

import click
import requests
import json
import time
import os
from typing import List

# Use environment variable or default to external URL
API_URL = os.environ.get('FORGE_API_URL', 'https://runtime-guardian-io.preview.emergentagent.com/api')

@click.group()
@click.version_option(version='1.0.0')
def cli():
    """SecureImage Forge - Build hardened Docker images with compliance"""
    pass

@cli.command()
@click.option('--name', required=True, help='Build configuration name')
@click.option('--runtime', type=click.Choice(['java', 'dotnet', 'go', 'nodejs']), required=True, help='Runtime environment')
@click.option('--base', type=click.Choice(['alpine', 'debian', 'distroless']), required=True, help='Base image type')
@click.option('--compliance', '-c', multiple=True, type=click.Choice(['hipaa', 'soc2', 'cis']), help='Compliance profiles (can be used multiple times)')
@click.option('--no-shell', is_flag=True, default=True, help='Remove shell binaries')
@click.option('--no-pkg-mgr', is_flag=True, default=True, help='Remove package managers')
@click.option('--sbom/--no-sbom', default=True, help='Generate SBOM')
@click.option('--sign/--no-sign', default=True, help='Sign image')
def build(name, runtime, base, compliance, no_shell, no_pkg_mgr, sbom, sign):
    """Build a new hardened container image"""
    
    compliance_list = list(compliance) if compliance else ['cis']
    
    payload = {
        "name": name,
        "runtime": runtime,
        "base_image": base,
        "compliance_profiles": compliance_list,
        "remove_shell": no_shell,
        "remove_package_manager": no_pkg_mgr,
        "enable_sbom": sbom,
        "enable_signing": sign
    }
    
    try:
        click.echo(f"🔨 Starting build: {name}")
        click.echo(f"   Runtime: {runtime}")
        click.echo(f"   Base: {base}")
        click.echo(f"   Compliance: {', '.join(compliance_list)}")
        click.echo()
        
        response = requests.post(f"{API_URL}/builds", json=payload)
        response.raise_for_status()
        
        build_data = response.json()
        build_id = build_data['id']
        
        click.echo(f"✓ Build queued with ID: {build_id}")
        click.echo(f"\nMonitoring build progress...\n")
        
        # Monitor build progress
        while True:
            time.sleep(2)
            status_response = requests.get(f"{API_URL}/builds/{build_id}")
            status_data = status_response.json()
            
            status = status_data['status']
            click.echo(f"Status: {status.upper()}", nl=False)
            
            if status in ['completed', 'failed']:
                click.echo()
                break
            click.echo('\r', nl=False)
        
        if status == 'completed':
            click.echo(f"\n✅ Build completed successfully!")
            click.echo(f"   Image: {status_data.get('image_tag', 'N/A')}")
            click.echo(f"   Compliance Score: {status_data.get('compliance_score', 0)}%")
            
            vuln = status_data.get('vulnerability_count', {})
            if vuln:
                click.echo(f"   Vulnerabilities: CRITICAL={vuln.get('CRITICAL', 0)}, HIGH={vuln.get('HIGH', 0)}, MEDIUM={vuln.get('MEDIUM', 0)}, LOW={vuln.get('LOW', 0)}")
        else:
            click.echo(f"\n❌ Build failed. Check logs with: forge logs {build_id}")
            
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.command()
@click.argument('build_id')
def scan(build_id):
    """View vulnerability scan results"""
    try:
        response = requests.get(f"{API_URL}/builds/{build_id}/scan")
        response.raise_for_status()
        
        scan_data = response.json()
        
        click.echo(f"\n🔍 Vulnerability Scan Results\n")
        click.echo(f"Build ID: {build_id}\n")
        
        total = scan_data.get('total_count', {})
        click.echo(f"Total Vulnerabilities:")
        click.echo(f"  CRITICAL: {total.get('CRITICAL', 0)}")
        click.echo(f"  HIGH: {total.get('HIGH', 0)}")
        click.echo(f"  MEDIUM: {total.get('MEDIUM', 0)}")
        click.echo(f"  LOW: {total.get('LOW', 0)}")
        
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.command()
def list():
    """List all builds"""
    try:
        response = requests.get(f"{API_URL}/builds")
        response.raise_for_status()
        
        builds = response.json()
        
        if not builds:
            click.echo("No builds found.")
            return
        
        click.echo(f"\n📋 Recent Builds\n")
        click.echo(f"{'ID':<36} {'Name':<20} {'Status':<12} {'Compliance':<6}")
        click.echo("-" * 80)
        
        for build in builds[:20]:
            build_id = build['id'][:8] + '...' if len(build['id']) > 8 else build['id']
            name = build['config_name'][:20]
            status = build['status']
            compliance = f"{build.get('compliance_score', 0)}%" if build.get('compliance_score') else 'N/A'
            
            click.echo(f"{build['id']:<36} {name:<20} {status:<12} {compliance:<6}")
            
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.command()
@click.argument('build_id')
def logs(build_id):
    """View build logs"""
    try:
        response = requests.get(f"{API_URL}/builds/{build_id}")
        response.raise_for_status()
        
        build_data = response.json()
        
        click.echo(f"\n📄 Build Logs: {build_data['config_name']}\n")
        
        logs = build_data.get('logs', [])
        for log in logs:
            click.echo(f"  {log}")
            
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.command()
def stats():
    """View dashboard statistics"""
    try:
        response = requests.get(f"{API_URL}/stats")
        response.raise_for_status()
        
        stats_data = response.json()
        
        click.echo(f"\n📊 SecureImage Forge Statistics\n")
        click.echo(f"Total Builds: {stats_data['total_builds']}")
        click.echo(f"Completed: {stats_data['completed_builds']}")
        click.echo(f"Failed: {stats_data['failed_builds']}")
        click.echo(f"In Progress: {stats_data['in_progress']}")
        click.echo(f"Average Compliance Score: {stats_data['avg_compliance_score']}%")
        
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

# Phase 2 Commands
@cli.command()
@click.argument('build_id')
def health(build_id):
    """View health score for a build"""
    try:
        response = requests.get(f"{API_URL}/builds/{build_id}/health")
        response.raise_for_status()
        
        health_data = response.json()
        
        click.echo(f"\n💚 Health Score Report\n")
        click.echo(f"Build ID: {build_id}")
        click.echo(f"Score: {health_data['score']}/100")
        click.echo(f"Grade: {health_data['grade']}")
        click.echo(f"Status: {health_data['status']}")
        
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.command()
@click.argument('build_id')
def remediation(build_id):
    """View remediation suggestions for a build"""
    try:
        response = requests.get(f"{API_URL}/builds/{build_id}/remediation")
        response.raise_for_status()
        
        data = response.json()
        
        click.echo(f"\n🔧 Remediation Suggestions\n")
        click.echo(f"CIS Benchmark Score: {data['cis_benchmark']['score']}/100 (Grade: {data['cis_benchmark']['grade']})")
        click.echo(f"Passed: {data['cis_benchmark']['passed']} | Failed: {data['cis_benchmark']['failed']} | Warnings: {data['cis_benchmark']['warnings']}\n")
        
        if data['remediation_suggestions']:
            click.echo("Suggested Remediations:\n")
            for i, suggestion in enumerate(data['remediation_suggestions'], 1):
                click.echo(f"{i}. {suggestion['title']} [{suggestion['severity'].upper()}]")
                click.echo(f"   Effort: {suggestion['effort']}")
                click.echo(f"   Impact: {suggestion['impact']}\n")
        else:
            click.echo("✅ No remediations needed - all checks passed!")
        
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.command()
@click.option('--days', default=30, help='Number of days to analyze')
def analytics(days):
    """View build analytics and trends"""
    try:
        trends_response = requests.get(f"{API_URL}/analytics/trends?days={days}")
        success_response = requests.get(f"{API_URL}/analytics/success-rate?days={days}")
        health_response = requests.get(f"{API_URL}/analytics/health-scores")
        
        trends_response.raise_for_status()
        success_response.raise_for_status()
        health_response.raise_for_status()
        
        trends = trends_response.json()
        success = success_response.json()
        health = health_response.json()
        
        click.echo(f"\n📈 Analytics Report ({days} days)\n")
        
        click.echo(f"Build Success Rate: {success['success_rate']}%")
        click.echo(f"Total Builds: {success['total_builds']} (Completed: {success['completed']}, Failed: {success['failed']})\n")
        
        click.echo(f"Average Health Score: {health['average_health_score']}/100")
        click.echo(f"Grade Distribution:")
        for grade, count in health['grade_distribution'].items():
            if count > 0:
                click.echo(f"  {grade}: {count}")
        
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.group()
def registry():
    """Manage container registries"""
    pass

@registry.command('list')
def list_registries():
    """List all configured registries"""
    try:
        response = requests.get(f"{API_URL}/registries")
        response.raise_for_status()
        
        registries = response.json()
        
        if not registries:
            click.echo("No registries configured.")
            return
        
        click.echo(f"\n📦 Configured Registries\n")
        click.echo(f"{'Name':<20} {'Type':<15} {'URL':<40}")
        click.echo("-" * 80)
        
        for reg in registries:
            click.echo(f"{reg['name']:<20} {reg['type']:<15} {reg['url']:<40}")
            
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@registry.command('add')
@click.option('--name', required=True, help='Registry name')
@click.option('--type', required=True, type=click.Choice(['jfrog', 'acr', 'dockerhub']), help='Registry type')
@click.option('--url', required=True, help='Registry URL')
@click.option('--username', required=True, help='Username')
@click.option('--password', required=True, prompt=True, hide_input=True, help='Password')
def add_registry(name, type, url, username, password):
    """Add a new registry"""
    try:
        payload = {
            "name": name,
            "type": type,
            "url": url,
            "username": username,
            "password": password
        }
        
        response = requests.post(f"{API_URL}/registries", json=payload)
        response.raise_for_status()
        
        click.echo(f"\n✅ Registry '{name}' added successfully!")
        
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

# Phase 3 Commands
@cli.group()
def policy():
    """Manage security policies"""
    pass

@policy.command('list')
def list_policies():
    """List all policies"""
    try:
        response = requests.get(f"{API_URL}/policies")
        response.raise_for_status()
        
        policies = response.json()
        
        if not policies:
            click.echo("No policies configured.")
            return
        
        click.echo(f"\n📋 Security Policies\n")
        click.echo(f"{'Name':<30} {'Type':<15} {'Enforcement':<12} {'Enabled':<8}")
        click.echo("-" * 70)
        
        for pol in policies:
            enabled = "✓" if pol['enabled'] else "✗"
            click.echo(f"{pol['name']:<30} {pol['type']:<15} {pol['enforcement']:<12} {enabled:<8}")
            
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@policy.command('templates')
def policy_templates():
    """View available policy templates"""
    try:
        response = requests.get(f"{API_URL}/policies/templates")
        response.raise_for_status()
        
        templates = response.json()['templates']
        
        click.echo(f"\n📚 Available Policy Templates\n")
        
        for key, template in templates.items():
            click.echo(f"Template: {key}")
            click.echo(f"  Name: {template['name']}")
            click.echo(f"  Type: {template['type']}")
            click.echo(f"  Enforcement: {template['enforcement']}")
            click.echo(f"  Description: {template['description']}\n")
            
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@policy.command('evaluate')
@click.argument('build_id')
def evaluate_policies(build_id):
    """Evaluate all policies against a build"""
    try:
        response = requests.post(f"{API_URL}/builds/{build_id}/evaluate-policies")
        response.raise_for_status()
        
        result = response.json()
        
        click.echo(f"\n🔍 Policy Evaluation Results\n")
        click.echo(f"Build ID: {build_id}")
        click.echo(f"Total Policies: {result['total_policies']}")
        click.echo(f"Passed: {result['passed']}")
        click.echo(f"Failed: {result['failed']}")
        click.echo(f"Overall Status: {result['overall_status'].upper()}\n")
        
        if result.get('blocks'):
            click.echo("❌ BLOCKING Issues:")
            for block in result['blocks']:
                click.echo(f"  - {block['message']}")
            click.echo()
        
        if result.get('warnings'):
            click.echo("⚠️  Warnings:")
            for warn in result['warnings']:
                click.echo(f"  - {warn['message']}")
            
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.command()
@click.argument('build_id')
def updates(build_id):
    """Check for base image and runtime updates"""
    try:
        response = requests.get(f"{API_URL}/builds/{build_id}/check-updates")
        response.raise_for_status()
        
        data = response.json()
        
        click.echo(f"\n🔄 Update Check Results\n")
        click.echo(f"Build ID: {build_id}")
        
        if not data['update_info']['has_updates']:
            click.echo("\n✅ No updates available - image is current!")
            return
        
        click.echo(f"\nBase Image Updates:")
        base = data['update_info']['base_image_updates']
        click.echo(f"  Current: {base['current']}")
        click.echo(f"  Latest: {base['latest']}")
        click.echo(f"  EOL Date: {base.get('eol_date', 'N/A')}")
        
        click.echo(f"\nRuntime Updates:")
        runtime = data['update_info']['runtime_updates']
        click.echo(f"  Current: {runtime['current']}")
        click.echo(f"  Latest: {runtime['latest']}")
        click.echo(f"  LTS Versions: {', '.join(runtime['lts_versions'])}")
        
        click.echo(f"\nRecommendation:")
        rec = data['recommendation']
        click.echo(f"  Action: {rec['action'].upper()}")
        click.echo(f"  Priority: {rec['priority']}")
        click.echo(f"  Message: {rec['message']}")
        
        if data.get('cves_fixed_by_update'):
            click.echo(f"\nCVEs Fixed by Update:")
            for cve in data['cves_fixed_by_update']:
                click.echo(f"  - {cve['id']} ({cve['severity']}): {cve['description']}")
        
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

@cli.command()
@click.argument('build_id')
def verify(build_id):
    """Verify image signature"""
    try:
        response = requests.get(f"{API_URL}/builds/{build_id}/signature")
        response.raise_for_status()
        
        data = response.json()
        sig = data['signature']
        ver = data['verification']
        
        click.echo(f"\n🔐 Image Signature Verification\n")
        click.echo(f"Build ID: {build_id}")
        click.echo(f"Image: {sig['image_tag']}")
        click.echo(f"Digest: {sig['digest']}")
        click.echo(f"Signature ID: {sig['signature_id']}")
        click.echo(f"Signed At: {sig['signed_at']}")
        click.echo(f"Signing Method: {sig['signing_method']}")
        
        click.echo(f"\nVerification Status:")
        if ver['verified']:
            click.echo(f"  ✅ Signature VALID")
            click.echo(f"  Trust Root: {ver['trust_root']}")
            click.echo(f"  Rekor Verified: {'✓' if ver['rekor_verified'] else '✗'}")
            click.echo(f"  Certificate Verified: {'✓' if ver['certificate_verified'] else '✗'}")
        else:
            click.echo(f"  ❌ Signature INVALID")
            click.echo(f"  Error: {ver.get('error', 'Unknown error')}")
        
    except requests.exceptions.RequestException as e:
        click.echo(f"❌ Error: {str(e)}", err=True)
        exit(1)

if __name__ == '__main__':
    cli()