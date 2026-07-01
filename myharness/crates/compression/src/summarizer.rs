//! `Summarizer` trait + `LlmSummarizer` (rig-core `LLMClient`) + `MockSummarizer` (test).
//!
//! Layer 1 의 정식 동작: budget 한계 도달 시 old message 들을 LLM 으로 요약하여
//! 단일 assistant message 로 history 에 보관. `ANTHROPIC_API_KEY` absent 환경에선
//! `MockSummarizer` 로 unit test.

use async_trait::async_trait;
use thiserror::Error;

use myharness_llm::client::{CompletionRequest, Message};
use myharness_llm::LLMClient;

#[derive(Debug, Error)]
pub enum SummarizerError {
    #[error("llm: {0}")]
    Llm(#[from] myharness_llm::LlmError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

#[async_trait]
pub trait Summarizer: Send + Sync {
    /// input text 를 요약하여 더 짧은 text 반환.
    async fn summarize(&self, input: &str) -> Result<String, SummarizerError>;
    fn name(&self) -> &'static str;
}

/// `LLMClient` 호출하여 요약. `ANTHROPIC_API_KEY` 있으면 실제 LLM, 없으면 fallback 시 Mock 사용 가능.
pub struct LlmSummarizer {
    client: std::sync::Arc<dyn LLMClient>,
    model: String,
    system: String,
}

impl LlmSummarizer {
    pub fn new(client: std::sync::Arc<dyn LLMClient>) -> Self {
        Self {
            client,
            model: String::new(), // client default
            system: "You are a concise summarizer. Reply with a brief summary in 1-3 sentences, preserving key facts and decisions. Do not add commentary.".into(),
        }
    }

    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }
}

#[async_trait]
impl Summarizer for LlmSummarizer {
    async fn summarize(&self, input: &str) -> Result<String, SummarizerError> {
        let req = CompletionRequest {
            model: self.model.clone(),
            system: Some(self.system.clone()),
            messages: vec![Message::user(input.to_string())],
            max_tokens: Some(512),
            temperature: Some(0.2),
            stop: vec![],
            stream: false,
            metadata: serde_json::Value::Null,
            tools: Vec::new(),
        };
        let resp = self.client.complete(req).await?;
        Ok(resp.content)
    }

    fn name(&self) -> &'static str {
        "llm"
    }
}

/// test 용 canned summarizer. queue 기반.
pub struct MockSummarizer {
    pub queue: std::sync::Mutex<Vec<String>>,
}

impl MockSummarizer {
    #[must_use] 
    pub fn new() -> Self {
        Self { queue: std::sync::Mutex::new(Vec::new()) }
    }

    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn push(&self, summary: impl Into<String>) {
        self.queue.lock().unwrap().push(summary.into());
    }
}

impl Default for MockSummarizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Summarizer for MockSummarizer {
    async fn summarize(&self, input: &str) -> Result<String, SummarizerError> {
        // 기본 동작: input 의 첫 문장 + 길이 축약
        let fallback: String = input
            .split_whitespace()
            .take(20)
            .collect::<Vec<_>>()
            .join(" ");
        let s = self.queue.lock().unwrap().pop().unwrap_or(fallback);
        Ok(s)
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}

/// trivial extractive summarizer (LLM 불필요, fallback).
/// 1) 첫 N 문장 keep, 2) 그 외 제거.
pub struct TrivialSummarizer {
    pub keep_sentences: usize,
    pub max_chars: usize,
}

impl Default for TrivialSummarizer {
    fn default() -> Self {
        Self { keep_sentences: 3, max_chars: 400 }
    }
}

#[async_trait]
impl Summarizer for TrivialSummarizer {
    async fn summarize(&self, input: &str) -> Result<String, SummarizerError> {
        // 문장 단위 분리 (간단: '. ', '! ', '? ' 기준)
        let mut sentences: Vec<&str> = Vec::new();
        let mut rest = input;
        while let Some(idx) = rest.find(['.', '!', '?'])
            .map(|i| (i, rest.as_bytes()[i] as char))
        {
            let (i, _) = idx;
            if i + 1 < rest.len() {
                let (s, r) = rest.split_at(i + 1);
                sentences.push(s.trim());
                rest = r;
            } else {
                sentences.push(rest.trim());
                break;
            }
            if sentences.len() >= self.keep_sentences {
                break;
            }
        }
        let mut out: String = sentences.into_iter().take(self.keep_sentences).collect::<Vec<_>>().join(" ");
        if out.len() > self.max_chars {
            out.truncate(self.max_chars);
        }
        if out.is_empty() {
            out = input.chars().take(self.max_chars).collect();
        }
        Ok(out)
    }

    fn name(&self) -> &'static str {
        "trivial"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myharness_llm::client_mock::{MockClient, MockResponse};
    use myharness_llm::provider::ProviderId;

    #[tokio::test]
    async fn llm_summarizer_calls_client() {
        let c = std::sync::Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c.push(MockResponse::Text("short summary".into()));
        let s = LlmSummarizer::new(c.clone());
        let out = s.summarize("long input text here").await.unwrap();
        assert_eq!(out, "short summary");
        assert_eq!(c.call_count(), 1);
    }

    #[tokio::test]
    async fn mock_summarizer_uses_queue() {
        let s = MockSummarizer::new();
        s.push("first canned");
        s.push("second canned");
        let a = s.summarize("long text").await.unwrap();
        let b = s.summarize("long text").await.unwrap();
        // queue 는 LIFO
        assert_eq!(b, "first canned");
        assert_eq!(a, "second canned");
    }

    #[tokio::test]
    async fn mock_summarizer_falls_back_to_truncate() {
        let s = MockSummarizer::new();
        let out = s.summarize("one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen twenty twentyone twentytwo twentythree twentyfour twentyfive").await.unwrap();
        // 20 단어까지만 keep
        assert!(out.split_whitespace().count() <= 20);
    }

    #[tokio::test]
    async fn trivial_summarizer_keeps_first_n_sentences() {
        let s = TrivialSummarizer { keep_sentences: 2, max_chars: 200 };
        let out = s.summarize("First sentence. Second sentence. Third sentence. Fourth.").await.unwrap();
        assert!(out.contains("First"));
        assert!(out.contains("Second"));
        assert!(!out.contains("Third"));
    }

    #[tokio::test]
    async fn trivial_summarizer_truncates_long_output() {
        let s = TrivialSummarizer { keep_sentences: 100, max_chars: 10 };
        let out = s.summarize("abcdefghijklmnop").await.unwrap();
        assert!(out.len() <= 10);
    }

    #[test]
    fn name_returns_distinct_values() {
        let c = std::sync::Arc::new(MockClient::new(ProviderId::Claude, "x"));
        assert_eq!(LlmSummarizer::new(c).name(), "llm");
        assert_eq!(MockSummarizer::new().name(), "mock");
        assert_eq!(TrivialSummarizer::default().name(), "trivial");
    }
}
