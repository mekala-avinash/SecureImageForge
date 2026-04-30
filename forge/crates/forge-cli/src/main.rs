use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use std::collections::BTreeSet;

use forge_core::adapters::buildkit::{BuildkitBuilder, BuildkitConfig};
use forge_core::adapters::cosign::{CosignConfig, CosignSigner};
use forge_core::adapters::grype::{GrypeConfig, GrypeScanner, MergedScanner};
use forge_core::adapters::opa::{OpaConfig, OpaPolicyEngine};
use forge_core::adapters::syft::{SyftConfig, SyftSbomGenerator};
use forge_core::adapters::trivy::{TrivyConfig, TrivyScanner};
use forge_core::config::Config;
use forge_core::domain::{
    Architecture, BaseImage, BuildSpec, ComplianceProfile, HardeningOptions, Runtime,
};
use forge_core::logs::LogStore;
use forge_core::orchestrator::BuildOrchestrator;
use forge_core::process::TokioRunner;
use forge_core::provenance::ProvenanceRepo;
use forge_core::repo::BuildRepo;
use forge_core::sarif;
use forge_core::storage::Storage;
use forge_core::telemetry;
use forge_core::toolchain::Toolchain;
use forge_core::tooling::PolicyDecision;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "forge",
    version,
    about = "SecureImage Forge CLI",
    propagate_version = true
)]
struct Cli {
    #[arg(long, value_enum, global = true, default_value_t = OutputFormat::Human)]
    output: OutputFormat,

    #[arg(short, long, action = ArgAction::Count, global = true)]
    verbose: u8,

    /// Override the data directory (default: $HOME/.forge).
    #[arg(long, env = "FORGE_DATA_DIR", global = true)]
    data_dir: Option<PathBuf>,

    /// BuildKit daemon address (env: FORGE_BUILDKIT_ADDR).
    #[arg(long, env = "FORGE_BUILDKIT_ADDR", global = true)]
    buildkit_addr: Option<String>,

    /// Vendor prefix for bundled tools (env: FORGE_VENDOR_PREFIX).
    #[arg(long, env = "FORGE_VENDOR_PREFIX", global = true)]
    vendor_prefix: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum OutputFormat {
    Human,
    Json,
    Sarif,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build, harden, scan, sign, and gate a container image.
    Build(BuildArgs),
    /// Show vulnerability scan results for a build.
    Scan { build_id: String },
    /// List recorded builds (most recent first).
    List,
    /// Print the persisted build log.
    Logs { build_id: String },
    /// Show aggregate statistics.
    Stats,
    /// Print the resolved toolchain (where each tool is found).
    Doctor,
    /// Run the local HTTP API daemon.
    Serve {
        #[arg(long, default_value = "127.0.0.1:7878")]
        addr: String,
    },
    /// Manage API principals (admin only).
    Principals {
        #[command(subcommand)]
        action: PrincipalsAction,
    },
    /// Generate shell completions.
    Completions { shell: Shell },
}

#[derive(Subcommand, Debug)]
enum PrincipalsAction {
    /// Create a principal and print the bearer token.
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, value_enum, default_value_t = RoleArg::Viewer)]
        role: RoleArg,
    },
    /// List configured principals (token hashes are not exposed).
    List,
    /// Revoke a principal by id.
    Revoke { id: String },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum RoleArg {
    Admin,
    Operator,
    Viewer,
}

impl From<RoleArg> for forge_core::rbac::Role {
    fn from(v: RoleArg) -> Self {
        match v {
            RoleArg::Admin => forge_core::rbac::Role::Admin,
            RoleArg::Operator => forge_core::rbac::Role::Operator,
            RoleArg::Viewer => forge_core::rbac::Role::Viewer,
        }
    }
}

