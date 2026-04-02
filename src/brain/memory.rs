// src/brain/memory.rs
use std::path::{Path, PathBuf};
use std::fs;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MemoryEntry {
    UserPrompt {
        timestamp: DateTime<Utc>,
        content: String,
    },
    AgentTrajectory {
        timestamp: DateTime<Utc>,
        agent_name: String,
        content: String,
    },
    Snapshot {
        timestamp: DateTime<Utc>,
        file_path: String,
        backup_path: String,
    }
}

pub struct SessionMemory {
    log_path: PathBuf,
    entries: Vec<MemoryEntry>,
}

impl SessionMemory {
    pub fn new(project_root: &Path) -> Result<Self> {
        let faber_dir = project_root.join(".faber");
        if !faber_dir.exists() {
            fs::create_dir_all(&faber_dir)?;
        }
        let log_path = faber_dir.join("session.json");
        
        let entries = if log_path.exists() {
            let data = fs::read_to_string(&log_path)?;
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Self { log_path, entries })
    }

    pub fn log(&mut self, entry: MemoryEntry) -> Result<()> {
        self.entries.push(entry);
        self.persist()?;
        Ok(())
    }

    fn persist(&self) -> Result<()> {
        let data = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.log_path, data)?;
        Ok(())
    }
}
