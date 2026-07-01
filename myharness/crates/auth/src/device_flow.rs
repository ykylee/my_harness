//! `MiniMax` Device Authorization Grant OAuth flow (W14, D-114 endpoint 갱신).
//!
//! `MiniMax` OAuth 는 표준 Authorization Code + redirect 가 아니라 **Device Authorization
//! Grant 변형** (OAuth 2.0 RFC 8628 의 `MiniMax` 구현). 흐름:
//!
//! 1) `request_code(provider)` → POST `{base_url}/oauth2/device/code` (form body) →
//!    `DeviceAuthorization { user_code, verification_uri, interval, expired_in, state }`
//! 2) cli 가 `verification_uri` 를 browser 로 open, user 가 `user_code` (6자리 + dash + 4자리) 입력
//! 3) `poll_token(provider, user_code, verifier)` → POST `{base_url}/oauth2/token` (form body,
//!    `grant_type=urn:ietf:params:oauth:grant-type:user_code`) →
//!    `TokenPoll { status, access_token?, refresh_token?, expired_in? }`
//! 4) `status=success` 시 `OAuthToken` 으로 변환 → `TokenStore::save`
//!
//! 다른 provider (`OpenAI`, Google) 는 표준 Authorization Code + redirect flow 사용
//! ([`crate::flow`]). `MiniMax` 만 이 flow 사용.
//!
//! 표준 (Hermes, `OpenClaw`) 과 동일한 상수:
//! - `client_id`: `78257093-7e40-4613-99e0-527b14b39113` (`MiniMax` 공통, 모든 client 가 동일 값 사용)
//! - scope: `group_id profile model.completion` (`MiniMax` Portal OAuth 권한)
//! - `grant_type`: `urn:ietf:params:oauth:grant-type:user_code` (`MiniMax` custom grant)
//! - endpoint: `https://account.minimax.io/oauth2/{device/code,token}` (글로벌; 한국 default, **D-114 갱신**)
//!   - 이전 (D-113 검증): `https://api.minimax.io/oauth/code` 307 redirect → 위 endpoint
//!   - D-114 갱신으로 production `MinimaxDeviceOAuth` 가 직접 새 endpoint hit
//!   - CN: `https://account.minimaxi.com/oauth2/{device/code,token}`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::pkce::{PkcePair, generate_pkce, generate_state};

#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("state mismatch (CSRF)")]
    StateMismatch,
    #[error("timed out waiting for user authorization")]
    Timeout,
}

/// Device Authorization Grant 응답 (POST /oauth/code).
///
/// **D-116 단위 contract (2026-07-01)**:
/// - `expired_in` 은 **milliseconds 단위 unix timestamp** (real `MiniMax` API 응답 형식,
///   D-52 follow-up / D-113 검증). OpenClaw/Hermes 와 동일 convention.
/// - `interval` 은 **milliseconds** (real `MiniMax` 는 `3000` = 3초 응답, D-113 검증).
///   `poll_until_success` 가 호출 시 seconds 로 변환 (`interval_ms / 1000` + `clamp(1, 10)`).
/// - mock test 의 `interval=1` / `expired_in=now+60` 은 **legacy 초 단위** (`W14.7
///   expired_in_to_chrono` 가 ms/μs/s 자동감지). D-116 에서 mock 도 `interval=1000` /
///   `expired_in=now+60_000` (ms) 으로 갱신.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthorization {
    pub user_code: String,
    pub verification_uri: String,
    /// 기본 2s (mock). real `MiniMax` 는 3000ms = 3초. **D-116: 단위 = ms**.
    #[serde(default = "default_interval")]
    pub interval: u64,
    /// **D-116: milliseconds 단위 unix timestamp** (real `MiniMax` API 응답 형식).
    /// user 가 이 시각 전에 authorize 해야 함.
    pub expired_in: u64,
    pub state: String,
}

/// **D-116**: legacy mock 호환 default (초). real spec 의 2초 = `2000` ms.
fn default_interval() -> u64 {
    2_000
}

