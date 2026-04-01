// src/state/mod.rs
pub mod snapshot;
use crate::llm::provider::LLMProvider;
use crate::tools::registry::ToolRegistry;
use anyhow::Result;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub enum PermissionState {
    Idle,
    Pending { tool_name: String, params: serde_json::Value },
    Approved { tool_name: String, params: serde_json::Value },
    Aborted,
}

pub struct AppState {
    pub chat_history: VecDeque<String>,
    pub current_input: String,
    pub preview_diff: String,
    pub permission_state: PermissionState,
    pub project_path: String,
    pub llm: std::sync::Arc<tokio::sync::Mutex<Box<dyn LLMProvider + Send + Sync>>>,
    pub tools: std::sync::Arc<ToolRegistry>,
    pub snapshot_manager: snapshot::SnapshotManager,
}

impl AppState {
    pub fn new(cli: crate::cli::Cli) -> Self {
        let project_path = cli.project.unwrap_or_else(|| ".".into());
        Self {
            chat_history: VecDeque::new(),
            current_input: String::new(),
            preview_diff: String::new(),
            permission_state: PermissionState::Idle,
            project_path,
            llm: std::sync::Arc::new(tokio::sync::Mutex::new(Box::new(crate::llm::provider::create_provider(cli.provider.clone(), cli.model.clone())))),
            tools: std::sync::Arc::new(ToolRegistry::new().expect("ToolRegistry init failed")),
            snapshot_manager: snapshot::SnapshotManager::new().expect("SnapshotManager init failed"),
        }
    }

    pub fn add_message(&mut self, msg: String) {
        self.chat_history.push_back(msg);
        if self.chat_history.len() > 50 {
            self.chat_history.pop_front();
        }
    }

    pub fn set_permission_pending(&mut self, name: String, params: serde_json::Value) {
        self.permission_state = PermissionState::Pending { tool_name: name, params };
    }
}