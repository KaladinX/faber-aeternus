// src/state/mod.rs
pub mod snapshot;

use crate::llm::provider::LLMProvider;
use crate::tools::registry::ToolRegistry;
use crate::brain::coordinator::Coordinator;
use crate::brain::context::ProjectIndex;
use anyhow::Result;
use std::collections::VecDeque;
use std::sync::Arc;
use serde_json::Value;

#[derive(Debug, Clone)]
pub enum PermissionState {
    Idle,
    Pending { tool_name: String, params: Value },
    Approved { tool_name: String, params: Value },
    Aborted,
}

pub struct AppState {
    pub chat_history: VecDeque<String>,
    pub current_input: String,
    pub preview_diff: String,
    pub permission_state: PermissionState,
    pub agent_status: Option<String>,
    pub project_path: String,
    pub llm: Arc<dyn LLMProvider + Sync + Send>,
    pub tools: Arc<ToolRegistry>,
    pub snapshot_manager: snapshot::SnapshotManager,
    pub p_index: Arc<ProjectIndex>,
    pub coordinator: Arc<tokio::sync::Mutex<Coordinator>>,
    pub config: crate::config::AppConfig,
}

impl AppState {
    pub fn new(cli: crate::cli::Cli, p_index: Arc<ProjectIndex>) -> Self {
        let project_path = cli.project.clone().unwrap_or_else(|| ".".into());
        let config = crate::config::AppConfig::load_from_dir(&project_path);
        let llm: Arc<dyn LLMProvider + Sync + Send> = Arc::from(crate::llm::provider::create_provider(cli.provider.clone(), cli.model.clone()));
        
        Self {
            chat_history: VecDeque::from(vec!["🔥 faber-aeternus v0.1.0 — the eternal craftsman".to_string()]),
            current_input: String::new(),
            preview_diff: String::new(),
            permission_state: PermissionState::Idle,
            agent_status: None,
            project_path,
            llm,
            tools: Arc::new(ToolRegistry::new().expect("ToolRegistry init failed")),
            snapshot_manager: snapshot::SnapshotManager::new().expect("SnapshotManager init failed"),
            p_index,
            coordinator: Arc::new(tokio::sync::Mutex::new(Coordinator::new())),
            config,
        }
    }

    pub fn add_message(&mut self, msg: String) {
        self.chat_history.push_back(msg);
        if self.chat_history.len() > 50 {
            self.chat_history.pop_front();
        }
    }
}