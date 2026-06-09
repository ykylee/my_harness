//! myharness-context — CLAUDE.md load + auto memory + /compact Layer 1 + headroom Layer 2
//!
//! 모듈:
//! - [`claude_md`]: project CLAUDE.md 자동 발견
//! - [`auto_memory`]: 작업 패턴 자동 학습
//! - [`budget`]: token budget + /compact Layer 1
//! - [`compression`]: Layer 2 (CacheAligner + ContentRouter + SmartCrusher + CodeCompressor)
//! - [`config`]: 통합 config + ContextOrchestrator

pub mod auto_memory;
pub mod budget;
pub mod claude_md;
pub mod compression;
pub mod config;

pub use auto_memory::{AutoMemory, MemoryKind, MemoryRecord};
pub use budget::{BudgetConfig, BudgetReport, CompactStrategy, ContextManager, Role, Message as ContextMessage};
pub use claude_md::{ContextLoader, ContextSource, DiscoveredContext};
pub use compression::{detect_content_type, smart_crush, code_compress, BuiltinConfig, BuiltinPipeline, ContentType};
pub use config::{BuiltinLayerConfig, ConfigError, ContextConfig, ContextOrchestrator};

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
