//! `OpenAI` 호환 provider wrapper (`DeepSeek` / Minimax / local-llm).
//!
//! rig-core 의 `openai::CompletionsClient` (Chat Completions API) 를 사용.
//! `DeepSeek`, Minimax, local-llm (Ollama) 모두 `base_url` 만 다르고 동일.

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
    /// Cached API key for the wire-format native tool calling path (D-108 follow-up).
    api_key: String,
}

impl OpenAiCompatProvider {
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// `base_url`: `OpenAI` 호환 root (예: `https://api.deepseek.com/v1`, `http://localhost:11434/v1`).
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
            api_key: effective_key.to_string(),
        })
    }

    #[must_use] 
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
        // D-108 follow-up: native tool calling path. When the caller
        // supplies `tools`, the rig-core builder does not yet expose
        // a way to attach them, so we hand-roll the OpenAI wire format
        // with reqwest. Plain text requests keep the rig-core path.
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

        let content = extract_text(&resp.choice);

        Ok(CompletionResponse {
            content,
            model,
            stop_reason: None,
            input_tokens: u32::try_from(resp.usage.input_tokens).ok(),
            output_tokens: u32::try_from(resp.usage.output_tokens).ok(),
            raw: None,
            tool_calls: Vec::new(),
        })
    }
}

impl OpenAiCompatProvider {
    /// D-108 follow-up: hand-rolled OpenAI-compatible /chat/completions
    /// POST that carries `tools` natively and parses
    /// `choices[0].message.tool_calls` back into `ToolCall`s.
    pub async fn complete_wire_format(
        &self,
        req: &CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        let model = if req.model.is_empty() {
            self.default_model.clone()
        } else {
            req.model.clone()
        };
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let payload = build_chat_payload(&model, req)?;
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| LlmError::ProviderInit(e.to_string()))?;
        let resp = http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
                "openai-compat wire: HTTP {status}: {body}"
            )));
        }
        parse_chat_response(&body, &model)
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

