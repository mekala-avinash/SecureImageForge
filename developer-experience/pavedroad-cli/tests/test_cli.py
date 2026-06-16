"""Regression tests for the pavedroad CLI.

These exercise scaffolding for every supported language without needing kubectl
or a live ArgoCD endpoint. Each language × concern is a separate parametrized
test so failures pinpoint exactly what regressed.
"""
from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

SUPPORTED: list[str] = ["python", "go", "nodejs", "java"]

# Files every language scaffold must produce.
COMMON_FILES: list[str] = [
    "Dockerfile",
    "Makefile",
    "README.md",
    "catalog-info.yaml",
    "helm/Chart.yaml",
    "helm/values.yaml",
    "helm/templates/all.yaml",
    ".github/workflows/build.yml",
]

# Language-specific entrypoint file.
LANGUAGE_ENTRYPOINT: dict[str, str] = {
    "python": "src/main.py",
    "go":     "cmd/server/main.go",
    "nodejs": "src/server.ts",
    "java":   "src/main/java/io/acme/Application.java",
}

PLACEHOLDER_TOKENS: tuple[str, ...] = ("{{service_name}}", "{{team}}", "{{language}}")
TEXT_SUFFIXES: set[str] = {".yaml", ".yml", ".md", ".json", ".xml", ".mk", ".py", ".go", ".ts", ".java"}


def _run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, "-m", "pavedroad.cli", *args],
        check=True, capture_output=True, text=True,
    )


def _scaffold(lang: str, tmp_path: Path, name: str | None = None) -> Path:
    """Scaffold a service of the given language and return its directory."""
    n = name or f"unit-{lang}"
    _run("new", "service", "-n", n, "-l", lang, "-t", "payments", "-o", str(tmp_path))
    return tmp_path / n


def test_version() -> None:
    out = _run("version").stdout
    assert "pavedroad" in out
    assert "1.1.0" in out


@pytest.mark.parametrize("lang", SUPPORTED)
def test_scaffold_succeeds(lang: str, tmp_path: Path) -> None:
    """Smoke: the CLI exits cleanly for every supported language."""
    target = _scaffold(lang, tmp_path)
    assert target.is_dir()


@pytest.mark.parametrize("lang", SUPPORTED)
@pytest.mark.parametrize("required", COMMON_FILES)
def test_scaffold_contains_required_file(lang: str, required: str, tmp_path: Path) -> None:
    """Every language must produce every common paved-road artifact."""
    target = _scaffold(lang, tmp_path)
    assert (target / required).is_file(), f"{lang}: missing {required}"


@pytest.mark.parametrize("lang", SUPPORTED)
def test_scaffold_has_language_entrypoint(lang: str, tmp_path: Path) -> None:
    target = _scaffold(lang, tmp_path)
    entrypoint = LANGUAGE_ENTRYPOINT[lang]
    assert (target / entrypoint).is_file(), f"{lang}: missing {entrypoint}"


@pytest.mark.parametrize("lang", SUPPORTED)
def test_scaffold_substitutes_all_tokens(lang: str, tmp_path: Path) -> None:
    """No raw `{{token}}` placeholders may leak into the rendered scaffold."""
    target = _scaffold(lang, tmp_path)
    leftovers = list(_find_token_leftovers(target))
    assert not leftovers, f"{lang}: leftover tokens in {leftovers[:5]}"


def _find_token_leftovers(target: Path) -> list[tuple[Path, str]]:
    """Yield (file, token) pairs for any rendered file still containing a placeholder."""
    leftovers: list[tuple[Path, str]] = []
    for f in target.rglob("*"):
        if not (f.is_file() and f.suffix in TEXT_SUFFIXES):
            continue
        text = f.read_text(errors="ignore")
        for token in PLACEHOLDER_TOKENS:
            if token in text:
                leftovers.append((f, token))
    return leftovers


def test_doctor_on_scaffold(tmp_path: Path) -> None:
    target = _scaffold("python", tmp_path, name="doctor-test")
    res = subprocess.run(
        [sys.executable, "-m", "pavedroad.cli", "doctor", str(target)],
        capture_output=True, text=True,
    )
    assert res.returncode == 0, res.stdout + res.stderr
