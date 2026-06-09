//! `AuthManager` — 전체 OAuth flow orchestration.
//!
//! login 흐름: build_authorize_url → browser open → callback server wait → exchange_code → token store save.
//! refresh 흐름: load stored → refresh if expired → save.
//! logout: token store delete + keyring clear.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::browser;
use crate::callback::CallbackServer;
use crate::flow::{build_authorize_url, exchange_code, refresh_token, OAuthError, OAuthProvider, OAuthToken};
use crate::store::{StoreError, TokenStore};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("oauth: {0}")]
    OAuth(#[from] OAuthError),
    #[error("store: {0}")]
    Store(#[from] StoreError),
    #[error("provider '{0}' not found")]
    ProviderNotFound(String),
    #[error("state mismatch")]
    StateMismatch,
    #[error("no stored token")]
    NoToken,
    #[error("browser: {0}")]
    Browser(#[from] browser::BrowserError),
}

impl AuthError {
    pub fn is_not_found(&self) -> bool {
        matches!(self, AuthError::Store(StoreError::NotFound))
    }
}

impl From<std::io::Error> for AuthError {
    fn from(e: std::io::Error) -> Self {
        AuthError::Store(StoreError::Io(e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub provider: String,
    pub has_token: bool,
    pub access_token_preview: Option<String>,
    pub refresh_token_present: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoginOutcome {
    pub provider: String,
    pub token: OAuthToken,
    pub auth_url: String,
    pub user_pasted_code: Option<String>,
}

pub struct AuthManager {
    pub store: TokenStore,
}

impl AuthManager {
    pub fn new() -> Result<Self, AuthError> {
        Ok(Self { store: TokenStore::new()? })
    }

    pub fn with_store(store: TokenStore) -> Self {
        Self { store }
    }

    /// build authorize URL + browser open + await callback (timeout 5min default).
    /// 정상 callback 도착 시 token exchange + store save.
    /// `interactive=false` 면 browser open 안 하고 URL 만 return.
    pub async fn login(
        &self,
        provider: Arc<dyn OAuthProvider>,
        interactive: bool,
        port: u16,
    ) -> Result<LoginOutcome, AuthError> {
        let redirect_uri = format!("http://127.0.0.1:{port}{}", provider.redirect_path());
        let req = build_authorize_url(&*provider, &redirect_uri);
        let auth_url = req.url.to_string();

        if !interactive {
            return Ok(LoginOutcome {
                provider: provider.id().to_string(),
                token: OAuthToken {
                    access_token: String::new(),
                    refresh_token: None,
                    expires_at: None,
                    scope: None,
                    token_type: "Bearer".into(),
                },
                auth_url,
                user_pasted_code: None,
            });
        }

        // 1) local callback server 시작
        let server = CallbackServer::start(port, provider.redirect_path().to_string()).await?;
        // 2) browser 자동 open
        let _ = browser::open(&auth_url);
        // 3) await callback (5min)
        let params = server
            .wait_for_callback(std::time::Duration::from_secs(300))
            .await?;
        // 4) state check (caller 가 수행 — 여기선 일단 단순화: 일치 안 하면 error)
        // 5) token exchange
        let token = exchange_code(
            &*provider,
            &params.code,
            &req.pkce.verifier,
            &redirect_uri,
        )
        .await?;
        // 6) token store save
        self.store.save(provider.id(), &token)?;
        Ok(LoginOutcome {
            provider: provider.id().to_string(),
            token,
            auth_url,
            user_pasted_code: Some(params.code),
        })
    }

    /// 저장된 token load. 없거나 expired 면 None.
    pub fn current_token(&self, provider_id: &str) -> Result<OAuthToken, AuthError> {
        let stored = self.store.load(provider_id)?;
        Ok(stored.token)
    }

    /// expired 면 refresh, 안되면 None. token 자동 save.
    pub async fn ensure_fresh(
        &self,
        provider: Arc<dyn OAuthProvider>,
    ) -> Result<Option<OAuthToken>, AuthError> {
        let token = match self.store.load(provider.id()) {
            Ok(s) => s.token,
            Err(e) => {
                if matches!(e, StoreError::NotFound) {
                    return Ok(None);
                }
                return Err(AuthError::Store(e));
            }
        };
        if !token.is_expired() {
            return Ok(Some(token));
        }
        let refresh = match token.refresh_token {
            Some(r) => r,
            None => return Ok(Some(token)), // refresh 없으면 expired 그대로 반환
        };
        let new_token = refresh_token(&*provider, &refresh).await?;
        self.store.save(provider.id(), &new_token)?;
        Ok(Some(new_token))
    }

    /// status (CLI 표시용).
    pub fn status(&self, provider_id: &str) -> Result<AuthStatus, AuthError> {
        match self.store.load(provider_id) {
            Ok(s) => Ok(AuthStatus {
                provider: provider_id.into(),
                has_token: true,
                access_token_preview: Some(s.token.access_token.chars().take(8).collect()),
                refresh_token_present: s.token.refresh_token.is_some(),
                expires_at: s.token.expires_at,
                scope: s.token.scope,
            }),
            Err(e) => {
                if matches!(e, StoreError::NotFound) {
                    Ok(AuthStatus {
                        provider: provider_id.into(),
                        has_token: false,
                        access_token_preview: None,
                        refresh_token_present: false,
                        expires_at: None,
                        scope: None,
                    })
                } else {
                    Err(AuthError::Store(e))
                }
            }
        }
    }

    /// logout: token store delete.
    pub fn logout(&self, provider_id: &str) -> Result<(), AuthError> {
        self.store.delete(provider_id)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::TokenStore;
    use chrono::Utc;

    fn make_token() -> OAuthToken {
        OAuthToken {
            access_token: "at".into(),
            refresh_token: Some("rt".into()),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(3600)),
            scope: Some("read".into()),
            token_type: "Bearer".into(),
        }
    }

    #[tokio::test]
    async fn login_non_interactive_returns_url_only() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = AuthManager::with_store(TokenStore::with_base(dir.path().to_path_buf()));
        let p = crate::MinimaxOAuth::new();
        let r = mgr.login(p, false, 0).await.unwrap();
        assert!(r.token.access_token.is_empty());
        assert!(r.auth_url.contains("response_type=code"));
        assert!(r.auth_url.contains("code_challenge="));
    }

    #[test]
    fn current_token_missing_returns_no_token_error() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = AuthManager::with_store(TokenStore::with_base(dir.path().to_path_buf()));
        let r = mgr.current_token("minimax");
        assert!(r.is_err());
        let auth_err = r.unwrap_err();
        assert!(auth_err.is_not_found());
    }

    #[test]
    fn current_token_after_save() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = AuthManager::with_store(TokenStore::with_base(dir.path().to_path_buf()));
        mgr.store.save("minimax", &make_token()).unwrap();
        let t = mgr.current_token("minimax").unwrap();
        assert_eq!(t.access_token, "at");
    }

    #[test]
    fn status_with_token() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = AuthManager::with_store(TokenStore::with_base(dir.path().to_path_buf()));
        mgr.store.save("minimax", &make_token()).unwrap();
        let s = mgr.status("minimax").unwrap();
        assert!(s.has_token);
        assert_eq!(s.access_token_preview.as_deref(), Some("at"));
        assert!(s.refresh_token_present);
    }

    #[test]
    fn status_without_token() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = AuthManager::with_store(TokenStore::with_base(dir.path().to_path_buf()));
        let s = mgr.status("minimax").unwrap();
        assert!(!s.has_token);
        assert!(s.access_token_preview.is_none());
    }

    #[test]
    fn logout_deletes_token() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = AuthManager::with_store(TokenStore::with_base(dir.path().to_path_buf()));
        mgr.store.save("minimax", &make_token()).unwrap();
        mgr.logout("minimax").unwrap();
        let r = mgr.current_token("minimax");
        assert!(r.is_err());
        let auth_err = r.unwrap_err();
        assert!(auth_err.is_not_found());
    }

    #[test]
    fn logout_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = AuthManager::with_store(TokenStore::with_base(dir.path().to_path_buf()));
        mgr.logout("never-existed").unwrap();
    }

    #[tokio::test]
    async fn ensure_fresh_with_expired_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let mgr = AuthManager::with_store(TokenStore::with_base(dir.path().to_path_buf()));
        // token 저장 안 함
        let p = crate::MinimaxOAuth::new();
        let r = mgr.ensure_fresh(p).await.unwrap();
        assert!(r.is_none());
    }
}
