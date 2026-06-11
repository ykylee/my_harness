//! OAuth 2.0 Authorization Code + PKCE flow (provider-agnostic core).
//!
//! 흐름:

#![allow(clippy::needless_lifetimes)] // explicit lifetimes are clearer for these OAuth parameter types
//! 1) `build_authorize_url(provider, redirect_uri, scopes)` → browser URL
//! 2) `start_callback_server(port, expected_state)` → await redirect
//! 3) user 가 provider 로그인 → provider 가 `redirect_uri?code=...&state=...` 로 redirect
//! 4) `exchange_code(provider, code, pkce_verifier, redirect_uri)` → access token + refresh token
//! 5) `refresh(provider, refresh_token)` → 새 access token
//!
//! 각 provider 가 `OAuthProvider` trait 으로 endpoint + `client_id` 채움.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::pkce::{generate_pkce, generate_state, PkcePair};

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("url: {0}")]
    Url(#[from] url::ParseError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("callback server: {0}")]
    CallbackServer(String),
    #[error("state mismatch (CSRF)")]
    StateMismatch,
    #[error("provider error: {0}")]
    Provider(String),
    #[error("no auth code in callback")]
    NoAuthCode,
}

/// OAuth access + refresh token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub scope: Option<String>,
    pub token_type: String,
}

impl OAuthToken {
    #[must_use]
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(t) => chrono::Utc::now() >= t,
            None => false,
        }
    }
}

/// Authorization URL 구성 결과.
#[derive(Debug, Clone)]
pub struct AuthorizeRequest {
    pub url: Url,
    pub state: String,
    pub pkce: PkcePair,
    pub redirect_uri: String,
}

/// callback query string.
#[derive(Debug, Clone)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

/// provider-agnostic OAuth endpoint 정의.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// provider 식별자 (token file 이름용).
    fn id(&self) -> &'static str;
    /// 표시 이름.
    fn display_name(&self) -> &'static str;
    /// authorize endpoint base URL.
    fn authorize_endpoint(&self) -> &str;
    /// token endpoint base URL.
    fn token_endpoint(&self) -> &str;
    /// OAuth `client_id`.
    fn client_id(&self) -> &str;
    /// OAuth `client_secret` (없으면 None for PKCE public client).
    fn client_secret(&self) -> Option<&str>;
    /// 기본 scope (공백 구분).
    fn default_scopes(&self) -> &[&str];
    /// redirect path (default "/callback").
    fn redirect_path(&self) -> &'static str {
        "/callback"
    }
    /// authorize URL 의 추가 parameters (e.g. audience, prompt).
    fn extra_authorize_params(&self) -> Vec<(&str, String)> {
        vec![]
    }
    /// token exchange 의 추가 parameters.
    fn extra_token_params(&self) -> Vec<(&str, String)> {
        vec![]
    }
}

/// 전체 authorize URL 생성.
///
/// # Panics
/// provider 의 `authorize_endpoint` 가 올바른 URL 형식이 아니면 panic 발생합니다.
pub fn build_authorize_url(
    provider: &dyn OAuthProvider,
    redirect_uri: &str,
) -> AuthorizeRequest {
    let state = generate_state();
    let pkce = generate_pkce();
    let mut url = Url::parse(provider.authorize_endpoint()).expect("invalid authorize endpoint");
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("response_type", "code");
        q.append_pair("client_id", provider.client_id());
        q.append_pair("redirect_uri", redirect_uri);
        q.append_pair("state", &state);
        q.append_pair("code_challenge", &pkce.challenge);
        q.append_pair("code_challenge_method", pkce.method.as_str());
        let scope = provider.default_scopes().join(" ");
        if !scope.is_empty() {
            q.append_pair("scope", &scope);
        }
        for (k, v) in provider.extra_authorize_params() {
            q.append_pair(k, &v);
        }
    }
    AuthorizeRequest { url, state, pkce, redirect_uri: redirect_uri.to_string() }
}

/// authorization code + PKCE verifier → access token.
///
/// # Errors
/// HTTP 요청 실패, 비정상 응답 (4xx/5xx), 또는 JSON 역직렬화 실패 시 `OAuthError` 를 반환합니다.
pub async fn exchange_code(
    provider: &dyn OAuthProvider,
    code: &str,
    pkce_verifier: &str,
    redirect_uri: &str,
) -> Result<OAuthToken, OAuthError> {
    let client = reqwest::Client::new();
    let mut form = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("redirect_uri", redirect_uri.to_string()),
        ("client_id", provider.client_id().to_string()),
        ("code_verifier", pkce_verifier.to_string()),
    ];
    if let Some(secret) = provider.client_secret() {
        form.push(("client_secret", secret.to_string()));
    }
    for (k, v) in provider.extra_token_params() {
        form.push((k, v));
    }
    let token_endpoint = format!("{}{}", provider.token_endpoint(), "");
    let resp = client.post(&token_endpoint).form(&form).send().await?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await?;
    if !status.is_success() {
        return Err(OAuthError::Provider(format!("token exchange failed: {body}")));
    }
    parse_token_response(&body)
}