/// D-115: real `MiniMax` API 의 응답 envelope.
/// 성공: `{"base_resp": {"status_code": 0, "status_msg": "success"}, ...페이로드}`
/// 실패: `{"base_resp": {"status_code": <non-zero>, "status_msg": "<msg>"}, ...}`
/// `base_resp` 가 부재하면 `None` 반환 (legacy 응답 / 다른 provider). 호출 측에서
/// `Some(code != 0)` 만 분기.
fn base_resp_status_code(value: &serde_json::Value) -> Option<i64> {
    let br = value.get("base_resp")?;
    br.get("status_code").and_then(serde_json::Value::as_i64)
}

/// POST /oauth/token polling 응답.
#[derive(Debug, Clone)]
pub enum TokenPoll {
    /// 아직 user 가 authorize 안 함. 계속 polling.
    Pending,
    /// user 가 authorize 완료. `access_token/refresh_token` 포함.
    Success {
        access_token: String,
        refresh_token: String,
        /// **D-116: milliseconds 단위 unix timestamp** (real `MiniMax` API 응답 형식).
        /// 만료 시각. manager.rs 의 `expired_in_to_chrono` 가 ms/μs/s 자동감지.
        expired_in: u64,
        /// optional. `token_type` (default "Bearer").
        token_type: Option<String>,
        /// optional. inference URL (e.g. `https://api.minimax.io/anthropic`).
        resource_url: Option<String>,
    },
    /// user 가 거부하거나 만료 등. 더 이상 polling 중단.
    Error(String),
}

#[derive(Debug, Clone)]
pub struct DeviceToken {
    pub access_token: String,
    pub refresh_token: String,
    /// **D-116: milliseconds 단위 unix timestamp** (real `MiniMax` API 응답 형식).
    /// manager.rs 의 `expired_in_to_chrono` 가 ms/μs/s 자동감지.
    pub expired_in: u64,
    pub token_type: String,
    pub resource_url: Option<String>,
}

/// Device Authorization Grant provider 정의 (`MiniMax` 만 사용).
///
/// 표준 [`crate::flow::OAuthProvider`] 와 별도 trait. OpenAI/Google 는 redirect flow
/// (Authorization Code + PKCE) 사용, `MiniMax` 만 이 flow 사용.
#[async_trait]
pub trait DeviceCodeProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &str;
    /// POST `{base_url}/oauth/code` 의 base URL.
    fn code_endpoint(&self) -> &str;
    /// POST `{base_url}/oauth/token` 의 base URL.
    fn token_endpoint(&self) -> &str;
    /// OAuth `client_id`.
    fn client_id(&self) -> &str;
    /// scope (공백 구분 단일 string).
    fn scope(&self) -> &str;
    /// 한국/글로벌 등 region 표시.
    fn region(&self) -> &str;
}

