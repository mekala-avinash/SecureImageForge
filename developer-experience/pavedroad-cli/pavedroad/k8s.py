"""Kubernetes adapter — thin wrapper over kubectl + the apps/v1 + argoproj APIs.

Design:
  * Uses `kubectl` as the transport (no kubeconfig parsing needed in-process).
  * Produces typed dataclasses so callers don't grovel over JSON.
  * All commands accept a kube-context override; `None` means current-context.
"""
from __future__ import annotations

import json
import shutil
import subprocess
from dataclasses import dataclass
from typing import Any, Optional


class KubectlNotFound(RuntimeError):
    pass


def _kubectl(*args: str, context: Optional[str] = None, namespace: Optional[str] = None) -> dict[str, Any]:
    if not shutil.which("kubectl"):
        raise KubectlNotFound("`kubectl` binary not found on PATH.")
    cmd = ["kubectl"]
    if context:
        cmd += ["--context", context]
    if namespace:
        cmd += ["-n", namespace]
    cmd += list(args)
    cmd += ["-o", "json"]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"kubectl failed: {res.stderr.strip() or res.stdout.strip()}")
    return json.loads(res.stdout) if res.stdout.strip() else {}


def _kubectl_raw(*args: str, context: Optional[str] = None, namespace: Optional[str] = None) -> str:
    if not shutil.which("kubectl"):
        raise KubectlNotFound("`kubectl` binary not found on PATH.")
    cmd = ["kubectl"]
    if context:
        cmd += ["--context", context]
    if namespace:
        cmd += ["-n", namespace]
    cmd += list(args)
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"kubectl failed: {res.stderr.strip() or res.stdout.strip()}")
    return res.stdout


@dataclass
class DeploymentStatus:
    name: str
    namespace: str
    replicas: int
    ready: int
    available: int
    image: str
    age: str


def get_deployment(name: str, *, namespace: str, context: Optional[str] = None) -> DeploymentStatus:
    obj = _kubectl("get", "deployment", name, context=context, namespace=namespace)
    s = obj.get("status", {})
    spec = obj.get("spec", {})
    image = ""
    try:
        image = spec["template"]["spec"]["containers"][0]["image"]
    except (KeyError, IndexError):
        pass
    return DeploymentStatus(
        name=name,
        namespace=namespace,
        replicas=spec.get("replicas", 0),
        ready=s.get("readyReplicas", 0),
        available=s.get("availableReplicas", 0),
        image=image,
        age=obj.get("metadata", {}).get("creationTimestamp", ""),
    )


def list_pods(label_selector: str, *, namespace: str, context: Optional[str] = None) -> list[dict[str, Any]]:
    obj = _kubectl("get", "pods", "-l", label_selector, context=context, namespace=namespace)
    return obj.get("items", [])


def stream_logs(label_selector: str, *, namespace: str, context: Optional[str] = None, tail: int = 100) -> str:
    return _kubectl_raw(
        "logs", "-l", label_selector, "--all-containers", "--tail", str(tail), "--prefix",
        context=context, namespace=namespace,
    )


def apply_manifest(path: str, *, namespace: Optional[str] = None, context: Optional[str] = None) -> str:
    return _kubectl_raw("apply", "-f", path, context=context, namespace=namespace)


def ensure_namespace(ns: str, *, context: Optional[str] = None) -> None:
    """Idempotently create a namespace with paved-road labels."""
    if not shutil.which("kubectl"):
        raise KubectlNotFound("`kubectl` not found.")
    cmd = ["kubectl"]
    if context:
        cmd += ["--context", context]
    # apply server-side from stdin so the operation is idempotent.
    body = (
        f"apiVersion: v1\nkind: Namespace\nmetadata:\n  name: {ns}\n"
        f"  labels:\n    pod-security.kubernetes.io/enforce: restricted\n"
        f"    pod-security.kubernetes.io/audit: restricted\n"
        f"    paved-road: \"true\"\n"
    )
    res = subprocess.run(cmd + ["apply", "-f", "-"], input=body, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"kubectl apply ns failed: {res.stderr.strip()}")
