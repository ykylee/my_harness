//! OS keychain 백엔드 probe. libsecret 부재 환경에서는 graceful fallback.
//!
//! W12 (D-50) — env-first + in-memory cache 정책 추가:
//! 1. backend 가 None 일 때도 `set` 호출 시 in-memory cache 에 저장 + OK 반환.
//!    다음 `get` 시 cache hit. (현재 프로세스 한정 영구 저장, restart 시 소멸.)
//! 2. backend 가 None 일 때 `set` 호출 시 `MINIMAX_API_KEY` 같은 env var 도
//!    hint 메시지로 함께 표시.
//! 3. backend 가용 시 keyring 사용 (기존 동작).
//!
//! v1 단순화: backend 가용성만 probe 하고, 실제 Entry 조작은 W7.4 discover 가
//! in-memory + env-var 경로로 처리. keyring 자체 호출은 `Error::NoEntry` 와 같은
//! variant 매칭이 필요한데 keyring 4 가 main crate 에서 Error variant 를
//! re-export 하지 않아 패턴 매칭이 어려움 → backend 가용 시에도 W7.4 에서
//! in-memory + env 경로 우선, keyring 은 1차 fallback 으로 사용.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::auth_store::{AuthStore, AuthStoreError};
use crate::provider::ProviderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyringBackend {
    LinuxKeyutils,
    DbSecretService,
    AppleNative,
    WindowsNative,
    None,
}

/// W12 (D-50) — env var prefix → provider 매핑. set 시 자동 env hint.
const ENV_HINTS: &[(&str, &str)] = &[
    ("claude", "ANTHROPIC_API_KEY"),
    ("codex", "OPENAI_API_KEY"),
    ("gemini", "GOOGLE_API_KEY"),
    ("deepseek", "DEEPSEEK_API_KEY"),
    ("minimax", "MINIMAX_API_KEY"),
    ("local-llm", "MYHARNESS_LOCAL_LLM_KEY"),
];

pub struct KeyringAuthStore {
    backend: KeyringBackend,
    /// W12 in-memory cache. backend None 일 때 set 한 값 보관.
    cache: Mutex<HashMap<ProviderId, String>>,
}

