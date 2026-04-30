//! xtask — repo automation entry point.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Parser, Debug)]
#[command(name = "xtask", about = "Repo automation tasks")]
struct Cli {
    #[command(subcommand)]
    command: Task,
}

#[derive(Subcommand, Debug)]
enum Task {
    /// Run `cargo deny check` to enforce the Apache-2.0/MIT license policy.
    LicenseAudit,
    /// Fetch and verify rootless buildkitd + trivy + syft + cosign + opa for
    /// the supported platforms, writing checksums to `vendor/manifest.json`.
    BundleBuildkit {
        /// Platforms in `os/arch` form (e.g. linux/amd64).
        #[arg(long, default_values_t = vec![
            "linux/amd64".to_string(),
            "linux/arm64".to_string(),
            "darwin/amd64".to_string(),
            "darwin/arm64".to_string(),
        ])]
        platforms: Vec<String>,
        /// Output directory (default: `forge/vendor`).
        #[arg(long, default_value = "vendor")]
        out: PathBuf,
        /// Skip checksum verification (NOT recommended).
        #[arg(long, default_value_t = false)]
        no_verify: bool,
        /// Tools to fetch (default: all).
        #[arg(long, default_values_t = vec![
            "buildkit".to_string(),
            "trivy".to_string(),
            "syft".to_string(),
            "cosign".to_string(),
            "opa".to_string(),
        ])]
        tools: Vec<String>,
    },
    /// Run `cargo llvm-cov` and emit an LCOV + summary at `target/coverage/`.
    Coverage {
        /// Workspace coverage floor; fail the command if total line coverage
        /// drops below this percentage.
        #[arg(long, default_value_t = 80.0)]
        min_percent: f64,
    },
    /// Run `cargo audit` for advisories on the locked dependency graph.
    Audit,
    /// Build release bundles for the current host, write SHA-256 digests, and
    /// optionally cosign-sign the artifacts.
    Dist {
        /// Output directory for built artifacts.
        #[arg(long, default_value = "dist")]
        out: PathBuf,
        /// Cosign key (file path or KMS reference). When set, all artifacts
        /// AND the manifest are signed.
        #[arg(long, env = "COSIGN_KEY")]
        cosign_key: Option<String>,
        /// Override the version stamped into the manifest (default: workspace
        /// `Cargo.toml`).
        #[arg(long)]
        version: Option<String>,
        /// Channel ("stable" | "beta") stamped into the manifest.
        #[arg(long, default_value = "stable")]
        channel: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Task::LicenseAudit => license_audit(),
        Task::BundleBuildkit {
            platforms,
            out,
            no_verify,
            tools,
        } => bundle_buildkit(&platforms, &out, no_verify, &tools),
        Task::Coverage { min_percent } => coverage(min_percent),
        Task::Audit => audit(),
        Task::Dist {
            out,
            cosign_key,
            version,
            channel,
        } => dist(&out, cosign_key.as_deref(), version.as_deref(), &channel),
    }
}

fn coverage(min_percent: f64) -> Result<()> {
    let workspace = workspace_root()?;
    let out_dir = workspace.join("target").join("coverage");
    std::fs::create_dir_all(&out_dir)?;

    // cargo llvm-cov produces lcov + a JSON summary with workspace totals.
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "llvm-cov",
            "--workspace",
            "--all-features",
            "--lcov",
            "--output-path",
        ])
        .arg(out_dir.join("lcov.info"))
        .status()
        .context("cargo llvm-cov not installed; run `cargo install cargo-llvm-cov`")?;
    if !status.success() {
        bail!("cargo llvm-cov failed");
    }

    let summary = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["llvm-cov", "report", "--workspace", "--summary-only"])
        .output()
        .context("cargo llvm-cov report failed")?;
    let body = String::from_utf8_lossy(&summary.stdout);
    println!("{body}");

    let percent = parse_total_line_coverage(&body).unwrap_or(0.0);
    if percent < min_percent {
        bail!(
            "coverage {percent:.2}% below floor {min_percent:.2}% — add tests or lower the floor"
        );
    }
    Ok(())
}

