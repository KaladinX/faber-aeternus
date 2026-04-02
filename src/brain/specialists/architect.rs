// src/brain/specialists/architect.rs
use crate::brain::agent::{SpecialistAgent, AgentOutput, AgentState};
use crate::brain::context::ProjectContext;
use crate::llm::provider::LLMProvider;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use futures::stream::StreamExt;

pub struct ArchitectAgent {}

impl ArchitectAgent {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SpecialistAgent for ArchitectAgent {
    fn name(&self) -> &'static str {
        "Architect"
    }

    fn description(&self) -> &'static str {
        "Long-horizon task decomposition and repository planning. Read-only."
    }

    async fn execute(
        &mut self,
        input: &str,
        context: &mut ProjectContext<'_>,
        llm: Arc<dyn LLMProvider + Send + Sync>,
        chat_history: &std::collections::VecDeque<String>,
    ) -> Result<AgentOutput> {
        let system_prompt = "You are the Architect agent, a master systems designer. You only read the project and output an ULTRAPLAN detailing the exact execution steps and required files. Do not emit tool calls, just structural plans.";

        // Inject reactive folded context
        let extra_files = context.reactive_search(input).unwrap_or_default();
        let origami_context = context.extract_origami_context(extra_files, input);
        
        let prompt_with_context = format!("{}\n\nContext files:\n{}", system_prompt, origami_context);

        let mut stream = llm.generate_stream(&prompt_with_context, chat_history).await?;
        
        let mut full_output = String::new();
        while let Some(chunk) = stream.next().await {
            if let Ok(text) = chunk {
                full_output.push_str(&text);
            }
        }

        Ok(AgentOutput {
            content: full_output,
            status: AgentState::Completed,
            requested_tools: vec![], // Architect is read-only in this stub, generates plans natively.
        })
    }
}
