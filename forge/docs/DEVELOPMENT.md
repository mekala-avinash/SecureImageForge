# Development setup

## Prerequisites

- **Rust 1.83.0** — install via [rustup](https://rustup.rs):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  The toolchain version is pinned in `rust-toolchain.toml`; rustup picks it up automatically when you run `cargo` inside `forge/`.

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