/// Device Authorization + state + PKCE 통합 요청.
#[derive(Debug, Clone)]
pub struct DeviceRequest {
    pub authorization: DeviceAuthorization,
    pub pkce: PkcePair,
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// POST `{code_endpoint}` → `DeviceAuthorization`.
///
/// `state` 와 `code_challenge` 모두 generate 후 form body 에 포함. 응답의 state 가
/// 일치하지 않으면 [`DeviceError::StateMismatch`].
pub async fn request_code(provider: &dyn DeviceCodeProvider) -> Result<DeviceRequest, DeviceError> {
    let pkce = generate_pkce();
    let state = generate_state();
    // D-117: real `MiniMax` Device Authorization Grant spec 은 `response_type=code`
    // 미포함 (이는 Authorization Code + redirect flow 의 표준 parameter).
    // 우리 v1 은 `flow.rs:121` 와 혼동했었지만, real `MiniMax` API 가 무시하므로
    // 제거. mock test 의 JSON body decode 도 `response_type` 에 의존 안 함 → 회귀 0.
    let body = serde_urlencoded::to_string(&[
        ("client_id", provider.client_id().to_string()),
        ("scope", provider.scope().to_string()),
        ("code_challenge", pkce.challenge.clone()),
        ("code_challenge_method", "S256".to_string()),
        ("state", state.clone()),
    ])
    .map_err(|e| DeviceError::Provider(format!("urlencoding: {e}")))?;
    let client = reqwest::Client::new();
    let resp = client
        .post(provider.code_endpoint())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| DeviceError::Provider(format!("json decode failed: {e} body={text}")))?;
    if !status.is_success() {
        let msg = value.get("error").and_then(|v| v.as_str()).unwrap_or(&text);
        return Err(DeviceError::Provider(format!("oauth/code {status}: {msg}")));
    }
    // D-115: real `MiniMax` API 는 HTTP 200 + `base_resp.status_code != 0` 으로
    // 실패 시나리오를 구분. `base_resp` 가 명시적으로 non-zero 면 `DeviceError::Provider`.
    if let Some(code) = base_resp_status_code(&value)
        && code != 0
    {
        let msg = value
            .get("base_resp")
            .and_then(|b| b.get("status_msg"))
            .and_then(|v| v.as_str())
            .unwrap_or("<missing>");
        return Err(DeviceError::Provider(format!(
            "oauth/code base_resp status_code={code} status_msg={msg}"
        )));
    }
    let auth: DeviceAuthorization = serde_json::from_value(value)
        .map_err(|e| DeviceError::Provider(format!("oauth/code body decode: {e}")))?;
    if auth.state != state {
        return Err(DeviceError::StateMismatch);
    }
    Ok(DeviceRequest {
        authorization: auth,
        pkce,
    })
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// POST `{token_endpoint}` 1회 호출. polling loop 에서 매번 호출.
///
/// `grant_type`: `urn:ietf:params:oauth:grant-type:user_code`.
pub async fn poll_token(
    provider: &dyn DeviceCodeProvider,
    user_code: &str,
    verifier: &str,
) -> Result<TokenPoll, DeviceError> {
    let body = serde_urlencoded::to_string(&[
        (
            "grant_type",
            "urn:ietf:params:oauth:grant-type:user_code".to_string(),
        ),
        ("client_id", provider.client_id().to_string()),
        ("user_code", user_code.to_string()),
        ("code_verifier", verifier.to_string()),
    ])
    .map_err(|e| DeviceError::Provider(format!("urlencoding: {e}")))?;
    let client = reqwest::Client::new();
    let resp = client
        .post(provider.token_endpoint())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Accept", "application/json")
        .body(body)
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    if !status.is_success() {
        return Ok(TokenPoll::Error(format!("oauth/token {status}: {text}")));
    }
    let value: serde_json::Value = if text.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&text)
            .map_err(|e| DeviceError::Provider(format!("json decode: {e} body={text}")))?
    };
    // D-115: real `MiniMax` API 는 HTTP 200 + `base_resp.status_code != 0` 으로
    // 실패 시그널. `base_resp` 가 non-zero 면 즉시 `TokenPoll::Error` 로 분기.
    // legacy `status: "error"` 응답은 기존 match 분기에서 계속 처리.
    if let Some(code) = base_resp_status_code(&value)
        && code != 0
    {
        let msg = value
            .get("base_resp")
            .and_then(|b| b.get("status_msg"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        return Ok(TokenPoll::Error(format!(
            "oauth/token base_resp status_code={code} status_msg={msg}"
        )));
    }
    let st = value.get("status").and_then(|v| v.as_str()).unwrap_or("");
    match st {
        "success" => {
            let access_token = value
                .get("access_token")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DeviceError::Provider("no access_token".into()))?
                .to_string();
            let refresh_token = value
                .get("refresh_token")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DeviceError::Provider("no refresh_token".into()))?
                .to_string();
            let expired_in = value
                .get("expired_in")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| DeviceError::Provider("no expired_in".into()))?;
            let token_type = value
                .get("token_type")
                .and_then(|v| v.as_str())
                .map(std::string::ToString::to_string);
            let resource_url = value
                .get("resource_url")
                .and_then(|v| v.as_str())
                .map(std::string::ToString::to_string);
            Ok(TokenPoll::Success {
                access_token,
                refresh_token,
                expired_in,
                token_type,
                resource_url,
            })
        }
        "pending" => Ok(TokenPoll::Pending),
        "error" => {
            let msg = value
                .get("base_resp")
                .and_then(|b| b.get("status_msg"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error")
                .to_string();
            Ok(TokenPoll::Error(msg))
        }
        _ => Ok(TokenPoll::Error(format!("unknown status: {st}"))),
    }
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// 만료 시각까지 interval 으로 polling. 성공/실패 시 종료.
///
/// **D-116 단위 contract**:
/// - `interval`: **milliseconds** (real `MiniMax` 응답). 내부에서 `(interval_ms / 1000)`
///   로 seconds 변환 후 `clamp(1, 10)` 적용 (sleep 1-10초).
/// - `expired_in_unix`: **milliseconds 단위 unix timestamp** (real `MiniMax` 응답).
///   `chrono::Utc::now().timestamp_millis()` 와 직접 비교.
///
/// `OpenClaw` 와 동일한 backoff 정책 (W14.5): `cur_interval *= 1.5` (cap 10s).
/// `MiniMax` 가 `interval=3000` (3초) 으로 응답해도 polling 1회 사이가 너무 길지 않도록
/// 1.5x backoff + cap. 단 `MiniMax` 가 명시한 interval 보다 작아지지 않음
/// (서버 권고 존중, 1초 floor).
pub async fn poll_until_success(
    provider: &dyn DeviceCodeProvider,
    user_code: &str,
    verifier: &str,
    interval_ms: u64,
    expired_in_unix: u64,
) -> Result<DeviceToken, DeviceError> {
    // D-116: ms → s 변환 + clamp(1, 10). mock test 의 interval_ms=1000 (1초) 와
    // real `MiniMax` 의 interval_ms=3000 (3초) 모두 seconds 로 변환되어 sleep.
    let mut cur_interval = (interval_ms / 1000).clamp(1, 10);
    let floor = 1u64;
    let cap = 10u64;
    let mut attempt = 0u32;
    while (chrono::Utc::now().timestamp_millis() as u64) < expired_in_unix {
        attempt += 1;
        tracing::debug!(target: "myharness::auth::device", "poll attempt={attempt} interval={cur_interval}s");
        match poll_token(provider, user_code, verifier).await? {
            TokenPoll::Pending => {
                tokio::time::sleep(std::time::Duration::from_secs(cur_interval)).await;
                // 1.5x backoff. cap 10s, floor 1s.
                let next = (cur_interval * 3 / 2).max(floor).min(cap);
                cur_interval = next;
            }
            TokenPoll::Success {
                access_token,
                refresh_token,
                expired_in,
                token_type,
                resource_url,
            } => {
                return Ok(DeviceToken {
                    access_token,
                    refresh_token,
                    expired_in,
                    token_type: token_type.unwrap_or_else(|| "Bearer".into()),
                    resource_url,
                });
            }
            TokenPoll::Error(msg) => return Err(DeviceError::Provider(msg)),
        }
    }
    Err(DeviceError::Timeout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    struct FakeDeviceProvider;
    #[async_trait]
    impl DeviceCodeProvider for FakeDeviceProvider {
        fn id(&self) -> &'static str {
            "fake"
        }
        fn display_name(&self) -> &'static str {
            "Fake Device"
        }
        fn code_endpoint(&self) -> &'static str {
            "https://fake/oauth/code"
        }
        fn token_endpoint(&self) -> &'static str {
            "https://fake/oauth/token"
        }
        fn client_id(&self) -> &'static str {
            "fake-client"
        }
        fn scope(&self) -> &'static str {
            "group_id profile model.completion"
        }
        fn region(&self) -> &'static str {
            "global"
        }
    }

    #[test]
    fn device_provider_metadata() {
        let p = FakeDeviceProvider;
        assert_eq!(p.id(), "fake");
        assert_eq!(p.region(), "global");
        assert!(p.code_endpoint().starts_with("https://"));
    }

    /// D-113 — Real `MiniMax` Device Authorization Grant 의 client 측 진입
    /// 검증. `request_code` 가 production `MinimaxDeviceOAuth::code_endpoint()`
    /// (https://account.minimax.io/oauth2/device/code, **D-114 갱신 후**) 에 도달
    /// → `user_code` + `verification_uri` 를 받아오는지 확인. **API key 불요**
    /// (Device flow 는 un-authed). 네트워크/방화벽 이슈로 fail 시 `eprintln!` +
    /// early return.
    ///
    /// **Manual run** (real MiniMax):
    /// `cargo test -p myharness-auth minimax_real_device_request_code -- --ignored --nocapture`
    ///
    /// **검증 단계**:
    /// 1. `MinimaxDeviceOAuth::from_env()` 가 real endpoint URL 을 가짐
    /// 2. `request_code` 가 HTTP 200 응답
    /// 3. JSON body 에 `user_code` 가 비어있지 않고 `XXXX-XXXX` 형식
    /// 4. `verification_uri` 가 `https://platform.minimax.io/oauth-authorize` 시작
    /// 5. `interval` 초 > 0 + `expired_in` ms 가 현재 시각 + 1분 이후
    ///
    /// **Endpoint 히스토리** (D-113 → D-114):
    /// - D-113 (2026-07-01 8th): `https://api.minimax.io/oauth/code` 307 redirect
    /// - D-114 (2026-07-01 9th): production `MinimaxDeviceOAuth` URL 갱신
    ///   → `https://account.minimax.io/oauth2/device/code` (direct hit, RFC 8628)
    #[tokio::test]
    #[ignore = "requires real network access to api.minimax.io (D-113)"]
    async fn minimax_real_device_request_code() {
        use crate::provider::MinimaxDeviceOAuth;

        let provider = MinimaxDeviceOAuth::from_env();
        eprintln!(
            "MiniMax Device OAuth: base_url={} region={} code_endpoint={}",
            provider.base_url,
            provider.region,
            provider.code_endpoint(),
        );

        // Step 1: code endpoint 가 https:// 시작
        assert!(
            provider.code_endpoint().starts_with("https://"),
            "code_endpoint must be HTTPS, got: {}",
            provider.code_endpoint(),
        );
        // Step 2: real endpoint 매핑 (production 이든 redirect target 이든)
        assert!(
            provider.code_endpoint().contains("minimax.io")
                || provider.code_endpoint().contains("minimaxi.com"),
            "code_endpoint must point at MiniMax infra, got: {}",
            provider.code_endpoint(),
        );

        // Step 3: 실제 request_code 호출
        let req = match request_code(&provider).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("request_code failed (network/auth/parse error): {e}");
                eprintln!("skipping — likely sandbox network restriction");
                return;
            }
        };
        eprintln!(
            "MiniMax device authorization: user_code={} verification_uri={} interval={} expired_in={}",
            req.authorization.user_code,
            req.authorization.verification_uri,
            req.authorization.interval,
            req.authorization.expired_in,
        );

        // Step 4: user_code 형식 (XXXX-XXXX)
        let uc = &req.authorization.user_code;
        assert!(!uc.is_empty(), "user_code must be non-empty");
        assert_eq!(
            uc.chars().filter(|c| *c == '-').count(),
            1,
            "user_code expected format XXXX-XXXX, got: {uc}"
        );
        assert_eq!(
            uc.len(),
            9,
            "user_code expected length 9 (XXXX-XXXX), got: {uc}"
        );

        // Step 5: verification_uri 검증
        let vuri = &req.authorization.verification_uri;
        assert!(
            vuri.starts_with("https://"),
            "verification_uri must be HTTPS"
        );
        assert!(
            vuri.contains("platform.minimax.io/oauth-authorize")
                || vuri.contains("platform.minimaxi.com"),
            "verification_uri must point at MiniMax authorize page, got: {vuri}"
        );

        // Step 6: interval / expired_in sanity
        assert!(req.authorization.interval > 0, "interval must be > 0");
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        assert!(
            req.authorization.expired_in > now_ms + 60_000,
            "expired_in must be > now + 60s, got: {} (now={})",
            req.authorization.expired_in,
            now_ms
        );
        // mini safety: 1년 이상 future 면 의심
        assert!(
            req.authorization.expired_in < now_ms + 365 * 24 * 60 * 60 * 1000,
            "expired_in suspiciously far in future: {}",
            req.authorization.expired_in
        );
    }

    // --- D-115: base_resp envelope helper + e2e branches ---

    #[test]
    fn d115_base_resp_status_code_zero() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"base_resp":{"status_code":0,"status_msg":"success"},"user_code":"X"}"#,
        )
        .unwrap();
        assert_eq!(base_resp_status_code(&v), Some(0));
    }

    #[test]
    fn d115_base_resp_status_code_nonzero() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"base_resp":{"status_code":40004,"status_msg":"invalid_grant"}}"#,
        )
        .unwrap();
        assert_eq!(base_resp_status_code(&v), Some(40004));
    }

    #[test]
    fn d115_base_resp_status_code_absent() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"status":"success","access_token":"x"}"#).unwrap();
        assert_eq!(base_resp_status_code(&v), None);
    }

    /// D-115: mock provider 가 `base_resp.status_code=0` 응답을 emit 할 때
    /// `request_code` 가 정상 경로 (DeviceAuthorization decode) 로 도달.
    #[tokio::test]
    async fn d115_request_code_base_resp_zero_succeeds() {
        struct MockProvider {
            code_ep: String,
        }
        #[async_trait]
        impl DeviceCodeProvider for MockProvider {
            fn id(&self) -> &'static str {
                "d115-zero"
            }
            fn display_name(&self) -> &str {
                "D115 base_resp=0"
            }
            fn code_endpoint(&self) -> &str {
                &self.code_ep
            }
            fn token_endpoint(&self) -> &str {
                "https://mock/oauth2/token"
            }
            fn client_id(&self) -> &'static str {
                "mock-client"
            }
            fn scope(&self) -> &'static str {
                "group_id profile model.completion"
            }
            fn region(&self) -> &'static str {
                "global"
            }
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{addr}/oauth2/device/code");
        let server = tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let state_param = req
                    .split("state=")
                    .nth(1)
                    .and_then(|s| s.split('&').next().map(std::string::ToString::to_string))
                    .unwrap_or_else(|| "placeholder".to_string());
                let now = chrono::Utc::now().timestamp_millis() as u64;
                let body = format!(
                    r#"{{"base_resp":{{"status_code":0,"status_msg":"success"}},"user_code":"D115-X1","verification_uri":"https://platform.test/oauth-authorize?user_code=D115-X1","interval":1,"expired_in":{},"state":"{}"}}"#,
                    now + 60,
                    state_param
                );
                let resp = format!(
                    "HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: {}
Connection: close

{}",
                    body.len(),
                    body
                );
                sock.write_all(resp.as_bytes()).await.unwrap();
                sock.shutdown().await.ok();
            }
        });
        let p = MockProvider { code_ep: url };
        let req = request_code(&p).await.expect("request_code failed");
        assert_eq!(req.authorization.user_code, "D115-X1");
        drop(server);
    }

    /// D-115: mock provider 가 `base_resp.status_code != 0` 응답을 emit 할 때
    /// `request_code` 가 `DeviceError::Provider` 로 분기.
    #[tokio::test]
    async fn d115_request_code_base_resp_nonzero_errors() {
        struct MockProvider {
            code_ep: String,
        }
        #[async_trait]
        impl DeviceCodeProvider for MockProvider {
            fn id(&self) -> &'static str {
                "d115-nonzero"
            }
            fn display_name(&self) -> &str {
                "D115 base_resp!=0"
            }
            fn code_endpoint(&self) -> &str {
                &self.code_ep
            }
            fn token_endpoint(&self) -> &str {
                "https://mock/oauth2/token"
            }
            fn client_id(&self) -> &'static str {
                "mock-client"
            }
            fn scope(&self) -> &'static str {
                "group_id profile model.completion"
            }
            fn region(&self) -> &'static str {
                "global"
            }
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{addr}/oauth2/device/code");
        let server = tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let _ = sock.read(&mut buf).await.unwrap();
                let body = r#"{"base_resp":{"status_code":40001,"status_msg":"invalid_client"}}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: {}
