# Developer Experience

The DX layer makes the paved road feel like an upgrade, not a tax. It includes:

- **`pavedroad-cli/`** — the single CLI a developer uses for scaffold/doctor/migrate/promote/watch/rollback/cleanup.
- **`devcontainer/`** — VS Code devcontainer pinned to the same tooling image CI uses.
- **`docker-compose/`** — local Postgres + Redis + OTel collector + Jaeger so traces flow even on a laptop.
- **`makefiles/standard.mk`** — common targets every service includes.
- **`docs/QUICKSTART.md`** — 0-to-prod walkthrough.

## Install (one time)

```bash
pipx install -e developer-experience/pavedroad-cli   # local checkout
# or
brew install acme/tap/pavedroad                      # released tap
```

## Try it now (no real org needed)

```bash
export PAVEDROAD_ROOT=/app
pavedroad version
pavedroad new service --name demo-svc --language python --team platform --out /tmp
cd /tmp/demo-svc && ls -la
pavedroad doctor /tmp/demo-svc
```

By default the CLI runs in **dry-run** mode. Set `ACME_LIVE=1` to perform real GitOps PRs / Argo CD interactions.

## CI integration

- CI runs the same `make` targets that developers run locally (`make bootstrap`, `make test`, `make lint`, `make helm-lint`, `make image-publish`).
- The devcontainer image is the same `registry.acme.io/tooling/devcontainer:1.0.0` used as base for runners — keeps "works on my machine" honest.
