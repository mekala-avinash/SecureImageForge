"""Regression tests for the pavedroad CLI.

These exercise scaffolding for every supported language without needing kubectl
or a live ArgoCD endpoint.
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

SUPPORTED = ["python", "go", "nodejs", "java"]


def _run(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, "-m", "pavedroad.cli", *args],
        check=True, capture_output=True, text=True,
    )


def test_version():
    out = _run("version").stdout
    assert "pavedroad" in out
    assert "1.1.0" in out


@pytest.mark.parametrize("lang", SUPPORTED)
def test_new_service_scaffolds(lang: str, tmp_path: Path):
    name = f"unit-{lang}"
    _run("new", "service", "-n", name, "-l", lang, "-t", "payments", "-o", str(tmp_path))
    target = tmp_path / name
    assert (target / "Dockerfile").is_file()
    assert (target / "Makefile").is_file()
    assert (target / "README.md").is_file()
    assert (target / "catalog-info.yaml").is_file()
    assert (target / "helm" / "Chart.yaml").is_file()
    assert (target / "helm" / "values.yaml").is_file()
    assert (target / ".github" / "workflows" / "build.yml").is_file()

    # No untemplated placeholders should remain
    for f in target.rglob("*"):
        if f.is_file() and f.suffix in {".yaml", ".yml", ".md", ".json", ".xml", ".mk", ".py", ".go", ".ts", ".java"}:
            text = f.read_text(errors="ignore")
            for token in ("{{service_name}}", "{{team}}", "{{language}}"):
                assert token not in text, f"{f}: leftover {token}"


def test_doctor_on_scaffold(tmp_path: Path):
    _run("new", "service", "-n", "doctor-test", "-l", "python", "-t", "payments", "-o", str(tmp_path))
    res = subprocess.run(
        [sys.executable, "-m", "pavedroad.cli", "doctor", str(tmp_path / "doctor-test")],
        capture_output=True, text=True,
    )
    # doctor returns 0 on full pass.
    assert res.returncode == 0, res.stdout + res.stderr
