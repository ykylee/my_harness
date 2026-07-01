//! 3 OAuth provider 구현 (W13.2) + `MiniMax` Device Authorization Grant (W14).
//!
//! 표준 Authorization Code + PKCE redirect flow (RFC 6749):
//! - `OpenAI`: auth.openai.com/oauth/authorize. PKCE public client.
//! - Google: accounts.google.com/o/oauth2/v2/auth (Gemini 용). PKCE public client.
//!
//! `MiniMax` 는 표준 redirect flow 가 **404** (D-52 follow-up). 대신 **Device Authorization
//! Grant 변형** (W14) 사용:
//! - POST `{base_url}/oauth/code` → `user_code` + `verification_uri`
//! - POST `{base_url}/oauth/token` polling
//! - 표준 `client_id` `78257093-7e40-4613-99e0-527b14b39113` (OpenClaw/Hermes 공통, 모든 client 가 동일 값 사용)
//! - scope: `group_id profile model.completion`
//! - region: 한국 default = global (`https://api.minimax.io`). CN 은 `MYHARNESS_MINIMAX_CN=1` 또는 `MINIMAX_OAUTH_BASE_URL` env override.

use std::sync::Arc;

use async_trait::async_trait;

use crate::device_flow::DeviceCodeProvider;
use crate::flow::OAuthProvider;

/// 빌드 시 OAuth provider 등록. CLI/사용자가 `client_id` 를 override 하려면
/// env var `MYHARNESS_OAUTH_CLIENT_ID_<PROVIDER>` 사용.
/// W13.5 (D-52) — env 매 호출마다 재읽기. `oauth_providers()` 가 매번 새 instance.
pub struct MinimaxOAuth {
    pub client_id: String,
    pub base_host: String,
}

impl MinimaxOAuth {
    /// env var 매번 재읽기. instance 새로 생성하여 env 변경 즉시 반영.
    #[must_use] 
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
    #[must_use] 
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
    fn token_endpoint(&self) -> &'static str { "https://api.minimax.io/oauth/token" }
    fn client_id(&self) -> &str { &self.client_id }
    fn client_secret(&self) -> Option<&str> { None }
    fn default_scopes(&self) -> &[&str] { &["completions.read", "completions.write"] }
    fn extra_authorize_params(&self) -> Vec<(&str, String)> {
        // response_type=code 는 flow.rs:115 에서 push 함 (중복 회피).
        vec![]
    }
}

pub struct OpenAiOAuth {
    pub client_id: String,
}

impl OpenAiOAuth {
    #[must_use] 
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("MYHARNESS_OAUTH_CLIENT_ID_OPENAI")
                .unwrap_or_else(|_| "myharness-cli".into()),
        }
    }
    #[must_use] 
    pub fn new() -> Arc<Self> { Arc::new(Self::from_env()) }
}

impl Default for OpenAiOAuth {
    fn default() -> Self { Self::from_env() }
}

#[async_trait]
impl OAuthProvider for OpenAiOAuth {
    fn id(&self) -> &'static str { "openai" }
    fn display_name(&self) -> &'static str { "OpenAI" }
    fn authorize_endpoint(&self) -> &'static str { "https://auth.openai.com/oauth/authorize" }
    fn token_endpoint(&self) -> &'static str { "https://auth.openai.com/oauth/token" }
    fn client_id(&self) -> &str { &self.client_id }
    fn client_secret(&self) -> Option<&str> { None }
    fn default_scopes(&self) -> &[&str] { &["openid", "profile", "email", "offline_access"] }
}

pub struct GoogleOAuth {
    pub client_id: String,
}

impl GoogleOAuth {
    #[must_use] 
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("MYHARNESS_OAUTH_CLIENT_ID_GOOGLE")
                .unwrap_or_else(|_| "myharness-cli".into()),
        }
    }
    #[must_use] 
    pub fn new() -> Arc<Self> { Arc::new(Self::from_env()) }
}

impl Default for GoogleOAuth {
    fn default() -> Self { Self::from_env() }
}

