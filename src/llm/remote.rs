use anyhow::Result;
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use reqwest::Client;

use crate::llm::provider::{LLMProvider, ProviderConfig};

pub struct RemoteProvider {
    pub config: ProviderConfig,
    client: reqwest:: Client,
}

impl RemoteProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMProvider for RemoteProvider {
    fn config(&self) -> &ProviderConfig {
        &self.config
    }

    async fn generate_stream(
        &self,
        _system_prompt: &str,
        _chat_history: &[crate::state::Message],
    ) -> Result<BoxStream<'static, Result<String>>> {
        // Concrete reqwest implementations would hook up into Grok/OpenAI format here.
        // For standard skeleton, we return a mock stream.
        let stream = stream::iter(vec![
            Ok("Generating...".to_string()),
            Ok(" (stub remote response)".to_string()),
        ]);
        Ok(stream.boxed())
    }

    fn update_config(&mut self, config: ProviderConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}