#[derive(clap::Args, Debug)]
struct BuildArgs {
    #[arg(long)]
    name: String,
    #[arg(long, value_enum)]
    runtime: RuntimeArg,
    #[arg(long, value_enum)]
    base: BaseImageArg,
    #[arg(long, value_enum)]
    compliance: Vec<ComplianceArg>,
    #[arg(long, value_enum)]
    arch: Vec<ArchArg>,
    #[arg(long)]
    no_sbom: bool,
    #[arg(long)]
    no_sign: bool,
    /// Push to a registry (uses --target or config default).
    #[arg(long)]
    push: bool,
    /// Registry reference for the produced image (e.g. ghcr.io/org/img:tag).
    #[arg(long)]
    target: Option<String>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum RuntimeArg {
    Java,
    Dotnet,
    Go,
    Node,
    Python,
}
#[derive(Copy, Clone, Debug, ValueEnum)]
enum BaseImageArg {
    Alpine,
    Debian,
    Distroless,
}
#[derive(Copy, Clone, Debug, ValueEnum)]
enum ComplianceArg {
    Hipaa,
    Soc2,
    Pcidss,
    Cis,
    FedrampModerate,
}
#[derive(Copy, Clone, Debug, ValueEnum)]
enum ArchArg {
    Amd64,
    Arm64,
}

impl From<RuntimeArg> for Runtime {
    fn from(v: RuntimeArg) -> Self {
        match v {
            RuntimeArg::Java => Runtime::Java,
            RuntimeArg::Dotnet => Runtime::Dotnet,
            RuntimeArg::Go => Runtime::Go,
            RuntimeArg::Node => Runtime::Node,
            RuntimeArg::Python => Runtime::Python,
        }
    }
}
impl From<BaseImageArg> for BaseImage {
    fn from(v: BaseImageArg) -> Self {
        match v {
            BaseImageArg::Alpine => BaseImage::Alpine,
            BaseImageArg::Debian => BaseImage::Debian,
            BaseImageArg::Distroless => BaseImage::Distroless,
        }
    }
}
impl From<ComplianceArg> for ComplianceProfile {
    fn from(v: ComplianceArg) -> Self {
        match v {
            ComplianceArg::Hipaa => ComplianceProfile::Hipaa,
            ComplianceArg::Soc2 => ComplianceProfile::Soc2,
            ComplianceArg::Pcidss => ComplianceProfile::PciDss,
            ComplianceArg::Cis => ComplianceProfile::Cis,
            ComplianceArg::FedrampModerate => ComplianceProfile::FedrampModerate,
        }
    }
}
impl From<ArchArg> for Architecture {
    fn from(v: ArchArg) -> Self {
        match v {
            ArchArg::Amd64 => Architecture::Amd64,
            ArchArg::Arm64 => Architecture::Arm64,
        }
    }
}

fn data_dir(cli: &Cli) -> PathBuf {
    if let Some(p) = &cli.data_dir {
        return p.clone();
    }
    if let Some(home) = dirs_home() {
        return home.join(".forge");
    }
    PathBuf::from(".forge")
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init();
    let cli = Cli::parse();

    // Completions can be generated without touching the filesystem.
    if let Command::Completions { shell } = cli.command {
        let mut cmd = Cli::command();
        generate(shell, &mut cmd, "forge", &mut io::stdout());
        return Ok(());
    }

    let dir = data_dir(&cli);
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let cfg_path = dir.join("config.toml");
    let mut cfg =
        Config::load(&cfg_path).with_context(|| format!("loading {}", cfg_path.display()))?;
    if let Some(addr) = &cli.buildkit_addr {
        cfg.buildkit.addr = addr.clone();
    }
    if let Some(prefix) = &cli.vendor_prefix {
        cfg.vendor.prefix = Some(prefix.clone());
    }

    let db_path = dir.join("forge.sqlite");
    let storage = Storage::open(&db_path)
        .await
        .with_context(|| format!("opening db at {}", db_path.display()))?;
    let repo = BuildRepo::new(storage.clone());
    let logs = LogStore::new(dir.join("logs"));
    let toolchain = Toolchain::new(cfg.vendor.prefix.clone());

    match cli.command {
        Command::Build(args) => cmd_build(args, cli.output, repo, logs, &cfg, &toolchain).await,
        Command::Scan { build_id } => cmd_scan(&build_id, cli.output, repo).await,
        Command::List => cmd_list(cli.output, repo).await,
        Command::Logs { build_id } => cmd_logs(&build_id, &logs).await,
        Command::Stats => cmd_stats(cli.output, repo).await,
        Command::Doctor => cmd_doctor(&toolchain),
        Command::Serve { addr } => cmd_serve(&addr, storage, logs, cfg, toolchain).await,
        Command::Principals { action } => cmd_principals(action, storage, cli.output).await,
        Command::Completions { .. } => Ok(()),
    }
}

async fn cmd_serve(
    addr: &str,
    storage: Storage,
    logs: LogStore,
    cfg: Config,
    toolchain: Toolchain,
) -> Result<()> {
    let socket: std::net::SocketAddr = addr.parse().context("invalid --addr")?;
    let state = forge_api::ApiState::new(
        std::sync::Arc::new(cfg),
        storage,
        logs,
        std::sync::Arc::new(toolchain),
    );
    forge_api::serve(state, socket).await
}

async fn cmd_principals(
    action: PrincipalsAction,
    storage: Storage,
    format: OutputFormat,
) -> Result<()> {
    let repo = forge_core::rbac::PrincipalRepo::new(storage);
    match action {
        PrincipalsAction::Create { name, role } => {
            let created = repo.create(&name, role.into()).await?;
            match format {
                OutputFormat::Json | OutputFormat::Sarif => {
                    println!("{}", serde_json::to_string_pretty(&created)?);
                }
                OutputFormat::Human => {
                    println!("created principal {}", created.principal.id);
                    println!("name:  {}", created.principal.name);
                    println!("role:  {:?}", created.principal.role);
                    println!("token: {}", created.token);
                    println!("(this token is shown ONCE — store it now)");
                }
            }
        }
        PrincipalsAction::List => {
            let rows = repo.list().await?;
            match format {
                OutputFormat::Json | OutputFormat::Sarif => {
                    println!("{}", serde_json::to_string_pretty(&rows)?);
                }
                OutputFormat::Human => {
                    for r in rows {
                        println!(
                            "{:<8} {:<20} {:?}",
                            &r.id[..8.min(r.id.len())],
                            r.name,
                            r.role
                        );
                    }
                }
            }
        }
        PrincipalsAction::Revoke { id } => {
            repo.revoke(&id).await?;
            println!("revoked {id}");
        }
    }
    Ok(())
}

async fn cmd_build(
    args: BuildArgs,
    format: OutputFormat,
    repo: BuildRepo,
    logs: LogStore,
    cfg: &Config,
    toolchain: &Toolchain,
) -> Result<()> {
    let mut archs = BTreeSet::new();
    for a in args.arch {
        archs.insert(a.into());
    }
    if archs.is_empty() {
        archs.insert(Architecture::Amd64);
    }
    let mut compliance = BTreeSet::new();
    for c in args.compliance {
        compliance.insert(c.into());
    }

    let spec = BuildSpec {
        name: args.name,
        runtime: args.runtime.into(),
        base_image: args.base.into(),
        architectures: archs,
        compliance: compliance.clone(),
        hardening: HardeningOptions::strict(),
        generate_sbom: !args.no_sbom,
        sign: !args.no_sign,
    };

    let runner: Arc<TokioRunner> = Arc::new(TokioRunner);
    let registry_target = args
        .target
        .clone()
        .or_else(|| cfg.registry.default_target.clone());
    let push = args.push || cfg.registry.default_push;
    let bundled_prefix = toolchain.prefix().map(|p| p.to_path_buf());

    let orchestrator = BuildOrchestrator {
        builder: Arc::new(BuildkitBuilder::new(
            runner.clone(),
            BuildkitConfig {
                addr: cfg.buildkit.addr.clone(),
                bundled_prefix: bundled_prefix.clone(),
                registry_target,
                push,
                ..Default::default()
            },
        )),
        scanner: Arc::new(MergedScanner {
            primary: Arc::new(TrivyScanner::new(
                runner.clone(),
                TrivyConfig {
                    bundled_prefix: bundled_prefix.clone(),
                    ..Default::default()
                },
            )),
            secondary: Arc::new(GrypeScanner::new(
                runner.clone(),
                GrypeConfig {
                    bundled_prefix: bundled_prefix.clone(),
                    ..Default::default()
                },
            )),
        }),
        sbom: Arc::new(SyftSbomGenerator::new(
            runner.clone(),
            SyftConfig {
                bundled_prefix: bundled_prefix.clone(),
                ..Default::default()
            },
        )),
        signer: Arc::new(CosignSigner::new(
            runner.clone(),
            CosignConfig {
                bundled_prefix: bundled_prefix.clone(),
                ..Default::default()
            },
        )),
        attestor: Some(Arc::new(CosignSigner::new(
            runner.clone(),
            CosignConfig {
                bundled_prefix: bundled_prefix.clone(),
                ..Default::default()
            },
        ))),
        policy: Arc::new(OpaPolicyEngine::new(
            runner.clone(),
            OpaConfig {
                bundled_prefix,
                profiles: compliance.into_iter().collect(),
                ..Default::default()
            },
        )),
        provenance: Some(ProvenanceRepo::new(repo.storage().clone())),
        repo,
        logs,
    };

    let outcome = orchestrator.run(spec).await?;

    match format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "build_id": outcome.record.id.to_string(),
                "status": format!("{:?}", outcome.record.status).to_lowercase(),
                "policy": match &outcome.policy {
                    PolicyDecision::Allow => serde_json::json!({"decision": "allow"}),
                    PolicyDecision::Deny { reasons } => serde_json::json!({"decision": "deny", "reasons": reasons}),
                },
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Sarif => {
            if let Some(scan) = &outcome.record.scan {
                let s = sarif::to_sarif(scan, "image");
                println!("{}", serde_json::to_string_pretty(&s)?);
            } else {
                println!("{{}}");
            }
        }
        OutputFormat::Human => {
            println!("build {} -> {:?}", outcome.record.id, outcome.record.status);
            if let PolicyDecision::Deny { reasons } = &outcome.policy {
                println!("policy denied:");
                for r in reasons {
                    println!("  - {r}");
                }
            }
        }
    }
    Ok(())
}