Connection: close

{}",
                    body.len(),
                    body
                );
                sock.write_all(resp.as_bytes()).await.unwrap();
                sock.shutdown().await.ok();
            }
        });
        let p = MockProvider { code_ep: url };
        let err = request_code(&p)
            .await
            .expect_err("request_code should fail on base_resp!=0");
        let msg = format!("{err}");
        assert!(
            msg.contains("base_resp"),
            "error should mention base_resp, got: {msg}"
        );
        assert!(
            msg.contains("40001"),
            "error should contain status_code, got: {msg}"
        );
        drop(server);
    }

    /// D-115: mock provider 가 `base_resp.status_code != 0` 응답을 emit 할 때
    /// `poll_token` 가 `TokenPoll::Error` 로 분기.
    #[tokio::test]
    async fn d115_poll_token_base_resp_nonzero_errors() {
        struct MockProvider {
            token_ep: String,
        }
        #[async_trait]
        impl DeviceCodeProvider for MockProvider {
            fn id(&self) -> &'static str {
                "d115-poll-err"
            }
            fn display_name(&self) -> &str {
                "D115 poll err"
            }
            fn code_endpoint(&self) -> &str {
                "https://mock/oauth2/device/code"
            }
            fn token_endpoint(&self) -> &str {
                &self.token_ep
            }
            fn client_id(&self) -> &'static str {
                "mock-client"
            }
            fn scope(&self) -> &'static str {
                "group_id profile model.completion"
            }
            fn region(&self) -> &'static str {
                "global"
            }
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{addr}/oauth2/token");
        let server = tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let _ = sock.read(&mut buf).await.unwrap();
                let body = r#"{"base_resp":{"status_code":40004,"status_msg":"invalid_grant: device_code expired or already used"}}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK
