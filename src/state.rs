use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Context, Result};
use chrono::Utc;
use crate::cli::Cli;

pub trait SnapshotManager: Send + Sync {
    fn create_snapshot(&self, file_path: &Path) -> Result<PathBuf>;
    fn restore_snapshot(&self, snapshot_path: &Path, original_path: &Path) -> Result<()>;
}

pub struct FileSnapshotManager {
    snapshot_dir: PathBuf,
}

impl FileSnapshotManager {
    pub fn new() -> Result<Self> {
        let snapshot_dir = PathBuf::from("/tmp/faber-aeternus-snapshots");
        if !snapshot_dir.exists() {
            fs::create_dir_all(&snapshot_dir).context("Failed to create snapshot directory")?;
        }
        Ok(Self { snapshot_dir })
    }
}

impl SnapshotManager for FileSnapshotManager {
    fn create_snapshot(&self, file_path: &Path) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
        let file_name = file_path.file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file path"))?
            .to_string_lossy();
        
        let snapshot_name = format!("{}_{}.snapshot", file_name, timestamp);
        let snapshot_path = self.snapshot_dir.join(snapshot_name);

        fs::copy(file_path, &snapshot_path).context("Failed to copy file to snapshot")?;
        Ok(snapshot_path)
    }

    fn restore_snapshot(&self, snapshot_path: &Path, original_path: &Path) -> Result<()> {
        fs::copy(snapshot_path, original_path).context("Failed to restore file from snapshot")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    ToolCall,
    ToolResult,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

pub struct AppState {
    pub config: Cli,
    pub chat_history: Vec<Message>,
    pub snapshot_manager: Box<dyn SnapshotManager>,
}

impl AppState {
    pub fn new(cli: Cli) -> Self {
        let snapshot_manager = Box::new(FileSnapshotManager::new().expect("Failed to initialize snapshot manager"));
        Self {
            config: cli,
            chat_history: vec![],
            snapshot_manager,
        }
    }
}

