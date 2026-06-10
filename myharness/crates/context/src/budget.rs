//! Token budget 추적 + /compact Layer 1.
//!
//! 정책 (D-30):
//! - 매 message 마다 token 사용량 추적 (chars/4 근사치)
//! - 한계 80% 도달 시 auto 압축
//! - 압축 전략: Truncate (keep_recent) / Summarize (stub) / Hybrid (둘 다)
//! - /compact slash command 는 user-callable 수동 압축

use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use myharness_compression::Summarizer;

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
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: Role::User, content: content.into() }
    }
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: Role::System, content: content.into() }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompactStrategy {
    /// 최근 N message 만 keep, 나머지 drop
    Truncate,
    /// (W8.5+) 오래된 message LLM 요약. v1 에선 stub.
    Summarize,
    /// 둘 다
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// 모델 한계 (예: 200_000 for Claude Sonnet 4)
    pub max_tokens: u32,
    /// auto trigger 비율 (예: 0.8 = 80%)
    pub warn_ratio: f32,
    /// 압축 시 keep 할 최근 message 수
    pub keep_recent: usize,
    /// 압축 전략
    pub strategy: CompactStrategy,
    /// 토큰 추정 분자 (chars per token). 기본 4.
    pub chars_per_token: u32,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            max_tokens: 200_000,
            warn_ratio: 0.8,
            keep_recent: 5,
            strategy: CompactStrategy::Hybrid,
            chars_per_token: 4,
        }
    }
}

#[derive(Debug, Error)]
pub enum BudgetError {
    #[error("invalid config: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetReport {
    pub current_tokens: u32,
    pub max_tokens: u32,
    pub ratio: f32,
    pub should_compact: bool,
}

#[derive(Clone)]
pub struct ContextManager {
    pub config: BudgetConfig,
    history: VecDeque<Message>,
    summarizer: Option<std::sync::Arc<dyn Summarizer>>,
}

impl std::fmt::Debug for ContextManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContextManager")
            .field("config", &self.config)
            .field("history_len", &self.history.len())
            .field("summarizer", &self.summarizer_name())
            .finish()
    }
}

impl ContextManager {
    pub fn new(config: BudgetConfig) -> Result<Self, BudgetError> {
        if config.max_tokens == 0 {
            return Err(BudgetError::Invalid("max_tokens must be > 0".into()));
        }
        if !(0.0..=1.0).contains(&config.warn_ratio) {
            return Err(BudgetError::Invalid("warn_ratio must be in 0..=1".into()));
        }
        if config.chars_per_token == 0 {
            return Err(BudgetError::Invalid("chars_per_token must be > 0".into()));
        }
        Ok(Self { config, history: VecDeque::new(), summarizer: None })
    }

    pub fn with_summarizer(mut self, summarizer: std::sync::Arc<dyn Summarizer>) -> Self {
        self.summarizer = Some(summarizer);
        self
    }

    pub fn set_summarizer(&mut self, summarizer: std::sync::Arc<dyn Summarizer>) {
        self.summarizer = Some(summarizer);
    }

    pub fn clear_summarizer(&mut self) {
        self.summarizer = None;
    }