async fn cmd_scan(build_id: &str, format: OutputFormat, repo: BuildRepo) -> Result<()> {
    let id = Uuid::parse_str(build_id).context("invalid build id")?;
    let scan = repo.get_scan(id).await?;
    match (format, scan) {
        (_, None) => {
            println!("no scan recorded for build {build_id}");
        }
        (OutputFormat::Json, Some(s)) => {
            println!("{}", serde_json::to_string_pretty(&s)?);
        }
        (OutputFormat::Sarif, Some(s)) => {
            let doc = sarif::to_sarif(&s, "image");
            println!("{}", serde_json::to_string_pretty(&doc)?);
        }
        (OutputFormat::Human, Some(s)) => {
            println!("scanner: {} ({})", s.scanner, s.scanned_at);
            println!("findings: {}", s.findings.len());
            for f in s.findings {
                println!(
                    "  [{:?}] {} {} -> {}",
                    f.severity,
                    f.id,
                    f.package,
                    f.fixed_version.unwrap_or_else(|| "no-fix".into())
                );
            }
        }
    }
    Ok(())
}

async fn cmd_list(format: OutputFormat, repo: BuildRepo) -> Result<()> {
    let rows = repo.list(50).await?;
    match format {
        OutputFormat::Json | OutputFormat::Sarif => {
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
        OutputFormat::Human => {
            for r in rows {
                println!(
                    "{:<8} {:<24} {:<8} {:<10} {}",
                    &r.id[..8.min(r.id.len())],
                    r.name,
                    r.runtime,
                    r.status,
                    r.created_at
                );
            }
        }
    }
    Ok(())
}

async fn cmd_logs(build_id: &str, logs: &LogStore) -> Result<()> {
    let id = Uuid::parse_str(build_id).context("invalid build id")?;
    match logs.read(id)? {
        Some(content) => {
            print!("{content}");
            Ok(())
        }
        None => {
            println!("no log file for build {build_id}");
            Ok(())
        }
    }
}

async fn cmd_stats(format: OutputFormat, repo: BuildRepo) -> Result<()> {
    let rows = repo.list(10_000).await?;
    let total = rows.len();
    let succeeded = rows.iter().filter(|r| r.status == "succeeded").count();
    let failed = rows.iter().filter(|r| r.status == "failed").count();
    let running = rows.iter().filter(|r| r.status == "running").count();
    match format {
        OutputFormat::Json | OutputFormat::Sarif => {
            let json = serde_json::json!({
                "total": total,
                "succeeded": succeeded,
                "failed": failed,
                "running": running,
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Human => {
            println!("total:     {total}");
            println!("succeeded: {succeeded}");
            println!("failed:    {failed}");
            println!("running:   {running}");
        }
    }
    Ok(())
}

fn cmd_doctor(toolchain: &Toolchain) -> Result<()> {
    println!("vendor prefix: {:?}", toolchain.prefix());
    if let Some(manifest) = toolchain.load_manifest() {
        for entry in manifest.tools {
            println!(
                "  {} {} ({})  {}",
                entry.name, entry.version, entry.platform, entry.relative_path
            );
        }
    } else {
        println!("(no vendor manifest found; tools will be resolved from PATH)");
    }
    for tool in ["buildctl", "trivy", "grype", "syft", "cosign", "opa"] {
        match toolchain.resolve(tool) {
            Ok(p) => println!("  [ok]   {tool} -> {}", p.display()),
            Err(e) => println!("  [miss] {tool}: {e}"),
        }
    }
    Ok(())
}
