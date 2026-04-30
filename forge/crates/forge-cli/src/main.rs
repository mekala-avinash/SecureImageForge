use std::io;
use std::path::PathBuf;

use anyhow::Result;
use clap::{ArgAction, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};

use forge_core::domain::{Architecture, BaseImage, ComplianceProfile, Runtime};
use forge_core::telemetry;

#[derive(Parser, Debug)]
#[command(
    name = "forge",
    version,
    about = "SecureImage Forge CLI",
    propagate_version = true
)]
struct Cli {
    /// Output format for command results.
    #[arg(long, value_enum, global = true, default_value_t = OutputFormat::Human)]
    output: OutputFormat,

    /// Increase verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short, long, action = ArgAction::Count, global = true)]
    verbose: u8,

    /// Override the data directory (default: platform-appropriate user dir).
    #[arg(long, env = "FORGE_DATA_DIR", global = true)]
    data_dir: Option<PathBuf>,

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
    /// Build and harden a container image.
    Build(BuildArgs),
    /// Show vulnerability scan results for a build.
    Scan { build_id: String },
    /// List recorded builds.
    List,
    /// Stream or print logs for a build.
    Logs { build_id: String },
    /// Show aggregate statistics.
    Stats,
    /// Generate shell completions.
    Completions { shell: Shell },
}

#[derive(clap::Args, Debug)]
struct BuildArgs {
    /// Friendly build name.
    #[arg(long)]
    name: String,

    /// Application runtime.
    #[arg(long, value_enum)]
    runtime: RuntimeArg,

    /// Base image family.
    #[arg(long, value_enum)]
    base: BaseImageArg,

    /// Compliance profiles to enforce (repeatable).
    #[arg(long, value_enum)]
    compliance: Vec<ComplianceArg>,

    /// Target architectures (repeatable, default: amd64).
    #[arg(long, value_enum)]
    arch: Vec<ArchArg>,

    /// Skip SBOM generation.
    #[arg(long)]
    no_sbom: bool,

    /// Skip image signing.
    #[arg(long)]
    no_sign: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum RuntimeArg { Java, Dotnet, Go, Node, Python }

#[derive(Copy, Clone, Debug, ValueEnum)]
enum BaseImageArg { Alpine, Debian, Distroless }

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ComplianceArg { Hipaa, Soc2, Pcidss, Cis, FedrampModerate }

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ArchArg { Amd64, Arm64 }

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

fn main() -> Result<()> {
    telemetry::init();
    let cli = Cli::parse();

    match cli.command {
        Command::Build(args) => cmd_build(args, cli.output),
        Command::Scan { build_id } => cmd_scan(&build_id, cli.output),
        Command::List => cmd_list(cli.output),
        Command::Logs { build_id } => cmd_logs(&build_id),
        Command::Stats => cmd_stats(cli.output),
        Command::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "forge", &mut io::stdout());
            Ok(())
        }
    }
}

fn cmd_build(_args: BuildArgs, _format: OutputFormat) -> Result<()> {
    // Phase 0: skeleton. Real orchestration lands in Phase 1 once buildkit
    // adapter is wired in forge-core::tooling.
    println!("[forge] build pipeline not yet implemented (Phase 1)");
    Ok(())
}

fn cmd_scan(_build_id: &str, _format: OutputFormat) -> Result<()> {
    println!("[forge] scan not yet implemented (Phase 1)");
    Ok(())
}

fn cmd_list(_format: OutputFormat) -> Result<()> {
    println!("[forge] list not yet implemented (Phase 1)");
    Ok(())
}

fn cmd_logs(_build_id: &str) -> Result<()> {
    println!("[forge] logs not yet implemented (Phase 1)");
    Ok(())
}

fn cmd_stats(_format: OutputFormat) -> Result<()> {
    println!("[forge] stats not yet implemented (Phase 1)");
    Ok(())
}