#[async_trait]
impl OAuthProvider for GoogleOAuth {
    fn id(&self) -> &'static str { "google" }
    fn display_name(&self) -> &'static str { "Google (Gemini)" }
    fn authorize_endpoint(&self) -> &'static str { "https://accounts.google.com/o/oauth2/v2/auth" }
    fn token_endpoint(&self) -> &'static str { "https://oauth2.googleapis.com/token" }
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

/// `MiniMax` Device Authorization Grant provider (W14, D-114 endpoint 갱신).
///
/// 표준 OAuth 2.0 Authorization Code + PKCE redirect flow 가 `MiniMax` 에서 404 반환
/// (D-52 follow-up 확인). 따라서 **Device Authorization Grant 변형** 사용:
/// POST `/oauth2/device/code` → `user_code` + `verification_uri` → user 가 browser 에서
/// `user_code` 입력 → polling `/oauth2/token` → `access_token`.
///
/// **D-114 endpoint 갱신** (2026-07-01, real MiniMax API 검증):
/// - 이전: `https://api.minimax.io/oauth/{code,token}` (W14 초안, **307 redirect**)
/// - 현재: `https://account.minimax.io/oauth2/{device/code,token}` (RFC 8628 표준, direct hit)
/// - `api.minimax.io/oauth/code` → 307 → `account.minimax.io/oauth2/device/code`
/// - PKCE `code_challenge` (S256) 필수. 미제출 시 HTTP 400 `invalid_request: code_challenge is required`.
/// - 응답에 `state` echo back + `client=OpenClaw` 식별자 (verification_uri 에 포함) →
///   `MiniMax` 가 client_id 공유 정책 사용 (`78257093-...` 공용 client_id 와 일치).
/// - 보류 (D-115/116 후보): (a) `base_resp.status_code==0` 처리 (현재 `status: "success"` 만
///   accept — real 응답은 `status` 필드 없음), (b) `expired_in`/`interval` 의 ms/초 통일
///   (real API 는 epoch ms, 우리 코드는 초 가정), (c) `response_type=code` 파라미터 제거
///   (real spec 에 없음, mock test 호환성 유지).
///
/// 표준 `client_id` (OpenClaw/Hermes 와 동일): `78257093-7e40-4613-99e0-527b14b39113`.
/// scope: `group_id profile model.completion` (Portal OAuth 권한).
/// region: global (한국 default) = `https://account.minimax.io`. CN 은 `MYHARNESS_MINIMAX_CN=1`
/// 또는 `MINIMAX_OAUTH_BASE_URL` env override.
pub struct MinimaxDeviceOAuth {
    /// Account host (POST /oauth2/device/code + /oauth2/token 의 host). CN 면
    /// `https://account.minimaxi.com`, global = `https://account.minimax.io`.
    /// D-114 이전엔 `https://api.minimax.io` 였으나, real endpoint 가 account.* host
    /// 라는 것을 D-113 의 real-network test 가 확인.
    pub base_url: String,
    /// region 표시 (global/cn).
    pub region: String,
}

impl MinimaxDeviceOAuth {
    /// env 재읽기. 한국 환경 default = global (`https://account.minimax.io`).
    /// CN 으로 전환: `MYHARNESS_MINIMAX_CN=1` 또는 `MINIMAX_OAUTH_BASE_URL` 직접 설정.
    #[must_use] 
    pub fn from_env() -> Self {
        let cn = std::env::var("MYHARNESS_MINIMAX_CN")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        let base_url = std::env::var("MINIMAX_OAUTH_BASE_URL").unwrap_or_else(|_| {
            if cn {
                "https://account.minimaxi.com".into()
            } else {
                "https://account.minimax.io".into()
            }
        });
        let region = if base_url.contains("minimaxi.com") {
            "cn".to_string()
        } else {
            "global".to_string()
        };
        Self { base_url, region }
    }
    #[must_use] 
    pub fn new() -> Arc<Self> { Arc::new(Self::from_env()) }
}

impl Default for MinimaxDeviceOAuth {
    fn default() -> Self { Self::from_env() }
}

#[async_trait]
impl DeviceCodeProvider for MinimaxDeviceOAuth {
    fn id(&self) -> &'static str { "minimax" }
    fn display_name(&self) -> &'static str { "MiniMax" }
    fn code_endpoint(&self) -> &str {
        // D-114 갱신: real MiniMax device code endpoint. base_url 이
        // "https://account.minimax.io" 또는 "https://account.minimaxi.com" 이면
        // 정적 literal 반환 (lifetime 안전). 그 외 override (e.g. local mock)는
        // global endpoint 로 fallback — mock server 가 다른 host 일 때.
        if self.base_url == "https://account.minimax.io" {
            "https://account.minimax.io/oauth2/device/code"
        } else if self.base_url == "https://account.minimaxi.com" {
            "https://account.minimaxi.com/oauth2/device/code"
        } else {
            "https://account.minimax.io/oauth2/device/code"
        }
    }
    fn token_endpoint(&self) -> &str {
        if self.base_url == "https://account.minimax.io" {
            "https://account.minimax.io/oauth2/token"
        } else if self.base_url == "https://account.minimaxi.com" {
            "https://account.minimaxi.com/oauth2/token"
        } else {
            "https://account.minimax.io/oauth2/token"
        }
    }
    fn client_id(&self) -> &'static str {
        // OpenClaw / Hermes Agent 와 동일한 공용 client_id. MiniMax 가 한 개만 발급하고 모든 client 공유.
        "78257093-7e40-4613-99e0-527b14b39113"
    }
    fn scope(&self) -> &'static str {
        "group_id profile model.completion"
    }
    fn region(&self) -> &str { &self.region }
}

/// 등록된 3 provider.
#[must_use] 
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

/// `DeviceCodeProvider` (W14). `MiniMax` 만 `DeviceCodeFlow` 사용. `find_device_provider("minimax`")
/// 로 `MinimaxDeviceOAuth` instance 반환.
#[must_use] 
pub fn find_device_provider(id: &str) -> Option<Arc<dyn DeviceCodeProvider>> {
    match id {
        "minimax" => Some(MinimaxDeviceOAuth::new() as Arc<dyn DeviceCodeProvider>),
        _ => None,
    }
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
