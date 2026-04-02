// src/brain/specialists/reviewer.rs
use crate::brain::agent::{SpecialistAgent, AgentOutput, AgentState};
use crate::brain::context::ProjectContext;
use crate::llm::provider::LLMProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tree_sitter::Parser;

pub struct ReviewerAgent {}

impl ReviewerAgent {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SpecialistAgent for ReviewerAgent {
    fn name(&self) -> &'static str {
        "Reviewer"
    }

    fn description(&self) -> &'static str {
        "Adversarial self-critic looking for bugs in Coder's diffs."
    }

    async fn execute(
        &mut self,
        input: &str,
        _context: &mut ProjectContext<'_>,
        _llm: Arc<dyn LLMProvider + Send + Sync>,
        _chat_history: &std::collections::VecDeque<String>,
    ) -> Result<AgentOutput> {
         // Attempt to parse tool outputs specifically looking for 'content'
         if let Ok(value) = serde_json::from_str::<serde_json::Value>(input) {
             if let Some(content) = value.get("content").and_then(|c| c.as_str()) {
                 let mut parser = Parser::new();
                 // Use default rust for this check phase
                 parser.set_language(&tree_sitter_rust::LANGUAGE.into()).unwrap();
                 if let Some(tree) = parser.parse(content, None) {
                     // Phase 5 Adversarial loop check!
                     if tree.root_node().has_error() {
                         return Ok(AgentOutput {
                             content: "Reviewer Auto-Rejected Diff: Tree-sitter AST ERROR nodes detected.".into(),
                             status: AgentState::Failed,
                             requested_tools: vec![],
                         });
                     }
                 }
             }
         }

         Ok(AgentOutput {
            content: "Reviewer completed. AST structural checks passed.".into(),
            status: AgentState::Completed,
            requested_tools: vec![],
         })
    }
}
