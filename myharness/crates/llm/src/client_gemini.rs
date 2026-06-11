//! rig-core Gemini client wrapper.

use std::sync::Arc;

use async_trait::async_trait;
use rig_core::OneOrMany;
use rig_core::client::CompletionClient;
use rig_core::completion::CompletionModel;
use rig_core::message::{AssistantContent, Message as RigMessage, UserContent};

use crate::client::{CompletionRequest, CompletionResponse, LLMClient, Message, Role};
use crate::error::LlmError;
use crate::provider::ProviderId;

pub struct GeminiProvider {
    id: ProviderId,
    client: Arc<rig_core::providers::gemini::Client>,
    default_model: String,
}

impl GeminiProvider {
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn new(api_key: &str) -> Result<Self, LlmError> {
        let client = rig_core::providers::gemini::Client::new(api_key)
            .map_err(|e| LlmError::ProviderInit(e.to_string()))?;
        Ok(Self {
            id: ProviderId::Gemini,
            client: Arc::new(client),
            default_model: "gemini-2.5-pro".into(),
        })
    }
}

#[async_trait]
impl LLMClient for GeminiProvider {
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

fn build_rig_prompt(messages: &[Message]) -> RigMessage {
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
    fn gemini_provider_new_with_dummy_key_succeeds() {
        let p = GeminiProvider::new("dummy-key");
        assert!(p.is_ok());
    }

    #[test]
    fn gemini_provider_default_model_is_2_5_pro() {
        let p = GeminiProvider::new("dummy").unwrap();
        assert_eq!(p.default_model, "gemini-2.5-pro");
    }
}