fn parse_total_line_coverage(report: &str) -> Option<f64> {
    // The "TOTAL" row of cargo llvm-cov's summary table looks like:
    //   TOTAL   123   45   90.00%   ...
    // We pick the first percentage column on that row.
    for line in report.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("TOTAL") {
            return rest
                .split_whitespace()
                .find_map(|tok| tok.strip_suffix('%').and_then(|n| n.parse::<f64>().ok()));
        }
    }
    None
}

fn audit() -> Result<()> {
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["audit", "--deny", "warnings"])
        .status()
        .context("cargo audit not installed; run `cargo install cargo-audit`")?;
    if !status.success() {
        bail!("cargo audit reported advisories");
    }
    Ok(())
}

fn license_audit() -> Result<()> {
    let status = Command::new("cargo")
        .args(["deny", "check"])
        .status()
        .context("cargo deny not installed; run `cargo install cargo-deny`")?;
    if !status.success() {
        bail!("license audit failed");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// dist
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReleaseManifest {
    version: String,
    published_at: String,
    channel: String,
    releases: Vec<ReleaseEntry>,
    min_required: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReleaseEntry {
    platform: String,
    url: String,
    sha256: String,
    signature_url: Option<String>,
    size_bytes: u64,
}

fn dist(out: &Path, cosign_key: Option<&str>, version: Option<&str>, channel: &str) -> Result<()> {
    std::fs::create_dir_all(out).with_context(|| format!("creating dist dir {}", out.display()))?;

    // Build the release binaries for the *current* host.
    let host_platform = host_platform();
    println!("[xtask] building release binaries for {host_platform}");
    run_cargo(&[
        "build",
        "--release",
        "-p",
        "forge-cli",
        "-p",
        "forge-desktop",
    ])?;

    let target_dir = workspace_root()?.join("target/release");
    let cli_bin = target_dir.join(if cfg!(windows) { "forge.exe" } else { "forge" });
    let desktop_bin = target_dir.join(if cfg!(windows) {
        "forge-desktop.exe"
    } else {
        "forge-desktop"
    });

    let mut entries = Vec::new();
    for (label, src) in [("cli", &cli_bin), ("desktop", &desktop_bin)] {
        if !src.exists() {
            eprintln!(
                "[xtask] WARN: expected {} at {} — skipping",
                label,
                src.display()
            );
            continue;
        }
        let file_name = artifact_name(label, &host_platform, src);
        let dest = out.join(&file_name);
        std::fs::copy(src, &dest)
            .with_context(|| format!("copying {} → {}", src.display(), dest.display()))?;
        let bytes = std::fs::read(&dest)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha = hex::encode(hasher.finalize());
        let signature_url = if let Some(key) = cosign_key {
            cosign_sign_blob(&dest, key)?;
            Some(format!("{file_name}.sig"))
        } else {
            None
        };
        entries.push(ReleaseEntry {
            platform: host_platform.clone(),
            url: file_name.clone(),
            sha256: sha,
            signature_url,
            size_bytes: bytes.len() as u64,
        });
    }

    let resolved_version = version
        .map(str::to_string)
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
    let manifest = ReleaseManifest {
        version: resolved_version,
        published_at: now_rfc3339(),
        channel: channel.to_string(),
        releases: entries,
        min_required: None,
    };
    let manifest_path = out.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    if let Some(key) = cosign_key {
        cosign_sign_blob(&manifest_path, key)?;
    }
    println!("[xtask] wrote {}", manifest_path.display());
    Ok(())
}

fn run_cargo(args: &[&str]) -> Result<()> {
    let status = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(args)
        .status()
        .context("invoking cargo")?;
    if !status.success() {
        bail!("cargo {args:?} failed");
    }
    Ok(())
}

fn host_platform() -> String {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        other => other,
    };
    format!("{os}/{arch}")
}

fn artifact_name(label: &str, platform: &str, src: &Path) -> String {
    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{e}"))
        .unwrap_or_default();
    let safe_platform = platform.replace('/', "-");
    format!("forge-{label}-{safe_platform}{ext}")
}

fn cosign_sign_blob(file: &Path, key: &str) -> Result<()> {
    println!("[xtask] cosign sign-blob {}", file.display());
    let sig_path = file.with_extension(format!(
        "{}.sig",
        file.extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
    ));
    let status = Command::new("cosign")
        .args([
            "sign-blob",
            "--yes",
            "--key",
            key,
            "--output-signature",
            sig_path.to_string_lossy().as_ref(),
            file.to_string_lossy().as_ref(),
        ])
        .status()
        .context("invoking cosign")?;
    if !status.success() {
        bail!("cosign sign-blob failed for {}", file.display());
    }
    Ok(())
}

fn now_rfc3339() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    // Avoid pulling chrono into xtask just for this; SystemTime + epoch is
    // good enough for a build manifest stamp.
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

fn workspace_root() -> Result<PathBuf> {
    // The xtask binary always runs from the workspace root in CI; CARGO_MANIFEST_DIR
    // points at xtask/. Walk one level up.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    Ok(manifest.parent().map(Path::to_path_buf).unwrap_or(manifest))
}

// ---------------------------------------------------------------------------
// bundle-buildkit
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VendorManifest {
    generated_at: String,
    tools: Vec<VendorEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VendorEntry {
    name: String,
    version: String,
    platform: String,
    sha256: String,
    relative_path: String,
}

/// Pinned versions and Apache-2.0 licensed source.
const TOOL_VERSIONS: &[(&str, &str)] = &[
    ("buildkit", "v0.16.0"),
    ("trivy", "0.55.2"),
    ("syft", "v1.14.1"),
    ("cosign", "v2.4.1"),
    ("opa", "v0.68.0"),
];

fn version_of(tool: &str) -> Result<&'static str> {
    TOOL_VERSIONS
        .iter()
        .find(|(t, _)| *t == tool)
        .map(|(_, v)| *v)
        .ok_or_else(|| anyhow!("unknown tool: {tool}"))
}

#[tokio::main(flavor = "current_thread")]
async fn bundle_buildkit(
    platforms: &[String],
    out: &Path,
    no_verify: bool,
    tools: &[String],
) -> Result<()> {
    std::fs::create_dir_all(out)
        .with_context(|| format!("creating output dir {}", out.display()))?;
    let client = reqwest::Client::builder()
        .user_agent("forge-xtask/0.1")
        .build()?;

    let mut entries = Vec::new();
    for platform in platforms {
        let (os, arch) = platform
            .split_once('/')
            .ok_or_else(|| anyhow!("bad platform '{platform}', expected os/arch"))?;
        for tool in tools {
            let version = version_of(tool)?;
            let task = ToolFetch {
                tool: tool.clone(),
                version: version.into(),
                os: os.into(),
                arch: arch.into(),
            };
            println!("[xtask] fetching {tool} {version} for {os}/{arch}");
            match download_and_install(&client, &task, out, no_verify).await {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    eprintln!("[xtask] WARN: skip {tool} {os}/{arch}: {e}");
                }
            }
        }
    }

    let manifest = VendorManifest {
        generated_at: chrono_like_now(),
        tools: entries,
    };
    let manifest_path = out.join("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;
    println!("[xtask] wrote {}", manifest_path.display());
    Ok(())
}

#[derive(Clone)]
struct ToolFetch {
    tool: String,
    version: String,
    os: String,
    arch: String,
}

fn url_for(t: &ToolFetch) -> Result<(String, String)> {
    let v_no_v = t.version.trim_start_matches('v');
    let os_release = match t.os.as_str() {
        "darwin" => "darwin",
        "linux" => "linux",
        other => bail!("unsupported os {other}"),
    };
    Ok(match t.tool.as_str() {
        "buildkit" => {
            let archive = format!(
                "buildkit-{v}.{os}-{a}.tar.gz",
                v = t.version,
                os = os_release,
                a = t.arch
            );
            (
                format!(
                    "https://github.com/moby/buildkit/releases/download/{}/{}",
                    t.version, archive
                ),
                archive,
            )
        }
        "trivy" => {
            // trivy uses capitalized OS in its archive names
            let os_cap = match os_release {
                "darwin" => "macOS",
                "linux" => "Linux",
                _ => unreachable!(),
            };
            let arch_cap = match t.arch.as_str() {
                "amd64" => "64bit",
                "arm64" => "ARM64",
                other => bail!("unsupported arch {other} for trivy"),
            };
            let archive = format!("trivy_{v_no_v}_{os_cap}-{arch_cap}.tar.gz");
            (
                format!(
                    "https://github.com/aquasecurity/trivy/releases/download/v{v_no_v}/{archive}"
                ),
                archive,
            )
        }
        "syft" => {
            let archive = format!(
                "syft_{v}_{os}_{a}.tar.gz",
                v = t.version.trim_start_matches('v'),
                os = os_release,
                a = t.arch
            );
            (
                format!(
                    "https://github.com/anchore/syft/releases/download/{}/{}",
                    t.version, archive
                ),
                archive,
            )
        }
        "cosign" => {
            // cosign distributes raw binaries, no archive
            let exe = format!("cosign-{}-{}", os_release, t.arch);
            (
                format!(
                    "https://github.com/sigstore/cosign/releases/download/{}/{}",
                    t.version, exe
                ),
                exe,
            )
        }
        "opa" => {
            // opa also raw binaries, suffixed _<os>_<arch>_static
            let exe = format!("opa_{}_{}_static", os_release, t.arch);
            (
                format!(
                    "https://github.com/open-policy-agent/opa/releases/download/{}/{}",
                    t.version, exe
                ),
                exe,
            )
        }
        other => bail!("unknown tool {other}"),
    })
}

async fn download_and_install(
    client: &reqwest::Client,
    t: &ToolFetch,
    out: &Path,
    no_verify: bool,
) -> Result<VendorEntry> {
    let (url, file_name) = url_for(t)?;
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("status for {url}"))?;
    let bytes = resp.bytes().await?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let archive_sha = hex::encode(hasher.finalize());

    let platform_dir = out.join(format!("{}/{}", t.os, t.arch));
    std::fs::create_dir_all(&platform_dir)?;

    // Verify against checksum file if available (best-effort).
    if !no_verify {
        if let Some(remote) = fetch_published_checksum(client, t, &file_name).await? {
            if remote != archive_sha {
                bail!("checksum mismatch for {file_name}: expected {remote}, got {archive_sha}");
            }
        }
    }

    let installed_path = match file_name.split('.').next_back() {
        Some("gz") => extract_tar_gz(&bytes, t, &platform_dir)?,
        _ => {
            let dest = platform_dir.join(binary_name(&t.tool));
            std::fs::write(&dest, &bytes)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&dest)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dest, perms)?;
            }
            dest
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(std::fs::read(&installed_path)?);
    let installed_sha = hex::encode(hasher.finalize());

    let relative = installed_path
        .strip_prefix(out)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| installed_path.display().to_string());

    Ok(VendorEntry {
        name: t.tool.clone(),
        version: t.version.clone(),
        platform: format!("{}/{}", t.os, t.arch),
        sha256: installed_sha,
        relative_path: relative,
    })
}

