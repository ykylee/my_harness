//! rig-core Anthropic client wrapper.

use std::sync::Arc;

use async_trait::async_trait;
use rig_core::client::CompletionClient;
use rig_core::completion::CompletionModel;
use rig_core::message::{Message as RigMessage, UserContent};
use rig_core::OneOrMany;

use crate::client::{CompletionRequest, CompletionResponse, LLMClient, Message, Role};
use crate::error::LlmError;
use crate::metadata::ProviderMetadata;
use crate::provider::ProviderId;

/// rig-core 의 `anthropic::Client` 를 우리 trait 으로 wrap.
pub struct AnthropicProvider {
    id: ProviderId,
    client: Arc<rig_core::providers::anthropic::Client>,
    default_model: String,
}

impl AnthropicProvider {
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn new(api_key: &str) -> Result<Self, LlmError> {
        let client = rig_core::providers::anthropic::Client::builder()
            .api_key(api_key)
            .build()
            .map_err(|e| LlmError::ProviderInit(e.to_string()))?;
        Ok(Self {
            id: ProviderId::Claude,
            client: Arc::new(client),
            default_model: "claude-sonnet-4-6".into(),
        })
    }

    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn from_metadata(meta: &ProviderMetadata, api_key: &str) -> Result<Self, LlmError> {
        let p = Self::new(api_key)?;
        Ok(Self { default_model: meta.default_model.clone(), ..p })
    }
}

#[async_trait]
impl LLMClient for AnthropicProvider {
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

        // rig-core response.choice: OneOrMany<AssistantContent> 에서 텍스트 추출
        let content = extract_text(&resp.choice);

        Ok(CompletionResponse {
            content,
            model: model.clone(),
            stop_reason: None,
            input_tokens: u32::try_from(resp.usage.input_tokens).ok(),
            output_tokens: u32::try_from(resp.usage.output_tokens).ok(),
            raw: None,
            tool_calls: Vec::new(),
        })
    }
}

fn build_rig_prompt(messages: &[Message]) -> RigMessage {
    // v1: 마지막 user 메시지만 사용 (multi-turn 은 v1.5+).
    // tool/system 메시지는 일단 무시 (preamble 로 system 분리, tool 결과는 v1.5+).
    let user_text = messages
        .iter()
        .rev()
        .find(|m| m.role == Role::User)
        .map(|m| m.content.clone())
        .unwrap_or_default();

    RigMessage::User {
        content: OneOrMany::one(UserContent::Text(rig_core::message::Text::new(user_text))),
    }
}

fn extract_text(choice: &OneOrMany<rig_core::message::AssistantContent>) -> String {
    let mut out = String::new();
    for item in choice.iter() {
        if let rig_core::message::AssistantContent::Text(t) = item {
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
    fn anthropic_provider_new_with_dummy_key_succeeds() {
        // rig-core 가 client build 까지만 검증 (실제 호출 없음)
        let p = AnthropicProvider::new("dummy-key-for-construction");
        assert!(p.is_ok());
    }

    #[test]
    fn anthropic_provider_default_model_is_sonnet_4_6() {
        let p = AnthropicProvider::new("dummy").unwrap();
        assert_eq!(p.default_model, "claude-sonnet-4-6");
    }

    #[test]
    fn from_metadata_overrides_default_model() {
        let meta = ProviderMetadata::builtin(ProviderId::Claude);
        let p = AnthropicProvider::from_metadata(&meta, "dummy").unwrap();
        assert_eq!(p.default_model, meta.default_model);
    }
}
