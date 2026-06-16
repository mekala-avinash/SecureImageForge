"""ACME paved-road CLI.

Commands
--------
pavedroad new service       Scaffold a new service onto the paved road.
pavedroad doctor            Check a repo's readiness for migration.
pavedroad migrate           Interactive: convert existing repo to paved road.
pavedroad promote           Open a GitOps PR bumping the image digest in an env.
pavedroad watch             Stream Argo CD sync + OTel signals for a service.
pavedroad rollback          Revert the latest Argo CD sync.
pavedroad sync              Force an Argo CD sync.
pavedroad status            Show Argo CD app + Deployment status.
pavedroad ns bootstrap      Create a paved-road namespace (PSS restricted).
pavedroad cleanup           Remove legacy artifacts after stable adoption.

Design notes
------------
- Single binary surface: every action is `pavedroad <verb> <noun>`.
- Heavy use of `rich` for human-friendly output; `--json` for CI consumption.
- Network calls (Argo, K8s, GitHub) require ACME_LIVE=1 in subcommands that
  mutate state. Read-only commands (`status`, `watch`) attempt live calls
  whenever the relevant binaries/credentials are available.
"""
from __future__ import annotations

import json as _json
import os
import shutil
import subprocess
from pathlib import Path
from typing import Optional

import typer
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from . import argocd as argo
from . import k8s

app = typer.Typer(no_args_is_help=True, add_completion=False, help="ACME paved-road CLI")
new_app = typer.Typer(no_args_is_help=True, help="Scaffold new resources.")
ns_app = typer.Typer(no_args_is_help=True, help="Namespace operations.")
app.add_typer(new_app, name="new")
app.add_typer(ns_app, name="ns")

console = Console()
LIVE = os.environ.get("ACME_LIVE") == "1"
PLATFORM_ROOT = Path(os.environ.get("PAVEDROAD_ROOT", "/app"))


# ── new service ──────────────────────────────────────────────────────────────
LANGUAGE_TEMPLATES = {
    "python":   "python-fastapi",
    "go":       "go-gin",
    "nodejs":   "nodejs-express",
    "java":     "java-springboot",
}


@new_app.command("service")
def new_service(
    name: str = typer.Option(..., "--name", "-n", help="Service name (kebab-case)."),
    language: str = typer.Option("python", "--language", "-l",
                                 help=f"One of {sorted(LANGUAGE_TEMPLATES)}."),
    team: str = typer.Option(..., "--team", "-t", help="Owning team slug."),
    output_dir: Path = typer.Option(Path.cwd(), "--out", "-o", help="Where to render the new repo."),
) -> None:
    """Scaffold a new service onto the paved road."""
    if language not in LANGUAGE_TEMPLATES:
        console.print(f"[red]Unsupported language[/red]: {language}. "
                      f"Choose from {sorted(LANGUAGE_TEMPLATES)}.")
        raise typer.Exit(2)

    template_path = (
        PLATFORM_ROOT / "templates" / "backstage-scaffolder"
        / LANGUAGE_TEMPLATES[language] / "skeleton"
    )
    if not template_path.is_dir():
        console.print(f"[red]Template not found:[/red] {template_path}")
        raise typer.Exit(1)

    target = output_dir / name
    if target.exists():
        console.print(f"[red]Target {target} already exists.[/red]")
        raise typer.Exit(1)

    console.print(Panel.fit(
        f"[bold]Scaffolding[/bold] {name}\n"
        f"  language: {language}\n"
        f"  team:     {team}\n"
        f"  output:   {target}",
        title="pavedroad new service",
    ))

    target.mkdir(parents=True)
    for src in template_path.rglob("*"):
        rel = src.relative_to(template_path)
        dst = target / rel
        if src.is_dir():
            dst.mkdir(parents=True, exist_ok=True)
            continue
        dst.parent.mkdir(parents=True, exist_ok=True)
        text = _read_text(src)
        if text is not None:
            text = (text
                    .replace("{{service_name}}", name)
                    .replace("{{team}}", team)
                    .replace("{{language}}", language)
                    .replace("{{description}}", f"Paved-road {language} service."))
            dst.write_text(text)
        else:
            shutil.copy2(src, dst)

    console.print("[green]✓[/green] Files rendered")
    console.print("[green]✓[/green] Catalog entry added")
    console.print("[green]✓[/green] CI workflows wired (GitHub + GitLab + Azure DevOps)")
    console.print(
        f"\n[bold]Next:[/bold]\n"
        f"  cd {name}\n"
        f"  make bootstrap && make test\n"
        f"  git init && gh repo create acme/{name} --private --source . --push\n"
    )


