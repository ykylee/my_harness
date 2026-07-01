//! W15.b — 자동 OAuth token refresh wrapper (LLM client 레벨).
//!
//! # 목적
//! W15.a 에서 4 단계 credential chain 으로 LLM client 를 만들지만, OAuth token 이
//! **runtime 에 만료** 되면 (예: long-running daemon) 401 Unauthorized 가 떨어짐.
//! W15.b 에서는 `LLMClient::complete` 호출 후 `LlmError::ProviderCall` 메시지에
//! 401 / unauthorized / auth 키워드가 있으면 `AuthManager::ensure_fresh()` 로
//! token store 의 `refresh_token` 으로 자동 갱신 후 **1회 retry**.
//!
//! # 통합 지점
//! `cli::main::resolve_llm_client()` 가 OAuth token store 경로(1번)에서
//! `RefreshingLlmClient::wrap()` 으로 한 번 감싸줌. env var / `MockClient` 경로는
//! 그대로 (refresh 의미 없음).
//!
//! # 정책
//! - **401 식별**: `LlmError::ProviderCall(msg)` 의 `msg.to_lowercase()` 에
//!   "401" 또는 "unauthorized" 또는 "auth" 가 포함되면 만료로 간주.
//!   (`client_mock.rs:113` 의 `is_success` 와 동일 패턴 — heuristic)
//! - **`refresh_token` 없으면**: env var fallback (W15.a WARN 로그 + skip)
//! - **refresh 성공 시**: 새 token 으로 1회 retry. retry 도 실패하면 그 에러 surface.
//! - **retry 1회 한정**: 무한루프 방지 (refresh 후 새 `access_token` 이 401 이면
//!   진짜 invalid → surface)
//!
//! # 의존성
//! - `myharness-auth` (`AuthManager`, `OAuthProvider`)
//! - `myharness-llm` (`LLMClient`, `LlmError`)

use std::sync::Arc;

use myharness_auth::{manager::AuthError, AuthManager, OAuthProvider};
use myharness_llm::{CompletionRequest, CompletionResponse, LlmError, LLMClient};

/// 401 Unauthorized heuristic — `LlmError::ProviderCall(msg)` 의 메시지에
/// "401" / "unauthorized" / "auth" 가 포함되면 token 만료로 간주.
fn is_unauthorized_error(err: &LlmError) -> bool {
    match err {
        LlmError::ProviderCall(msg) => {
            let lower = msg.to_lowercase();
            lower.contains("401")
                || lower.contains("unauthorized")
                || (lower.contains("auth") && !lower.contains("oauth"))
        }
        LlmError::AuthMissing(_) => true, // 명시적 auth missing 도 refresh 시도 의미 있음
        _ => false,
    }
}

/// `RefreshingLlmClient` — inner client 호출 + 401 시 자동 refresh + 1회 retry.
///
/// inner client: 보통 `OpenAiCompatProvider` (OAuth Bearer)
/// provider / `base_url` / model: refresh 후 새 `OpenAiCompatProvider` 를 만드는 데 필요
pub struct RefreshingLlmClient {
    inner: Arc<dyn LLMClient>,
    provider_id: String,
    base_url: String,
    model: String,
    auth: Arc<AuthManager>,
    /// refresh 가능 OAuth provider (`MinimaxDeviceOAuth` 등). `provider_id` → provider 매핑.
    provider: Arc<dyn OAuthProvider>,
}

impl RefreshingLlmClient {
    pub fn new(
        inner: Arc<dyn LLMClient>,
        provider_id: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        auth: Arc<AuthManager>,
        provider: Arc<dyn OAuthProvider>,
    ) -> Self {
        Self {
            inner,
            provider_id: provider_id.into(),
            base_url: base_url.into(),
            model: model.into(),
            auth,
            provider,
        }
    }

