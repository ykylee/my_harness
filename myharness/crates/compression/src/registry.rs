//! BuiltinAlgorithm registry — 6 알고리즘 (CacheAligner/ContentRouter/SmartCrusher/CodeCompressor/Ccr/KompressBase)
//! 중 어떤 게 enabled 인지 통합 관리.
//!
//! Note: W8.4 에서 context 의 compression 모듈에 CacheAligner/ContentRouter/SmartCrusher/CodeCompressor 4종
//! 구현됨. CCR + KompressBase 는 W9.3/W9.4 에서 compression crate 의 신규 모듈.
//! W9.5 에서 두 crate 의 알고리즘을 단일 `BuiltinAlgorithm` enum 으로 통합 view 제공.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ccr::CcrStore;
use crate::kompress::{kompress_v1, KompressConfig};

/// W9.5 — 단일 알고리즘 식별자. CONCEPT §5.6 의 6 알고리즘.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuiltinAlgorithm {
    CacheAligner,
    ContentRouter,
    SmartCrusher,
    CodeCompressor,
    Ccr,
    KompressBase,
}

impl BuiltinAlgorithm {
    pub const ALL: &'static [BuiltinAlgorithm] = &[
        BuiltinAlgorithm::CacheAligner,
        BuiltinAlgorithm::ContentRouter,
        BuiltinAlgorithm::SmartCrusher,
        BuiltinAlgorithm::CodeCompressor,
        BuiltinAlgorithm::Ccr,
        BuiltinAlgorithm::KompressBase,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            BuiltinAlgorithm::CacheAligner => "cache-aligner",
            BuiltinAlgorithm::ContentRouter => "content-router",
            BuiltinAlgorithm::SmartCrusher => "smart-crusher",
            BuiltinAlgorithm::CodeCompressor => "code-compressor",
            BuiltinAlgorithm::Ccr => "ccr",
            BuiltinAlgorithm::KompressBase => "kompress-base",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cache-aligner" => Some(BuiltinAlgorithm::CacheAligner),
            "content-router" => Some(BuiltinAlgorithm::ContentRouter),
            "smart-crusher" => Some(BuiltinAlgorithm::SmartCrusher),
            "code-compressor" => Some(BuiltinAlgorithm::CodeCompressor),
            "ccr" => Some(BuiltinAlgorithm::Ccr),
            "kompress-base" => Some(BuiltinAlgorithm::KompressBase),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BuiltinFlags {
    pub cache_aligner: bool,
    pub content_router: bool,
    pub smart_crusher: bool,
    pub code_compressor: bool,
    pub ccr: bool,
    pub kompress_base: bool,
}

impl BuiltinFlags {
    pub fn from_slice(enabled: &[BuiltinAlgorithm]) -> Self {
        let mut f = Self::default();
        for a in enabled {
            match a {
                BuiltinAlgorithm::CacheAligner => f.cache_aligner = true,
                BuiltinAlgorithm::ContentRouter => f.content_router = true,
                BuiltinAlgorithm::SmartCrusher => f.smart_crusher = true,
                BuiltinAlgorithm::CodeCompressor => f.code_compressor = true,
                BuiltinAlgorithm::Ccr => f.ccr = true,
                BuiltinAlgorithm::KompressBase => f.kompress_base = true,
            }
        }
        f
    }

    pub fn enabled_list(&self) -> Vec<BuiltinAlgorithm> {
        let mut v = Vec::new();
        if self.cache_aligner {
            v.push(BuiltinAlgorithm::CacheAligner);
        }
        if self.content_router {
            v.push(BuiltinAlgorithm::ContentRouter);
        }
        if self.smart_crusher {
            v.push(BuiltinAlgorithm::SmartCrusher);
        }
        if self.code_compressor {
            v.push(BuiltinAlgorithm::CodeCompressor);
        }
        if self.ccr {
            v.push(BuiltinAlgorithm::Ccr);
        }
        if self.kompress_base {
            v.push(BuiltinAlgorithm::KompressBase);
        }
        v
    }
}

/// Layer 2 (W9.5) — CCR + KompressBase 의 통합 registry.
/// W8.4 의 CacheAligner/ContentRouter/SmartCrusher/CodeCompressor 는 context crate 에서 처리.
pub struct BuiltinRegistry {
    pub ccr: CcrStore,
    pub kompress_config: KompressConfig,
    pub flags: BuiltinFlags,
}

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self {
            ccr: CcrStore::new(),
            kompress_config: KompressConfig::default(),
            flags: BuiltinFlags::default(),
        }
    }
}

impl BuiltinRegistry {
    pub fn new(flags: BuiltinFlags) -> Self {
        Self {
            flags,
            ..Default::default()
        }
    }

