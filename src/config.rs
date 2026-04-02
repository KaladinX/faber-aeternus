// src/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_architect_prompt")]
    pub architect_prompt: String,
    #[serde(default = "default_coder_prompt")]
    pub coder_prompt: String,
    #[serde(default = "default_reviewer_prompt")]
    pub reviewer_prompt: String,
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

fn default_architect_prompt() -> String {
    "You are the Architect agent, a master systems designer. You only read the project and output an ULTRAPLAN detailing the exact execution steps and required files. Do not emit tool calls, just structural plans.".into()
}

fn default_coder_prompt() -> String {
    "You are the Coder agent. You strictly implement changes by outputting <tool_call name=\"edit_file\">{\"path\": \"...\", \"diff\": \"...\"}</tool_call>. YOU MAY NEVER USE run_shell.".into()
}

fn default_reviewer_prompt() -> String {
    "Adversarial self-critic looking for bugs in Coder's diffs.".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            architect_prompt: default_architect_prompt(),
            coder_prompt: default_coder_prompt(),
            reviewer_prompt: default_reviewer_prompt(),
            ignore_patterns: vec![],
        }
    }
}

impl AppConfig {
    pub fn load_from_dir(dir: &str) -> Self {
        let path = std::path::Path::new(dir).join(".faber.toml");
        if let Ok(content) = std::fs::read_to_string(&path) {
            toml::from_str(&content).unwrap_or_else(|_| Self::default())
        } else {
            let fallback = std::env::var("HOME")
                .map(|p| std::path::Path::new(&p).join(".config").join("faber-aeternus").join("config.toml"))
                .unwrap_or_else(|_| PathBuf::from(".faber.toml"));
            
            if let Ok(content) = std::fs::read_to_string(&fallback) {
                toml::from_str(&content).unwrap_or_else(|_| Self::default())
            } else {
                Self::default()
            }
        }
    }
}
