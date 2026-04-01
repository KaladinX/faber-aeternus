use async_trait::async_trait;
use anyhow::Result;
use futures::stream::BoxStream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderType {
    Local,
    Remote,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub model_name: String,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Return the static config for this provider session
    fn config(&self) -> &ProviderConfig;
    
    /// Stream a response from the LLM given a system prompt and complete history
    async fn generate_stream(
        &self,
        system_prompt: &str,
        chat_history: &std::collections::VecDeque<String>,
    ) -> Result<BoxStream<'static, Result<String>>>;

    /// Update the configuration at runtime (e.g. switching models or endpoint)
    fn update_config(&mut self, config: ProviderConfig) -> Result<()>;
}

pub fn create_provider(_name: String, model: Option<String>) -> impl LLMProvider + Send + Sync + 'static {
    // stub — wire your RemoteProvider / CandleProvider here
    let config = ProviderConfig {
        provider_type: ProviderType::Remote,
        model_name: model.unwrap_or_else(|| "default-model".to_string()),
        endpoint: None,
        api_key: None,
    };
    crate::llm::remote::RemoteProvider::new(config)
}