async fn fetch_published_checksum(
    client: &reqwest::Client,
    t: &ToolFetch,
    archive_file: &str,
) -> Result<Option<String>> {
    let checksum_url = match t.tool.as_str() {
        "buildkit" => format!(
            "https://github.com/moby/buildkit/releases/download/{}/buildkit-{}.{}-{}.tar.gz.sha256sum",
            t.version, t.version, t.os, t.arch
        ),
        "syft" => format!(
            "https://github.com/anchore/syft/releases/download/{}/syft_{}_checksums.txt",
            t.version,
            t.version.trim_start_matches('v')
        ),
        "trivy" => format!(
            "https://github.com/aquasecurity/trivy/releases/download/v{}/trivy_{}_checksums.txt",
            t.version.trim_start_matches('v'),
            t.version.trim_start_matches('v')
        ),
        "cosign" => format!(
            "https://github.com/sigstore/cosign/releases/download/{}/cosign_checksums.txt",
            t.version
        ),
        "opa" => return Ok(None), // opa publishes detached signatures, not a flat sums file
        _ => return Ok(None),
    };
    let resp = client.get(&checksum_url).send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let body = resp.text().await?;
    Ok(parse_sums_for(&body, archive_file))
}

fn parse_sums_for(body: &str, file_name: &str) -> Option<String> {
    for line in body.lines() {
        let mut parts = line.split_whitespace();
        let sha = parts.next()?;
        let path = parts.next().unwrap_or("");
        if path.trim_start_matches('*').ends_with(file_name)
            || path.trim_start_matches('*') == file_name
        {
            return Some(sha.to_string());
        }
    }
    None
}