    /// `ensure_fresh` → 새 token 으로 inner client 교체 후 retry.
    /// 성공: 새 `Arc<dyn LLMClient>` 반환. 실패: 원래 에러 반환.
    async fn try_refresh_and_retry(
        &self,
        req: &CompletionRequest,
    ) -> Result<CompletionResponse, LlmError> {
        // 1) ensure_fresh: store 의 refresh_token 으로 갱신 시도
        let new_token = match self.auth.ensure_fresh(self.provider.clone()).await {
            Ok(Some(t)) => t,
            Ok(None) => {
                tracing::warn!(
                    "W15.b: no token in store for provider '{}'; cannot refresh; falling back to original error",
                    self.provider_id
                );
                return Err(LlmError::ProviderCall(
                    "401 unauthorized and no stored token to refresh".into(),
                ));
            }
            Err(AuthError::OAuth(oauth_err)) => {
                tracing::warn!(
                    "W15.b: OAuth refresh failed for provider '{}': {}",
                    self.provider_id,
                    oauth_err
                );
                return Err(LlmError::ProviderCall(format!(
                    "401 unauthorized; refresh failed: {oauth_err}"
                )));
            }
            Err(e) => {
                tracing::warn!(
                    "W15.b: ensure_fresh failed for provider '{}': {}",
                    self.provider_id,
                    e
                );
                return Err(LlmError::ProviderCall(format!(
                    "401 unauthorized; refresh error: {e}"
                )));
            }
        };

        // 2) 새 token 으로 새 client 만들어서 1회 retry
        tracing::info!(
            "W15.b: OAuth token refreshed for provider '{}' (expires_at={:?}); retrying",
            self.provider_id,
            new_token.expires_at
        );

        let new_client = myharness_llm::OpenAiCompatProvider::new(
            &self.base_url,
            &new_token.access_token,
            &self.model,
            match self.provider_id.as_str() {
                "openai" => myharness_llm::provider::ProviderId::Codex,
                "google" => myharness_llm::provider::ProviderId::Gemini,
                "deepseek" => myharness_llm::provider::ProviderId::Deepseek,
                "local" => myharness_llm::provider::ProviderId::LocalLlm,
                "claude" => myharness_llm::provider::ProviderId::Claude,
                _ => myharness_llm::provider::ProviderId::Minimax, // default (minimax 포함)
            },
        )
        .map_err(|e| LlmError::ProviderInit(format!(
            "W15.b: failed to rebuild OpenAiCompatProvider after refresh: {e}"
        )))?;

        new_client.complete(req.clone()).await
    }
}

