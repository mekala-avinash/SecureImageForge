"""Runtime Version and Distribution Matrix"""
from typing import Dict, List, Any

# Runtime version matrices with distributions
RUNTIME_VERSIONS = {
    "java": {
        "versions": {
            "8": {
                "distributions": {
                    "temurin": {
                        "image": "eclipse-temurin:8-jre",
                        "fips_supported": False,
                        "eol_date": "2026-11-30"
                    },
                    "corretto": {
                        "image": "amazoncorretto:8",
                        "fips_supported": True,
                        "eol_date": "2026-05-31"
                    }
                }
            },
            "11": {
                "distributions": {
                    "temurin": {
                        "image": "eclipse-temurin:11-jre",
                        "fips_supported": False,
                        "eol_date": "2027-09-30"
                    },
                    "corretto": {
                        "image": "amazoncorretto:11",
                        "fips_supported": True,
                        "eol_date": "2027-09-30"
                    },
                    "microsoft": {
                        "image": "mcr.microsoft.com/openjdk/jdk:11-ubuntu",
                        "fips_supported": True,
                        "eol_date": "2027-09-30"
                    }
                }
            },
            "17": {
                "distributions": {
                    "temurin": {
                        "image": "eclipse-temurin:17-jre",
                        "fips_supported": False,
                        "eol_date": "2029-09-30"
                    },
                    "corretto": {
                        "image": "amazoncorretto:17",
                        "fips_supported": True,
                        "eol_date": "2029-10-31"
                    },
                    "microsoft": {
                        "image": "mcr.microsoft.com/openjdk/jdk:17-ubuntu",
                        "fips_supported": True,
                        "eol_date": "2029-09-30"
                    }
                },
                "recommended": True,
                "lts": True
            },
            "21": {
                "distributions": {
                    "temurin": {
                        "image": "eclipse-temurin:21-jre",
                        "fips_supported": False,
                        "eol_date": "2031-09-30"
                    },
                    "corretto": {
                        "image": "amazoncorretto:21",
                        "fips_supported": True,
                        "eol_date": "2031-09-30"
                    },
                    "microsoft": {
                        "image": "mcr.microsoft.com/openjdk/jdk:21-ubuntu",
                        "fips_supported": True,
                        "eol_date": "2031-09-30"
                    }
                },
                "recommended": True,
                "lts": True
            }
        },
        "default_version": "17",
        "default_distribution": "temurin"
    },
    "dotnet": {
        "versions": {
            "6.0": {
                "distributions": {
                    "microsoft": {
                        "image": "mcr.microsoft.com/dotnet/aspnet:6.0",
                        "fips_supported": True,
                        "eol_date": "2024-11-12"
                    }
                },
                "lts": True
            },
            "7.0": {
                "distributions": {
                    "microsoft": {
                        "image": "mcr.microsoft.com/dotnet/aspnet:7.0",
                        "fips_supported": True,
                        "eol_date": "2024-05-14"
                    }
                }
            },
            "8.0": {
                "distributions": {
                    "microsoft": {
                        "image": "mcr.microsoft.com/dotnet/aspnet:8.0",
                        "fips_supported": True,
                        "eol_date": "2026-11-10"
                    }
                },
                "recommended": True,
                "lts": True
            }
        },
        "default_version": "8.0",
        "default_distribution": "microsoft"
    },
    "go": {
        "versions": {
            "1.20": {
                "distributions": {
                    "official": {
                        "image": "golang:1.20",
                        "fips_supported": False,
                        "eol_date": "2024-02-06"
                    }
                }
            },
            "1.21": {
                "distributions": {
                    "official": {
                        "image": "golang:1.21",
                        "fips_supported": False,
                        "eol_date": "2024-08-06"
                    }
                }
            },
            "1.22": {
                "distributions": {
                    "official": {
                        "image": "golang:1.22",
                        "fips_supported": False,
                        "eol_date": "2025-02-06"
                    }
                },
                "recommended": True
            }
        },
        "default_version": "1.22",
        "default_distribution": "official"
    },
    "nodejs": {
        "versions": {
            "18": {
                "distributions": {
                    "official": {
                        "image": "node:18",
                        "fips_supported": False,
                        "eol_date": "2025-04-30"
                    }
                },
                "lts": True
            },
            "20": {
                "distributions": {
                    "official": {
                        "image": "node:20",
                        "fips_supported": False,
                        "eol_date": "2026-04-30"
                    }
                },
                "recommended": True,
                "lts": True
            },
            "21": {
                "distributions": {
                    "official": {
                        "image": "node:21",
                        "fips_supported": False,
                        "eol_date": "2024-06-01"
                    }
                }
            }
        },
        "default_version": "20",
        "default_distribution": "official"
    }
}

