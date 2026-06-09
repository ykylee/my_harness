//! OS keychain 백엔드 probe. libsecret 부재 환경에서는 graceful fallback.
//!
//! v1 단순화: backend 가용성만 probe 하고, 실제 Entry 조작은 W7.4 discover 가
//! in-memory + env-var 경로로 처리. keyring 자체 호출은 `Error::NoEntry` 와 같은
//! variant 매칭이 필요한데 keyring 4 가 main crate 에서 Error variant 를
//! re-export 하지 않아 패턴 매칭이 어려움 → backend 가용 시에도 W7.4 에서
//! in-memory + env 경로 우선, keyring 은 1차 fallback 으로 사용.

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

pub struct KeyringAuthStore {
    backend: KeyringBackend,
    service_prefix: String,
}

impl KeyringAuthStore {
    pub fn probe() -> Self {
        Self {
            backend: detect_backend(),
            service_prefix: "myharness".into(),
        }
    }

    pub fn backend(&self) -> KeyringBackend {
        self.backend
    }

    #[allow(dead_code)]
    fn service_name(&self, provider: ProviderId) -> String {
        format!("{}:{}", self.service_prefix, provider)
    }
}

#[async_trait::async_trait]
impl AuthStore for KeyringAuthStore {
    async fn get(&self, _provider: ProviderId) -> Result<Option<String>, AuthStoreError> {
        match self.backend {
            KeyringBackend::None => Err(AuthStoreError::BackendUnavailable(
                "no keyring backend available (install libsecret-1-dev + gnome-keyring or set env var)".into(),
            )),
            _ => Ok(None),
        }
    }

    async fn set(&self, _provider: ProviderId, _value: &str) -> Result<(), AuthStoreError> {
        match self.backend {
            KeyringBackend::None => Err(AuthStoreError::BackendUnavailable(
                "no keyring backend available".into(),
            )),
            _ => Ok(()),
        }
    }

    async fn clear(&self, _provider: ProviderId) -> Result<(), AuthStoreError> {
        match self.backend {
            KeyringBackend::None => Err(AuthStoreError::BackendUnavailable(
                "no keyring backend available".into(),
            )),
            _ => Ok(()),
        }
    }

    async fn list(&self) -> Result<Vec<ProviderId>, AuthStoreError> {
        match self.backend {
            KeyringBackend::None => Err(AuthStoreError::BackendUnavailable(
                "no keyring backend available".into(),
            )),
            _ => Ok(Vec::new()),
        }
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
            assert!(s.set(ProviderId::Claude, "v").await.is_err());
            assert!(s.clear(ProviderId::Claude).await.is_err());
        } else {
            s.set(ProviderId::Claude, "v").await.unwrap();
            s.clear(ProviderId::Claude).await.unwrap();
        }
    }
}
