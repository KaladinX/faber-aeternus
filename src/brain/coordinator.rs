// src/brain/coordinator.rs
use petgraph::graph::{NodeIndex, DiGraph};
use anyhow::Result;
use std::collections::HashMap;

use crate::brain::agent::{SpecialistAgent, AgentOutput, AgentState};
use crate::brain::context::ProjectContext;
use crate::llm::provider::LLMProvider;
use std::sync::Arc;

pub struct Coordinator {
    pub graph: DiGraph<String, ()>,
    pub agents: HashMap<String, Box<dyn SpecialistAgent>>,
}

impl Coordinator {
    pub fn new() -> Self {
        let mut coordinator = Self {
            graph: DiGraph::new(),
            agents: HashMap::new(),
        };

        let arch = Box::new(crate::brain::specialists::ArchitectAgent::new());
        let coder = Box::new(crate::brain::specialists::CoderAgent::new());
        let rev = Box::new(crate::brain::specialists::ReviewerAgent::new());

        let a_idx = coordinator.register_agent(arch);
        let c_idx = coordinator.register_agent(coder);
        let r_idx = coordinator.register_agent(rev);

        coordinator.connect(a_idx, c_idx);
        coordinator.connect(c_idx, r_idx);

        coordinator
    }

    fn register_agent(&mut self, agent: Box<dyn SpecialistAgent>) -> NodeIndex {
        let name = agent.name().to_string();
        self.agents.insert(name.clone(), agent);
        self.graph.add_node(name)
    }

    fn connect(&mut self, from: NodeIndex, to: NodeIndex) {
        self.graph.add_edge(from, to, ());
    }

    pub async fn direct_execute(
        &mut self,
        target_agent: &str,
        input: &str,
        context: &mut ProjectContext<'_>,
        llm: Arc<dyn LLMProvider + Send + Sync>,
        history: &std::collections::VecDeque<String>,
    ) -> Result<AgentOutput> {
		// Can't mutably borrow two agents perfectly at once from HashMap, so we execute sequentially and extract.
        let mut coder_out = {
             let agent = self.agents.get_mut(target_agent).ok_or_else(|| anyhow::anyhow!("Unknown agent"))?;
             agent.execute(input, context, llm.clone(), history).await?
        };

        if target_agent == "Coder" {
            let mut all_valid = true;
            let mut reject_notes = String::new();
            
            for tool in &coder_out.requested_tools {
                 let reviewer = self.agents.get_mut("Reviewer").unwrap();
                 let rev_in = tool.params.to_string(); 
                 
                 if let Ok(rev_out) = reviewer.execute(&rev_in, context, llm.clone(), history).await {
                     if rev_out.status == AgentState::Failed {
                         all_valid = false;
                         reject_notes.push_str(&rev_out.content);
                     }
                 }
            }
            
            if !all_valid {
                coder_out.content.push_str(&format!("\n\n❌ [COORDINATOR] Auto-rejection triggered.\nReason: {}", reject_notes));
                coder_out.status = AgentState::Failed;
                coder_out.requested_tools.clear(); // Safe rejection - User never sees modal, tools are wiped
            }
        }

        Ok(coder_out)
    }
}
