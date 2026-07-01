//! provider-agnostic `LLMClient` trait + 메시지 타입.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::LlmError;
use crate::metadata::ProviderCapabilities;
use crate::provider::ProviderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
        }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
        }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
        }
    }
    pub fn tool(content: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: Some(name.into()),
        }
    }
}

/// A-proper native tool calling (D-108, v1.5+): tool specification sent
/// to the LLM in the completion request.
///
/// `input_schema` is a JSON Schema object describing the arguments the
/// LLM is allowed to emit for this tool. Providers that do not
/// support native tool calling (text-only models) will simply ignore
/// this field on the request side.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema object (e.g. `{"type":"object","properties":{...}}`).
    #[serde(default = "empty_object_schema")]
    pub input_schema: serde_json::Value,
}

fn empty_object_schema() -> serde_json::Value {
    serde_json::json!({"type": "object", "properties": {}})
}

impl ToolSpec {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            input_schema: empty_object_schema(),
        }
    }
}

/// A-proper native tool calling (D-108): a single structured tool call
/// the LLM emitted in its completion response. `id` is provider-opaque
/// (OpenAI uses "call_xxx", Anthropic uses "toolu_xxx"); `arguments`
/// is the parsed JSON object the LLM chose for the tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompletionRequest {
    #[serde(default)]
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stop: Vec<String>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub metadata: serde_json::Value,
    /// A-proper native tool calling (D-108): tool specs the model may
    /// invoke. Empty for text-only completions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
    /// A-proper native tool calling (D-108): structured tool calls the
    /// LLM emitted. Empty when the model responded with plain text.
    /// Callers should treat the presence of any `tool_calls` as
    /// authoritative — `content` may be empty or contain
    /// intermediate text the model said before invoking the tool.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
}

#[async_trait]
pub trait LLMClient: Send + Sync {
    fn provider_id(&self) -> ProviderId;
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError>;
    fn supports(&self) -> ProviderCapabilities {
        ProviderCapabilities::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_constructors() {
        let u = Message::user("hi");
        assert_eq!(u.role, Role::User);
        assert_eq!(u.content, "hi");
        assert!(u.name.is_none());

        let t = Message::tool("result", "bash");
        assert_eq!(t.role, Role::Tool);
        assert_eq!(t.name.as_deref(), Some("bash"));
    }

    #[test]
    fn completion_request_default() {
        let r = CompletionRequest::default();
        assert_eq!(r.model, "");
        assert!(r.messages.is_empty());
        assert!(!r.stream);
    }

    #[test]
    fn completion_response_serde() {
        let r = CompletionResponse {
            content: "ok".into(),
            model: "claude-sonnet-4-6".into(),
            stop_reason: Some("end_turn".into()),
            input_tokens: Some(10),
            output_tokens: Some(3),
            raw: None,
            tool_calls: Vec::new(),
        };
        let s = serde_json::to_string(&r).unwrap();
        let back: CompletionResponse = serde_json::from_str(&s).unwrap();
        assert_eq!(back.content, "ok");
        assert_eq!(back.input_tokens, Some(10));
    }
}
