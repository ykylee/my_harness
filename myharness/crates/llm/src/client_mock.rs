//! 테스트용 mock client. queue 에 적힌 응답을 순서대로 반환.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::client::{CompletionRequest, CompletionResponse, LLMClient};
use crate::error::LlmError;
use crate::metadata::ProviderCapabilities;
use crate::provider::ProviderId;

#[derive(Debug, Clone)]
pub enum MockResponse {
    Text(String),
    /// A-proper native tool calling (D-108): emit structured tool
    /// calls exactly as a wire-format provider would. Optional
    /// `text_before` carries any text the model said before invoking
    /// the tool.
    ToolCalls {
        text_before: String,
        calls: Vec<crate::client::ToolCall>,
    },
    Error(String),
    Delay {
        ms: u64,
        then: Box<MockResponse>,
    },
}

pub struct MockClient {
    pub id: ProviderId,
    pub model: String,
    queue: Mutex<VecDeque<MockResponse>>,
    pub calls: Mutex<Vec<CompletionRequest>>,
}

impl MockClient {
    pub fn new(id: ProviderId, model: impl Into<String>) -> Self {
        Self {
            id,
            model: model.into(),
            queue: Mutex::new(VecDeque::new()),
            calls: Mutex::new(Vec::new()),
        }
    }

    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn push(&self, resp: MockResponse) {
        self.queue.lock().unwrap().push_back(resp);
    }

    ///
    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    /// `push` 와 동일 (`VecDeque::push_back` → `pop_front` = FIFO).
    /// W11.3 (D-49) 에서 명시적 FIFO 메서드로 추가. push 와 동작이 같아서
    /// 함수 이름으로 의도만 명시.
    pub fn push_fifo(&self, resp: MockResponse) {
        self.queue.lock().unwrap().push_back(resp);
    }

    /// FIFO 순서로 여러 push.
    pub fn push_fifo_many(&self, responses: impl IntoIterator<Item = MockResponse>) {
        for r in responses {
            self.push_fifo(r);
        }
    }

    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }

    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn last_call(&self) -> Option<CompletionRequest> {
        self.calls.lock().unwrap().last().cloned()
    }
}

#[async_trait]
impl LLMClient for MockClient {
    fn provider_id(&self) -> ProviderId {
        self.id
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.calls.lock().unwrap().push(req.clone());
        let next = self.queue.lock().unwrap().pop_front();
        match next {
            Some(MockResponse::Text(s)) => {
                let out_tokens = s.len() as u32;
                Ok(CompletionResponse {
                    content: s,
                    model: self.model.clone(),
                    stop_reason: Some("end_turn".into()),
                    input_tokens: Some(10),
                    output_tokens: Some(out_tokens),
                    raw: None,
                    tool_calls: Vec::new(),
                })
            }
            Some(MockResponse::ToolCalls { text_before, calls }) => {
                let out_tokens = text_before.len() as u32;
                Ok(CompletionResponse {
                    content: text_before,
                    model: self.model.clone(),
                    stop_reason: Some("tool_use".into()),
                    input_tokens: Some(10),
                    output_tokens: Some(out_tokens),
                    raw: None,
                    tool_calls: calls,
                })
            }
            Some(MockResponse::Error(e)) => Err(mock_error_to_llm_error(&e)),
            Some(MockResponse::Delay { ms, then }) => {
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                self.queue.lock().unwrap().push_back(*then);
                Box::pin(self.complete(req)).await
            }
            None => Err(LlmError::ProviderUnavailable(format!(
                "MockClient({}): queue empty",
                self.id
            ))),
        }
    }

    fn supports(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            tool_use: true,
            vision: false,
            thinking: false,
            prompt_cache: false,
            streaming: true,
        }
    }
}

