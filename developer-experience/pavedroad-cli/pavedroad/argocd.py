"""ArgoCD adapter.

Two transports:
  * `argocd` CLI (preferred when present).
  * REST API via httpx (fallback when only ARGOCD_SERVER + ARGOCD_AUTH_TOKEN are set).

Both surfaces share the same dataclasses.
"""
from __future__ import annotations

import os
import shutil
import subprocess
from dataclasses import dataclass
from typing import Any, Optional

import httpx


class ArgoCDError(RuntimeError):
    pass


@dataclass
class AppStatus:
    name: str
    sync: str
    health: str
    revision: str
    repo: str
    path: str


def _via_cli() -> bool:
    return shutil.which("argocd") is not None


def _server() -> str:
    s = os.environ.get("ARGOCD_SERVER", "")
    if not s:
        raise ArgoCDError("ARGOCD_SERVER not set and `argocd` CLI not found.")
    return s.rstrip("/")


def _token() -> str:
    t = os.environ.get("ARGOCD_AUTH_TOKEN", "")
    if not t:
        raise ArgoCDError("ARGOCD_AUTH_TOKEN not set.")
    return t


def _api() -> httpx.Client:
    return httpx.Client(
        base_url=f"https://{_server()}/api/v1",
        headers={"Authorization": f"Bearer {_token()}"},
        timeout=30.0,
        verify=os.environ.get("ARGOCD_INSECURE", "0") != "1",
    )


def get_app(name: str) -> AppStatus:
    if _via_cli():
        res = subprocess.run(
            ["argocd", "app", "get", name, "-o", "json"],
            capture_output=True, text=True,
        )
        if res.returncode != 0:
            raise ArgoCDError(res.stderr.strip() or res.stdout.strip())
        import json as _json
        obj = _json.loads(res.stdout)
    else:
        with _api() as c:
            r = c.get(f"/applications/{name}")
            r.raise_for_status()
            obj = r.json()
    src = obj.get("spec", {}).get("source", {})
    st = obj.get("status", {})
    return AppStatus(
        name=name,
        sync=st.get("sync", {}).get("status", "Unknown"),
        health=st.get("health", {}).get("status", "Unknown"),
        revision=st.get("sync", {}).get("revision", ""),
        repo=src.get("repoURL", ""),
        path=src.get("path", ""),
    )


def sync_app(name: str, *, prune: bool = False) -> dict[str, Any]:
    if _via_cli():
        cmd = ["argocd", "app", "sync", name]
        if prune:
            cmd.append("--prune")
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode != 0:
            raise ArgoCDError(res.stderr.strip() or res.stdout.strip())
        return {"ok": True, "stdout": res.stdout}
    with _api() as c:
        r = c.post(f"/applications/{name}/sync", json={"prune": prune})
        r.raise_for_status()
        return r.json()


def rollback_app(name: str, *, deployment_id: Optional[int] = None) -> dict[str, Any]:
    """Roll back to the previous deployment (or a specific id when provided)."""
    if _via_cli():
        cmd = ["argocd", "app", "rollback", name]
        if deployment_id is not None:
            cmd.append(str(deployment_id))
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode != 0:
            raise ArgoCDError(res.stderr.strip() or res.stdout.strip())
        return {"ok": True, "stdout": res.stdout}
    with _api() as c:
        if deployment_id is None:
            # find previous successful deploy from history
            r = c.get(f"/applications/{name}")
            r.raise_for_status()
            history = r.json().get("status", {}).get("history") or []
            if len(history) < 2:
                raise ArgoCDError("no prior deployment to roll back to.")
            deployment_id = int(history[-2]["id"])
        r = c.post(f"/applications/{name}/rollback", json={"id": deployment_id})
        r.raise_for_status()
        return r.json()


def wait_synced(name: str, *, timeout_seconds: int = 600) -> AppStatus:
    if _via_cli():
        res = subprocess.run(
            ["argocd", "app", "wait", name, "--timeout", str(timeout_seconds)],
            capture_output=True, text=True,
        )
        if res.returncode != 0:
            raise ArgoCDError(res.stderr.strip() or res.stdout.strip())
        return get_app(name)
    # API: poll
    import time
    deadline = time.time() + timeout_seconds
    while time.time() < deadline:
        s = get_app(name)
        if s.sync == "Synced" and s.health == "Healthy":
            return s
        time.sleep(5)
    raise ArgoCDError(f"timed out waiting for {name} to become Synced+Healthy.")
