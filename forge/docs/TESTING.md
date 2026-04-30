# Testing strategy

## Test pyramid

| Layer | Where | What it covers |
|---|---|---|
| **Unit** | `crates/*/src/**/*` `#[cfg(test)] mod tests` | Pure functions, parsers, repo CRUD with `Storage::open_memory`, mocked subprocess adapters via `MockRunner` |
| **Property** | `crates/forge-core/tests/dockerfile_props.rs` | Invariants of the Dockerfile generator across every `(runtime × base × hardening)` combination using `proptest` |
| **Integration** | `crates/forge-core/tests/orchestrator_e2e.rs`, `crates/forge-api/tests/api.rs`, `crates/forge-cli/tests/cli.rs` | Full pipelines or HTTP flows wired against in-memory storage and `MockRunner` |

## Coverage gate

Run locally:

```bash
cd forge
cargo install cargo-llvm-cov --locked
cargo xtask coverage --min-percent 80
```

`cargo xtask coverage` runs `cargo llvm-cov --workspace --all-features`, writes
`target/coverage/lcov.info`, and parses the `TOTAL` row from the summary
report — the command exits non-zero when total line coverage drops below the
`--min-percent` floor (default 80%).

The same gate runs on every PR via [`.github/workflows/coverage.yml`](../../.github/workflows/coverage.yml).

## Local CI shortcuts

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo xtask coverage --min-percent 80
cargo xtask audit              # cargo audit on the locked graph
cargo xtask license-audit      # cargo deny check
```

## Mock conventions

- **Subprocess adapters** (`buildkit`, `trivy`, `grype`, `syft`, `cosign`,
  `opa`) are constructed with an `Arc<dyn ProcessRunner>`. Tests inject
  `MockRunner` and register `expect(matcher_fn, output)` rules; the runner
  records every call so assertions can verify the exact CLI invocations.
- **Storage** uses `Storage::open_memory()` for SQLite in-memory mode so each
  test starts on a fresh schema with all migrations applied.
- **Tools on PATH** are bypassed by setting an explicit `*_path` field in the
  adapter config (e.g. `TrivyConfig { trivy_path: Some("/bin/trivy".into()), .. }`)
  so the tests never call `which::which`.

## Flaky / slow / ignored

We don't currently quarantine any tests. Anything that takes >1s on the CI
runner gets refactored to use mocks rather than real subprocess execution; if
you find yourself reaching for `#[ignore]`, file an issue first.

## What's *not* tested in CI

| Area | Why | Where it gets tested |
|---|---|---|
| Real BuildKit / Trivy / Syft / Cosign / OPA invocations | Need rootful daemon access; we cover their CLI surface via parser-focused unit tests instead | Manual smoke tests + `xtask bundle-buildkit` artifact verification |
| Dioxus desktop UI | `dioxus-desktop` requires a windowing system in CI | Manual via `cargo run -p forge-desktop` |
| Tray menu | Same as above | Manual; gated behind `FORGE_ENABLE_TRAY=1` on macOS |

## Adding tests

When adding new functionality:

1. Pure functions → unit tests in the same module.
2. Subprocess wrappers → an adapter test using `MockRunner`.
3. New API route → an `axum` integration test in `crates/forge-api/tests/api.rs`
   driven by `reqwest` against a `tokio::spawn`-launched server.
4. New invariants of a generator → a `proptest!` block.
5. Update the [coverage gate](../../.github/workflows/coverage.yml) only if
   the new code makes the existing 80% floor unreachable; prefer adding tests
   to keep the floor meaningful.
