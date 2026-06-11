//! `FallbackRouter` — chain 순서대로 시도, fallback 가능한 에러면 다음 provider 로 cascade.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::{Mutex, RwLock};

use crate::chain::ActiveProviderChain;
use crate::client::{CompletionRequest, CompletionResponse, LLMClient};
use crate::error::LlmError;
use crate::provider::ProviderId;

#[derive(Debug, Clone, Default)]
pub struct ProviderStatus {
    pub last_error: Option<String>,
    pub consecutive_failures: u32,
    pub last_success_at: Option<DateTime<Utc>>,
}

pub struct FallbackRouter {
    chain: Arc<RwLock<ActiveProviderChain>>,
    clients: Arc<RwLock<HashMap<ProviderId, Arc<dyn LLMClient>>>>,
    statuses: Arc<Mutex<HashMap<ProviderId, ProviderStatus>>>,
    max_consecutive_failures_skip: u32,
}

#[derive(Debug, Clone)]
pub struct RouterResponse {
    pub response: CompletionResponse,
    pub served_by: ProviderId,
    pub attempts: u32,
}

impl FallbackRouter {
    #[must_use] 
    pub fn new(chain: ActiveProviderChain) -> Self {
        Self {
            chain: Arc::new(RwLock::new(chain)),
            clients: Arc::new(RwLock::new(HashMap::new())),
            statuses: Arc::new(Mutex::new(HashMap::new())),
            max_consecutive_failures_skip: 3,
        }
    }

    #[must_use] 
    pub fn with_consecutive_failure_threshold(mut self, n: u32) -> Self {
        self.max_consecutive_failures_skip = n;
        self
    }

    pub async fn with_client(&self, provider: ProviderId, client: Arc<dyn LLMClient>) {
        self.clients.write().await.insert(provider, client);
    }

    pub async fn chain_snapshot(&self) -> ActiveProviderChain {
        self.chain.read().await.clone()
    }

