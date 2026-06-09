//! 3 OAuth provider 구현 (W13.2).
//!
//! - MiniMax: api.minimax.io/oauth/authorize, OpenAI-style. PKCE public client.
//! - OpenAI: auth.openai.com/oauth/authorize, OpenAI Platform. PKCE public client.
//! - Google: accounts.google.com/o/oauth2/v2/auth (Gemini 용). PKCE public client.

use std::sync::Arc;

use async_trait::async_trait;

use crate::flow::OAuthProvider;

/// 빌드 시 OAuth provider 등록. CLI/사용자가 client_id 를 override 하려면
/// env var `MYHARNESS_OAUTH_CLIENT_ID_<PROVIDER>` 사용.
/// W13.5 (D-52) — env 매 호출마다 재읽기. `oauth_providers()` 가 매번 새 instance.
pub struct MinimaxOAuth {
    pub client_id: String,
    pub base_host: String,
}

impl MinimaxOAuth {
    /// env var 매번 재읽기. instance 새로 생성하여 env 변경 즉시 반영.
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("MYHARNESS_OAUTH_CLIENT_ID_MINIMAX")
                .unwrap_or_else(|_| "myharness-cli".into()),
            base_host: std::env::var("MINIMAX_API_HOST")
                .unwrap_or_else(|_| "https://api.minimax.io".into()),
        }
    }
}

impl Default for MinimaxOAuth {
    fn default() -> Self {
        Self::from_env()
    }
}

impl MinimaxOAuth {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::from_env())
    }
}

#[async_trait]
impl OAuthProvider for MinimaxOAuth {
    fn id(&self) -> &'static str { "minimax" }
    fn display_name(&self) -> &'static str { "MiniMax" }
    fn authorize_endpoint(&self) -> &str {
        if self.base_host == "https://api.minimax.io" {
            "https://api.minimax.io/oauth/authorize"
        } else {
            // CN endpoint (또는 override) — base_host 기반
            // &str 반환이지만 String 의 self.base_host 와 lifetime 동일
            // → 안전하지 않음. unsafe transmute 회피: Box::leak 패턴 또는 &'static str 캐시.
            // W13.5 단순화: 글로벌 endpoint 만 사용. CN 은 별도 env MYHARNESS_MINIMAX_CN=1
            "https://api.minimax.io/oauth/authorize"
        }
    }
    fn token_endpoint(&self) -> &str { "https://api.minimax.io/oauth/token" }
    fn client_id(&self) -> &str { &self.client_id }
    fn client_secret(&self) -> Option<&str> { None }
    fn default_scopes(&self) -> &[&str] { &["completions.read", "completions.write"] }
    fn extra_authorize_params(&self) -> Vec<(&str, String)> {
        vec![("response_type", "code".into())]
    }
}

pub struct OpenAiOAuth {
    pub client_id: String,
}

impl OpenAiOAuth {
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("MYHARNESS_OAUTH_CLIENT_ID_OPENAI")
                .unwrap_or_else(|_| "myharness-cli".into()),
        }
    }
    pub fn new() -> Arc<Self> { Arc::new(Self::from_env()) }
}

impl Default for OpenAiOAuth {
    fn default() -> Self { Self::from_env() }
}

#[async_trait]
impl OAuthProvider for OpenAiOAuth {
    fn id(&self) -> &'static str { "openai" }
    fn display_name(&self) -> &'static str { "OpenAI" }
    fn authorize_endpoint(&self) -> &str { "https://auth.openai.com/oauth/authorize" }
    fn token_endpoint(&self) -> &str { "https://auth.openai.com/oauth/token" }
    fn client_id(&self) -> &str { &self.client_id }
    fn client_secret(&self) -> Option<&str> { None }
    fn default_scopes(&self) -> &[&str] { &["openid", "profile", "email", "offline_access"] }
}

pub struct GoogleOAuth {
    pub client_id: String,
}

impl GoogleOAuth {
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("MYHARNESS_OAUTH_CLIENT_ID_GOOGLE")
                .unwrap_or_else(|_| "myharness-cli".into()),
        }
    }
    pub fn new() -> Arc<Self> { Arc::new(Self::from_env()) }
}

impl Default for GoogleOAuth {
    fn default() -> Self { Self::from_env() }
}

#[async_trait]
impl OAuthProvider for GoogleOAuth {
    fn id(&self) -> &'static str { "google" }
    fn display_name(&self) -> &'static str { "Google (Gemini)" }
    fn authorize_endpoint(&self) -> &str { "https://accounts.google.com/o/oauth2/v2/auth" }
    fn token_endpoint(&self) -> &str { "https://oauth2.googleapis.com/token" }
    fn client_id(&self) -> &str { &self.client_id }
    fn client_secret(&self) -> Option<&str> { None }
    fn default_scopes(&self) -> &[&str] {
        &["openid", "https://www.googleapis.com/auth/userinfo.email", "https://www.googleapis.com/auth/cloud-platform"]
    }
    fn extra_authorize_params(&self) -> Vec<(&str, String)> {
        vec![
            ("access_type", "offline".into()),
            ("prompt", "consent".into()),
        ]
    }
}

/// 등록된 3 provider.
pub fn oauth_providers() -> Vec<Arc<dyn OAuthProvider>> {
    vec![
        MinimaxOAuth::new(),
        OpenAiOAuth::new(),
        GoogleOAuth::new(),
    ]
}

pub static OAUTH_PROVIDERS: std::sync::LazyLock<Vec<std::sync::Arc<dyn OAuthProvider>>> =
    std::sync::LazyLock::new(oauth_providers);

pub fn find_provider(id: &str) -> Option<Arc<dyn OAuthProvider>> {
    for p in OAUTH_PROVIDERS.iter() {
        if p.id() == id {
            return Some(Arc::clone(p));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_providers_list_has_three() {
        assert_eq!(OAUTH_PROVIDERS.len(), 3);
    }

    #[test]
    fn find_provider_by_id() {
        let p = find_provider("minimax").unwrap();
        assert_eq!(p.id(), "minimax");
        assert_eq!(p.display_name(), "MiniMax");
    }

    #[test]
    fn find_provider_unknown_returns_none() {
        assert!(find_provider("nonexistent").is_none());
    }

    #[test]
    fn minimax_authorize_endpoint_https() {
        let p = MinimaxOAuth::default();
        assert!(p.authorize_endpoint().starts_with("https://"));
        assert!(p.token_endpoint().starts_with("https://"));
    }

    #[test]
    fn openai_default_scopes_include_offline() {
        let p = OpenAiOAuth::default();
        let s = p.default_scopes();
        assert!(s.contains(&"offline_access"));
    }

    #[test]
    fn google_extra_params_offline() {
        let p = GoogleOAuth::default();
        let extras = p.extra_authorize_params();
        let mut has_offline = false;
        let mut has_consent = false;
        for (k, _) in &extras {
            if *k == "access_type" { has_offline = true; }
            if *k == "prompt" { has_consent = true; }
        }
        assert!(has_offline);
        assert!(has_consent);
    }
}
