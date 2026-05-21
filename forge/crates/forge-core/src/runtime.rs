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
            let vendor_dir = find_vendor_dir(self.bundled_prefix.as_deref());
            let mut mounts_section = String::from("- location: \"~\"\n  writable: true");
            if let Some(ref path) = vendor_dir {
                mounts_section.push_str(&format!(
                    "\n- location: \"{}\"\n  mountPoint: \"/mnt/forge-vendor\"\n  writable: false",
                    path.display()
                ));
            }

            let template = format!(r#"
images:
- location: "https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-amd64.img"
  arch: "x86_64"
- location: "https://cloud-images.ubuntu.com/releases/24.04/release/ubuntu-24.04-server-cloudimg-arm64.img"
  arch: "aarch64"
mounts:
{mounts_section}
containerd:
  system: false
  user: false
provision:
- mode: system
  script: |
    #!/bin/sh
    set -e
    if [ -d /mnt/forge-vendor ]; then
        echo "Installing from bundled vendor..."
        ARCH=$(uname -m)
        if [ "$ARCH" = "x86_64" ]; then
            VDIR="linux/amd64"
        else
            VDIR="linux/arm64"
        fi
        cp /mnt/forge-vendor/$VDIR/buildkitd /usr/local/bin/
        cp /mnt/forge-vendor/$VDIR/buildctl /usr/local/bin/
        cp /mnt/forge-vendor/$VDIR/runc /usr/local/bin/
        if [ -f /mnt/forge-vendor/$VDIR/containerd ]; then
            cp /mnt/forge-vendor/$VDIR/containerd* /usr/local/bin/
        fi
    else
        echo "Bundled vendor not found, falling back to apt-get..."
        apt-get update
        apt-get install -y buildkit
    fi
    
    # Create systemd service for buildkitd
    echo '[Unit]' > /etc/systemd/system/buildkit.service
    echo 'Description=BuildKit' >> /etc/systemd/system/buildkit.service
    echo 'Documentation=https://github.com/moby/buildkit' >> /etc/systemd/system/buildkit.service
    echo '' >> /etc/systemd/system/buildkit.service
    echo '[Service]' >> /etc/systemd/system/buildkit.service
    echo 'ExecStart=/usr/local/bin/buildkitd --addr unix:///run/buildkit/buildkitd.sock --group 1000' >> /etc/systemd/system/buildkit.service
    echo 'Restart=always' >> /etc/systemd/system/buildkit.service
    echo '' >> /etc/systemd/system/buildkit.service
    echo '[Install]' >> /etc/systemd/system/buildkit.service
    echo 'WantedBy=multi-user.target' >> /etc/systemd/system/buildkit.service

    mkdir -p /run/buildkit
    chmod +x /usr/local/bin/buildkitd /usr/local/bin/buildctl /usr/local/bin/runc
    if [ -f /usr/local/bin/containerd ]; then
        chmod +x /usr/local/bin/containerd*
    fi
    systemctl daemon-reload
    systemctl enable --now buildkit
portForwards:
- guestSocket: "/run/buildkit/buildkitd.sock"
  hostSocket: "{{{{.Dir}}}}/sock/buildkitd.sock"
"#, mounts_section = mounts_section);

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

        let mut path_env = std::env::var("PATH").unwrap_or_default();
        if let Some(vendor_dir) = find_vendor_dir(self.bundled_prefix.as_deref()) {
            let arch = if cfg!(target_arch = "x86_64") { "amd64" } else { "arm64" };
            let platform_vendor = vendor_dir.join("linux").join(arch);
            if platform_vendor.exists() {
                path_env = format!("{}:{}", platform_vendor.display(), path_env);
            }
            if vendor_dir.exists() {
                path_env = format!("{}:{}", vendor_dir.display(), path_env);
            }
        }

        info!("Starting rootless buildkitd...");
        tokio::spawn(async move {
            let mut cmd = Command::new(rootlesskit);
            cmd.env("PATH", path_env);
            cmd.args([
                "--net=slirp4netns",
                "--copy-up=/etc",
                "--disable-host-loopback",
                buildkitd.to_str().unwrap(),
                "--addr",
                &addr_clone,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
            
            let mut child = cmd.spawn().expect("Failed to spawn rootlesskit");
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

fn find_vendor_dir(bundled_prefix: Option<&Path>) -> Option<PathBuf> {
    if let Some(prefix) = bundled_prefix {
        if prefix.exists() {
            if let Ok(abs) = std::fs::canonicalize(prefix) {
                return Some(abs);
            }
            return Some(prefix.to_path_buf());
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let base_vendor = parent.join("vendor");
            if base_vendor.exists() {
                return Some(base_vendor);
            }
            let mut current = parent;
            while let Some(up) = current.parent() {
                let candidate = current.join("vendor");
                if candidate.exists() {
                    return Some(candidate);
                }
                current = up;
            }
        }
    }
    None
}