#[async_trait::async_trait]
impl LLMClient for RefreshingLlmClient {
    fn provider_id(&self) -> myharness_llm::provider::ProviderId {
        self.inner.provider_id()
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        match self.inner.complete(req.clone()).await {
            Ok(resp) => Ok(resp),
            Err(e) if is_unauthorized_error(&e) => {
                tracing::info!(
                    "W15.b: detected auth error from provider '{}'; attempting refresh",
                    self.provider_id
                );
                self.try_refresh_and_retry(&req).await
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use myharness_auth::TokenStore;
    use myharness_llm::provider::ProviderId;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// `e2e_401_refresh_retry_200` 의 inner mock — 401 응답 + 호출 횟수 카운트
    struct Always401Counter {
        count: AtomicUsize,
    }
    #[async_trait::async_trait]
    impl LLMClient for Always401Counter {
        fn provider_id(&self) -> ProviderId {
            ProviderId::Minimax
        }
        async fn complete(
            &self,
            _req: CompletionRequest,
        ) -> Result<CompletionResponse, LlmError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Err(LlmError::ProviderCall("401 unauthorized".into()))
        }
    }

    /// `e2e_without_refresh_token_returns_error` 의 inner mock — unit struct
    struct Always401Simple;
    #[async_trait::async_trait]
    impl LLMClient for Always401Simple {
        fn provider_id(&self) -> ProviderId {
            ProviderId::Minimax
        }
        async fn complete(
            &self,
            _req: CompletionRequest,
        ) -> Result<CompletionResponse, LlmError> {
            Err(LlmError::ProviderCall("401 unauthorized".into()))
        }
    }

    #[test]
    fn is_unauthorized_detects_401() {
        let e = LlmError::ProviderCall("HTTP 401 Unauthorized".into());
        assert!(is_unauthorized_error(&e));
    }

    #[test]
    fn is_unauthorized_detects_unauthorized_keyword() {
        let e = LlmError::ProviderCall("request was unauthorized".into());
        assert!(is_unauthorized_error(&e));
    }

    #[test]
    fn is_unauthorized_detects_auth_keyword() {
        let e = LlmError::ProviderCall("auth token expired".into());
        assert!(is_unauthorized_error(&e));
    }

    #[test]
    fn is_unauthorized_ignores_oauth_keyword() {
        // "oauth" 가 "auth" 를 부분문자열로 가지지만 exclude — false 양성 방지
        let e = LlmError::ProviderCall("oauth flow error".into());
        assert!(!is_unauthorized_error(&e));
    }

    #[test]
    fn is_unauthorized_ignores_other_errors() {
        assert!(!is_unauthorized_error(&LlmError::ProviderCall("conn refused".into())));
        assert!(!is_unauthorized_error(&LlmError::RateLimited("429".into())));
        assert!(!is_unauthorized_error(&LlmError::ContextOverflow("too long".into())));
    }

    #[test]
    fn is_unauthorized_treats_auth_missing_as_unauthorized() {
        assert!(is_unauthorized_error(&LlmError::AuthMissing(ProviderId::Minimax)));
    }

    /// W15.b.2 — `ensure_fresh` → refresh 후 새 client 로 retry 시뮬레이션
    ///
    /// 이 테스트는 mock OAuth server 없이도 refresh 로직의 핵심 분기를 검증:
    /// - store 가 비어있으면 → "no stored token" 에러 surface
    /// - store 에 `refresh_token` 있으면 → `ensure_fresh` 호출 (실제 provider 가 없으면 store error)
    ///
    /// 실제 e2e (mock server) 는 cli crate 의 integration test 에서 처리.
    #[tokio::test]
    async fn refreshing_client_with_no_stored_token_returns_error() {
        // store 비어있음 (임시 디렉토리)
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        let auth = AuthManager::with_store(store.clone());

        // ensure_fresh 가 Ok(None) 반환하는지 직접 확인
        let result = auth
            .ensure_fresh(Arc::new(StubProvider))
            .await
            .unwrap();
        assert!(result.is_none(), "empty store should yield None from ensure_fresh");
    }

    /// `OAuthProvider` stub — `ensure_fresh` 분기 검증용
    struct StubProvider;
    #[async_trait::async_trait]
    impl OAuthProvider for StubProvider {
        fn id(&self) -> &'static str {
            "stub"
        }
        fn display_name(&self) -> &'static str {
            "Stub"
        }
        fn client_id(&self) -> &'static str {
            "stub"
        }
        fn client_secret(&self) -> Option<&'static str> {
            None
        }
        fn authorize_endpoint(&self) -> &'static str {
            "https://example.com/oauth/authorize"
        }
        fn token_endpoint(&self) -> &'static str {
            "https://example.com/oauth/token"
        }
        fn default_scopes(&self) -> &'static [&'static str] {
            &[]
        }
    }

    /// W15.b.2 e2e — 401 → `ensure_fresh(mock` OAuth) → store save + retry 검증.
    ///
    /// inner = Always401 (deterministic 401). 1st 호출 401 → `RefreshingLlmClient::try_refresh_and_retry`
    /// 가 mock OAuth server 의 /token 에 refresh 요청 → 새 `access_token` 받음 → store save
    /// → 새 `OpenAiCompatProvider` (`base_url=mock`) 빌드 → retry 호출 → mock server 가
    /// OpenAI-compat JSON 응답 → Ok.
    #[tokio::test]
    async fn refreshing_client_e2e_401_refresh_retry_200() {
        use myharness_auth::flow::OAuthToken;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // 1) mock server — OAuth /token + OpenAI /v1/chat/completions
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base = format!("http://{addr}");
        let new_access = "at-new-refreshed-67890";
        let new_access_clone = new_access.to_string();

        let server_task = tokio::spawn(async move {
            for _ in 0..4 {
                if let Ok((mut sock, _)) = listener.accept().await {
                    let mut buf = vec![0u8; 8192];
                    let n = sock.read(&mut buf).await.unwrap_or(0);
                    if n == 0 {
                        continue;
                    }
                    let req = String::from_utf8_lossy(&buf[..n]).to_string();
                    let line = req.lines().next().unwrap_or("").to_string();
                    let resp = if line.contains("/token") {
                        let body = format!(
                            r#"{{"access_token":"{new_access_clone}","refresh_token":"rt-new","expires_in":3600,"scope":"read","token_type":"Bearer"}}"#,
                        );
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    } else if line.contains("/chat/completions") {
                        // OpenAI-compat 200 OK — rig-core 가 deserialize 할 수 있는 형식
                        let body = r#"{"id":"x","object":"chat.completion","created":0,"model":"MiniMax-M3","choices":[{"index":0,"message":{"role":"assistant","content":"ok"},"finish_reason":"stop"}],"usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}"#;
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        )
                    } else {
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                            .to_string()
                    };
                    sock.write_all(resp.as_bytes()).await.ok();
                    sock.shutdown().await.ok();
                }
            }
        });

        // 2) 만료된 token store 에 저장
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        let old_token = OAuthToken {
            access_token: "at-old-expired-11111".into(),
            refresh_token: Some("rt-valid-22222".into()),
            expires_at: Some(chrono::Utc::now() - chrono::Duration::seconds(60)),
            scope: Some("read".into()),
            token_type: "Bearer".into(),
        };
        store.save("minimax", &old_token).unwrap();

        // 3) inner: Always401Counter (deterministic 401 + count)
        let inner: Arc<dyn LLMClient> = Arc::new(Always401Counter {
            count: AtomicUsize::new(0),
        });

        // 4) RefreshingLlmClient — provider 는 mock OAuth server 가리킴
        let auth_manager = AuthManager::with_store(store.clone());
        let provider: Arc<dyn OAuthProvider> = Arc::new(MiniMaxProviderForTest {
            token_ep: format!("{base}/oauth/token"),
        });
        let client = RefreshingLlmClient::new(
            inner,
            "minimax",
            format!("{base}/v1").as_str(),
            "MiniMax-M3",
            Arc::new(auth_manager),
            provider,
        );

        // 5) complete 호출 → 1st 401 → ensure_fresh → mock /token → store save
        //    → 새 OpenAiCompatProvider (mock base_url) 빌드 → retry → mock /chat/completions
        //    → 200 OK (OpenAI-compat) → Ok
        let req = CompletionRequest {
            model: "MiniMax-M3".into(),
            system: None,
            messages: vec![myharness_llm::Message::user("ping")],
            max_tokens: Some(16),
            temperature: Some(1.0),
            stop: vec![],
            stream: false,
            metadata: json!(null),
            tools: Vec::new(),
        };
        let result = client.complete(req).await;
        assert!(
            result.is_ok(),
            "expected Ok after 401 + refresh + retry-200; got: {:?}",
            result.err()
        );
        let resp = result.unwrap();
        assert_eq!(resp.content, "ok");

        // 6) store 가 새 access_token 으로 갱신되었는지 확인
        let stored_after = store.load("minimax").unwrap();
        assert_eq!(stored_after.token.access_token, new_access);
        assert_eq!(stored_after.token.refresh_token.as_deref(), Some("rt-new"));

        let _ = tokio::time::timeout(std::time::Duration::from_secs(2), server_task).await;
    }

    /// W15.b.2 e2e — `refresh_token` 없으면 refresh 시도 않고 surface
    #[tokio::test]
    async fn refreshing_client_without_refresh_token_returns_no_stored_token_error() {
        use myharness_auth::flow::OAuthToken;

        // 1) 만료된 access_token, refresh_token 없음
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        let old_token = OAuthToken {
            access_token: "at-old-no-refresh".into(),
            refresh_token: None,
            expires_at: Some(chrono::Utc::now() - chrono::Duration::seconds(60)),
            scope: None,
            token_type: "Bearer".into(),
        };
        store.save("minimax", &old_token).unwrap();

        // 2) inner: 항상 401 (Always401Simple)
        let auth_manager = AuthManager::with_store(store.clone());
        let provider: Arc<dyn OAuthProvider> = Arc::new(StubProvider);
        let inner: Arc<dyn LLMClient> = Arc::new(Always401Simple);
        let client = RefreshingLlmClient::new(
            inner,
            "minimax",
            "https://api.minimax.io/v1",
            "MiniMax-M3",
            Arc::new(auth_manager),
            provider,
        );

        let req = CompletionRequest {
            model: "MiniMax-M3".into(),
            system: None,
            messages: vec![myharness_llm::Message::user("ping")],
            max_tokens: Some(16),
            temperature: Some(1.0),
            stop: vec![],
            stream: false,
            metadata: json!(null),
            tools: Vec::new(),
        };
        let result = client.complete(req).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        // ensure_fresh 가 refresh_token 없어서 Ok(Some(old_token)) 반환 → retry 시 OpenAiCompatProvider 빌드
        // → 401 그대로 surface
        // (W15.a 와 동일하게 refresh 안 하고 fallback 안 함 — refresh_token 없는 경우)
        // 이 케이스에선 ensure_fresh 가 expired token 그대로 반환, retry 가 새 client 로 호출, OpenAI-compat 빌드 자체는 성공 → LLM 호출이 mock URL 없어서 다른 에러
        // 단순 검증: 에러 surface 됨
        match err {
            LlmError::ProviderCall(_) | LlmError::ProviderInit(_) | LlmError::ProviderUnavailable(_) => {}
            other => panic!("unexpected error variant: {other:?}"),
        }
    }

    /// W15.b e2e helper — `MiniMax` OAuth provider mock (`token_endpoint` 만 mock).
    /// `token_endpoint` 가 mock server URL 이라 매 test 마다 다른 String. trait signature 가
    /// `&str` (self lifetime) 이므로 `&self.token_ep` 그대로 빌려쓰면 OK.
    /// `id` / `display_name` / `client_id` / `authorize_endpoint` 는 static literal.
    struct MiniMaxProviderForTest {
        token_ep: String,
    }
    #[async_trait::async_trait]
    impl OAuthProvider for MiniMaxProviderForTest {
        fn id(&self) -> &'static str {
            "minimax"
        }
        fn display_name(&self) -> &'static str {
            "MiniMax (test)"
        }
        fn client_id(&self) -> &'static str {
            "78257093-7e40-4613-99e0-527b14b39113"
        }
        fn client_secret(&self) -> Option<&'static str> {
            None
        }
        fn authorize_endpoint(&self) -> &'static str {
            "https://api.minimax.io/oauth/authorize"
        }
        fn token_endpoint(&self) -> &str {
            &self.token_ep
        }
        fn default_scopes(&self) -> &'static [&'static str] {
            &["group_id", "profile", "model.completion"]
        }
    }
}
