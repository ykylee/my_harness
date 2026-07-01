//! provider 정적 메타데이터 + 6 built-in factory.

use serde::{Deserialize, Serialize};

use crate::provider::{ProviderId, ProviderKind};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // capability flags are intrinsic to provider metadata
pub struct ProviderCapabilities {
    pub tool_use: bool,
    pub vision: bool,
    pub thinking: bool,
    pub prompt_cache: bool,
    pub streaming: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub id: ProviderId,
    pub display_name: String,
    /// 환경변수 이름 (예: "`ANTHROPIC_API_KEY`"). None 이면 env var 기반 인증 안 함 (local-llm 등).
    pub env_var: Option<String>,
    /// keychain service 이름 (예: "myharness:anthropic").
    pub keychain_service: String,
    /// keychain account 이름 (v1: 단일 "default").
    pub keychain_account: String,
    /// API base URL.
    pub base_url: String,
    /// 기본 모델.
    pub default_model: String,
    /// 사용 가능 모델 (probe 시 갱신 가능).
    pub available_models: Vec<String>,
    /// API key 필요 여부 (local-llm: false).
    pub requires_key: bool,
    pub kind: ProviderKind,
    pub supports: ProviderCapabilities,
}

impl ProviderMetadata {
    /// `ProviderId` → built-in metadata.
    #[must_use]
    pub fn builtin(id: ProviderId) -> Self {
        match id {
            ProviderId::Claude => Self::builtin_claude(),
            ProviderId::Codex => Self::builtin_codex(),
            ProviderId::Gemini => Self::builtin_gemini(),
            ProviderId::Deepseek => Self::builtin_deepseek(),
            ProviderId::Minimax => Self::builtin_minimax(),
            ProviderId::LocalLlm => Self::builtin_local_llm(),
        }
    }

    /// 6 built-in 의 Vec (id 순서).
    #[must_use]
    pub fn all_builtins() -> Vec<Self> {
        ProviderId::ALL
            .iter()
            .map(|id| Self::builtin(*id))
            .collect()
    }

    fn builtin_claude() -> Self {
        Self {
            id: ProviderId::Claude,
            display_name: "Anthropic Claude".into(),
            env_var: Some("ANTHROPIC_API_KEY".into()),
            keychain_service: "myharness:anthropic".into(),
            keychain_account: "default".into(),
            base_url: "https://api.anthropic.com".into(),
            default_model: "claude-sonnet-4-6".into(),
            available_models: vec![
                "claude-sonnet-4-6".into(),
                "claude-opus-4-1".into(),
                "claude-haiku-4-5".into(),
            ],
            requires_key: true,
            kind: ProviderKind::Native,
            supports: ProviderCapabilities {
                tool_use: true,
                vision: true,
                thinking: true,
                prompt_cache: true,
                streaming: true,
            },
        }
    }

    fn builtin_codex() -> Self {
        Self {
            id: ProviderId::Codex,
            display_name: "OpenAI Codex".into(),
            env_var: Some("OPENAI_API_KEY".into()),
            keychain_service: "myharness:openai".into(),
            keychain_account: "default".into(),
            base_url: "https://api.openai.com/v1".into(),
            default_model: "gpt-4o".into(),
            available_models: vec![
                "gpt-4o".into(),
                "gpt-4o-mini".into(),
                "o3".into(),
                "o4-mini".into(),
            ],
            requires_key: true,
            kind: ProviderKind::OpenAiCompat,
            supports: ProviderCapabilities {
                tool_use: true,
                vision: true,
                thinking: true,
                prompt_cache: false,
                streaming: true,
            },
        }
    }

    fn builtin_gemini() -> Self {
        Self {
            id: ProviderId::Gemini,
            display_name: "Google Gemini".into(),
            env_var: Some("GOOGLE_API_KEY".into()),
            keychain_service: "myharness:google".into(),
            keychain_account: "default".into(),
            base_url: "https://generativelanguage.googleapis.com".into(),
            default_model: "gemini-2.5-pro".into(),
            available_models: vec!["gemini-2.5-pro".into(), "gemini-2.5-flash".into()],
            requires_key: true,
            kind: ProviderKind::Native,
            supports: ProviderCapabilities {
                tool_use: true,
                vision: true,
                thinking: true,
                prompt_cache: false,
                streaming: true,
            },
        }
    }

    fn builtin_deepseek() -> Self {
        Self {
            id: ProviderId::Deepseek,
            display_name: "DeepSeek".into(),
            env_var: Some("DEEPSEEK_API_KEY".into()),
            keychain_service: "myharness:deepseek".into(),
            keychain_account: "default".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            default_model: "deepseek-chat".into(),
            available_models: vec!["deepseek-chat".into(), "deepseek-reasoner".into()],
            requires_key: true,
            kind: ProviderKind::OpenAiCompat,
            supports: ProviderCapabilities {
                tool_use: true,
                vision: false,
                thinking: true,
                prompt_cache: false,
                streaming: true,
            },
        }
    }