# Base image tag catalog
BASE_IMAGE_TAGS = {
    "alpine": {
        "tags": {
            "3.18.4": {"security_status": "patched", "release_date": "2023-09-28"},
            "3.18.5": {"security_status": "patched", "release_date": "2023-11-30"},
            "3.19.0": {"security_status": "patched", "release_date": "2023-12-07"},
            "3.19.1": {"security_status": "current", "release_date": "2024-01-26"},
            "3.20.0": {"security_status": "latest", "release_date": "2024-05-23"},
            "latest": {"security_status": "rolling", "release_date": "rolling"}
        },
        "recommended_tag": "3.20.0",
        "stable_tag": "3.19.1"
    },
    "debian": {
        "tags": {
            "11-slim": {"codename": "bullseye", "security_status": "old_stable", "eol": "2026-06-30"},
            "12-slim": {"codename": "bookworm", "security_status": "stable", "eol": "2028-06-10"},
            "bookworm-slim": {"codename": "bookworm", "security_status": "stable", "eol": "2028-06-10"},
            "trixie-slim": {"codename": "trixie", "security_status": "testing", "eol": "TBD"},
            "latest": {"security_status": "rolling", "eol": "rolling"}
        },
        "recommended_tag": "12-slim",
        "stable_tag": "bookworm-slim"
    },
    "distroless": {
        "tags": {
            "latest": {"security_status": "rolling"},
            "debug": {"security_status": "rolling", "has_shell": True},
            "nonroot": {"security_status": "rolling", "user": "nonroot"}
        },
        "recommended_tag": "latest",
        "stable_tag": "nonroot"
    }
}

# CIS Benchmark configurations
CIS_LEVELS = {
    "level1": {
        "name": "CIS Level 1 - Base Security",
        "description": "Baseline hardening suitable for most environments",
        "controls": [
            "non_root_user",
            "no_ssh_server",
            "minimal_packages",
            "security_updates"
        ],
        "strictness": "moderate"
    },
    "level2": {
        "name": "CIS Level 2 - High Security",
        "description": "Aggressive hardening for high-security environments",
        "controls": [
            "non_root_user",
            "no_ssh_server",
            "minimal_packages",
            "security_updates",
            "remove_shell",
            "remove_package_manager",
            "read_only_filesystem",
            "drop_all_capabilities",
            "no_new_privileges",
            "seccomp_profile"
        ],
        "strictness": "strict"
    }
}

# SBOM configuration options
SBOM_FORMATS = {
    "cyclonedx": {
        "name": "CycloneDX",
        "version": "1.4",
        "mime_type": "application/vnd.cyclonedx+json",
        "extension": ".cdx.json",
        "spec_url": "https://cyclonedx.org/specification/overview/"
    },
    "spdx": {
        "name": "SPDX",
        "version": "2.3",
        "mime_type": "application/spdx+json",
        "extension": ".spdx.json",
        "spec_url": "https://spdx.github.io/spdx-spec/"
    }
}

SBOM_SCAN_DEPTHS = {
    "os_only": {
        "name": "OS Packages Only",
        "description": "Scan only operating system packages",
        "includes": ["os_packages"],
        "performance": "fast"
    },
    "os_and_runtime": {
        "name": "OS + Runtime Dependencies",
        "description": "Include runtime-specific dependencies (Maven, npm, etc.)",
        "includes": ["os_packages", "runtime_deps"],
        "performance": "medium"
    },
    "full": {
        "name": "Full Application Scan",
        "description": "Deep scan including application dependencies and transitive deps",
        "includes": ["os_packages", "runtime_deps", "app_deps", "transitive_deps"],
        "performance": "slow"
    }
}

def get_runtime_versions(runtime: str) -> Dict[str, Any]:
    """Get available versions for a runtime"""
    return RUNTIME_VERSIONS.get(runtime, {})

def get_distributions(runtime: str, version: str) -> Dict[str, Any]:
    """Get available distributions for a runtime version"""
    runtime_data = RUNTIME_VERSIONS.get(runtime, {})
    version_data = runtime_data.get("versions", {}).get(version, {})
    return version_data.get("distributions", {})

def validate_runtime_config(runtime: str, version: str, distribution: str, fips_mode: bool = False) -> Dict[str, Any]:
    """Validate runtime configuration compatibility"""
    errors = []
    warnings = []
    
    if runtime not in RUNTIME_VERSIONS:
        errors.append(f"Unknown runtime: {runtime}")
        return {"valid": False, "errors": errors}
    
    runtime_data = RUNTIME_VERSIONS[runtime]
    
    if version not in runtime_data["versions"]:
        errors.append(f"Unknown version {version} for {runtime}")
        available = list(runtime_data["versions"].keys())
        warnings.append(f"Available versions: {', '.join(available)}")
    
    version_data = runtime_data["versions"].get(version, {})
    distributions = version_data.get("distributions", {})
    
    if distribution not in distributions:
        errors.append(f"Unknown distribution {distribution} for {runtime} {version}")
        available = list(distributions.keys())
        warnings.append(f"Available distributions: {', '.join(available)}")
    
    dist_data = distributions.get(distribution, {})
    
    if fips_mode and not dist_data.get("fips_supported", False):
        errors.append(f"{distribution} distribution does not support FIPS mode for {runtime} {version}")
        fips_dists = [d for d, data in distributions.items() if data.get("fips_supported")]
        if fips_dists:
            warnings.append(f"FIPS-compatible distributions: {', '.join(fips_dists)}")
    
    return {
        "valid": len(errors) == 0,
        "errors": errors,
        "warnings": warnings,
        "image": dist_data.get("image"),
        "eol_date": dist_data.get("eol_date")
    }
