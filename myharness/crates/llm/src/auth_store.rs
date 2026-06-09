//! `AuthStore` trait + `InMemoryAuthStore` 구현.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use thiserror::Error;

use crate::provider::ProviderId;

#[async_trait]
pub trait AuthStore: Send + Sync {
    async fn get(&self, provider: ProviderId) -> Result<Option<String>, AuthStoreError>;
    async fn set(&self, provider: ProviderId, value: &str) -> Result<(), AuthStoreError>;
    async fn clear(&self, provider: ProviderId) -> Result<(), AuthStoreError>;
    async fn list(&self) -> Result<Vec<ProviderId>, AuthStoreError>;
    fn backend_name(&self) -> &'static str;
}

#[derive(Debug, Error)]
pub enum AuthStoreError {
    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("other: {0}")]
    Other(String),
}

#[derive(Debug, Default)]
pub struct InMemoryAuthStore {
    map: Mutex<HashMap<ProviderId, String>>,
}

impl InMemoryAuthStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl AuthStore for InMemoryAuthStore {
    async fn get(&self, provider: ProviderId) -> Result<Option<String>, AuthStoreError> {
        Ok(self.map.lock().unwrap().get(&provider).cloned())
    }

    async fn set(&self, provider: ProviderId, value: &str) -> Result<(), AuthStoreError> {
        self.map.lock().unwrap().insert(provider, value.to_string());
        Ok(())
    }

    async fn clear(&self, provider: ProviderId) -> Result<(), AuthStoreError> {
        self.map.lock().unwrap().remove(&provider);
        Ok(())
    }

    async fn list(&self) -> Result<Vec<ProviderId>, AuthStoreError> {
        Ok(self.map.lock().unwrap().keys().copied().collect())
    }

    fn backend_name(&self) -> &'static str {
        "memory"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inmemory_set_get_clear_list() {
        let s = InMemoryAuthStore::new();
        assert!(s.get(ProviderId::Claude).await.unwrap().is_none());
        s.set(ProviderId::Claude, "k1").await.unwrap();
        s.set(ProviderId::Codex, "k2").await.unwrap();
        assert_eq!(s.get(ProviderId::Claude).await.unwrap().as_deref(), Some("k1"));
        let ids = s.list().await.unwrap();
        assert_eq!(ids.len(), 2);
        s.clear(ProviderId::Claude).await.unwrap();
        assert!(s.get(ProviderId::Claude).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn inmemory_overwrites() {
        let s = InMemoryAuthStore::new();
        s.set(ProviderId::Claude, "v1").await.unwrap();
        s.set(ProviderId::Claude, "v2").await.unwrap();
        assert_eq!(s.get(ProviderId::Claude).await.unwrap().as_deref(), Some("v2"));
    }

    #[tokio::test]
    async fn inmemory_backend_name() {
        let s = InMemoryAuthStore::new();
        assert_eq!(s.backend_name(), "memory");
    }

    #[tokio::test]
    async fn inmemory_is_send_sync() {
        // 컴파일 타임 검증
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<InMemoryAuthStore>();
    }
}