    pub async fn update_chain(&self, chain: ActiveProviderChain) {
        *self.chain.write().await = chain;
    }

    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub async fn complete(&self, req: CompletionRequest) -> Result<RouterResponse, LlmError> {
        let chain_snapshot = self.chain.read().await.clone();
        let entries: Vec<_> = chain_snapshot.iter().cloned().collect();
        let total = entries.len();
        let mut attempts: u32 = 0;
        let mut last_err: Option<LlmError> = None;

        for entry in entries {
            attempts += 1;
            let client = if let Some(c) = self.clients.read().await.get(&entry.provider) { Arc::clone(c) } else {
                last_err = Some(LlmError::ProviderUnavailable(format!(
                    "no client for {}",
                    entry.provider
                )));
                continue;
            };

            // consecutive_failures 가 임계값 이상이고 마지막 fallback 이 아니면 skip
            let status = {
                let s = self.statuses.lock().await;
                s.get(&entry.provider).cloned().unwrap_or_default()
            };
            if status.consecutive_failures >= self.max_consecutive_failures_skip
                && (attempts as usize) < total
            {
                continue;
            }

            let req_for_provider = CompletionRequest {
                model: entry.default_model.clone(),
                ..req.clone()
            };

            match client.complete(req_for_provider).await {
                Ok(resp) => {
                    let mut statuses = self.statuses.lock().await;
                    statuses.insert(
                        entry.provider,
                        ProviderStatus {
                            consecutive_failures: 0,
                            last_success_at: Some(Utc::now()),
                            ..status
                        },
                    );
                    return Ok(RouterResponse {
                        response: resp,
                        served_by: entry.provider,
                        attempts,
                    });
                }
                Err(e) if !e.is_fallbackable() => {
                    return Err(e);
                }
                Err(e) => {
                    let mut statuses = self.statuses.lock().await;
                    let prev = statuses.get(&entry.provider).cloned().unwrap_or_default();
                    statuses.insert(
                        entry.provider,
                        ProviderStatus {
                            consecutive_failures: prev.consecutive_failures + 1,
                            last_error: Some(e.to_string()),
                            last_success_at: prev.last_success_at,
                        },
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or(LlmError::Other("no providers in chain".into())))
    }

    pub async fn status_snapshot(&self) -> Vec<(ProviderId, ProviderStatus)> {
        let s = self.statuses.lock().await;
        let mut out: Vec<_> = s.iter().map(|(k, v)| (*k, v.clone())).collect();
        out.sort_by_key(|(k, _)| *k);
        out
    }

    pub async fn reset_status(&self, provider: ProviderId) {
        self.statuses.lock().await.remove(&provider);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_state::AuthState;
    use crate::chain::DiscoveredProvider;
    use crate::client::{CompletionRequest, Message};
    use crate::client_mock::{MockClient, MockResponse};

    fn req() -> CompletionRequest {
        CompletionRequest {
            messages: vec![Message::user("hi")],
            ..Default::default()
        }
    }

    fn chain3() -> ActiveProviderChain {
        // order: claude (env) → gemini (env) → codex (manual = lowest priority)
        ActiveProviderChain::from_discovered(vec![
            DiscoveredProvider {
                provider: ProviderId::Claude,
                auth_state: AuthState::EnvVar,
                default_model: "claude-sonnet-4-6".into(),
            },
            DiscoveredProvider {
                provider: ProviderId::Gemini,
                auth_state: AuthState::EnvVar,
                default_model: "gemini-2.5-pro".into(),
            },
            DiscoveredProvider {
                provider: ProviderId::Codex,
                auth_state: AuthState::Manual,
                default_model: "gpt-4o".into(),
            },
        ])
    }

    #[tokio::test]
    async fn router_primary_succeeds_first_try() {
        let c1 = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c1.push(MockResponse::Text("from claude".into()));
        let router = FallbackRouter::new(chain3());
        router.with_client(ProviderId::Claude, c1.clone()).await;
        router.with_client(ProviderId::Codex, Arc::new(MockClient::new(ProviderId::Codex, "gpt-4o"))).await;
        router.with_client(ProviderId::Gemini, Arc::new(MockClient::new(ProviderId::Gemini, "gemini-2.5-pro"))).await;
        let r = router.complete(req()).await.unwrap();
        assert_eq!(r.served_by, ProviderId::Claude);
        assert_eq!(r.attempts, 1);
        assert_eq!(r.response.content, "from claude");
    }

    #[tokio::test]
    async fn router_falls_back_to_second_when_first_unavailable() {
        let c1 = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c1.push(MockResponse::Error("dns failure".into()));
        let c2 = Arc::new(MockClient::new(ProviderId::Gemini, "gemini-2.5-pro"));
        c2.push(MockResponse::Text("from gemini".into()));
        let c3 = Arc::new(MockClient::new(ProviderId::Codex, "gpt-4o"));
        let router = FallbackRouter::new(chain3());
        router.with_client(ProviderId::Claude, c1).await;
        router.with_client(ProviderId::Gemini, c2).await;
        router.with_client(ProviderId::Codex, c3).await;
        let r = router.complete(req()).await.unwrap();
        assert_eq!(r.served_by, ProviderId::Gemini);
        assert_eq!(r.attempts, 2);
    }

    #[tokio::test]
    async fn router_falls_back_to_third_when_first_two_fail() {
        let c1 = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c1.push(MockResponse::Error("dns".into()));
        let c2 = Arc::new(MockClient::new(ProviderId::Gemini, "gemini-2.5-pro"));
        c2.push(MockResponse::Error("dns".into()));
        let c3 = Arc::new(MockClient::new(ProviderId::Codex, "gpt-4o"));
        c3.push(MockResponse::Text("from codex".into()));
        let router = FallbackRouter::new(chain3());
        router.with_client(ProviderId::Claude, c1).await;
        router.with_client(ProviderId::Gemini, c2).await;
        router.with_client(ProviderId::Codex, c3).await;
        let r = router.complete(req()).await.unwrap();
        assert_eq!(r.served_by, ProviderId::Codex);
        assert_eq!(r.attempts, 3);
        assert_eq!(r.response.content, "from codex");
    }

    #[tokio::test]
    async fn router_surfaces_rate_limit_immediately_no_fallback() {
        let c1 = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c1.push(MockResponse::Error("HTTP 429".into()));
        let c2 = Arc::new(MockClient::new(ProviderId::Gemini, "gemini-2.5-pro"));
        c2.push(MockResponse::Text("ok".into()));
        let c3 = Arc::new(MockClient::new(ProviderId::Codex, "gpt-4o"));
        let router = FallbackRouter::new(chain3());
        router.with_client(ProviderId::Claude, c1).await;
        router.with_client(ProviderId::Gemini, c2).await;
        router.with_client(ProviderId::Codex, c3).await;
        let e = router.complete(req()).await.unwrap_err();
        assert!(matches!(e, LlmError::RateLimited(_)));
    }

    #[tokio::test]
    async fn router_surfaces_auth_missing_immediately_no_fallback() {
        let c1 = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c1.push(MockResponse::Error("401 auth missing".into()));
        let c2 = Arc::new(MockClient::new(ProviderId::Gemini, "gemini-2.5-pro"));
        c2.push(MockResponse::Text("ok".into()));
        let router = FallbackRouter::new(chain3());
        router.with_client(ProviderId::Claude, c1).await;
        router.with_client(ProviderId::Gemini, c2).await;
        router.with_client(ProviderId::Codex, Arc::new(MockClient::new(ProviderId::Codex, "gpt-4o"))).await;
        let e = router.complete(req()).await.unwrap_err();
        assert!(matches!(e, LlmError::AuthMissing(_)));
    }

    #[tokio::test]
    async fn router_skips_provider_with_3_consecutive_failures() {
        let c1 = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c1.push(MockResponse::Error("dns".into()));
        c1.push(MockResponse::Error("dns".into()));
        c1.push(MockResponse::Error("dns".into()));
        let c2 = Arc::new(MockClient::new(ProviderId::Gemini, "gemini-2.5-pro"));
        for _ in 0..4 {
            c2.push(MockResponse::Text("from gemini".into()));
        }
        let c3 = Arc::new(MockClient::new(ProviderId::Codex, "gpt-4o"));
        let router = FallbackRouter::new(chain3());
        router.with_client(ProviderId::Claude, c1.clone()).await;
        router.with_client(ProviderId::Gemini, c2.clone()).await;
        router.with_client(ProviderId::Codex, c3).await;
        for _ in 0..4 {
            let _ = router.complete(req()).await.unwrap();
        }
        assert_eq!(c1.call_count(), 3);
    }

    #[tokio::test]
    async fn router_status_snapshot_reflects_attempts() {
        let c1 = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c1.push(MockResponse::Error("dns".into()));
        let c2 = Arc::new(MockClient::new(ProviderId::Gemini, "gemini-2.5-pro"));
        c2.push(MockResponse::Text("ok".into()));
        let c3 = Arc::new(MockClient::new(ProviderId::Codex, "gpt-4o"));
        let router = FallbackRouter::new(chain3());
        router.with_client(ProviderId::Claude, c1).await;
        router.with_client(ProviderId::Gemini, c2).await;
        router.with_client(ProviderId::Codex, c3).await;
        router.complete(req()).await.unwrap();
        let snap: Vec<(ProviderId, ProviderStatus)> = router.status_snapshot().await;
        let claude = snap.iter().find(|(k, _)| *k == ProviderId::Claude).unwrap();
        assert_eq!(claude.1.consecutive_failures, 1);
        let gemini = snap.iter().find(|(k, _)| *k == ProviderId::Gemini).unwrap();
        assert_eq!(gemini.1.consecutive_failures, 0);
        assert!(gemini.1.last_success_at.is_some());
    }

    #[tokio::test]
    async fn router_reset_status_clears_consecutive_failures() {
        let c1 = Arc::new(MockClient::new(ProviderId::Claude, "claude-sonnet-4-6"));
        c1.push(MockResponse::Error("dns".into()));
        let c2 = Arc::new(MockClient::new(ProviderId::Gemini, "gemini-2.5-pro"));
        c2.push(MockResponse::Text("ok".into()));
        let c3 = Arc::new(MockClient::new(ProviderId::Codex, "gpt-4o"));
        let router = FallbackRouter::new(chain3());
        router.with_client(ProviderId::Claude, c1).await;
        router.with_client(ProviderId::Gemini, c2).await;
        router.with_client(ProviderId::Codex, c3).await;
        router.complete(req()).await.unwrap();
        router.reset_status(ProviderId::Claude).await;
        let snap: Vec<(ProviderId, ProviderStatus)> = router.status_snapshot().await;
        let claude = snap.iter().find(|(k, _)| *k == ProviderId::Claude);
        assert!(claude.is_none());
    }
}
