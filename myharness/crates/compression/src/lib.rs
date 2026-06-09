//! myharness-compression — built-in 압축
//!
//! 모듈:
//! - [`summarizer`]: Summarizer trait + LlmSummarizer + MockSummarizer + TrivialSummarizer (W9.1)
//! - [`ccr`]: CCR (reversible compression with retrieval) (W9.3)
//! - [`kompress`]: Kompress-base v1 simple (W9.4)
//! - [`registry`]: BuiltinAlgorithm registry + BuiltinConfig 통합 (W9.5)

pub mod ccr;
pub mod kompress;
pub mod registry;
pub mod summarizer;

pub use ccr::{CcrStats, CcrStore};
pub use kompress::{kompress_v1, KompressConfig, KompressStats};
pub use registry::{flags_to_map, BuiltinAlgorithm, BuiltinFlags, BuiltinRegistry};
pub use summarizer::{LlmSummarizer, MockSummarizer, Summarizer, SummarizerError, TrivialSummarizer};

/// Crate 버전.
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
