// src/sandbox.rs
use std::process::Command;
use anyhow::{Context, Result};
use tracing::{info, warn};

#[derive(Clone)]
pub struct SandboxExecutor {
    strict_mode: bool,
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
                warn!("⚠️  bubblewrap not found — running in UNSANDBOXED mode.");
                warn!("Install with: sudo apt install bubblewrap (or brew install bubblewrap on macOS)");
                warn!("For maximum security, rebuild with --features sandbox-strict");
            }
        } else {
            info!("✅ Sandbox executor initialized with bubblewrap.");
        }

        Ok(Self { strict_mode, bwrap_available })
    }

    pub fn execute(&self, command: &str, args: &[&str]) -> Result<String> {
        if self.bwrap_available {
            let mut cmd = Command::new("bwrap");
            cmd.args([
                "--ro-bind", "/", "/",
                "--dev", "/dev",
                "--proc", "/proc",
                "--bind", "/tmp", "/tmp",
                "--unshare-all",
                "--die-with-parent",
                "--",
                command,
            ])
            .args(args);

            let output = cmd.output().context("Failed to execute sandboxed command")?;
            if !output.status.success() {
                anyhow::bail!(
                    "Sandboxed command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let mut cmd = Command::new(command);
            cmd.args(args);
            let output = cmd.output().context(format!("Failed to execute fallback command: {command}"))?;
            if !output.status.success() {
                anyhow::bail!(
                    "Command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        }
    }
}