fn mock_error_to_llm_error(s: &str) -> LlmError {
    let lower = s.to_ascii_lowercase();
    if lower.contains("429") || lower.contains("rate") {
        LlmError::RateLimited(s.into())
    } else if lower.contains("401") || lower.contains("auth") || lower.contains("missing") {
        LlmError::AuthMissing(ProviderId::Claude) // mock 은 provider 무관하게 처리
    } else if lower.contains("context") || lower.contains("overflow") || lower.contains("too long")
    {
        LlmError::ContextOverflow(s.into())
    } else if lower.contains("model") && lower.contains("not") {
        LlmError::ModelNotFound(s.into())
    } else {
        LlmError::ProviderUnavailable(s.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Message;

    #[tokio::test]
    async fn mock_client_returns_queued_text() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push(MockResponse::Text("hi".into()));
        let r = c.complete(CompletionRequest::default()).await.unwrap();
        assert_eq!(r.content, "hi");
        assert_eq!(r.model, "claude-sonnet-4-6");
    }

    #[tokio::test]
    async fn mock_client_records_calls() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push(MockResponse::Text("ok".into()));
        let req = CompletionRequest {
            messages: vec![Message::user("ping")],
            ..Default::default()
        };
        c.complete(req).await.unwrap();
        assert_eq!(c.call_count(), 1);
        assert_eq!(c.last_call().unwrap().messages[0].content, "ping");
    }

    #[tokio::test]
    async fn mock_client_queue_empty_returns_unavailable() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        let e = c.complete(CompletionRequest::default()).await.unwrap_err();
        assert!(matches!(e, LlmError::ProviderUnavailable(_)));
    }

    #[tokio::test]
    async fn mock_client_error_response_maps_to_provider_unavailable() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push(MockResponse::Error("dns failure".into()));
        let e = c.complete(CompletionRequest::default()).await.unwrap_err();
        assert!(matches!(e, LlmError::ProviderUnavailable(_)));
    }

    #[tokio::test]
    async fn mock_client_error_with_429_maps_to_rate_limited() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push(MockResponse::Error("HTTP 429".into()));
        let e = c.complete(CompletionRequest::default()).await.unwrap_err();
        assert!(matches!(e, LlmError::RateLimited(_)));
    }

    #[tokio::test]
    async fn mock_client_error_with_context_overflow_maps_correctly() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push(MockResponse::Error("context overflow".into()));
        let e = c.complete(CompletionRequest::default()).await.unwrap_err();
        assert!(matches!(e, LlmError::ContextOverflow(_)));
    }

    #[tokio::test]
    async fn mock_client_multiple_responses_served_in_order() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push(MockResponse::Text("first".into()));
        c.push(MockResponse::Text("second".into()));
        let r1 = c.complete(CompletionRequest::default()).await.unwrap();
        let r2 = c.complete(CompletionRequest::default()).await.unwrap();
        assert_eq!(r1.content, "first");
        assert_eq!(r2.content, "second");
    }

    #[tokio::test]
    async fn mock_client_push_fifo_serves_in_order() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push_fifo(MockResponse::Text("first".into()));
        c.push_fifo(MockResponse::Text("second".into()));
        c.push_fifo(MockResponse::Text("third".into()));
        let r1 = c.complete(CompletionRequest::default()).await.unwrap();
        let r2 = c.complete(CompletionRequest::default()).await.unwrap();
        let r3 = c.complete(CompletionRequest::default()).await.unwrap();
        // FIFO: push 한 순서대로 pop
        assert_eq!(r1.content, "first");
        assert_eq!(r2.content, "second");
        assert_eq!(r3.content, "third");
    }

    #[tokio::test]
    async fn mock_client_push_fifo_many() {
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push_fifo_many(vec![
            MockResponse::Text("a".into()),
            MockResponse::Text("b".into()),
        ]);
        let r1 = c.complete(CompletionRequest::default()).await.unwrap();
        let r2 = c.complete(CompletionRequest::default()).await.unwrap();
        assert_eq!(r1.content, "a");
        assert_eq!(r2.content, "b");
    }

    #[tokio::test]
    async fn mock_client_default_push_is_fifo() {
        // 기본 push 는 FIFO (VecDeque push_back + pop_front)
        let c = MockClient::new(ProviderId::Claude, "claude-sonnet-4-6");
        c.push(MockResponse::Text("first".into()));
        c.push(MockResponse::Text("second".into()));
        let r1 = c.complete(CompletionRequest::default()).await.unwrap();
        let r2 = c.complete(CompletionRequest::default()).await.unwrap();
        assert_eq!(r1.content, "first");
        assert_eq!(r2.content, "second");
    }
}