    fn builtin_minimax() -> Self {
        // D-50 (W12) — librarian 조사 결과로 갱신:
        // - base_url: `https://api.minimax.io/v1` (글로벌, .io not .chat)
        //   CN 사용자는 `MINIMAX_API_HOST=https://api.minimaxi.com/v1` env override 가능
        // - default_model: `MiniMax-M3` (1M context, 2026-05-31 출시, coding SOTA)
        // - tool_use: M3 + M2.x 모두 지원
        // - vision: M3 만 지원
        // - thinking: M3 는 toggle, M2.x 는 always on
        Self {
            id: ProviderId::Minimax,
            display_name: "MiniMax".into(),
            env_var: Some("MINIMAX_API_KEY".into()),
            keychain_service: "myharness:minimax".into(),
            keychain_account: "default".into(),
            base_url: "https://api.minimax.io/v1".into(),
            default_model: "MiniMax-M3".into(),
            available_models: vec![
                "MiniMax-M3".into(),
                "MiniMax-M2.7".into(),
                "MiniMax-M2.7-highspeed".into(),
                "MiniMax-M2.5".into(),
                "MiniMax-M2.5-highspeed".into(),
                "MiniMax-M2.1".into(),
                "MiniMax-M2".into(),
            ],
            requires_key: true,
            kind: ProviderKind::OpenAiCompat,
            supports: ProviderCapabilities {
                tool_use: true,
                vision: true,
                thinking: true,
                prompt_cache: false,
                streaming: true,
            },
        }
    }

    fn builtin_local_llm() -> Self {
        Self {
            id: ProviderId::LocalLlm,
            display_name: "Local LLM (Ollama / vLLM / LM Studio / llama.cpp)".into(),
            env_var: None,
            keychain_service: "myharness:local-llm".into(),
            keychain_account: "default".into(),
            base_url: "http://localhost:11434/v1".into(),
            default_model: "llama3.1".into(),
            available_models: vec![],
            requires_key: false,
            kind: ProviderKind::OpenAiCompat,
            supports: ProviderCapabilities {
                tool_use: true,
                vision: false,
                thinking: false,
                prompt_cache: false,
                streaming: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_builtins() {
        let all = ProviderMetadata::all_builtins();
        assert_eq!(all.len(), 6);
        let ids: Vec<_> = all.iter().map(|m| m.id).collect();
        assert!(ids.contains(&ProviderId::Claude));
        assert!(ids.contains(&ProviderId::Codex));
        assert!(ids.contains(&ProviderId::Gemini));
        assert!(ids.contains(&ProviderId::Deepseek));
        assert!(ids.contains(&ProviderId::Minimax));
        assert!(ids.contains(&ProviderId::LocalLlm));
    }

    #[test]
    fn claude_has_anthropic_env() {
        let m = ProviderMetadata::builtin(ProviderId::Claude);
        assert_eq!(m.env_var.as_deref(), Some("ANTHROPIC_API_KEY"));
        assert!(m.requires_key);
        assert_eq!(m.kind, ProviderKind::Native);
    }

    #[test]
    fn local_llm_no_env_no_key() {
        let m = ProviderMetadata::builtin(ProviderId::LocalLlm);
        assert!(m.env_var.is_none());
        assert!(!m.requires_key);
        assert_eq!(m.kind, ProviderKind::OpenAiCompat);
    }

    #[test]
    fn deepseek_is_openai_compat() {
        let m = ProviderMetadata::builtin(ProviderId::Deepseek);
        assert_eq!(m.kind, ProviderKind::OpenAiCompat);
        assert_eq!(m.env_var.as_deref(), Some("DEEPSEEK_API_KEY"));
    }

    #[test]
    fn minimax_registered_but_minimal() {
        let m = ProviderMetadata::builtin(ProviderId::Minimax);
        assert!(m.requires_key);
        // D-50: librarian 조사 결과로 업데이트
        assert_eq!(m.base_url, "https://api.minimax.io/v1");
        assert_eq!(m.default_model, "MiniMax-M3");
        assert!(m.supports.tool_use);
        assert!(m.supports.vision);
        assert!(m.supports.thinking);
        assert!(m.supports.streaming);
        assert_eq!(m.env_var.as_deref(), Some("MINIMAX_API_KEY"));
    }

    #[test]
    fn serde_roundtrip() {
        let m = ProviderMetadata::builtin(ProviderId::Claude);
        let s = toml::to_string(&m).unwrap();
        let back: ProviderMetadata = toml::from_str(&s).unwrap();
        assert_eq!(m, back);
    }

    #[test]
    fn dispatch_by_id() {
        for id in ProviderId::ALL {
            let m = ProviderMetadata::builtin(*id);
            assert_eq!(m.id, *id);
        }
    }
}
