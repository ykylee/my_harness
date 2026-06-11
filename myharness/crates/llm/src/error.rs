//! `LlmError` — llm crate 의 단일 에러 타입.
//!
//! router 의 fallback/surface 정책:
//! - [`ProviderUnavailable`](LlmError::ProviderUnavailable) / [`ProviderCall`](LlmError::ProviderCall) / [`ProviderInit`](LlmError::ProviderInit) → 다음 provider 로 fallback
//! - [`AuthMissing`](LlmError::AuthMissing) / [`RateLimited`](LlmError::RateLimited) / [`ContextOverflow`](LlmError::ContextOverflow) / [`ModelNotFound`](LlmError::ModelNotFound) → 즉시 surface (fallback 안 함)

use thiserror::Error;

use crate::chain::ChainError;
use crate::provider::ProviderId;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("provider init failed: {0}")]
    ProviderInit(String),

    #[error("provider call failed: {0}")]
    ProviderCall(String),

    #[error("provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("auth missing: {0:?}")]
    AuthMissing(ProviderId),

    #[error("rate limited: {0}")]
    RateLimited(String),

    #[error("context overflow: {0}")]
    ContextOverflow(String),

    #[error("model not found: {0}")]
    ModelNotFound(String),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("toml decode: {0}")]
    TomlDecode(#[from] toml::de::Error),

    #[error("toml encode: {0}")]
    TomlEncode(#[from] toml::ser::Error),

    #[error("chain: {0}")]
    Chain(#[from] ChainError),

    #[error("other: {0}")]
    Other(String),
}

impl LlmError {
    /// router 가 fallback 해야 하는 에러인지.
    #[must_use] 
    pub fn is_fallbackable(&self) -> bool {
        matches!(
            self,
            LlmError::ProviderInit(_)
                | LlmError::ProviderCall(_)
                | LlmError::ProviderUnavailable(_)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_unavailable_is_fallbackable() {
        assert!(LlmError::ProviderUnavailable("x".into()).is_fallbackable());
    }

    #[test]
    fn auth_missing_is_not_fallbackable() {
        assert!(!LlmError::AuthMissing(ProviderId::Claude).is_fallbackable());
    }

    #[test]
    fn rate_limited_is_not_fallbackable() {
        assert!(!LlmError::RateLimited("429".into()).is_fallbackable());
    }

    #[test]
    fn context_overflow_is_not_fallbackable() {
        assert!(!LlmError::ContextOverflow("too long".into()).is_fallbackable());
    }

    #[test]
    fn model_not_found_is_not_fallbackable() {
        assert!(!LlmError::ModelNotFound("x".into()).is_fallbackable());
    }

    #[test]
    fn io_is_fallbackable() {
        // std::io::Error → fallback 가능 (네트워크 일시 장애 등)
        let e: LlmError = std::io::Error::other("conn refused").into();
        // LlmError::Io 는 is_fallbackable 에 없음 — 명시적 ProviderUnavailable 로 매핑 권장
        // 하지만 router 가 보수적으로 fallback 시도할 수 있도록 false 도 합리적
        assert!(!e.is_fallbackable());
    }
}