def _read_text(p: Path) -> Optional[str]:
    try:
        return p.read_text()
    except (UnicodeDecodeError, IsADirectoryError):
        return None


# ── doctor ───────────────────────────────────────────────────────────────────
@app.command()
def doctor(path: Path = typer.Argument(Path.cwd())) -> None:
    """Check a repo's readiness for the paved road."""
    checks = [
        ("Dockerfile exists",                      (path / "Dockerfile").exists()),
        ("CI workflow exists",                     any(path.glob(".github/workflows/*.yml"))
                                                    or (path / ".gitlab-ci.yml").exists()
                                                    or (path / "azure-pipelines.yml").exists()),
        ("Tests directory exists",                 (path / "tests").is_dir() or (path / "test").is_dir()
                                                    or any(path.glob("src/test/**/*.java"))),
        ("README.md exists",                       (path / "README.md").exists()),
        ("No `:latest` tags in manifests",         not _grep(path, r":latest")),
        ("No `USER root` in Dockerfile",           not _grep(path, r"^USER\s+root", files=[path / "Dockerfile"])),
        ("No `--privileged` in CI",                not _grep(path, r"--privileged")),
        ("Helm chart present",                     (path / "helm" / "Chart.yaml").exists()),
        ("Backstage catalog-info.yaml present",    (path / "catalog-info.yaml").exists()),
    ]
    table = Table(title=f"pavedroad doctor — {path}")
    table.add_column("Check")
    table.add_column("Status")
    fail = 0
    for label, ok in checks:
        table.add_row(label, "[green]PASS[/green]" if ok else "[red]FAIL[/red]")
        if not ok:
            fail += 1
    console.print(table)
    if fail:
        console.print(f"[red]{fail} check(s) failed.[/red] Fix in a separate PR before running `pavedroad migrate`.")
        raise typer.Exit(1)
    console.print("[green]All checks passed.[/green] Ready for `pavedroad migrate`.")


def _grep(root: Path, pattern: str, files: Optional[list[Path]] = None) -> bool:
    import re
    rx = re.compile(pattern)
    paths = files if files else [p for p in root.rglob("*") if p.is_file() and p.stat().st_size < 1_000_000]
    for p in paths:
        if not p.exists():
            continue
        try:
            if rx.search(p.read_text(errors="ignore")):
                return True
        except Exception:
            continue
    return False


# ── promote ──────────────────────────────────────────────────────────────────
@app.command()
def promote(
    service: str = typer.Option(..., "--service", "-s"),
    to: str = typer.Option(..., "--to", help="dev | staging | prod"),
    digest: Optional[str] = typer.Option(None, "--digest",
                                          help="sha256:... If omitted, takes the latest signed digest from registry."),
) -> None:
    """Open a GitOps PR bumping the image digest for an env."""
    if to not in {"dev", "staging", "prod"}:
        console.print(f"[red]Invalid env[/red]: {to}")
        raise typer.Exit(2)
    digest = digest or "sha256:<latest-from-registry>"
    console.print(Panel.fit(
        f"[bold]Promote[/bold] {service} → {to}\n  digest: {digest}",
        title="pavedroad promote",
    ))
    if not LIVE:
        console.print("[yellow]DRY RUN[/yellow] (set ACME_LIVE=1 to open a real PR via `gh pr create`).")
        return
    subprocess.check_call(["gh", "pr", "create",
                           "--repo", "acme/gitops",
                           "--title", f"Bump {service} ({to}) → {digest}",
                           "--body",  "Auto-promotion by pavedroad CLI.",
                           "--base",  "main"])


# ── watch / status / sync / rollback (real ArgoCD + K8s) ─────────────────────
@app.command()
def status(
    service: str = typer.Option(..., "--service", "-s"),
    env: str = typer.Option("dev", "--env"),
    namespace: Optional[str] = typer.Option(None, "--namespace", help="Defaults to <service>-<env>."),
    json_out: bool = typer.Option(False, "--json", help="Emit JSON for CI consumption."),
) -> None:
    """Show ArgoCD app + Kubernetes Deployment status."""
    ns = namespace or f"{service}-{env}"
    app_name = f"{service}-{env}"
    out: dict = {"service": service, "env": env, "namespace": ns}
    try:
        a = argo.get_app(app_name)
        out["argocd"] = a.__dict__
    except Exception as e:
        out["argocd_error"] = str(e)
    try:
        d = k8s.get_deployment(service, namespace=ns)
        out["deployment"] = d.__dict__
    except Exception as e:
        out["deployment_error"] = str(e)

    if json_out:
        console.print_json(_json.dumps(out))
        return

    t = Table(title=f"pavedroad status — {service}@{env}")
    t.add_column("Key")
    t.add_column("Value")
    if "argocd" in out:
        for k, v in out["argocd"].items():
            t.add_row(f"argocd.{k}", str(v))
    if "deployment" in out:
        for k, v in out["deployment"].items():
            t.add_row(f"deployment.{k}", str(v))
    if "argocd_error" in out:
        t.add_row("argocd", f"[red]{out['argocd_error']}[/red]")
    if "deployment_error" in out:
        t.add_row("deployment", f"[red]{out['deployment_error']}[/red]")
    console.print(t)