/// refresh token 으로 새 access token 발급.
///
/// # Errors
/// HTTP 요청 실패, 비정상 응답 (4xx/5xx), 또는 JSON 역직렬화 실패 시 `OAuthError` 를 반환합니다.
pub async fn refresh_token(
    provider: &dyn OAuthProvider,
    refresh: &str,
) -> Result<OAuthToken, OAuthError> {
    let client = reqwest::Client::new();
    let mut form = vec![
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh.to_string()),
        ("client_id", provider.client_id().to_string()),
    ];
    if let Some(secret) = provider.client_secret() {
        form.push(("client_secret", secret.to_string()));
    }
    for (k, v) in provider.extra_token_params() {
        form.push((k, v));
    }
    let resp = client.post(provider.token_endpoint()).form(&form).send().await?;
    let status = resp.status();
    let body: serde_json::Value = resp.json().await?;
    if !status.is_success() {
        return Err(OAuthError::Provider(format!("refresh failed: {body}")));
    }
    parse_token_response(&body)
}

fn parse_token_response(body: &serde_json::Value) -> Result<OAuthToken, OAuthError> {
    let access_token = body
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OAuthError::Provider("no access_token in response".into()))?
        .to_string();
    let refresh_token = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string);
    let expires_in = body.get("expires_in").and_then(serde_json::Value::as_u64);
    #[allow(clippy::cast_possible_wrap)]
    let expires_at = expires_in.map(|s| chrono::Utc::now() + chrono::Duration::seconds(s as i64));
    let scope = body
        .get("scope")
        .and_then(|v| v.as_str())
        .map(std::string::ToString::to_string);
    let token_type = body
        .get("token_type")
        .and_then(|v| v.as_str())
        .unwrap_or("Bearer")
        .to_string();
    Ok(OAuthToken {
        access_token,
        refresh_token,
        expires_at,
        scope,
        token_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider;
    impl OAuthProvider for MockProvider {
        fn id(&self) -> &'static str { "mock" }
        fn display_name(&self) -> &'static str { "Mock Provider" }
        fn authorize_endpoint(&self) -> &'static str { "https://example.com/authorize" }
        fn token_endpoint(&self) -> &'static str { "https://example.com/token" }
        fn client_id(&self) -> &'static str { "client-123" }
        fn client_secret(&self) -> Option<&str> { Some("secret-456") }
        fn default_scopes(&self) -> &[&str] { &["read", "write"] }
    }

    #[test]
    fn build_authorize_url_includes_pkce_and_state() {
        let p = MockProvider;
        let req = build_authorize_url(&p, "http://localhost:8085/callback");
        assert!(req.url.as_str().contains("response_type=code"));
        assert!(req.url.as_str().contains("client_id=client-123"));
        assert!(req.url.as_str().contains("code_challenge="));
        assert!(req.url.as_str().contains("code_challenge_method=S256"));
        assert!(req.url.as_str().contains("scope=read+write")); // URL encoding
        assert_eq!(req.state.len(), 22);
    }

    #[test]
    fn build_authorize_url_without_secret_works() {
        struct NoSecret;
        impl OAuthProvider for NoSecret {
            fn id(&self) -> &'static str { "ns" }
            fn display_name(&self) -> &'static str { "No Secret" }
            fn authorize_endpoint(&self) -> &'static str { "https://example.com/authorize" }
            fn token_endpoint(&self) -> &'static str { "https://example.com/token" }
            fn client_id(&self) -> &'static str { "client-789" }
            fn client_secret(&self) -> Option<&str> { None }
            fn default_scopes(&self) -> &[&str] { &[] }
        }
        let p = NoSecret;
        let req = build_authorize_url(&p, "http://localhost/cb");
        assert!(req.url.as_str().contains("client_id=client-789"));
    }

    #[test]
    fn parse_token_response_basic() {
        let body = serde_json::json!({
            "access_token": "at-123",
            "refresh_token": "rt-456",
            "expires_in": 3600,
            "scope": "read write",
            "token_type": "Bearer"
        });
        let t = parse_token_response(&body).unwrap();
        assert_eq!(t.access_token, "at-123");
        assert_eq!(t.refresh_token.as_deref(), Some("rt-456"));
        assert_eq!(t.token_type, "Bearer");
        assert_eq!(t.scope.as_deref(), Some("read write"));
        assert!(!t.is_expired());
    }

    #[test]
    fn parse_token_response_no_expiry() {
        let body = serde_json::json!({"access_token": "at"});
        let t = parse_token_response(&body).unwrap();
        assert!(!t.is_expired());
    }

    #[test]
    fn parse_token_response_missing_access_token_errors() {
        let body = serde_json::json!({"refresh_token": "rt"});
        assert!(matches!(parse_token_response(&body), Err(OAuthError::Provider(_))));
    }
}
