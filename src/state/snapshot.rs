use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use chrono::Utc;

#[derive(Clone)]
pub struct SnapshotManager {
    snapshot_dir: PathBuf,
}

impl SnapshotManager {
    pub fn new() -> Result<Self> {
        let snapshot_dir = PathBuf::from("/tmp/faber-aeternus-snapshots");
        if !snapshot_dir.exists() {
            fs::create_dir_all(&snapshot_dir).context("Failed to create snapshot directory")?;
        }
        Ok(Self { snapshot_dir })
    }

    pub fn create_snapshot(&self, file_path: &Path) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let file_name = file_path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string_lossy();
        
        let snapshot_name = format!("{}_{}.snapshot", file_name, timestamp);
        let snapshot_path = self.snapshot_dir.join(&snapshot_name);

        // It might not exist if we're creating a new file
        if file_path.exists() {
            fs::copy(file_path, &snapshot_path).context("Failed to copy file to snapshot")?;
        } else {
            fs::write(&snapshot_path, "").context("Failed to create empty snapshot proxy")?;
        }
        Ok(snapshot_path)
    }

    pub fn restore_snapshot(&self, snapshot_path: &Path, original_path: &Path) -> Result<()> {
        fs::copy(snapshot_path, original_path).context("Failed to restore file from snapshot")?;
        Ok(())
    }
}