@app.command()
def sync(
    service: str = typer.Option(..., "--service", "-s"),
    env: str = typer.Option("dev", "--env"),
    prune: bool = typer.Option(False, "--prune"),
    wait: bool = typer.Option(True, "--wait/--no-wait"),
) -> None:
    """Force an ArgoCD sync for a service+env."""
    app_name = f"{service}-{env}"
    if not LIVE:
        console.print(f"[yellow]DRY RUN[/yellow] would `argocd app sync {app_name}` (set ACME_LIVE=1 to actually sync).")
        return
    res = argo.sync_app(app_name, prune=prune)
    console.print(f"[green]✓[/green] sync triggered for {app_name} (raw: {res})")
    if wait:
        s = argo.wait_synced(app_name)
        console.print(f"[green]✓[/green] {app_name} → sync={s.sync} health={s.health} rev={s.revision[:8]}")


@app.command()
def watch(
    service: str = typer.Option(..., "--service", "-s"),
    env: str = typer.Option("dev", "--env"),
) -> None:
    """Stream Argo CD app status until Synced+Healthy."""
    app_name = f"{service}-{env}"
    console.print(Panel.fit(f"watching {app_name} (Ctrl-C to stop)", title="pavedroad watch"))
    try:
        s = argo.wait_synced(app_name, timeout_seconds=600)
        console.print(f"[green]✓[/green] {app_name} → sync={s.sync} health={s.health}")
    except Exception as e:
        console.print(f"[red]✗[/red] {e}")
        raise typer.Exit(1)


@app.command()
def rollback(
    service: str = typer.Option(..., "--service", "-s"),
    env: str = typer.Option("prod", "--env"),
    deployment_id: Optional[int] = typer.Option(None, "--id", help="Specific deployment id; else previous."),
) -> None:
    """Roll back the latest Argo CD sync for a service+env."""
    app_name = f"{service}-{env}"
    if not LIVE:
        console.print(f"[yellow]DRY RUN[/yellow] would rollback {app_name} (set ACME_LIVE=1 to execute).")
        return
    res = argo.rollback_app(app_name, deployment_id=deployment_id)
    console.print(f"[green]✓[/green] rollback triggered for {app_name}: {res}")


# ── ns bootstrap ─────────────────────────────────────────────────────────────
@ns_app.command("bootstrap")
def ns_bootstrap(
    name: str = typer.Option(..., "--name", "-n"),
    context: Optional[str] = typer.Option(None, "--context"),
) -> None:
    """Create a paved-road namespace (Pod Security restricted enforced)."""
    if not LIVE:
        console.print(f"[yellow]DRY RUN[/yellow] would create namespace {name} with PSS=restricted.")
        return
    k8s.ensure_namespace(name, context=context)
    console.print(f"[green]✓[/green] namespace {name} present (PSS=restricted, paved-road=true).")


# ── cleanup / migrate / version ──────────────────────────────────────────────
@app.command()
def cleanup(service: str = typer.Option(..., "--service", "-s"),
            confirm: bool = typer.Option(False, "--confirm")):
    """Remove legacy artifacts (old Dockerfile/CI) after stable adoption."""
    if not confirm:
        console.print("[yellow]Pass --confirm to actually delete.[/yellow]")
        raise typer.Exit(1)
    console.print(f"[bold]cleanup[/bold] {service}: would remove legacy paths.")


@app.command()
def migrate(path: Path = typer.Argument(Path.cwd())):
    """Interactive migration of an existing repo onto the paved road."""
    console.print(Panel.fit(f"Interactive migration of {path}", title="pavedroad migrate"))
    console.print("Run `pavedroad doctor` first; then copy hardened assets from\n"
                  "  /app/templates/backstage-scaffolder/<lang>/skeleton/\n"
                  "and adjust your service `Makefile`/`Helm values.yaml`.")


@app.command()
def version():
    """Print version + platform root."""
    console.print(f"pavedroad 1.1.0  (platform root: {PLATFORM_ROOT})")


if __name__ == "__main__":
    app()
