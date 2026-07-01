//! myharness-llm — rig-core integration + provider registry + auth + discover
//!
//! v1 MVP (TASK-005-1 W7).
//!
//! 모듈:
//! - [`provider`]: 6 provider 식별자 + 종류 분류
//! - [`metadata`]: provider 정적 메타데이터
//! - [`registry`]: in-memory provider CRUD + TOML 영속화
//! - [`client`]: provider-agnostic `LLMClient` trait + 메시지 타입
//! - [`client_anthropic`]: rig-core Anthropic 래퍼
//! - [`client_openai_compat`]: `OpenAI` 호환 (DeepSeek/Ollama/local-llm) 래퍼
//! - [`client_gemini`]: rig-core Gemini 래퍼
//! - [`client_mock`]: 테스트용 mock client
//! - [`auth_state`]: per-provider auth 상태 (Unset/EnvVar/Keychain/Manual/LocalDetected/Error)
//! - [`auth_store`]: `AuthStore` trait + in-memory 구현
//! - [`auth_keyring`]: OS keychain 백엔드 (graceful fallback)
//! - [`paths`]: `~/.myharness/` 경로 헬퍼 + `init_home_dir()` §5.12 자동 생성
//! - `error` 모듈: `LlmError` (thiserror)
//! - [`add_local`]: W16 `auth add-local` subcommand 의 register API + probe

pub mod add_local;
pub mod auth_keyring;
pub mod auth_state;
pub mod auth_store;
pub mod chain;
pub mod client;
pub mod client_anthropic;
pub mod client_gemini;
pub mod client_mock;
pub mod client_openai_compat;
pub mod discover;
pub mod error;
pub mod hash8;
pub mod metadata;
pub mod paths;
pub mod provider;
pub mod registry;
pub mod router;
pub mod scan_local;

pub use add_local::{
    ModelInfo, RegisterError, RegisterReport, backup_providers_toml, probe_local_models,
    register_local_provider, register_local_provider_non_interactive,
    register_local_provider_non_interactive_with_store, register_local_provider_with_store,
};
pub use auth_keyring::{KeyringAuthStore, KeyringBackend};
pub use auth_state::{AuthState, AuthStatus};
pub use auth_store::{AuthStore, AuthStoreError, InMemoryAuthStore};
pub use chain::{ActiveProviderChain, ChainEntry, ChainError, ChainSource, DiscoveredProvider};
pub use client::{CompletionRequest, CompletionResponse, LLMClient, Message, Role};
pub use client_anthropic::AnthropicProvider;
pub use client_gemini::GeminiProvider;
pub use client_mock::{MockClient, MockResponse};
pub use client_openai_compat::OpenAiCompatProvider;
pub use discover::{DiscoverOpts, DiscoverReport, EnvVarHit, discover};
pub use error::LlmError;
pub use metadata::{ProviderCapabilities, ProviderMetadata};
pub use paths::init_home_dir;
pub use provider::{ProviderId, ProviderKind};
pub use registry::{ProviderRegistry, RegistryError};
pub use router::{FallbackRouter, ProviderStatus, RouterResponse};

/// Crate 버전.
#[must_use]
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// 자주 같이 쓰이는 re-export.
pub mod prelude {
    pub use crate::auth_keyring::{KeyringAuthStore, KeyringBackend};
    pub use crate::auth_state::{AuthState, AuthStatus};
    pub use crate::auth_store::{AuthStore, AuthStoreError, InMemoryAuthStore};
    pub use crate::chain::{ActiveProviderChain, ChainEntry, ChainSource, DiscoveredProvider};
    pub use crate::client::{CompletionRequest, CompletionResponse, LLMClient, Message, Role};
    pub use crate::client_anthropic::AnthropicProvider;
    pub use crate::client_gemini::GeminiProvider;
    pub use crate::client_mock::{MockClient, MockResponse};
    pub use crate::client_openai_compat::OpenAiCompatProvider;
    pub use crate::discover::{DiscoverOpts, DiscoverReport, EnvVarHit, discover};
    pub use crate::error::LlmError;
    pub use crate::metadata::{ProviderCapabilities, ProviderMetadata};
    pub use crate::provider::{ProviderId, ProviderKind};
    pub use crate::registry::{ProviderRegistry, RegistryError};
    pub use crate::router::{FallbackRouter, ProviderStatus, RouterResponse};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }

    #[test]
    fn prelude_brings_common_types() {
        let _: Option<ProviderId> = Some(ProviderId::Claude);
    }
}
