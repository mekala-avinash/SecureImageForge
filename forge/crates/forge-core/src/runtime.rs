use std::path::{Path, PathBuf};
use std::process::Stdio;
use anyhow::{anyhow, Context};
use tokio::process::Command;
use tracing::{info, warn};
use crate::Result;

pub struct RuntimeManager {
    bundled_prefix: Option<PathBuf>,
}

impl RuntimeManager {
    pub fn new(bundled_prefix: Option<PathBuf>) -> Self {
        Self { bundled_prefix }
    }

    pub async fn ensure_running(&self) -> Result<String> {
        if cfg!(target_os = "macos") {
            self.ensure_lima().await
        } else if cfg!(target_os = "linux") {
            self.ensure_rootlesskit().await
        } else {
            Err(crate::Error::Internal(anyhow!("unsupported platform for managed runtime")))
        }
    }

    async fn ensure_lima(&self) -> Result<String> {
        let limactl = crate::process::resolve_tool("limactl", self.bundled_prefix.as_deref())?;
        
        let output = Command::new(&limactl)
            .args(["list", "--json"])
            .output()
            .await?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut exists = false;
        let mut running = false;
        
        for line in stdout.lines() {
            if let Ok(vm) = serde_json::from_str::<serde_json::Value>(line) {
                if vm["name"] == "forge-buildkit" {
                    exists = true;
                    if vm["status"] == "Running" {
                        running = true;
                    }
                }
            }
        }
        
        let lima_home = dirs::home_dir().unwrap_or_default().join(".lima");
        let sock_path = lima_home.join("forge-buildkit").join("sock").join("buildkitd.sock");
        let addr = format!("unix://{}", sock_path.display());

        if running {
            return Ok(addr);
        }

        if !exists {
            info!("Provisioning Lima VM for buildkit...");
            let template = r#"
images:
- location: "https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-amd64.img"
  arch: "x86_64"
- location: "https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-arm64.img"
  arch: "aarch64"
mounts:
- location: "~"
  writable: true
containerd:
  system: false
  user: false
provision:
- mode: system
  script: |
    #!/bin/sh
    apt-get update
    apt-get install -y buildkit
    systemctl enable --now buildkit
portForwards:
- guestSocket: "/run/buildkit/buildkitd.sock"
  hostSocket: "{{.Dir}}/sock/buildkitd.sock"
"#;
            let temp_dir = std::env::temp_dir();
            let template_path = temp_dir.join("forge-lima.yaml");
            std::fs::write(&template_path, template)?;
            
            let status = Command::new(&limactl)
                .args(["start", "--tty=false", "--name", "forge-buildkit", template_path.to_str().unwrap()])
                .status()
                .await?;
            
            if !status.success() {
                return Err(crate::Error::Internal(anyhow!("Failed to create lima VM")));
            }
        } else {
            info!("Starting Lima VM...");
            let status = Command::new(&limactl)
                .args(["start", "--tty=false", "forge-buildkit"])
                .status()
                .await?;
            if !status.success() {
                return Err(crate::Error::Internal(anyhow!("Failed to start lima VM")));
            }
        }
        
        // Wait for socket
        for _ in 0..10 {
            if sock_path.exists() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        Ok(addr)
    }

    async fn ensure_rootlesskit(&self) -> Result<String> {
        let rootlesskit = crate::process::resolve_tool("rootlesskit", self.bundled_prefix.as_deref())?;
        let buildkitd = crate::process::resolve_tool("buildkitd", self.bundled_prefix.as_deref())?;
        
        let sock_dir = dirs::runtime_dir().unwrap_or_else(|| std::env::temp_dir()).join("forge-buildkit");
        std::fs::create_dir_all(&sock_dir)?;
        let sock_path = sock_dir.join("buildkitd.sock");
        let addr = format!("unix://{}", sock_path.display());
        let addr_clone = addr.clone();
        
        #[cfg(unix)]
        if std::os::unix::net::UnixStream::connect(&sock_path).is_ok() {
            return Ok(addr);
        }

        info!("Starting rootless buildkitd...");
        tokio::spawn(async move {
            let mut child = Command::new(rootlesskit)
                .args([
                    "--net=slirp4netns",
                    "--copy-up=/etc",
                    "--disable-host-loopback",
                    buildkitd.to_str().unwrap(),
                    "--addr",
                    &addr_clone,
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .expect("Failed to spawn rootlesskit");
            let _ = child.wait().await;
        });
        
        for _ in 0..10 {
            if sock_path.exists() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        Ok(addr)
    }
}
