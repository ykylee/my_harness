//! myharness-auth — keyring + per-provider auth state + OAuth (W13)
//!
//! 모듈:
//! - [`pkce`]: PKCE + state 생성 (RFC 7636)
//! - [`flow`]: OAuth 2.0 Authorization Code + PKCE core (provider-agnostic)
//! - [`callback`]: local HTTP callback server (loopback redirect URI)
//! - [`browser`]: OS 기본 browser 자동 open
//! - [`store`]: token 영구 저장 (`~/.myharness/oauth/{provider}.toml`, chmod 600)
//! - [`provider`]: 3 provider impl (minimax/openai/google)
//! - [`manager`]: 전체 OAuth flow orchestration (authorize → callback → exchange → save)

pub mod browser;
pub mod callback;
pub mod device_flow;
pub mod flow;
pub mod manager;
pub mod pkce;
pub mod provider;
pub mod store;

pub use browser::{open as open_browser, BrowserError};
pub use callback::CallbackServer;
pub use device_flow::{
    poll_token, poll_until_success, request_code, DeviceAuthorization, DeviceCodeProvider, DeviceError,
    DeviceRequest, DeviceToken, TokenPoll,
};
pub use flow::{
    build_authorize_url, exchange_code, refresh_token, AuthorizeRequest, CallbackParams, OAuthError,
    OAuthProvider, OAuthToken,
};
pub use manager::{AuthManager, AuthStatus, LoginOutcome};
pub use provider::{
    find_device_provider, find_provider, GoogleOAuth, MinimaxDeviceOAuth, MinimaxOAuth, OpenAiOAuth,
    OAUTH_PROVIDERS,
};
pub use pkce::{generate_pkce, generate_state, PkceMethod, PkcePair};
pub use store::{StoreError, StoredToken, TokenStore};

/// Crate 버전.
#[must_use] 
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }
}
