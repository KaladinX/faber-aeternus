// src/llm/remote.rs
use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};

use crate::llm::provider::{LLMProvider, ProviderConfig};

pub struct RemoteProvider {
    pub config: ProviderConfig,
    _client: Client,
}

impl RemoteProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            _client: Client::new(),
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
        system_prompt: &str,
        chat_history: &std::collections::VecDeque<String>,
    ) -> Result<BoxStream<'static, Result<String>>> {
        let endpoint = self.config.endpoint.as_deref().unwrap_or("https://api.openai.com/v1/chat/completions");
        
        let mut messages = vec![
            json!({ "role": "system", "content": system_prompt })
        ];
        
        for msg in chat_history {
            // Primitive role mapping based on generic emoji tags used in app.rs
            let role = if msg.starts_with("👤 You:") { "user" } else { "assistant" };
            messages.push(json!({ "role": role, "content": msg }));
        }

        let mut req = self._client.post(endpoint)
            .json(&json!({
                "model": &self.config.model_name,
                "messages": messages,
                "stream": true,
            }));

        if let Some(key) = &self.config.api_key {
            req = req.bearer_auth(key);
        }

        let response = req.send().await.context("Failed to send LLM request")?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("LLM Error: {} - {}", status, text);
        }

        let stream = response.bytes_stream().map(|chunk_res| {
            let chunk = chunk_res.map_err(|e| anyhow::anyhow!("Stream error: {}", e))?;
            let text = String::from_utf8_lossy(&chunk);
            
            let mut output = String::new();
            for line in text.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { continue; }
                    if let Ok(json) = serde_json::from_str::<Value>(data) {
                        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                            output.push_str(content);
                        }
                    }
                }
            }
            Ok(output)
        });

        Ok(stream.boxed())
    }

    fn update_config(&mut self, config: ProviderConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }
}
