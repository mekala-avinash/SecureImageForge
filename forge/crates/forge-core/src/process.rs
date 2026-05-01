//! Subprocess abstraction. `ProcessRunner` is the only thing adapters depend on
//! for spawning external tools, which keeps them unit-testable via
//! `MockRunner` without a real BuildKit / Trivy / Syft / Cosign / OPA install.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio_stream::wrappers::LinesStream;
use tokio_stream::Stream;

use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct ProcessSpec {
    pub program: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<PathBuf>,
}

impl ProcessSpec {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: HashMap::new(),
            cwd: None,
        }
    }

    pub fn arg(mut self, a: impl Into<String>) -> Self {
        self.args.push(a.into());
        self
    }

    pub fn args<I, S>(mut self, items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(items.into_iter().map(Into::into));
        self
    }

    pub fn env(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.env.insert(k.into(), v.into());
        self
    }

    pub fn cwd(mut self, p: impl Into<PathBuf>) -> Self {
        self.cwd = Some(p.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Runner contract. Adapters MUST go through this — never spawn directly.
#[async_trait]
pub trait ProcessRunner: Send + Sync {
    /// Run to completion, capture stdout+stderr.
    async fn run(&self, spec: ProcessSpec) -> Result<ProcessOutput>;

    /// Stream merged stdout+stderr line-by-line; returns final exit status
    /// alongside the stream so callers can both tail logs and gate on success.
    async fn stream(&self, spec: ProcessSpec) -> Result<StreamingChild>;
}

/// Result of a streaming spawn: an async line stream + a join handle that
/// resolves to the final exit code. Adapters typically forward `lines` to
/// the orchestrator's log channel and await `wait()` for the exit code.
pub struct StreamingChild {
    pub lines: Box<dyn Stream<Item = Result<String>> + Send + Unpin>,
    pub wait: tokio::task::JoinHandle<Result<i32>>,
    pub abort_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl Drop for StreamingChild {
    fn drop(&mut self) {
        if let Some(tx) = self.abort_tx.take() {
            let _ = tx.send(());
        }
    }
}

#[derive(Default)]
pub struct TokioRunner;

#[async_trait]
impl ProcessRunner for TokioRunner {
    async fn run(&self, spec: ProcessSpec) -> Result<ProcessOutput> {
        let mut cmd = build_command(&spec);
        let output = cmd.output().await.map_err(Error::Io)?;
        Ok(ProcessOutput {
            status: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }

    async fn stream(&self, spec: ProcessSpec) -> Result<StreamingChild> {
        let mut cmd = build_command(&spec);
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn().map_err(Error::Io)?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::Internal(anyhow::anyhow!("failed to capture stdout")))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| Error::Internal(anyhow::anyhow!("failed to capture stderr")))?;

        let stdout_lines = LinesStream::new(BufReader::new(stdout).lines());
        let stderr_lines = LinesStream::new(BufReader::new(stderr).lines());
        let merged = futures::stream::select(stdout_lines, stderr_lines);

        use futures::StreamExt as _;
        let mapped = merged.map(|r| r.map_err(Error::Io));

        let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<()>();

        let wait = tokio::spawn(async move {
            tokio::select! {
                res = child.wait() => {
                    let status = res.map_err(Error::Io)?;
                    Ok(status.code().unwrap_or(-1))
                }
                _ = abort_rx => {
                    let _ = child.start_kill();
                    let _ = child.wait().await;
                    Err(Error::Internal(anyhow::anyhow!("Process was aborted")))
                }
            }
        });

        Ok(StreamingChild {
            lines: Box::new(Box::pin(mapped)),
            wait,
            abort_tx: Some(abort_tx),
        })
    }
}

fn build_command(spec: &ProcessSpec) -> Command {
    let mut cmd = Command::new(&spec.program);
    cmd.args(&spec.args);
    for (k, v) in &spec.env {
        cmd.env(k, v);
    }
    if let Some(cwd) = &spec.cwd {
        cmd.current_dir(cwd);
    }
    cmd
}

/// Resolve `program` against PATH (and optionally an explicit prefix dir for
/// bundled binaries). Returns `Error::ToolMissing` if absent.
pub fn resolve_tool(program: &str, bundled_prefix: Option<&Path>) -> Result<PathBuf> {
    if let Some(prefix) = bundled_prefix {
        let candidate = prefix.join(program);
        if candidate.is_file() {
            return Ok(candidate);
        }
        let exe_candidate = prefix.join(format!("{program}.exe"));
        if exe_candidate.is_file() {
            return Ok(exe_candidate);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let base_vendor = parent.join("vendor");
            
            // Check flat vendor
            let candidate = base_vendor.join(program);
            if candidate.is_file() { return Ok(candidate); }
            let exe_candidate = base_vendor.join(format!("{program}.exe"));
            if exe_candidate.is_file() { return Ok(exe_candidate); }

            // Check platform-specific vendor (dev mode)
            let os = if cfg!(target_os = "macos") { "darwin" } else { "linux" };
            let arch = if cfg!(target_arch = "x86_64") { "amd64" } else { "arm64" };
            let platform_vendor = base_vendor.join(os).join(arch);
            
            let candidate = platform_vendor.join(program);
            if candidate.is_file() { return Ok(candidate); }
            let exe_candidate = platform_vendor.join(format!("{program}.exe"));
            if exe_candidate.is_file() { return Ok(exe_candidate); }
        }
    }

    which::which(program).map_err(|_| Error::ToolMissing {
        tool: program.to_string(),
    })
}

// ---------------------------------------------------------------------------
// MockRunner: deterministic fake for adapter unit tests.
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
pub struct MockRunner {
    inner: Arc<Mutex<MockState>>,
}

#[derive(Default)]
struct MockState {
    expectations: Vec<Expectation>,
    calls: Vec<ProcessSpec>,
}

struct Expectation {
    matcher: Box<dyn Fn(&ProcessSpec) -> bool + Send + Sync>,
    output: ProcessOutput,
}

impl MockRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn expect<F>(&self, matcher: F, output: ProcessOutput) -> &Self
    where
        F: Fn(&ProcessSpec) -> bool + Send + Sync + 'static,
    {
        self.inner
            .lock()
            .expect("mock state poisoned")
            .expectations
            .push(Expectation {
                matcher: Box::new(matcher),
                output,
            });
        self
    }

    pub fn calls(&self) -> Vec<ProcessSpec> {
        self.inner
            .lock()
            .expect("mock state poisoned")
            .calls
            .clone()
    }
}

#[async_trait]
impl ProcessRunner for MockRunner {
    async fn run(&self, spec: ProcessSpec) -> Result<ProcessOutput> {
        let mut guard = self.inner.lock().expect("mock state poisoned");
        guard.calls.push(spec.clone());
        for ex in &guard.expectations {
            if (ex.matcher)(&spec) {
                return Ok(ex.output.clone());
            }
        }
        Err(Error::ToolFailure {
            tool: spec.program,
            code: -1,
            stderr: "no MockRunner expectation matched".into(),
        })
    }

    async fn stream(&self, spec: ProcessSpec) -> Result<StreamingChild> {
        // For the mock, run synchronously and surface output as a single line
        // so adapters that prefer the streaming API still get deterministic
        // behavior in tests.
        let out = self.run(spec).await?;
        let line = if out.stderr.is_empty() {
            out.stdout.clone()
        } else {
            format!("{}{}", out.stdout, out.stderr)
        };
        let stream = futures::stream::iter(vec![Ok(line)]);
        let status = out.status;
        let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<()>();
        let wait = tokio::spawn(async move {
            tokio::select! {
                _ = abort_rx => Err(Error::Internal(anyhow::anyhow!("Process was aborted"))),
                _ = std::future::ready(()) => Ok(status),
            }
        });
        use futures::StreamExt as _;
        Ok(StreamingChild {
            lines: Box::new(Box::pin(stream.map(|r: Result<String>| r))),
            wait,
            abort_tx: Some(abort_tx),
        })
    }
}