    pub fn summarizer_name(&self) -> Option<&'static str> {
        self.summarizer.as_ref().map(|s| s.name())
    }

    pub fn push(&mut self, msg: Message) {
        self.history.push_back(msg);
    }

    pub fn history(&self) -> &VecDeque<Message> {
        &self.history
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// 현재 token 사용량 추정 (모든 message content 의 char 합 / chars_per_token).
    pub fn estimate_tokens(&self) -> u32 {
        let total_chars: usize = self.history.iter().map(|m| m.content.chars().count()).sum();
        (total_chars as u32) / self.config.chars_per_token
    }

    pub fn budget_report(&self) -> BudgetReport {
        let current = self.estimate_tokens();
        let max = self.config.max_tokens;
        let ratio = current as f32 / max as f32;
        BudgetReport {
            current_tokens: current,
            max_tokens: max,
            ratio,
            should_compact: ratio >= self.config.warn_ratio,
        }
    }

    /// auto check + 필요 시 압축. 호출자가 push 한 직후 확인.
    pub fn maybe_auto_compact(&mut self) -> Option<BudgetReport> {
        let r = self.budget_report();
        if r.should_compact {
            self.compact();
            Some(r)
        } else {
            None
        }
    }

    /// user-callable 수동 압축.
    pub fn compact(&mut self) {
        match self.config.strategy {
            CompactStrategy::Truncate => self.truncate_keep_recent(),
            CompactStrategy::Summarize => self.run_summarize(),
            CompactStrategy::Hybrid => {
                self.truncate_keep_recent();
                self.run_summarize();
            }
        }
    }

    fn truncate_keep_recent(&mut self) {
        let keep = self.config.keep_recent;
        let len = self.history.len();
        if len > keep {
            let drop_count = len - keep;
            self.history.drain(0..drop_count);
        }
    }

    /// W9.2 — LLM-driven summarize. summarizer 가 없으면 no-op.
    /// drop 할 old message 들을 join → summarizer.summarize() → 결과를 단일 assistant message 로
    /// history 맨 앞에 삽입. 단, summarizer 없을 때는 기존 truncate 만 fallback.
    fn run_summarize(&mut self) {
        let keep = self.config.keep_recent;
        let len = self.history.len();
        if len <= keep {
            return;
        }
        let drop_count = len - keep;
        // summarizer 없으면 truncate fallback
        let Some(summarizer) = self.summarizer.clone() else {
            self.history.drain(0..drop_count);
            return;
        };
        let to_summarize: Vec<Message> = self.history.drain(0..drop_count).collect();
        let combined: String = to_summarize
            .iter()
            .map(|m| format!("[{:?}] {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        // blocking call via tokio runtime handle (이 context 는 sync API)
        let summary = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(summarizer.summarize(&combined))
        });
        match summary {
            Ok(s) => {
                self.history.push_front(Message::assistant(format!("[Summary] {s}")));
            }
            Err(_) => {
                // 실패 시 fallback: drop 만
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mgr() -> ContextManager {
        ContextManager::new(BudgetConfig {
            max_tokens: 100,
            warn_ratio: 0.8,
            keep_recent: 3,
            strategy: CompactStrategy::Truncate,
            chars_per_token: 1, // 테스트 단순화: 1 char = 1 token
        })
        .unwrap()
    }

    #[test]
    fn estimate_tokens_simple() {
        let mut m = mgr();
        m.push(Message::user("abcd")); // 4 chars / 1 = 4
        assert_eq!(m.estimate_tokens(), 4);
    }

    #[test]
    fn budget_report_below_warn() {
        let mut m = mgr();
        for _ in 0..5 {
            m.push(Message::user("a")); // 1 token
        }
        // 5 tokens / 100 = 5%
        let r = m.budget_report();
        assert!(!r.should_compact);
    }

    #[test]
    fn budget_report_above_warn_triggers_compact() {
        let mut m = mgr();
        for _ in 0..90 {
            m.push(Message::user("a"));
        }
        // 90/100 = 90% ≥ 80%
        let r = m.maybe_auto_compact();
        assert!(r.is_some());
        // 압축 후 keep_recent=3 만 남음
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn truncate_keep_recent() {
        let mut m = mgr();
        for i in 0..10 {
            m.push(Message::user(format!("msg{i}")));
        }
        m.compact();
        assert_eq!(m.len(), 3);
        // 마지막 3개 keep
        assert!(m.history[0].content.contains("msg7"));
        assert!(m.history[2].content.contains("msg9"));
    }

    #[test]
    fn invalid_config_zero_max() {
        let r = ContextManager::new(BudgetConfig {
            max_tokens: 0,
            ..BudgetConfig::default()
        });
        assert!(r.is_err());
    }

    #[test]
    fn invalid_config_warn_ratio() {
        let r = ContextManager::new(BudgetConfig {
            warn_ratio: 1.5,
            ..BudgetConfig::default()
        });
        assert!(r.is_err());
    }

    #[test]
    fn invalid_config_zero_chars_per_token() {
        let r = ContextManager::new(BudgetConfig {
            chars_per_token: 0,
            ..BudgetConfig::default()
        });
        assert!(r.is_err());
    }

    #[test]
    fn hybrid_compact_runs_both() {
        let mut m = ContextManager::new(BudgetConfig {
            max_tokens: 100,
            warn_ratio: 0.8,
            keep_recent: 2,
            strategy: CompactStrategy::Hybrid,
            chars_per_token: 1,
        })
        .unwrap();
        for i in 0..10 {
            m.push(Message::user(format!("m{i}")));
        }
        m.compact();
        assert_eq!(m.len(), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn summarize_strategy_uses_summarizer() {
        use myharness_compression::MockSummarizer;
        let s = std::sync::Arc::new(MockSummarizer::new());
        s.push("concise summary");
        let mut m = ContextManager::new(BudgetConfig {
            max_tokens: 100,
            warn_ratio: 0.8,
            keep_recent: 2,
            strategy: CompactStrategy::Summarize,
            chars_per_token: 1,
        })
        .unwrap()
        .with_summarizer(s.clone());
        for i in 0..5 {
            m.push(Message::user(format!("old{i}")));
        }
        m.push(Message::user("keep1"));
        m.push(Message::user("keep2"));
        // run_summarize 는 block_in_place 사용 — async runtime 필요
        m.compact();
        // 결과: [Summary] message + keep1 + keep2 = 3 messages
        assert_eq!(m.len(), 3);
        assert!(m.history[0].content.contains("[Summary]"));
        assert!(m.history[0].content.contains("concise summary"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn summarize_falls_back_to_truncate_without_summarizer() {
        let mut m = ContextManager::new(BudgetConfig {
            max_tokens: 100,
            warn_ratio: 0.8,
            keep_recent: 2,
            strategy: CompactStrategy::Summarize,
            chars_per_token: 1,
        })
        .unwrap();
        for i in 0..5 {
            m.push(Message::user(format!("m{i}")));
        }
        m.compact();
        assert_eq!(m.len(), 2);
    }

    #[test]
    fn empty_manager_no_compact() {
        let m = mgr();
        let r = m.budget_report();
        assert_eq!(r.current_tokens, 0);
        assert!(!r.should_compact);
    }

    #[test]
    fn message_role_preserved_after_compact() {
        let mut m = mgr();
        m.push(Message::user("u1"));
        m.push(Message::assistant("a1"));
        m.push(Message::user("u2"));
        m.push(Message::assistant("a2"));
        m.push(Message::user("u3"));
        m.push(Message::assistant("a3"));
        m.push(Message::user("u4"));
        m.compact(); // keep_recent=3 → [u3, a3, u4]
        assert_eq!(m.len(), 3);
        assert_eq!(m.history[0].role, Role::User);
        assert_eq!(m.history[1].role, Role::Assistant);
        assert_eq!(m.history[2].role, Role::User);
        assert_eq!(m.history[0].content, "u3");
        assert_eq!(m.history[2].content, "u4");
    }
}
