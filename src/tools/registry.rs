// src/tools/registry.rs
use serde_json::Value;
use std::collections::HashMap;
use crate::sandbox::SandboxExecutor;
use anyhow::{Context, Result};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum ToolPermissionLevel {
    Low,    // read-only
    Medium, // file edits, git
    High,   // shell, network, external
}

#[derive(Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub schema: Value,
    pub permission_level: ToolPermissionLevel,
    pub requires_approval: bool,
}

pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    sandbox: SandboxExecutor,
}

impl ToolRegistry {
    pub fn new() -> Result<Self> {
        let sandbox = SandboxExecutor::new()?;
        let mut registry = Self {
            tools: HashMap::new(),
            sandbox,
        };
        registry.register_core_tools();
        Ok(registry)
    }

    fn register_core_tools(&mut self) {
        self.tools.insert("read_file".into(), Tool {
            name: "read_file".into(),
            description: "Read a file from the project (LOW risk)".into(),
            schema: serde_json::json!({
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }),
            permission_level: ToolPermissionLevel::Low,
            requires_approval: false,
        });

        self.tools.insert("edit_file".into(), Tool {
            name: "edit_file".into(),
            description: "Apply a reversible diff edit (MEDIUM risk)".into(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" },
                    "diff": { "type": "string" }
                },
                "required": ["path", "diff"]
            }),
            permission_level: ToolPermissionLevel::Medium,
            requires_approval: true,
        });

        self.tools.insert("run_shell".into(), Tool {
            name: "run_shell".into(),
            description: "Run a shell command in sandbox (HIGH risk)".into(),
            schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "args": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["command"]
            }),
            permission_level: ToolPermissionLevel::High,
            requires_approval: true,
        });
    }

    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    pub fn list_for_llm(&self) -> Vec<Value> {
        self.tools.values().map(|t| serde_json::json!({
            "name": t.name,
            "description": t.description,
            "parameters": t.schema
        })).collect()
    }

    pub async fn execute(&self, name: &str, params: Value) -> Result<String> {
        let tool = self.get_tool(name).context("unknown tool")?;

        if tool.requires_approval && tool.permission_level != ToolPermissionLevel::Low {
            println!("🔒 [Permission] {} ({:?}) — auto-approved for demo", name, tool.permission_level);
        }

        match name {
            "run_shell" => {
                let command = params["command"].as_str().context("missing command")?;
                let args: Vec<&str> = params["args"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
                    .unwrap_or_default();
                self.sandbox.execute(command, &args)
            }
            _ => Ok(format!("Tool '{name}' executed (full impl coming next)")),
        }
    }
}