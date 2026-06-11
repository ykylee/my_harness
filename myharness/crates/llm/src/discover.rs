//! `provider-auto-config` v1 simple: env + keychain + local scan → active chain.

use std::path::PathBuf;

use crate::auth_state::AuthState;
use crate::auth_store::AuthStore;
use crate::chain::{ActiveProviderChain, DiscoveredProvider};
use crate::metadata::ProviderMetadata;
use crate::paths;
use crate::provider::ProviderId;
use crate::registry::ProviderRegistry;
use crate::scan_local::{scan_local_servers, LocalHit};

#[derive(Debug, Clone, Default)]
pub struct DiscoverOpts {
    pub probe_timeout_ms: u64,
    pub home_override: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct DiscoverReport {
    pub chain: ActiveProviderChain,
    pub env_hits: Vec<EnvVarHit>,
    pub keychain_hits: Vec<ProviderId>,
    pub local_hits: Vec<LocalHit>,
}

#[derive(Debug, Clone)]
pub struct EnvVarHit {
    pub provider: ProviderId,
    pub env_var: String,
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// 메인 진입점. env + (keychain) + local scan 병렬 → 우선순위 merge → persist.
pub async fn discover(
    registry: &ProviderRegistry,
    _opts: DiscoverOpts,
) -> Result<DiscoverReport, crate::error::LlmError> {
    // 1) env scan
    let env_hits = scan_env(registry);

    // 2) local scan
    let local_hits = scan_local_servers().await;

    // 3) keychain (W7.3 단순화: backend 가 None 이면 empty)
    let keychain_hits: Vec<ProviderId> = crate::auth_keyring::KeyringAuthStore::probe()
        .list()
        .await
        .unwrap_or_default();

    // 4) merge: per provider
    let discovered = merge(registry, &env_hits, &keychain_hits, &local_hits);

    // 5) build chain
    let chain = ActiveProviderChain::from_discovered(discovered);

    // 6) persist
    let path = paths::state_active_providers_toml();
    chain.save(&path).map_err(crate::error::LlmError::from)?;

    Ok(DiscoverReport {
        chain,
        env_hits,
        keychain_hits,
        local_hits,
    })
}

fn scan_env(registry: &ProviderRegistry) -> Vec<EnvVarHit> {
    let mut hits = Vec::new();
    for meta in registry.list() {
        if let Some(var) = &meta.env_var
            && std::env::var(var).is_ok() {
                hits.push(EnvVarHit {
                    provider: meta.id,
                    env_var: var.clone(),
                });
            }
    }
    hits
}

fn merge(
    registry: &ProviderRegistry,
    env: &[EnvVarHit],
    keychain: &[ProviderId],
    local: &[LocalHit],
) -> Vec<DiscoveredProvider> {
    let mut out = Vec::new();

    for meta in registry.list() {
        let m: &ProviderMetadata = meta;
        // env 우선
        if let Some(hit) = env.iter().find(|h| h.provider == m.id) {
            out.push(DiscoveredProvider {
                provider: m.id,
                auth_state: AuthState::EnvVar,
                default_model: m.default_model.clone(),
            });
            let _ = hit; // suppress unused
            continue;
        }
        // keychain
        if keychain.contains(&m.id) {
            out.push(DiscoveredProvider {
                provider: m.id,
                auth_state: AuthState::Keychain,
                default_model: m.default_model.clone(),
            });
            continue;
        }
        // local (local-llm 만)
        if m.id == ProviderId::LocalLlm
            && let Some(_hit) = local.iter().find(|h| h.available) {
                out.push(DiscoveredProvider {
                    provider: m.id,
                    auth_state: AuthState::LocalDetected,
                    default_model: m.default_model.clone(),
                });
            }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn discover_with_no_env_vars_returns_only_local_or_empty() {
        // Rust 2024: env::set_var/remove_var 는 unsafe. 테스트에서는
        // 현재 env 상태 그대로 두고 scan_env 가 그 값을 그대로 본다.
        // 결과는 0~6개 사이.
        let reg = ProviderRegistry::with_builtins();
        let r = discover(&reg, DiscoverOpts::default()).await.unwrap();
        // 최소한 panic 없이 결과 반환
        assert!(r.env_hits.len() <= 6);
        assert!(r.local_hits.len() == 4);
    }

    /// W12 (D-50) — integration test: `discover()` 가 6 provider 의 default `model/base_url` 로 동작.
    /// 실제 network call 은 안 함 (--ignored). real test 는 `MINIMAX_API_KEY` 등 env 주입 후 수동 실행.
    #[tokio::test]
    #[ignore = "requires real network and env var; run manually with MINIMAX_API_KEY set"]
    async fn discover_minimax_integration_smoke() {
        // 가정: env MINIMAX_API_KEY=... 가 설정되어 있어야 함 (CI 환경 아닐 때만)
        let reg = ProviderRegistry::with_builtins();
        let r = discover(&reg, DiscoverOpts::default()).await.unwrap();
        // env_hits 에 minimax 가 있어야 함 (MINIMAX_API_KEY set 가정)
        let has_minimax = r.env_hits.iter().any(|h| h.provider == ProviderId::Minimax);
        assert!(has_minimax, "MINIMAX_API_KEY not detected; set env first");
        // chain 에 minimax primary
        let primary = r.chain.primary().unwrap();
        assert_eq!(primary.provider, ProviderId::Minimax);
    }
}
