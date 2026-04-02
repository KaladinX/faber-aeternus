// src/brain/agent.rs
use anyhow::Result;
use async_trait::async_trait;
use crate::brain::context::ProjectContext;
use crate::tools::ToolCall;
use crate::llm::provider::LLMProvider;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    Active,
    AwaitingApproval,
    Completed,
    Failed,
}

pub struct AgentOutput {
    pub content: String,
    pub status: AgentState,
    pub requested_tools: Vec<ToolCall>,
}

#[async_trait]
pub trait SpecialistAgent: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    
    /// Execute one cognitive step of the agent.
    async fn execute(
        &mut self,
        input: &str,
        context: &mut ProjectContext<'_>,
        llm: Arc<dyn LLMProvider + Send + Sync>,
        chat_history: &std::collections::VecDeque<String>,
    ) -> Result<AgentOutput>;
}
