# Development setup

## Prerequisites

- **Rust 1.88.0** — install via [rustup](https://rustup.rs):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  The toolchain version is pinned in `rust-toolchain.toml`; rustup picks it up automatically when you run `cargo` inside `forge/`. The 1.88 floor is required by transitive deps (`idna_adapter`, `icu_*`, `home`) that ship `edition2024` and require rustc ≥1.86 / ≥1.88.

- **cargo-deny** for license/advisory checks:
  ```bash
  cargo install cargo-deny --locked
  ```

- **System packages** (Linux desktop builds): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libsoup-3.0-dev`, `pkg-config`.

## Common commands

All commands run from the `forge/` workspace root.

| Command | Purpose |
|---|---|
| `cargo build --workspace` | Build every crate |
| `cargo run -p forge-cli -- --help` | Run the CLI |
| `cargo run -p forge-desktop` | Launch the desktop app |
| `cargo run -p forge-api` | (Phase 1+) start the local HTTP API |
| `cargo test --workspace` | Run all tests |
| `cargo fmt --all` | Format |
| `cargo clippy --workspace --all-targets -- -D warnings` | Lint |
| `cargo xtask license-audit` | Run cargo-deny |
| `cargo xtask bundle-buildkit` | Download + verify pinned tool binaries into `forge/vendor/` |
| `cargo run -p forge-cli -- doctor` | Print resolved toolchain (bundled prefix + PATH fallback) |
| `cargo run -p forge-cli -- completions zsh` | Print shell completions |

## Configuration

`forge` reads `<data_dir>/config.toml` (default `~/.forge/config.toml`):

```toml
[buildkit]
addr = "unix:///run/user/1000/buildkit/buildkitd.sock"

[registry]
default_push   = false
default_target = "ghcr.io/example/forge"

[vendor]
prefix = "/path/to/forge/vendor"
```

Env overrides: `FORGE_BUILDKIT_ADDR`, `FORGE_VENDOR_PREFIX`, `FORGE_REGISTRY_TARGET`, `FORGE_REGISTRY_PUSH`, `FORGE_DATA_DIR`.

## Cross-compilation

Targets pinned in `rust-toolchain.toml`. CI matrix covers:
- `x86_64-apple-darwin`, `aarch64-apple-darwin`
- `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`

For local cross-builds use [`cross`](https://github.com/cross-rs/cross):
```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-gnu
```

## Repository layout

See [ARCHITECTURE.md](./ARCHITECTURE.md).

## Phase status

Phase 0 (foundation) ships only the skeleton — every CLI subcommand currently prints a "Phase 1" placeholder. Adapter implementations land in subsequent PRs.
