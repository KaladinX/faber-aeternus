use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolPermissionLevel {
    Low,
    Medium,
    High,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
}

pub trait Tool: Send + Sync {
    fn schema(&self) -> ToolSchema;
    fn permission_level(&self) -> ToolPermissionLevel;
    fn execute(&self, arguments: &serde_json::Value) -> Result<String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    max_permission: ToolPermissionLevel,
}

impl ToolRegistry {
    pub fn new(max_permission: ToolPermissionLevel) -> Self {
        Self {
            tools: HashMap::new(),
            max_permission,
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        if tool.permission_level() <= self.max_permission {
            self.tools.insert(tool.schema().name, tool);
        }
    }

    pub fn execute_tool(&self, name: &str, args: &serde_json::Value) -> Result<String> {
        let tool = self.tools.get(name).ok_or_else(|| anyhow::anyhow!("Tool not found"))?;
        tool.execute(args)
    }

    pub fn schemas(&self) -> Vec<ToolSchema> {
        self.tools.values().map(|t| t.schema()).collect()
    }
}
