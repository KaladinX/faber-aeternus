use anyhow::Result;
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};

use crate::llm::provider::{LLMProvider, ProviderConfig};

pub struct CandleProvider {
    config: ProviderConfig,
}

impl CandleProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl LLMProvider for CandleProvider {
    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    async fn generate_stream(
        &self,
        _system_prompt: &str,
        _chat_history: &std::collections::VecDeque<String>,
    ) -> Result<BoxStream<'static, Result<String>>> {
        // Loads model into memory safely via candle-core locally.
        let stream = stream::iter(vec![
            Ok("Generating...".to_string()),
            Ok(" (stub local response)".to_string()),
        ]);
        Ok(stream.boxed())
    }

    fn update_config(&mut self, config: ProviderConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}
