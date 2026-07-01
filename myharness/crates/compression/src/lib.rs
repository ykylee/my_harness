//! myharness-compression — built-in 압축
//!
//! 모듈:
//! - [`summarizer`]: Summarizer trait + `LlmSummarizer` + `MockSummarizer` + `TrivialSummarizer` (W9.1)
//! - [`ccr`]: CCR (reversible compression with retrieval) (W9.3)

#![allow(clippy::struct_excessive_bools)] // 3-4 bool fields are appropriate for these state structs
//! - [`kompress`]: Kompress-base v1 simple (W9.4)
//! - [`registry`]: `BuiltinAlgorithm` registry + `BuiltinConfig` 통합 (W9.5)
//! - [`onnx_model`]: v2.0 ONNX `ModelManager` (tract 0.23 + all-MiniLM-L6-v2) (W23, D-67)

pub mod ccr;
pub mod kompress;
pub mod onnx_model;
pub mod registry;
pub mod summarizer;

pub use ccr::{CcrStats, CcrStore};
pub use kompress::{KompressConfig, KompressStats, kompress_v1};
pub use onnx_model::{ModelInfo, ModelManager};
pub use registry::{BuiltinAlgorithm, BuiltinFlags, BuiltinRegistry, flags_to_map};
pub use summarizer::{
    LlmSummarizer, MockSummarizer, Summarizer, SummarizerError, TrivialSummarizer,
};

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
