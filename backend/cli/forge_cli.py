#!/usr/bin/env python3
"""SecureImage Forge CLI Tool"""

import click
import requests
import json
import time
import os
from typing import List

# Use environment variable or default to external URL
API_URL = os.environ.get('FORGE_API_URL', 'https://docker-vault.preview.emergentagent.com/api')

@click.group()
@click.version_option(version='1.0.0')
def cli():
    """SecureImage Forge - Build hardened Docker images with compliance"""
    pass

@cli.command()
@click.option('--name', required=True, help='Build configuration name')
@click.option('--runtime', type=click.Choice(['java', 'dotnet']), required=True, help='Runtime environment')
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

if __name__ == '__main__':
    cli()