    /// CCR 활성화 시 압축. in-place: ccr store 에 marker 등록.
    pub fn compress_with_ccr(&mut self, text: &str, min_length: usize) -> (String, crate::CcrStats) {
        self.ccr.compress(text, min_length)
    }

    /// CCR 로 압축된 text 복원.
    pub fn expand_with_ccr(&self, compressed: &str) -> String {
        self.ccr.expand(compressed)
    }

    /// Kompress-base v1 simple.
    pub fn kompress(&self, text: &str) -> String {
        kompress_v1(text, &self.kompress_config)
    }

    pub fn is_enabled(&self, algo: BuiltinAlgorithm) -> bool {
        match algo {
            BuiltinAlgorithm::CacheAligner => self.flags.cache_aligner,
            BuiltinAlgorithm::ContentRouter => self.flags.content_router,
            BuiltinAlgorithm::SmartCrusher => self.flags.smart_crusher,
            BuiltinAlgorithm::CodeCompressor => self.flags.code_compressor,
            BuiltinAlgorithm::Ccr => self.flags.ccr,
            BuiltinAlgorithm::KompressBase => self.flags.kompress_base,
        }
    }
}

/// `flags` 를 HashMap 으로 표현 (TOML 호환).
pub fn flags_to_map(flags: &BuiltinFlags) -> HashMap<&'static str, bool> {
    let mut m = HashMap::new();
    m.insert("cache-aligner", flags.cache_aligner);
    m.insert("content-router", flags.content_router);
    m.insert("smart-crusher", flags.smart_crusher);
    m.insert("code-compressor", flags.code_compressor);
    m.insert("ccr", flags.ccr);
    m.insert("kompress-base", flags.kompress_base);
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_algorithms() {
        assert_eq!(BuiltinAlgorithm::ALL.len(), 6);
    }

    #[test]
    fn as_str_roundtrip() {
        for a in BuiltinAlgorithm::ALL {
            assert_eq!(BuiltinAlgorithm::from_str(a.as_str()), Some(*a));
        }
    }

    #[test]
    fn unknown_str_returns_none() {
        assert_eq!(BuiltinAlgorithm::from_str("nonexistent"), None);
    }

    #[test]
    fn flags_from_slice() {
        let f = BuiltinFlags::from_slice(&[BuiltinAlgorithm::Ccr, BuiltinAlgorithm::KompressBase]);
        assert!(f.ccr);
        assert!(f.kompress_base);
        assert!(!f.cache_aligner);
    }

    #[test]
    fn flags_enabled_list_roundtrip() {
        let f = BuiltinFlags::from_slice(&[
            BuiltinAlgorithm::CacheAligner,
            BuiltinAlgorithm::SmartCrusher,
        ]);
        let list = f.enabled_list();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&BuiltinAlgorithm::CacheAligner));
        assert!(list.contains(&BuiltinAlgorithm::SmartCrusher));
    }

    #[test]
    fn registry_is_enabled() {
        let f = BuiltinFlags::from_slice(&[BuiltinAlgorithm::Ccr]);
        let r = BuiltinRegistry::new(f);
        assert!(r.is_enabled(BuiltinAlgorithm::Ccr));
        assert!(!r.is_enabled(BuiltinAlgorithm::KompressBase));
    }

    #[test]
    fn registry_compress_with_ccr() {
        let f = BuiltinFlags::from_slice(&[BuiltinAlgorithm::Ccr]);
        let mut r = BuiltinRegistry::new(f);
        let (compressed, stats) = r.compress_with_ccr("anthropic_claude_sonnet_4_6 here", 20);
        assert!(compressed.contains("{ccr:0}"));
        assert!(stats.markers > 0);
    }

    #[test]
    fn registry_expand_with_ccr() {
        let f = BuiltinFlags::from_slice(&[BuiltinAlgorithm::Ccr]);
        let mut r = BuiltinRegistry::new(f);
        let original = "anthropic_claude_sonnet_4_6 and google_gemini_2_5_pro";
        let (compressed, _) = r.compress_with_ccr(original, 20);
        let expanded = r.expand_with_ccr(&compressed);
        assert_eq!(expanded, original);
    }

    #[test]
    fn registry_kompress_reduces_size() {
        let r = BuiltinRegistry::default();
        let original = "the quick brown fox is jumping over the lazy dog";
        let out = r.kompress(original);
        let stats = crate::kompress::KompressStats::from(original, &out);
        assert!(stats.savings_ratio() > 0.1);
    }

    #[test]
    fn flags_to_map_all_keys_present() {
        let f = BuiltinFlags::default();
        let m = flags_to_map(&f);
        assert_eq!(m.len(), 6);
        assert!(m.contains_key("cache-aligner"));
        assert!(m.contains_key("kompress-base"));
    }
}
