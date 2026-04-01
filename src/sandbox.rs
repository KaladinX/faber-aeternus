use std::process::Command;
use anyhow::{Context, Result};
use tracing::{info, warn};

pub struct SandboxExecutor {
    _strict_mode: bool,
    bwrap_available: bool,
}

impl SandboxExecutor {
    pub fn new() -> Result<Self> {
        let strict_mode = cfg!(feature = "sandbox-strict");
        
        let bwrap_available = match Command::new("bwrap").arg("--version").output() {
            Ok(output) => output.status.success(),
            Err(_) => false,
        };

        if !bwrap_available {
            if strict_mode {
                anyhow::bail!("Strict sandbox mode is enabled, but bubblewrap (bwrap) is not found.");
            } else {
                warn!("⚠️ bubblewrap not found — running in UNSANDBOXED mode.");
                warn!("Install with: sudo apt install bubblewrap (or brew install bubblewrap on macOS)");
                warn!("For maximum security, use --sandbox-strict flag.");
            }
        } else {
            info!("Sandbox executor initialized with bubblewrap.");
        }

        Ok(Self { strict_mode, bwrap_available })
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> Result<String> {
        if self.bwrap_available {
            // Basic bwrap strategy: Read-only root, read-write /tmp
            let mut cmd = Command::new("bwrap");
            cmd.arg("--ro-bind")
               .arg("/")
               .arg("/")
               .arg("--dev")
               .arg("/dev")
               .arg("--proc")
               .arg("/proc")
               .arg("--bind")
               .arg("/tmp")
               .arg("/tmp")
               .arg("--unshare-all")
               .arg("--")
               .arg(command)
               .args(args);

            let output = cmd.output().context("Failed to execute sandboxed command")?;
            if !output.status.success() {
                anyhow::bail!("Sandboxed command failed: {}", String::from_utf8_lossy(&output.stderr));
            }
            Ok(String::from_utf8(output.stdout)?)
        } else {
            // Graceful fallback
            let mut cmd = Command::new(command);
            cmd.args(args);
            let output = cmd.output().context(format!("Failed to execute fallback command: {}", command))?;
            if !output.status.success() {
                anyhow::bail!("Command failed: {}", String::from_utf8_lossy(&output.stderr));
            }
            Ok(String::from_utf8(output.stdout)?)
        }
    }
}