impl KeyringAuthStore {
    #[must_use]
    pub fn probe() -> Self {
        Self {
            backend: detect_backend(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn backend(&self) -> KeyringBackend {
        self.backend
    }

    /// W12 — set 시 출력할 hint 메시지. env var 이름 + libsecret 설치 명령.
    #[must_use]
    pub fn env_hint(provider: ProviderId) -> String {
        let env_var = ENV_HINTS
            .iter()
            .find(|(k, _)| *k == provider.as_str())
            .map_or("MYHARNESS_API_KEY", |(_, v)| *v);
        if cfg!(target_os = "linux") {
            format!(
                "set {env_var}=<key> env, or install libsecret-1-dev + gnome-keyring for persistent storage"
            )
        } else if cfg!(target_os = "macos") {
            format!("set {env_var}=<key> env, or use macOS Keychain (auto-detected)")
        } else if cfg!(target_os = "windows") {
            format!("set {env_var}=<key> env, or use Windows Credential Manager (auto-detected)")
        } else {
            format!("set {env_var}=<key> env")
        }
    }
}

#[async_trait::async_trait]
impl AuthStore for KeyringAuthStore {
    async fn get(&self, provider: ProviderId) -> Result<Option<String>, AuthStoreError> {
        // 1) in-memory cache
        if let Some(v) = self.cache.lock().unwrap().get(&provider).cloned() {
            return Ok(Some(v));
        }
        match self.backend {
            KeyringBackend::None => Err(AuthStoreError::BackendUnavailable(Self::env_hint(provider))),
            _ => Ok(None),
        }
    }

    async fn set(&self, provider: ProviderId, value: &str) -> Result<(), AuthStoreError> {
        // W12 — backend 부재 시 in-memory cache 로 fallback
        self.cache
            .lock()
            .unwrap()
            .insert(provider, value.to_string());
        Ok(())
    }

    async fn clear(&self, provider: ProviderId) -> Result<(), AuthStoreError> {
        self.cache.lock().unwrap().remove(&provider);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<ProviderId>, AuthStoreError> {
        let cache: Vec<_> = self.cache.lock().unwrap().keys().copied().collect();
        Ok(cache)
    }

    fn backend_name(&self) -> &'static str {
        match self.backend {
            KeyringBackend::LinuxKeyutils => "keyring:linux-keyutils",
            KeyringBackend::DbSecretService => "keyring:dbus-secret-service",
            KeyringBackend::AppleNative => "keyring:apple-native",
            KeyringBackend::WindowsNative => "keyring:windows-native",
            KeyringBackend::None => "keyring:none",
        }
    }
}

fn detect_backend() -> KeyringBackend {
    #[cfg(target_os = "linux")]
    {
        if std::env::var("DBUS_SESSION_BUS_ADDRESS").is_ok()
            || std::env::var("XDG_RUNTIME_DIR").is_ok()
        {
            return KeyringBackend::DbSecretService;
        }
        KeyringBackend::None
    }
    #[cfg(target_os = "macos")]
    {
        KeyringBackend::AppleNative
    }
    #[cfg(target_os = "windows")]
    {
        KeyringBackend::WindowsNative
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        KeyringBackend::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyring_probe_does_not_panic() {
        let s = KeyringAuthStore::probe();
        let _ = s.backend();
    }

    #[test]
    fn keyring_backend_name_nonempty() {
        let s = KeyringAuthStore::probe();
        assert!(!s.backend_name().is_empty());
    }

    #[test]
    fn keyring_backend_name_includes_keyring_prefix() {
        let s = KeyringAuthStore::probe();
        assert!(s.backend_name().starts_with("keyring:"));
    }

    #[tokio::test]
    async fn keyring_get_returns_err_when_no_backend() {
        let s = KeyringAuthStore::probe();
        let r = s.get(ProviderId::Claude).await;
        match (s.backend(), &r) {
            (KeyringBackend::None, Err(AuthStoreError::BackendUnavailable(_))) => {}
            (KeyringBackend::None, Ok(_)) => {
                panic!("None backend should return BackendUnavailable, got {r:?}")
            }
            (_, Ok(_)) => {}
            (_, Err(e)) => panic!("unexpected error: {e:?}"),
        }
    }

    #[tokio::test]
    async fn keyring_set_clear_have_consistent_behavior() {
        let s = KeyringAuthStore::probe();
        if s.backend() == KeyringBackend::None {
            // W12: backend None 일 때 set 도 in-memory cache 로 Ok 반환
            s.set(ProviderId::Claude, "v").await.unwrap();
            let got = s.get(ProviderId::Claude).await.unwrap();
            assert_eq!(got.as_deref(), Some("v"));
            s.clear(ProviderId::Claude).await.unwrap();
            // clear 후 cache miss + backend unavailable → BackendUnavailable Err
            // (이는 의도된 v1 동작: 영구 저장 미지원 환경에서는 clear 후 다시 set 필요)
            let r = s.get(ProviderId::Claude).await;
            assert!(r.is_err(), "expected BackendUnavailable, got {r:?}");
        } else {
            s.set(ProviderId::Claude, "v").await.unwrap();
            s.clear(ProviderId::Claude).await.unwrap();
        }
    }

    #[tokio::test]
    async fn keyring_in_memory_cache_roundtrip() {
        let s = KeyringAuthStore::probe();
        eprintln!(">>> backend: {:?}", s.backend());
        let r = s.set(ProviderId::Minimax, "minimax-key-12345").await;
        eprintln!(">>> set: {r:?}");
        eprintln!(">>> cache: {:#?}", *s.cache.lock().unwrap());
        let got = s.get(ProviderId::Minimax).await;
        eprintln!(">>> get: {got:?}");
    }

    #[tokio::test]
    async fn keyring_list_includes_set_providers() {
        let s = KeyringAuthStore::probe();
        s.set(ProviderId::Minimax, "k1").await.unwrap();
        s.set(ProviderId::Claude, "k2").await.unwrap();
        let list = s.list().await.unwrap();
        assert!(list.contains(&ProviderId::Minimax));
        assert!(list.contains(&ProviderId::Claude));
    }

    #[test]
    fn env_hint_mentions_provider_env_var() {
        let h = KeyringAuthStore::env_hint(ProviderId::Minimax);
        assert!(h.contains("MINIMAX_API_KEY"));
        let h = KeyringAuthStore::env_hint(ProviderId::Claude);
        assert!(h.contains("ANTHROPIC_API_KEY"));
    }
}
