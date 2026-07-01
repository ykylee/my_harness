//! rig-core Anthropic client wrapper.

use std::sync::Arc;

use async_trait::async_trait;
use rig_core::OneOrMany;
use rig_core::client::CompletionClient;
use rig_core::completion::CompletionModel;
use rig_core::message::{Message as RigMessage, UserContent};

use crate::client::{CompletionRequest, CompletionResponse, LLMClient, Message, Role};
use crate::error::LlmError;
use crate::metadata::ProviderMetadata;
use crate::provider::ProviderId;

/// rig-core 의 `anthropic::Client` 를 우리 trait 으로 wrap.
/// D-122: hand-rolled Anthropic wire format (tool_use block parse) 지원
/// 위해 `api_key` + `base_url` field 노출. rig-core path 는 plain text
/// 전용으로 유지.
pub struct AnthropicProvider {
    id: ProviderId,
    /// rig-core client (plain text path 전용)
    client: Arc<rig_core::providers::anthropic::Client>,
    /// D-122: hand-rolled wire format path 에서 사용
    api_key: String,
    /// D-122: 기본 `"https://api.anthropic.com"`. mock server test 시 override.
    base_url: String,
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
            api_key: api_key.to_string(),
            base_url: "https://api.anthropic.com".to_string(),
            default_model: "claude-sonnet-4-6".into(),
        })
    }

    /// D-122: hand-rolled wire format 의 endpoint override (test-only mock server).
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn from_metadata(meta: &ProviderMetadata, api_key: &str) -> Result<Self, LlmError> {
        let p = Self::new(api_key)?;
        Ok(Self {
            default_model: meta.default_model.clone(),
            ..p
        })
    }
}

#[async_trait]
impl LLMClient for AnthropicProvider {
    fn provider_id(&self) -> ProviderId {
        self.id
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        // D-122: native tool calling path. Caller 가 `tools` 를 supply 하면
        // rig-core builder 가 아직 tools 를 expose 안 하므로 hand-rolled
        // Anthropic `/v1/messages` POST 로 전환. plain text 는 rig-core path 유지.
        if !req.tools.is_empty() {
            return self.complete_wire_format(&req).await;
        }

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

        // D-122: rig-core native path 에서도 AssistantContent::ToolCall 추출.
        // 우리 v1 의 wire format path (hand-rolled) 와 동등하게 tool_use 보존.
        let tool_calls = extract_tool_calls(&resp.choice);

        Ok(CompletionResponse {
            content,
            model: model.clone(),
            stop_reason: None,
            input_tokens: u32::try_from(resp.usage.input_tokens).ok(),
            output_tokens: u32::try_from(resp.usage.output_tokens).ok(),
            raw: None,
            tool_calls,
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

/// D-122: `AssistantContent::ToolCall` variant 를 우리 `ToolCall` 로 변환.
/// rig-core native path 와 hand-rolled wire format path 모두 동일한
/// shape (id, name, arguments: serde_json::Value) 으로 normalize.
fn extract_tool_calls(
    choice: &OneOrMany<rig_core::message::AssistantContent>,
) -> Vec<crate::client::ToolCall> {
    let mut out = Vec::new();
    for item in choice.iter() {
        if let rig_core::message::AssistantContent::ToolCall(tc) = item {
            out.push(crate::client::ToolCall {
                id: tc.id.clone(),
                name: tc.function.name.clone(),
                arguments: tc.function.arguments.clone(),
            });
        }
    }
    out
}

impl AnthropicProvider {
    /// D-122: hand-rolled Anthropic `/v1/messages` POST with native
    /// `tools` block. OpenAI-compat 의 `complete_wire_format` 와 동일 패턴.
    /// response.content[] 중 `type:"tool_use"` block 을 `ToolCall` 로 변환.
    pub async fn complete_wire_format(
        &self,
        req: &CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let model = if req.model.is_empty() {
            self.default_model.clone()
        } else {
            req.model.clone()
        };
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let payload = build_anthropic_payload(&model, req)?;
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| LlmError::ProviderInit(e.to_string()))?;
        let resp = http
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| LlmError::ProviderCall(e.to_string()))?;
        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| LlmError::ProviderCall(e.to_string()))?;
        if !status.is_success() {
            return Err(LlmError::ProviderCall(format!(
                "anthropic wire: HTTP {status}: {body}"
            )));
        }
        parse_anthropic_response(&body, &model)
    }
}

/// D-122: Anthropic `/v1/messages` request payload.
fn build_anthropic_payload(
    model: &str,
    req: &CompletionRequest,
) -> Result<serde_json::Value, LlmError> {
    let mut messages: Vec<serde_json::Value> = Vec::new();
    for m in &req.messages {
        let role = match m.role {
            crate::client::Role::System => continue, // Anthropic: system 은 top-level
            crate::client::Role::User => "user",
            crate::client::Role::Assistant => "assistant",
            crate::client::Role::Tool => "user", // Anthropic tool_result 는 user content[] 안에
        };
        if m.role == crate::client::Role::Tool {
            // tool_result content block
            let tool_use_id = m
                .name
                .as_deref()
                .and_then(|n| n.strip_prefix("call_id:"))
                .unwrap_or("");
            messages.push(serde_json::json!({
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": tool_use_id,
                    "content": m.content,
                }]
            }));
        } else {
            messages.push(serde_json::json!({"role": role, "content": m.content}));
        }
    }
    let tools: Vec<serde_json::Value> = req
        .tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.input_schema,
            })
        })
        .collect();
    let mut payload = serde_json::json!({
        "model": model,
        "messages": messages,
    });
    if let Some(sys) = &req.system
        && !sys.is_empty()
    {
        payload["system"] = serde_json::Value::String(sys.clone());
    }
    if let Some(mt) = req.max_tokens {
        payload["max_tokens"] = serde_json::Value::from(mt);
    }
    if let Some(t) = req.temperature {
        payload["temperature"] = serde_json::json!(t);
    }
    if !tools.is_empty() {
        payload["tools"] = serde_json::Value::Array(tools);
    }
    Ok(payload)
}

