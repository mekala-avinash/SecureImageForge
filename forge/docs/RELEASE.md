# Release & code signing

Phase 3.5 ships a tag-driven release pipeline that produces signed binaries
for macOS / Linux / Windows on both x86_64 and arm64, plus a Sigstore-signed
update manifest the in-app updater consumes.

## Tagged release flow

1. Bump `version` in `forge/Cargo.toml` (workspace package).
2. `git tag v0.X.Y && git push origin v0.X.Y`.
3. [`.github/workflows/release.yml`](../../.github/workflows/release.yml) runs:
   - Cross-builds `forge-cli` + `forge-desktop` for the 5 targets.
   - cosign keyless-signs every artifact (`*.sig` next to each binary).
   - Assembles `manifest.json` listing every artifact + SHA-256 + signature URL.
   - cosign-signs `manifest.json` itself → `manifest.json.sig`.
   - Publishes a GitHub Release with all assets attached.

The same artifacts can be produced locally:

```bash
cd forge
cargo xtask dist --cosign-key cosign.key --version 0.2.0
ls dist/
# forge-cli-darwin-arm64        forge-cli-darwin-arm64.sig
# forge-desktop-darwin-arm64    forge-desktop-darwin-arm64.sig
# manifest.json                 manifest.json.sig
```

## macOS code signing & notarization

Inputs (env vars):

| Variable | Source |
|---|---|
| `SIGNING_IDENTITY` | `Developer ID Application: <Name> (<TEAMID>)` from Apple Developer |
| `APPLE_ID` | Apple ID owning the team |
| `APPLE_TEAM_ID` | 10-char team ID |
| `APPLE_PASSWORD` | App-specific password generated at appleid.apple.com |

```bash
APP_BUNDLE=dist/SecureImageForge.app \
SIGNING_IDENTITY="Developer ID Application: SecureImage (TEAMID)" \
APPLE_ID="releases@secureimage.dev" \
APPLE_TEAM_ID="ABCDE12345" \
APPLE_PASSWORD="$(security find-generic-password -a releases -s notarytool -w)" \
forge/crates/forge-desktop/packaging/macos/notarize.sh
```

Entitlements live in [`packaging/macos/entitlements.plist`](../crates/forge-desktop/packaging/macos/entitlements.plist) — disabled library validation is required so the desktop binary can launch the bundled `buildctl`/`trivy`/`syft`/`cosign`/`opa`.

## Windows Authenticode

```powershell
$env:CERT_PATH = "C:\secrets\codesign.pfx"
$env:CERT_PASS = (Get-Content -Raw .\cert.pass)
.\forge\crates\forge-desktop\packaging\windows\sign.ps1 -Target ".\SecureImageForge-0.2.0.exe"
```

The NSIS template at [`packaging/windows/forge-desktop.nsi`](../crates/forge-desktop/packaging/windows/forge-desktop.nsi) produces the installer; `sign.ps1` then attaches the Authenticode signature with an RFC 3161 timestamp.

## Linux (.deb / AppImage)

```bash
forge/crates/forge-desktop/packaging/linux/build-deb.sh 0.2.0
```

The script uses [`fpm`](https://github.com/jordansissel/fpm) to assemble a `.deb` declaring runtime dependencies on `buildkit` and `trivy`. AppImage / RPM follow the same pattern (Phase 4).

## Update manifest format

The auto-updater consumes JSON of this shape:

```json
{
  "version": "0.2.0",
  "published_at": "2026-04-30T12:00:00Z",
  "channel": "stable",
  "min_required": null,
  "releases": [
    {
      "platform": "darwin/arm64",
      "url": "https://updates.secureimage-forge.dev/v0.2.0/forge-desktop-darwin-arm64",
      "sha256": "deadbeef...",
      "signature_url": "https://updates.secureimage-forge.dev/v0.2.0/forge-desktop-darwin-arm64.sig",
      "size_bytes": 28311552
    }
  ]
}
```

The desktop app:
1. Fetches `feed_url` (default `https://updates.secureimage-forge.dev/manifest.json`).
2. Verifies `manifest.json.sig` with `cosign verify-blob` against the embedded public key (or `updater.cosign_key_path` from config).
3. Picks the entry matching the host platform and proposes the update.

Set `[updater] allow_unsigned = true` only in dev builds.

## Tray menu

The desktop binary registers a tray icon on startup with three actions:

- **Show window** — restores the dashboard if minimized.
- **Check for updates** — runs the same flow as Settings → Check for updates.
- **Quit** — exits the process cleanly.

Tray installation is best-effort; on hosts without a system tray the app continues without it.
