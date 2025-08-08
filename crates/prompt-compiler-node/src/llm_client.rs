use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, debug, error};

use crate::ChatCompletionRequest;

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<crate::ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

pub struct LLMResponse {
    pub content: String,
    pub usage: TokenUsage,
}

pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub struct LLMClient {
    client: Client,
    api_key: String,
    base_url: String,
}

impl LLMClient {
    pub async fn new() -> Result<Self> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .or_else(|_| std::env::var("LLM_API_KEY"))
            .expect("OpenAI API key not found in environment");

        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()?;

        info!("LLM Client initialized with base URL: {}", base_url);

        Ok(Self {
            client,
            api_key,
            base_url,
        })
    }

    pub async fn complete(
        &self,
        request: &ChatCompletionRequest,
        messages: &[crate::ChatMessage],
    ) -> Result<LLMResponse> {
        debug!("Calling LLM with {} messages", messages.len());

        // 🔧 新增：打印实际传给LLM的消息内容
        info!("📤 Sending request to LLM API:");
        info!("  Model: {}", request.model);
        info!("  Temperature: {:?}", request.temperature);
        info!("  Max tokens: {:?}", request.max_tokens);
        info!("  Messages count: {}", messages.len());

        for (i, message) in messages.iter().enumerate() {
            let content_preview = if message.content.len() > 200 {
                // 🔧 修复：使用字符边界安全的截断方法
                let truncated = message.content.char_indices()
                    .take_while(|(byte_idx, _)| *byte_idx <= 200)
                    .last()
                    .map(|(byte_idx, ch)| byte_idx + ch.len_utf8())
                    .unwrap_or(0);
                format!("{}...", &message.content[..truncated])
            } else {
                message.content.clone()
            };
            info!("  Message {}: [{}] {}", i + 1, message.role, content_preview);
        }

        // 🔧 打印完整的消息内容（用于调试）
        debug!("📋 Full message details:");
        for (i, message) in messages.iter().enumerate() {
            debug!("Message {}: Role={}, Content='{}'", i + 1, message.role, message.content);
        }

        let openai_request = OpenAIRequest {
            model: request.model.clone(),
            messages: messages.to_vec(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&openai_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            error!("LLM API error: {}", error_text);
            return Err(anyhow::anyhow!("LLM API error: {}", error_text));
        }

        let openai_response: OpenAIResponse = response.json().await?;

        let content = openai_response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .ok_or_else(|| anyhow::anyhow!("No response from LLM"))?;

        debug!("LLM response received: {} tokens", openai_response.usage.total_tokens);

        Ok(LLMResponse {
            content,
            usage: TokenUsage {
                prompt_tokens: openai_response.usage.prompt_tokens,
                completion_tokens: openai_response.usage.completion_tokens,
                total_tokens: openai_response.usage.total_tokens,
            },
        })
    }

    pub async fn health_check(&self) -> Result<bool> {
        // Simple health check by listing models
        let response = self
            .client
            .get(&format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}
