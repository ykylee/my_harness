//! per-provider auth 상태 (Unset/EnvVar/Keychain/Manual/LocalDetected/Error) +
//! 영속화 (per-provider TOML at `~/.myharness/state/auth/<provider>.toml`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::provider::ProviderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthState {
    Unset,
    EnvVar,
    Keychain,
    Manual,
    LocalDetected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub provider: ProviderId,
    pub state: AuthState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_checked: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_prefix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_test_latency_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_test_ok: Option<bool>,
}

impl AuthStatus {
    pub fn new(provider: ProviderId) -> Self {
        Self {
            provider,
            state: AuthState::Unset,
            last_checked: None,
            error_message: None,
            key_prefix: None,
            source_detail: None,
            last_test_latency_ms: None,
            last_test_ok: None,
        }
    }

    /// 호출 가능한 상태인지 (Unset/Error 가 아니면 true).
    pub fn is_usable(&self) -> bool {
        !matches!(self.state, AuthState::Unset | AuthState::Error)
    }

    /// prefix 만 남기고 나머지 마스킹 (operator-visible 진단용).
    pub fn redact_key(key: &str) -> String {
        if key.len() <= 7 {
            return "***".into();
        }
        let mut s = String::with_capacity(10);
        s.push_str(&key[..7]);
        s.push('…');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_state_serde_kebab_case() {
        let s = serde_json::to_string(&AuthState::LocalDetected).unwrap();
        assert_eq!(s, "\"local-detected\"");
        let back: AuthState = serde_json::from_str("\"local-detected\"").unwrap();
        assert_eq!(back, AuthState::LocalDetected);
    }

    #[test]
    fn auth_status_is_usable_for_each_state() {
        for s in [
            AuthState::Unset,
            AuthState::EnvVar,
            AuthState::Keychain,
            AuthState::Manual,
            AuthState::LocalDetected,
            AuthState::Error,
        ] {
            let status = AuthStatus { state: s, ..AuthStatus::new(ProviderId::Claude) };
            let expected = !matches!(s, AuthState::Unset | AuthState::Error);
            assert_eq!(status.is_usable(), expected, "state: {s:?}");
        }
    }

    #[test]
    fn auth_status_serde_roundtrip() {
        let s = AuthStatus {
            provider: ProviderId::Claude,
            state: AuthState::EnvVar,
            last_checked: Some(Utc::now()),
            error_message: None,
            key_prefix: Some("sk-ant-…".into()),
            source_detail: Some("env:ANTHROPIC_API_KEY".into()),
            last_test_latency_ms: Some(123),
            last_test_ok: Some(true),
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: AuthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.state, AuthState::EnvVar);
        assert_eq!(back.key_prefix.as_deref(), Some("sk-ant-…"));
    }

    #[test]
    fn auth_status_last_checked_optional() {
        let s = AuthStatus::new(ProviderId::Codex);
        let json = serde_json::to_string(&s).unwrap();
        let back: AuthStatus = serde_json::from_str(&json).unwrap();
        assert!(back.last_checked.is_none());
        assert!(back.error_message.is_none());
    }

    #[test]
    fn key_prefix_redaction() {
        assert_eq!(AuthStatus::redact_key("sk-1234567890abcdef"), "sk-1234…");
        assert_eq!(AuthStatus::redact_key("short"), "***");
        assert_eq!(AuthStatus::redact_key(""), "***");
    }

    #[test]
    fn auth_state_unset_to_env_var_transition() {
        let mut s = AuthStatus::new(ProviderId::Claude);
        assert_eq!(s.state, AuthState::Unset);
        assert!(!s.is_usable());
        s.state = AuthState::EnvVar;
        assert!(s.is_usable());
    }

    #[test]
    fn auth_state_env_var_to_error_on_test_fail() {
        let mut s = AuthStatus::new(ProviderId::Claude);
        s.state = AuthState::EnvVar;
        assert!(s.is_usable());
        s.state = AuthState::Error;
        s.error_message = Some("401 unauthorized".into());
        assert!(!s.is_usable());
    }
}
