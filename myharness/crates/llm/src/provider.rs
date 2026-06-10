//! 6 provider 식별자 + 종류 분류 (Native / OpenAI 호환).

use serde::{Deserialize, Serialize};

/// v1 의 6 built-in provider. v1.5+ 에서 Custom 추가 가능.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderId {
    /// Anthropic Claude (native SDK)
    Claude,
    /// OpenAI Codex / GPT (native SDK)
    Codex,
    /// Google Gemini (native SDK)
    Gemini,
    /// DeepSeek (OpenAI 호환)
    Deepseek,
    /// Minimax (OpenAI 호환, base_url 미검증 — D-28 TBD)
    Minimax,
    /// local LLM — Ollama / vLLM / LM Studio / llama.cpp (OpenAI 호환)
    LocalLlm,
}

impl ProviderId {
    /// 6 built-in 모두.
    pub const ALL: &'static [ProviderId] = &[
        ProviderId::Claude,
        ProviderId::Codex,
        ProviderId::Gemini,
        ProviderId::Deepseek,
        ProviderId::Minimax,
        ProviderId::LocalLlm,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderId::Claude => "claude",
            ProviderId::Codex => "codex",
            ProviderId::Gemini => "gemini",
            ProviderId::Deepseek => "deepseek",
            ProviderId::Minimax => "minimax",
            ProviderId::LocalLlm => "local-llm",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "claude" => Some(ProviderId::Claude),
            "codex" => Some(ProviderId::Codex),
            "gemini" => Some(ProviderId::Gemini),
            "deepseek" => Some(ProviderId::Deepseek),
            "minimax" => Some(ProviderId::Minimax),
            "local-llm" => Some(ProviderId::LocalLlm),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProviderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// SDK 의 종류. fallback 가능 여부 + base_url 사용 결정.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderKind {
    /// rig-core native SDK 사용 (Anthropic, Gemini)
    Native,
    /// OpenAI 호환 API (DeepSeek, Minimax, local-llm)
    OpenAiCompat,
}

impl ProviderId {
    /// native vs OpenAI 호환 분류. Codex 도 v1 에서는 OpenAI 호환 (`CompletionsClient`) 으로 wrap.
    pub fn kind(&self) -> ProviderKind {
        match self {
            ProviderId::Claude => ProviderKind::Native,
            ProviderId::Gemini => ProviderKind::Native,
            ProviderId::Codex => ProviderKind::OpenAiCompat,
            ProviderId::Deepseek => ProviderKind::OpenAiCompat,
            ProviderId::Minimax => ProviderKind::OpenAiCompat,
            ProviderId::LocalLlm => ProviderKind::OpenAiCompat,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_has_six() {
        assert_eq!(ProviderId::ALL.len(), 6);
    }

    #[test]
    fn as_str_roundtrip() {
        for id in ProviderId::ALL {
            assert_eq!(ProviderId::from_str(id.as_str()), Some(*id));
        }
    }

    #[test]
    fn unknown_str_returns_none() {
        assert_eq!(ProviderId::from_str("bogus"), None);
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(ProviderId::Claude.to_string(), "claude");
        assert_eq!(ProviderId::LocalLlm.to_string(), "local-llm");
    }

    #[test]
    fn serde_kebab_case() {
        let s = serde_json::to_string(&ProviderId::LocalLlm).unwrap();
        assert_eq!(s, "\"local-llm\"");
        let back: ProviderId = serde_json::from_str("\"local-llm\"").unwrap();
        assert_eq!(back, ProviderId::LocalLlm);
    }

    #[test]
    fn kind_classification() {
        assert_eq!(ProviderId::Claude.kind(), ProviderKind::Native);
        assert_eq!(ProviderId::Gemini.kind(), ProviderKind::Native);
        assert_eq!(ProviderId::Deepseek.kind(), ProviderKind::OpenAiCompat);
        assert_eq!(ProviderId::LocalLlm.kind(), ProviderKind::OpenAiCompat);
        assert_eq!(ProviderId::Codex.kind(), ProviderKind::OpenAiCompat);
        assert_eq!(ProviderId::Minimax.kind(), ProviderKind::OpenAiCompat);
    }
}
