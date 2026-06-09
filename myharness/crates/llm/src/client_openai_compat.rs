//! OpenAI 호환 provider wrapper (DeepSeek / Minimax / local-llm).
//!
//! rig-core 의 `openai::CompletionsClient` (Chat Completions API) 를 사용.
//! DeepSeek, Minimax, local-llm (Ollama) 모두 base_url 만 다르고 동일.

use std::sync::Arc;

use async_trait::async_trait;
use rig_core::OneOrMany;
use rig_core::client::CompletionClient;
use rig_core::completion::CompletionModel;
use rig_core::message::{AssistantContent, Message as RigMessage, UserContent};

use crate::client::{CompletionRequest, CompletionResponse, LLMClient};
use crate::error::LlmError;
use crate::provider::ProviderId;

pub struct OpenAiCompatProvider {
    id: ProviderId,
    client: Arc<rig_core::providers::openai::CompletionsClient>,
    default_model: String,
    base_url: String,
}

impl OpenAiCompatProvider {
    /// `base_url`: OpenAI 호환 root (예: `https://api.deepseek.com/v1`, `http://localhost:11434/v1`).
    /// `api_key`: 빈 문자열이면 "not-needed" 로 채워짐 (Ollama 등 key-less local 서버용).
    pub fn new(
        base_url: &str,
        api_key: &str,
        default_model: &str,
        id: ProviderId,
    ) -> Result<Self, LlmError> {
        let effective_key = if api_key.is_empty() { "not-needed" } else { api_key };
        let client = rig_core::providers::openai::CompletionsClient::builder()
            .api_key(effective_key)
            .base_url(base_url)
            .build()
            .map_err(|e| LlmError::ProviderInit(e.to_string()))?;
        Ok(Self {
            id,
            client: Arc::new(client),
            default_model: default_model.into(),
            base_url: base_url.into(),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[async_trait]
impl LLMClient for OpenAiCompatProvider {
    fn provider_id(&self) -> ProviderId {
        self.id
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let model = if req.model.is_empty() {
            self.default_model.clone()
        } else {
            req.model
        };

        let prompt = build_rig_prompt(&req.messages);
        let preamble = req.system.clone();
        let max_tokens: u64 = req.max_tokens.unwrap_or(1024).into();

        let model_handle = self.client.completion_model(&model);
        let request = model_handle.completion_request(prompt);
        let request = if let Some(pre) = preamble {
            request.preamble(pre)
        } else {
            request
        };
        let request = request.max_tokens(max_tokens);

        let resp = request
            .send()
            .await
            .map_err(|e| LlmError::ProviderCall(e.to_string()))?;

        let content = extract_text(&resp.choice);

        Ok(CompletionResponse {
            content,
            model,
            stop_reason: None,
            input_tokens: u32::try_from(resp.usage.input_tokens).ok(),
            output_tokens: u32::try_from(resp.usage.output_tokens).ok(),
            raw: None,
        })
    }
}

fn build_rig_prompt(messages: &[crate::client::Message]) -> RigMessage {
    let user_text = messages
        .iter()
        .rev()
        .find(|m| m.role == crate::client::Role::User)
        .map(|m| m.content.clone())
        .unwrap_or_default();
    RigMessage::User {
        content: OneOrMany::one(UserContent::Text(rig_core::message::Text::new(user_text))),
    }
}

fn extract_text(choice: &OneOrMany<AssistantContent>) -> String {
    let mut out = String::new();
    for item in choice.iter() {
        if let AssistantContent::Text(t) = item {
            out.push_str(&t.text);
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_compat_provider_builds_with_dummy() {
        let p = OpenAiCompatProvider::new(
            "https://api.deepseek.com/v1",
            "dummy",
            "deepseek-chat",
            ProviderId::Deepseek,
        )
        .unwrap();
        assert_eq!(p.base_url(), "https://api.deepseek.com/v1");
        assert_eq!(p.provider_id(), ProviderId::Deepseek);
    }

    #[test]
    fn local_llm_with_empty_key_uses_not_needed() {
        let p = OpenAiCompatProvider::new(
            "http://localhost:11434/v1",
            "",
            "llama3.1",
            ProviderId::LocalLlm,
        )
        .unwrap();
        assert_eq!(p.provider_id(), ProviderId::LocalLlm);
    }
}