/// Build the OpenAI /v1/chat/completions JSON body. We use
/// `serde_json::Value` directly so unknown server-side fields are
/// preserved in `raw` for debugging.
fn build_chat_payload(
    model: &str,
    req: &CompletionRequest,
) -> Result<serde_json::Value, LlmError> {
    let mut messages: Vec<serde_json::Value> = Vec::new();
    if let Some(sys) = &req.system
        && !sys.is_empty()
    {
        messages.push(serde_json::json!({"role": "system", "content": sys}));
    }
    for m in &req.messages {
        let role = match m.role {
            crate::client::Role::System => "system",
            crate::client::Role::User => "user",
            crate::client::Role::Assistant => "assistant",
            crate::client::Role::Tool => "tool",
        };
        let mut obj = serde_json::json!({"role": role, "content": m.content});
        if role == "tool"
            && let Some(name) = &m.name
        {
            if let Some(id) = name.strip_prefix("call_id:") {
                obj["tool_call_id"] = serde_json::Value::String(id.to_string());
            } else {
                obj["name"] = serde_json::Value::String(name.clone());
            }
        }
        messages.push(obj);
    }
    let tools: Vec<serde_json::Value> = req
        .tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.input_schema,
                }
            })
        })
        .collect();
    let mut payload = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": false,
    });
    if let Some(mt) = req.max_tokens {
        payload["max_tokens"] = serde_json::Value::from(mt);
    }
    if let Some(t) = req.temperature {
        payload["temperature"] = serde_json::json!(t);
    }
    if !tools.is_empty() {
        payload["tools"] = serde_json::Value::Array(tools);
    }
    if !req.stop.is_empty() {
        payload["stop"] = serde_json::json!(req.stop);
    }
    Ok(payload)
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct ChatResponse {
    #[serde(default)]
    model: String,
    #[serde(default)]
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<ChatUsage>,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct ChatChoice {
    #[serde(default)]
    message: ChatMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(serde::Deserialize, Default)]
#[allow(dead_code)]
struct ChatMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<ChatToolCall>>,
    #[serde(flatten)]
    extra: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct ChatToolCall {
    #[serde(default)]
    id: String,
    #[serde(rename = "type", default)]
    #[allow(dead_code)]
    call_type: String,
    function: ChatToolCallFunction,
}

#[derive(serde::Deserialize)]
struct ChatToolCallFunction {
    name: String,
    arguments: String,
}

#[derive(serde::Deserialize, Default)]
struct ChatUsage {
    #[serde(default)]
    prompt_tokens: Option<u32>,
    #[serde(default)]
    completion_tokens: Option<u32>,
}

/// Parse an OpenAI-compatible chat completion response body into our
/// `CompletionResponse`. Tool call `arguments` strings are decoded
/// to JSON; a malformed payload yields `LlmError::ProviderCall`.
fn parse_chat_response(body: &str, model: &str) -> Result<CompletionResponse, LlmError> {
    let parsed: ChatResponse = serde_json::from_str(body).map_err(|e| {
        LlmError::ProviderCall(format!(
            "openai-compat wire: invalid JSON body: {e}; body={body}"
        ))
    })?;
    let parsed_model = if !parsed.model.is_empty() { parsed.model } else { model.to_string() };
    let choice = parsed.choices.into_iter().next().ok_or_else(|| {
        LlmError::ProviderCall("openai-compat wire: response has no choices".to_string())
    })?;
    let content = choice.message.content.unwrap_or_default();
    let stop_reason = choice.finish_reason;
    let mut tool_calls: Vec<crate::client::ToolCall> = Vec::new();
    if let Some(calls) = choice.message.tool_calls {
        for c in calls {
            let arguments: serde_json::Value = serde_json::from_str(&c.function.arguments)
                .map_err(|e| {
                    LlmError::ProviderCall(format!(
                        "openai-compat wire: tool_call `{}` has invalid arguments JSON: {e}; raw={}",
                        c.function.name, c.function.arguments
                    ))
                })?;
            tool_calls.push(crate::client::ToolCall {
                id: c.id,
                name: c.function.name,
                arguments,
            });
        }
    }
    let raw = serde_json::from_str::<serde_json::Value>(body).ok();
    let (input_tokens, output_tokens) = match parsed.usage {
        Some(u) => (u.prompt_tokens, u.completion_tokens),
        None => (None, None),
    };
    Ok(CompletionResponse {
        content,
        model: parsed_model,
        stop_reason,
        input_tokens,
        output_tokens,
        raw,
        tool_calls,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Message, Role, ToolSpec};

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

    /// W12 (D-50) — `MiniMax` OpenAI-compat client 구성 검증 (real network 없음).
    /// `base_url/모델/key` 가 rig-core `CompletionsClient` 로 정확히 전달되는지.
    #[test]
    fn minimax_provider_builds_with_correct_metadata() {
        let p = OpenAiCompatProvider::new(
            "https://api.minimax.io/v1",
            "sk-cp-fake-test-key-12345",
            "MiniMax-M3",
            ProviderId::Minimax,
        )
        .unwrap();
        assert_eq!(p.provider_id(), ProviderId::Minimax);
        assert_eq!(p.base_url(), "https://api.minimax.io/v1");
        assert_eq!(p.default_model, "MiniMax-M3");
    }

    /// W12 (D-50) — CN endpoint (api.minimaxi.com) 도 동일하게 빌드 가능.
    #[test]
    fn minimax_cn_endpoint_builds() {
        let p = OpenAiCompatProvider::new(
            "https://api.minimaxi.com/v1",
            "sk-cp-cn-key",
            "MiniMax-M3",
            ProviderId::Minimax,
        )
        .unwrap();
        assert_eq!(p.base_url(), "https://api.minimaxi.com/v1");
    }

    /// W12 (D-50) — real `MiniMax` API 호출. `MINIMAX_API_KEY` env 가 있어야 동작.
    /// network test — CI 환경에선 #[ignore], 수동 실행:
    /// `MINIMAX_API_KEY=... cargo test minimax_real_api_smoke -- --ignored --nocapture`
    #[tokio::test]
    #[ignore = "requires real MINIMAX_API_KEY + network access"]
    async fn minimax_real_api_smoke() {
        let Ok(api_key) = std::env::var("MINIMAX_API_KEY") else {
            eprintln!("MINIMAX_API_KEY not set; skipping");
            return;
        };
        let base_url = std::env::var("MINIMAX_API_HOST")
            .unwrap_or_else(|_| "https://api.minimax.io/v1".into());
        let p = OpenAiCompatProvider::new(
            &base_url,
            &api_key,
            "MiniMax-M3",
            ProviderId::Minimax,
        )
        .unwrap();
        let req = CompletionRequest {
            model: "MiniMax-M3".into(),
            system: Some("You are a concise assistant. Reply in 1 sentence.".into()),
            messages: vec![Message::user("Reply with the single word: PONG")],
            max_tokens: Some(32),
            temperature: Some(1.0),
            stop: vec![],
            stream: false,
            metadata: serde_json::Value::Null,
            tools: Vec::new(),
        };
        let resp = p.complete(req).await.expect("MiniMax API call failed");
        eprintln!("MiniMax response: model={} content={:?}", resp.model, resp.content);
        assert!(!resp.content.is_empty(), "empty response from MiniMax");
    }

    // --- D-108 follow-up: OpenAI wire format payload + response parse --------


    #[test]
    fn build_chat_payload_includes_tools_and_messages() {
        let req = CompletionRequest {
            model: "MiniMax-M3".into(),
            system: Some("you are a tool user".into()),
            messages: vec![
                Message::user(String::from("list files")),
            ],
            max_tokens: Some(64),
            temperature: Some(0.0),
            stop: vec![],
            stream: false,
            metadata: serde_json::Value::Null,
            tools: vec![ToolSpec::new("Read", "Read a file")],
        };
        let payload = build_chat_payload("MiniMax-M3", &req).unwrap();
        let obj = payload.as_object().expect("payload must be object");
        assert_eq!(obj["model"], "MiniMax-M3");
        assert_eq!(obj["stream"], false);
        assert_eq!(obj["max_tokens"], 64);
        let messages = obj["messages"].as_array().expect("messages");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[1]["role"], "user");
        let tools = obj["tools"].as_array().expect("tools");
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
        assert_eq!(tools[0]["function"]["name"], "Read");
        assert_eq!(tools[0]["function"]["description"], "Read a file");
    }

    #[test]
    fn build_chat_payload_omits_tools_when_empty() {
        let req = CompletionRequest::default();
        let payload = build_chat_payload("m", &req).unwrap();
        assert!(payload.get("tools").is_none());
        assert!(payload.get("max_tokens").is_none());
        assert!(payload.get("temperature").is_none());
    }

    #[test]
    fn build_chat_payload_tool_message_carries_call_id() {
        // Tool result messages need the matching `tool_call_id` so the
        // server can route the result back to the right call.
        let mut m = Message::tool(String::from("file content"), "call_id:call_42");
        m.role = Role::Tool;
        let req = CompletionRequest {
            messages: vec![m],
            ..CompletionRequest::default()
        };
        let payload = build_chat_payload("m", &req).unwrap();
        let msg = &payload["messages"][0];
        assert_eq!(msg["role"], "tool");
        assert_eq!(msg["tool_call_id"], "call_42");
    }

    #[test]
    fn parse_chat_response_plain_text() {
        let body = r#"{
            "id": "x",
            "model": "MiniMax-M3",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "hello"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 5, "completion_tokens": 1}
        }"#;
        let resp = parse_chat_response(body, "fallback").unwrap();
        assert_eq!(resp.content, "hello");
        assert_eq!(resp.model, "MiniMax-M3");
        assert_eq!(resp.input_tokens, Some(5));
        assert_eq!(resp.output_tokens, Some(1));
        assert_eq!(resp.stop_reason.as_deref(), Some("stop"));
        assert!(resp.tool_calls.is_empty());
    }

    #[test]
    fn parse_chat_response_with_tool_calls() {
        let body = r#"{
            "model": "MiniMax-M3",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [
                        {
                            "id": "call_1",
                            "type": "function",
                            "function": {
                                "name": "Bash",
                                "arguments": "{\"command\":\"echo hi\"}"
                            }
                        },
                        {
                            "id": "call_2",
                            "type": "function",
                            "function": {
                                "name": "Read",
                                "arguments": "{\"file_path\":\"/tmp/x\"}"
                            }
                        }
                    ]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 0}
        }"#;
        let resp = parse_chat_response(body, "fallback").unwrap();
        assert_eq!(resp.tool_calls.len(), 2);
        assert_eq!(resp.tool_calls[0].id, "call_1");
        assert_eq!(resp.tool_calls[0].name, "Bash");
        assert_eq!(resp.tool_calls[0].arguments["command"], "echo hi");
        assert_eq!(resp.tool_calls[1].name, "Read");
        assert_eq!(resp.tool_calls[1].arguments["file_path"], "/tmp/x");
        assert_eq!(resp.stop_reason.as_deref(), Some("tool_calls"));
    }

    #[test]
    fn parse_chat_response_invalid_tool_args_yields_error() {
        // Tool call arguments that are not valid JSON should fail with
        // a clear, actionable error.
        let body = r#"{
            "model": "MiniMax-M3",
            "choices": [{
                "message": {
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {"name": "Bash", "arguments": "not-json"}
                    }]
                }
            }]
        }"#;
        let err = parse_chat_response(body, "m").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("tool_call") && msg.contains("invalid arguments"), "msg was: {msg}");
    }

    #[test]
    fn parse_chat_response_no_choices_yields_error() {
        let body = r#"{"model": "m", "choices": []}"#;
        let err = parse_chat_response(body, "m").unwrap_err();
        assert!(format!("{err}").contains("no choices"));
    }

    #[test]
    fn parse_chat_response_mixed_text_and_tool_calls() {
        // Some servers (e.g. Anthropic-via-OpenAI-compat) emit BOTH
        // text content and tool_calls. We surface both.
        let body = r#"{
            "model": "m",
            "choices": [{
                "message": {
                    "content": "I will check the env.",
                    "tool_calls": [{
                        "id": "c1",
                        "type": "function",
                        "function": {"name": "Bash", "arguments": "{}"}
                    }]
                }
            }]
        }"#;
        let resp = parse_chat_response(body, "m").unwrap();
        assert_eq!(resp.content, "I will check the env.");
        assert_eq!(resp.tool_calls.len(), 1);
        assert_eq!(resp.tool_calls[0].name, "Bash");
    }
}