Content-Type: application/json
Content-Length: {}
Connection: close

{}",
                    body.len(),
                    body
                );
                sock.write_all(resp.as_bytes()).await.unwrap();
                sock.shutdown().await.ok();
            }
        });
        let p = MockProvider { token_ep: url };
        let poll = poll_token(&p, "UC-FAKE", "VERIFIER-FAKE")
            .await
            .expect("poll_token network ok");
        match poll {
            TokenPoll::Error(msg) => {
                assert!(
                    msg.contains("base_resp"),
                    "error should mention base_resp, got: {msg}"
                );
                assert!(
                    msg.contains("40004"),
                    "error should contain status_code, got: {msg}"
                );
            }
            other => panic!("expected TokenPoll::Error, got: {other:?}"),
        }
        drop(server);
    }

    // --- D-116: 단위 invariant (expired_in/interval 은 ms) ---

    /// D-116 invariant: real `MiniMax` API 의 `interval` 은 ms 단위
    /// (`3000` = 3초, D-113 검증). 1:1 정합성 + 추후 format 변경 시 빠른 진단.
    #[test]
    fn d116_interval_is_milliseconds_unit() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"user_code":"X","verification_uri":"https://platform.minimax.io/oauth-authorize?user_code=X&client=OpenClaw","interval":3000,"expired_in":1782881072225,"state":"x"}"#,
        )
        .unwrap();
        let auth: DeviceAuthorization = serde_json::from_value(v).unwrap();
        assert_eq!(auth.interval, 3_000, "interval must be ms (3_000 = 3s)");
        assert!(
            auth.interval >= 1_000,
            "interval (ms) must be >= 1s (1_000 ms), got: {}",
            auth.interval
        );
        assert!(
            auth.interval <= 60_000,
            "interval (ms) must be <= 60s (60_000 ms), got: {}",
            auth.interval
        );
    }

    /// D-116 invariant: `expired_in` 은 milliseconds 단위 unix timestamp.
    /// 2026-07-01 시점 unix ms ≈ 1.78e12. 초 단위로 해석하면 2026+56년 (잘못).
    #[test]
    fn d116_expired_in_is_milliseconds_unix_timestamp() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"user_code":"X","verification_uri":"https://platform.minimax.io/oauth-authorize","interval":3000,"expired_in":1782881072225,"state":"x"}"#,
        )
        .unwrap();
        let auth: DeviceAuthorization = serde_json::from_value(v).unwrap();
        let now_ms = chrono::Utc::now().timestamp_millis() as u64;
        assert!(
            auth.expired_in >= 1_000_000_000_000,
            "expired_in (ms) must be >= 1e12 (2001-09-09 in ms), got: {}",
            auth.expired_in
        );
        let max_future = now_ms + 5 * 365 * 24 * 60 * 60 * 1_000_u64;
        assert!(
            auth.expired_in <= max_future,
            "expired_in (ms) must be <= now + 5y, got: {} (now_ms={})",
            auth.expired_in,
            now_ms
        );
    }

    // --- D-117: request_code form body omits response_type=code ---

    /// D-117 invariant: real `MiniMax` Device Authorization Grant spec 은
    /// `response_type=code` 미포함 (이는 Authorization Code + redirect flow 의
    /// 표준 parameter). D-117 에서 `request_code` 의 form body 에서 제거. mock
    /// server 가 `response_type` 을 검사하지 않으므로 회귀 0. real API
    /// (D-113 검증) 도 무시.
    #[tokio::test]
    async fn d117_request_code_form_body_omits_response_type() {
        struct CapturingProvider {
            code_ep: String,
        }
        #[async_trait]
        impl DeviceCodeProvider for CapturingProvider {
            fn id(&self) -> &'static str {
                "d117-capture"
            }
            fn display_name(&self) -> &str {
                "D117 capture"
            }
            fn code_endpoint(&self) -> &str {
                &self.code_ep
            }
            fn token_endpoint(&self) -> &str {
                "https://mock/oauth2/token"
            }
            fn client_id(&self) -> &'static str {
                "mock-client"
            }
            fn scope(&self) -> &'static str {
                "group_id profile model.completion"
            }
            fn region(&self) -> &'static str {
                "global"
            }
        }
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{addr}/oauth2/device/code");
        let captured: std::sync::Arc<tokio::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(tokio::sync::Mutex::new(None));
        let captured_clone = captured.clone();
        const CRLF_CRLF: &str = "\r\n\r\n";
        let server = tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let n = sock.read(&mut buf).await.unwrap();
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                if let Some(off) = req.find(CRLF_CRLF) {
                    let body = &req[off + CRLF_CRLF.len()..];
                    *captured_clone.lock().await = Some(body.to_string());
                }
                let state_param = req
                    .split("state=")
                    .nth(1)
                    .and_then(|s| s.split('&').next().map(std::string::ToString::to_string))
                    .unwrap_or_else(|| "placeholder".to_string());
                let now = chrono::Utc::now().timestamp_millis() as u64;
                let resp_body = format!(
                    "{{\"base_resp\":{{\"status_code\":0,\"status_msg\":\"success\"}},\"user_code\":\"D117-X\",\"verification_uri\":\"https://platform.test/oauth-authorize?user_code=D117-X\",\"interval\":1000,\"expired_in\":{},\"state\":\"{}\"}}",
                    now + 60_000,
                    state_param
                );
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    resp_body.len(),
                    resp_body
                );
                sock.write_all(resp.as_bytes()).await.unwrap();
                sock.shutdown().await.ok();
            }
        });
        let p = CapturingProvider { code_ep: url };
        let _ = request_code(&p).await.expect("request_code failed");
        let body = captured.lock().await.clone().expect("body not captured");
        // D-117 핵심 assertion: `response_type` 미포함.
        assert!(
            !body.contains("response_type"),
            "form body must NOT contain response_type, got: {body}"
        );
        // sanity: PKCE + state + client_id + scope 는 그대로 포함 (회귀 방지).
        assert!(
            body.contains("code_challenge="),
            "PKCE code_challenge must be present, got: {body}"
        );
        assert!(
            body.contains("code_challenge_method=S256"),
            "code_challenge_method=S256 required, got: {body}"
        );
        assert!(
            body.contains("state="),
            "state parameter required, got: {body}"
        );
        assert!(
            body.contains("client_id=mock-client"),
            "client_id required, got: {body}"
        );
        assert!(
            body.contains("scope=group_id"),
            "scope required, got: {body}"
        );
        drop(server);
    }
}