#[derive(serde::Deserialize)]
struct AnthropicWireResponse {
    #[serde(default)]
    model: String,
    #[serde(default)]
    content: Vec<AnthropicContentBlock>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
enum AnthropicContentBlock {
    Text { text: String },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(other)]
    Other,
}

#[derive(serde::Deserialize)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
}

fn parse_anthropic_response(body: &str, model: &str) -> Result<CompletionResponse, LlmError> {
    let parsed: AnthropicWireResponse = serde_json::from_str(body)
        .map_err(|e| LlmError::ProviderCall(format!("anthropic wire parse: {e}: {body}")))?;
    let mut content = String::new();
    let mut tool_calls = Vec::new();
    for block in parsed.content {
        match block {
            AnthropicContentBlock::Text { text } => {
                content.push_str(&text);
                content.push('\n');
            }
            AnthropicContentBlock::ToolUse { id, name, input } => {
                tool_calls.push(crate::client::ToolCall {
                    id,
                    name,
                    arguments: input,
                });
            }
            AnthropicContentBlock::Other => {}
        }
    }
    let content = content.trim_end().to_string();
    let (input_tokens, output_tokens) = match parsed.usage {
        Some(u) => (
            u32::try_from(u.input_tokens).ok(),
            u32::try_from(u.output_tokens).ok(),
        ),
        None => (None, None),
    };
    Ok(CompletionResponse {
        content,
        model: if parsed.model.is_empty() { model.to_string() } else { parsed.model },
        stop_reason: parsed.stop_reason,
        input_tokens,
        output_tokens,
        raw: None,
        tool_calls,
    })
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

    // --- D-122: Anthropic wire format + tool_use block parse (옵션 B) ---

    /// Mock server 가 `text` + `tool_use` block 을 동시 emit → 우리
    /// `parse_anthropic_response` 가 text content + Vec<ToolCall> 로
    /// 정확히 분리. mock server 는 `127.0.0.1` TcpListener 로 띄움.
    #[tokio::test]
    async fn d122_wire_format_parses_tool_use_block() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = vec![0u8; 16_384];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                // request sanity: `tools` 가 wire 에 emit 됨
                assert!(req.contains("\"tools\""), "request missing tools: {req}");
                assert!(req.contains("Read"), "request missing tool name: {req}");

                // mock response: text + tool_use 동시
                let body = r#"{
                    "model": "claude-sonnet-4-6",
                    "stop_reason": "tool_use",
                    "content": [
                        {"type": "text", "text": "Reading the file..."},
                        {
                            "type": "tool_use",
                            "id": "toolu_test_01",
                            "name": "Read",
                            "input": {"path": "/tmp/x.rs", "limit": 50}
                        }
                    ],
                    "usage": {"input_tokens": 42, "output_tokens": 7}
                }"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                sock.write_all(resp.as_bytes()).await.unwrap();
                sock.shutdown().await.ok();
            }
        });

        let provider = AnthropicProvider::new("dummy-key")
            .unwrap()
            .with_base_url(format!("http://{addr}"));
        let req = CompletionRequest {
            model: "claude-sonnet-4-6".to_string(),
            system: None,
            messages: vec![Message {
                role: Role::User,
                content: "read /tmp/x.rs".to_string(),
                name: None,
            }],
            max_tokens: Some(128),
            temperature: None,
            stop: Vec::new(),
            stream: false,
            metadata: serde_json::Value::Null,
            tools: vec![crate::client::ToolSpec::new("Read", "Read a file")],
        };
        let resp = provider.complete_wire_format(&req).await.unwrap();
        server.await.ok();

        // text content
        assert_eq!(resp.content, "Reading the file...");
        assert_eq!(resp.tool_calls.len(), 1, "expected 1 tool_call");
        let tc = &resp.tool_calls[0];
        assert_eq!(tc.id, "toolu_test_01");
        assert_eq!(tc.name, "Read");
        assert_eq!(tc.arguments["path"], "/tmp/x.rs");
        assert_eq!(tc.arguments["limit"], 50);
        assert_eq!(resp.stop_reason.as_deref(), Some("tool_use"));
        assert_eq!(resp.input_tokens, Some(42));
        assert_eq!(resp.output_tokens, Some(7));
    }
}
