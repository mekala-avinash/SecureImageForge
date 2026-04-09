"""Image Signing Service with Cosign Simulation"""
from typing import Dict, Any, Optional, List
from datetime import datetime, timezone
import hashlib
import json
import uuid

def generate_image_digest(image_tag: str) -> str:
    """Generate a SHA256 digest for an image"""
    # In production, this would be the actual image digest
    # For simulation, we hash the image tag
    return hashlib.sha256(image_tag.encode()).hexdigest()

def sign_image(image_tag: str, build_id: str, sbom_path: Optional[str] = None) -> Dict[str, Any]:
    """Sign an image using keyless signing (simulated)"""
    digest = generate_image_digest(image_tag)
    signature_id = str(uuid.uuid4())
    
    signature = {
        "signature_id": signature_id,
        "image_tag": image_tag,
        "digest": f"sha256:{digest}",
        "signing_method": "keyless",
        "oidc_issuer": "https://token.actions.githubusercontent.com",
        "oidc_subject": "repo:secureforge/builds:ref:refs/heads/main",
        "signed_at": datetime.now(timezone.utc).isoformat(),
        "build_id": build_id,
        "transparency_log": {
            "log_id": str(uuid.uuid4()),
            "log_index": abs(hash(signature_id)) % 1000000,
            "integrated_time": datetime.now(timezone.utc).timestamp(),
            "log_entry_url": f"https://rekor.sigstore.dev/api/v1/log/entries/{str(uuid.uuid4())}"
        },
        "certificate": {
            "issuer": "CN=sigstore-intermediate,O=sigstore.dev",
            "subject": "secureforge-ci",
            "not_before": datetime.now(timezone.utc).isoformat(),
            "not_after": (datetime.now(timezone.utc).replace(year=datetime.now(timezone.utc).year + 1)).isoformat()
        }
    }
    
    # If SBOM exists, sign it too
    if sbom_path:
        sbom_digest = hashlib.sha256(sbom_path.encode()).hexdigest()
        signature["sbom_signature"] = {
            "sbom_digest": f"sha256:{sbom_digest}",
            "signed_at": datetime.now(timezone.utc).isoformat()
        }
    
    return signature

def verify_signature(signature_data: Dict[str, Any]) -> Dict[str, Any]:
    """Verify an image signature"""
    # Simulated verification
    is_valid = True
    
    # Check if signature is not expired
    try:
        not_after = datetime.fromisoformat(signature_data['certificate']['not_after'])
        if datetime.now(timezone.utc) > not_after:
            is_valid = False
    except:
        pass
    
    verification_result = {
        "verified": is_valid,
        "signature_id": signature_data.get('signature_id'),
        "image_tag": signature_data.get('image_tag'),
        "digest": signature_data.get('digest'),
        "verified_at": datetime.now(timezone.utc).isoformat(),
        "trust_root": "Sigstore Public Good",
        "rekor_verified": True,
        "certificate_verified": is_valid,
        "oidc_verified": is_valid
    }
    
    if not is_valid:
        verification_result["error"] = "Certificate expired or invalid"
    
    return verification_result

def get_signature_chain(image_tag: str) -> List[Dict[str, Any]]:
    """Get the signature chain for an image (all historical signatures)"""
    # Simulated chain - in production would query signature storage
    return [
        {
            "version": 1,
            "signed_at": datetime.now(timezone.utc).isoformat(),
            "signer": "secureforge-ci",
            "verified": True
        }
    ]

def generate_attestation(build_id: str, build_data: Dict[str, Any]) -> Dict[str, Any]:
    """Generate SLSA provenance attestation"""
    attestation = {
        "_type": "https://in-toto.io/Statement/v0.1",
        "predicateType": "https://slsa.dev/provenance/v0.2",
        "subject": [
            {
                "name": build_data.get('image_tag', ''),
                "digest": {
                    "sha256": generate_image_digest(build_data.get('image_tag', ''))
                }
            }
        ],
        "predicate": {
            "builder": {
                "id": "https://secureforge.dev/builder/v1"
            },
            "buildType": "https://secureforge.dev/build/v1",
            "invocation": {
                "configSource": {
                    "entryPoint": "build.yaml",
                    "digest": {
                        "sha256": build_id
                    }
                }
            },
            "metadata": {
                "buildStartedOn": build_data.get('started_at'),
                "buildFinishedOn": build_data.get('completed_at'),
                "completeness": {
                    "parameters": True,
                    "environment": True,
                    "materials": True
                },
                "reproducible": False
            },
            "materials": [
                {
                    "uri": f"base:{build_data.get('base_image', 'alpine')}",
                    "digest": {
                        "sha256": hashlib.sha256(build_data.get('base_image', 'alpine').encode()).hexdigest()
                    }
                }
            ]
        }
    }
    
    return attestation