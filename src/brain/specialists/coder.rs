// src/brain/specialists/coder.rs
use crate::brain::agent::{SpecialistAgent, AgentOutput, AgentState};
use crate::brain::context::ProjectContext;
use crate::llm::provider::LLMProvider;
use crate::tools::StreamingParser;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use futures::stream::StreamExt;

pub struct CoderAgent {}

impl CoderAgent {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SpecialistAgent for CoderAgent {
    fn name(&self) -> &'static str {
        "Coder"
    }

    fn description(&self) -> &'static str {
        "Diff-only isolated editor. Strictly restricted to edit_file."
    }

    async fn execute(
        &mut self,
        _input: &str,
        _context: &mut ProjectContext<'_>,
        llm: Arc<dyn LLMProvider + Send + Sync>,
        chat_history: &std::collections::VecDeque<String>,
    ) -> Result<AgentOutput> {
        let system_prompt = "You are the Coder agent. You strictly implement changes by outputting <tool_call name=\"edit_file\">{\"path\": \"...\", \"content\": \"<full updated file text>\"}</tool_call>. YOU MAY NEVER USE run_shell.";

        let mut stream = llm.generate_stream(system_prompt, chat_history).await?;
        let mut parser = StreamingParser::new();
        let mut full_text = String::new();
        let mut all_tools = vec![];

        while let Some(chunk) = stream.next().await {
            if let Ok(text) = chunk {
                let (safe_text, tools) = parser.push(&text);
                full_text.push_str(&safe_text);
                
                for tool in tools {
                    if tool.name == "edit_file" {
                        all_tools.push(tool);
                    } else {
                        full_text.push_str(&format!("\n[SECURITY] Blocked attempt to use illegal tool: {}", tool.name));
                    }
                }
            }
        }

        let state = if all_tools.is_empty() { AgentState::Completed } else { AgentState::AwaitingApproval };

        Ok(AgentOutput {
            content: full_text,
            status: state,
            requested_tools: all_tools,
        })
    }
}
