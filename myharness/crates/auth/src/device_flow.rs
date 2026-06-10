//! MiniMax Device Authorization Grant OAuth flow (W14).
//!
//! MiniMax OAuth 는 표준 Authorization Code + redirect 가 아니라 **Device Authorization
//! Grant 변형** (OAuth 2.0 RFC 8628 의 MiniMax 구현). 흐름:
//!
//! 1) `request_code(provider)` → POST `{base_url}/oauth/code` (form body) →
//!    `DeviceAuthorization { user_code, verification_uri, interval, expired_in, state }`
//! 2) cli 가 `verification_uri` 를 browser 로 open, user 가 `user_code` (6자리 + dash + 4자리) 입력
//! 3) `poll_token(provider, user_code, verifier)` → POST `{base_url}/oauth/token` (form body,
//!    `grant_type=urn:ietf:params:oauth:grant-type:user_code`) →
//!    `TokenPoll { status, access_token?, refresh_token?, expired_in? }`
//! 4) `status=success` 시 `OAuthToken` 으로 변환 → `TokenStore::save`
//!
//! 다른 provider (OpenAI, Google) 는 표준 Authorization Code + redirect flow 사용
//! ([`crate::flow`]). MiniMax 만 이 flow 사용.
//!
//! 표준 (Hermes, OpenClaw) 과 동일한 상수:
//! - client_id: `78257093-7e40-4613-99e0-527b14b39113` (MiniMax 공통, 모든 client 가 동일 값 사용)
//! - scope: `group_id profile model.completion` (MiniMax Portal OAuth 권한)
//! - grant_type: `urn:ietf:params:oauth:grant-type:user_code` (MiniMax custom grant)
//! - endpoint: `https://api.minimax.io/oauth/{code,token}` (글로벌; 한국 default)

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::pkce::{generate_pkce, generate_state, PkcePair};

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
/// `expired_in` 은 **unix timestamp** (초). OpenClaw/Hermes 와 동일 convention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthorization {
    pub user_code: String,
    pub verification_uri: String,
    /// 기본 2s. OpenClaw 는 1.5x backoff, 우리 v1 은 고정값 사용.
    #[serde(default = "default_interval")]
    pub interval: u64,
    /// unix timestamp (초). user 가 이 시각 전에 authorize 해야 함.
    pub expired_in: u64,
    pub state: String,
}

fn default_interval() -> u64 {
    2
}

/// POST /oauth/token polling 응답.
#[derive(Debug, Clone)]
pub enum TokenPoll {
    /// 아직 user 가 authorize 안 함. 계속 polling.
    Pending,
    /// user 가 authorize 완료. access_token/refresh_token 포함.
    Success {
        access_token: String,
        refresh_token: String,
        /// unix timestamp (초). 만료 시각.
        expired_in: u64,
        /// optional. token_type (default "Bearer").
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
    pub expired_in: u64,
    pub token_type: String,
    pub resource_url: Option<String>,
}

/// Device Authorization Grant provider 정의 (MiniMax 만 사용).
///
/// 표준 [`crate::flow::OAuthProvider`] 와 별도 trait. OpenAI/Google 는 redirect flow
/// (Authorization Code + PKCE) 사용, MiniMax 만 이 flow 사용.
#[async_trait]
pub trait DeviceCodeProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &str;
    /// POST `{base_url}/oauth/code` 의 base URL.
    fn code_endpoint(&self) -> &str;
    /// POST `{base_url}/oauth/token` 의 base URL.
    fn token_endpoint(&self) -> &str;
    /// OAuth client_id.
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

/// POST `{code_endpoint}` → `DeviceAuthorization`.
///
/// `state` 와 `code_challenge` 모두 generate 후 form body 에 포함. 응답의 state 가
/// 일치하지 않으면 [`DeviceError::StateMismatch`].
pub async fn request_code(
    provider: &dyn DeviceCodeProvider,
) -> Result<DeviceRequest, DeviceError> {
    let pkce = generate_pkce();
    let state = generate_state();
    let body = serde_urlencoded::to_string(&[
        ("response_type", "code".to_string()),
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
        let msg = value
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or(&text);
        return Err(DeviceError::Provider(format!("oauth/code {status}: {msg}")));
    }
    let auth: DeviceAuthorization = serde_json::from_value(value)
        .map_err(|e| DeviceError::Provider(format!("oauth/code body decode: {e}")))?;
    if auth.state != state {
        return Err(DeviceError::StateMismatch);
    }
    Ok(DeviceRequest { authorization: auth, pkce })
}

/// POST `{token_endpoint}` 1회 호출. polling loop 에서 매번 호출.
///
/// grant_type: `urn:ietf:params:oauth:grant-type:user_code`.
pub async fn poll_token(
    provider: &dyn DeviceCodeProvider,
    user_code: &str,
    verifier: &str,
) -> Result<TokenPoll, DeviceError> {
    let body = serde_urlencoded::to_string(&[
        ("grant_type", "urn:ietf:params:oauth:grant-type:user_code".to_string()),
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
                .and_then(|v| v.as_u64())
                .ok_or_else(|| DeviceError::Provider("no expired_in".into()))?;
            let token_type = value
                .get("token_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let resource_url = value
                .get("resource_url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
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

/// 만료 시각까지 interval 으로 polling. 성공/실패 시 종료.
///
/// OpenClaw 와 동일한 backoff 정책 (W14.5): `cur_interval *= 1.5` (cap 10s).
/// MiniMax 가 `interval=3000` 으로 응답해도 polling 1회 사이가 너무 길지 않도록
/// 1.5x backoff + cap. 단 MiniMax 가 명시한 interval 보다 작아지지 않음
/// (서버 권고 존중, 1초 floor).
///
/// `expired_in` 은 **milliseconds 단위 unix timestamp** (MiniMax 응답, D-52 follow-up 확인).
pub async fn poll_until_success(
    provider: &dyn DeviceCodeProvider,
    user_code: &str,
    verifier: &str,
    interval: u64,
    expired_in_unix: u64,
) -> Result<DeviceToken, DeviceError> {
    let mut cur_interval = interval.clamp(1, 10);
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
                continue;
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

    struct FakeDeviceProvider;
    #[async_trait]
    impl DeviceCodeProvider for FakeDeviceProvider {
        fn id(&self) -> &'static str { "fake" }
        fn display_name(&self) -> &str { "Fake Device" }
        fn code_endpoint(&self) -> &str { "https://fake/oauth/code" }
        fn token_endpoint(&self) -> &str { "https://fake/oauth/token" }
        fn client_id(&self) -> &str { "fake-client" }
        fn scope(&self) -> &str { "group_id profile model.completion" }
        fn region(&self) -> &str { "global" }
    }

    #[test]
    fn device_provider_metadata() {
        let p = FakeDeviceProvider;
        assert_eq!(p.id(), "fake");
        assert_eq!(p.region(), "global");
        assert!(p.code_endpoint().starts_with("https://"));
    }
}