fn extract_tar_gz(bytes: &[u8], t: &ToolFetch, dest_dir: &Path) -> Result<PathBuf> {
    use flate2::read::GzDecoder;
    let dec = GzDecoder::new(bytes);
    let mut archive = tar::Archive::new(dec);
    let target_name = binary_name(&t.tool);
    let mut found: Option<PathBuf> = None;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.to_path_buf();
        let file = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        if file == target_name || file == format!("{target_name}.exe") {
            let dest = dest_dir.join(&file);
            entry.unpack(&dest)?;
            found = Some(dest);
        }
    }
    found.ok_or_else(|| anyhow!("binary {target_name} not found in archive"))
}

fn binary_name(tool: &str) -> &str {
    match tool {
        "buildkit" => "buildctl", // we only need the client, daemon is shipped separately
        other => other,
    }
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_checksum_line() {
        let body = "deadbeef  ./trivy_0.55.2_Linux-64bit.tar.gz\nbadbeef  trivy_0.55.2_macOS-64bit.tar.gz\n";
        assert_eq!(
            parse_sums_for(body, "trivy_0.55.2_Linux-64bit.tar.gz"),
            Some("deadbeef".to_string())
        );
    }

    #[test]
    fn url_for_buildkit_linux() {
        let t = ToolFetch {
            tool: "buildkit".into(),
            version: "v0.16.0".into(),
            os: "linux".into(),
            arch: "amd64".into(),
        };
        let (url, archive) = url_for(&t).unwrap();
        assert!(url.contains("moby/buildkit/releases/download/v0.16.0"));
        assert_eq!(archive, "buildkit-v0.16.0.linux-amd64.tar.gz");
    }

    #[test]
    fn url_for_cosign_is_raw_binary() {
        let t = ToolFetch {
            tool: "cosign".into(),
            version: "v2.4.1".into(),
            os: "darwin".into(),
            arch: "arm64".into(),
        };
        let (_, archive) = url_for(&t).unwrap();
        assert_eq!(archive, "cosign-darwin-arm64");
    }

    #[test]
    fn parses_total_line_coverage() {
        let body = "Filename                          Regions    Missed   Cover\n\
                    TOTAL                                 100       10  85.50%   ...\n";
        assert_eq!(parse_total_line_coverage(body), Some(85.50));
    }

    #[test]
    fn missing_total_row_returns_none() {
        assert_eq!(parse_total_line_coverage("no total row here\n"), None);
    }

    #[test]
    fn binary_name_buildkit_to_buildctl() {
        assert_eq!(binary_name("buildkit"), "buildctl");
        assert_eq!(binary_name("trivy"), "trivy");
    }

    // walkdir + zip are wired for Phase 4 packaging; keep the imports alive
    // here so cargo check stays green even when not used yet.
    #[allow(dead_code)]
    fn _unused_imports_keepalive() {
        let _ = walkdir::WalkDir::new(".");
        let _ = zip::ZipArchive::new(std::io::Cursor::new(Vec::<u8>::new())).is_err();
    }
}
