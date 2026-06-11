//! CCR (Compress-Cache-Retrieve) — reversible + retrieval
//!
//! v1 simple: in-memory dictionary 로 {`marker_N`} ↔ `original_text` 매핑.
//! 압축: 특정 threshold (예: 30+ char) 단어/구를 `{marker_N}` 로 치환.
//! 복원: marker → `original_text` lookup.
//!
//! v1.5+: persistence (`SQLite`), LLM-based segment selection, round-trip 비용 trade-off.

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct CcrStore {
    inner: Mutex<CcrInner>,
}

#[derive(Debug, Default)]
struct CcrInner {
    next_id: u32,
    /// marker id → original text
    forward: HashMap<u32, String>,
    /// marker text (예: "{ccr:0}") → marker id
    reverse: HashMap<String, u32>,
    /// original text (lowercase) → marker id (중복 방지; optional)
    dedup: HashMap<String, u32>,
}

impl CcrStore {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use] 
    pub fn marker_format() -> &'static str {
        "{ccr:%d}"
    }

    fn marker_for(id: u32) -> String {
        format!("{{ccr:{id}}}")
    }

    ///
    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    /// original text 등록. marker 텍스트 반환. 중복이면 기존 marker.
    pub fn intern(&self, text: &str) -> String {
        let mut g = self.inner.lock().unwrap();
        if let Some(&id) = g.dedup.get(text) {
            return Self::marker_for(id);
        }
        let id = g.next_id;
        g.next_id += 1;
        let marker = Self::marker_for(id);
        g.forward.insert(id, text.to_string());
        g.reverse.insert(marker.clone(), id);
        g.dedup.insert(text.to_string(), id);
        marker
    }

    ///
    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    /// marker → original 복원. 없으면 None.
    pub fn retrieve(&self, marker: &str) -> Option<String> {
        let g = self.inner.lock().unwrap();
        g.reverse.get(marker).and_then(|id| g.forward.get(id).cloned())
    }

    /// 압축: `min_length` 이상이고 알파벳/숫자 위주 segment 만 marker 화.
    /// segment 분리: 공백 기준. punctuation 포함 단어도 통째로.
    pub fn compress(&self, text: &str, min_length: usize) -> (String, CcrStats) {
        let mut out = String::with_capacity(text.len());
        let mut stats = CcrStats::default();
        let mut current = String::new();
        for c in text.chars() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                current.push(c);
            } else {
                if current.len() >= min_length && is_compressable_token(&current) {
                    let marker = self.intern(&current);
                    stats.original_chars += current.len();
                    stats.compressed_chars += marker.len();
                    stats.markers += 1;
                    out.push_str(&marker);
                } else {
                    out.push_str(&current);
                }
                current.clear();
                out.push(c);
            }
        }
        if !current.is_empty() {
            if current.len() >= min_length && is_compressable_token(&current) {
                let marker = self.intern(&current);
                stats.original_chars += current.len();
                stats.compressed_chars += marker.len();
                stats.markers += 1;
                out.push_str(&marker);
            } else {
                out.push_str(&current);
            }
        }
        (out, stats)
    }

    /// 압축된 text 의 모든 marker 를 original 로 복원.
    pub fn expand(&self, compressed: &str) -> String {
        let mut out = String::with_capacity(compressed.len() * 2);
        let bytes = compressed.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'{' {
                // marker 후보: {ccr:N}
                if let Some(end_rel) = find_marker_end(&bytes[i..]) {
                    let marker = &compressed[i..i + end_rel];
                    if let Some(orig) = self.retrieve(marker) {
                        out.push_str(&orig);
                        i += end_rel;
                        continue;
                    }
                }
            }
            out.push(bytes[i] as char);
            i += 1;
        }
        out
    }

    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().forward.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn clear(&self) {
        let mut g = self.inner.lock().unwrap();
        g.next_id = 0;
        g.forward.clear();
        g.reverse.clear();
        g.dedup.clear();
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct CcrStats {
    pub markers: usize,
    pub original_chars: usize,
    pub compressed_chars: usize,
}

impl CcrStats {
    #[must_use] 
    pub fn savings_ratio(&self) -> f32 {
        if self.original_chars == 0 {
            return 0.0;
        }
        1.0 - (self.compressed_chars as f32 / self.original_chars as f32)
    }
}

fn is_compressable_token(s: &str) -> bool {
    // 단일 char 반복, 숫자만, 너무 흔한 short word 는 skip
    if s.is_empty() {
        return false;
    }
    if s.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if s.chars().all(|c| c == s.chars().next().unwrap()) {
        return false; // "aaaa" 같은
    }
    true
}

fn find_marker_end(bytes: &[u8]) -> Option<usize> {
    // "{ccr:N}" 패턴
    if bytes.len() < 7 {
        return None;
    }
    if &bytes[0..5] != b"{ccr:" {
        return None;
    }
    let mut i = 5;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'}' {
        Some(i + 1)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intern_and_retrieve_roundtrip() {
        let s = CcrStore::new();
        let m = s.intern("anthropic_claude_sonnet_4_6");
        assert_eq!(m, "{ccr:0}");
        assert_eq!(s.retrieve(&m).unwrap(), "anthropic_claude_sonnet_4_6");
    }

    #[test]
    fn intern_dedups() {
        let s = CcrStore::new();
        let m1 = s.intern("hello_world_long");
        let m2 = s.intern("hello_world_long");
        assert_eq!(m1, m2);
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn distinct_text_gets_distinct_markers() {
        let s = CcrStore::new();
        let m1 = s.intern("alpha_long_string");
        let m2 = s.intern("beta_long_string");
        assert_ne!(m1, m2);
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn compress_replaces_long_tokens() {
        let s = CcrStore::new();
        let input = "see anthropic_claude_sonnet_4_6 docs for details";
        let (out, stats) = s.compress(input, 20);
        assert!(out.contains("{ccr:0}"));
        assert!(!out.contains("anthropic_claude_sonnet_4_6"));
        assert_eq!(stats.markers, 1);
        assert!(stats.savings_ratio() > 0.0);
    }

    #[test]
    fn compress_skips_short_tokens() {
        let s = CcrStore::new();
        let (out, _stats) = s.compress("hi hi hi", 20);
        assert_eq!(out, "hi hi hi");
    }

    #[test]
    fn compress_skips_pure_digits() {
        let s = CcrStore::new();
        let (out, stats) = s.compress("count is 1234567890 here", 5);
        assert!(out.contains("1234567890"));
        // "count" + "1234567890" + "here" — "count"(5)≥5 marker, "here"(4)<5 skip, digits skip
        // 즉 marker 는 "count" 만
        assert!(!out.contains("count"));
        assert_eq!(stats.markers, 1);
    }

    #[test]
    fn expand_restores_originals() {
        let s = CcrStore::new();
        let original = "see anthropic_claude_sonnet_4_6 docs for more details";
        let (compressed, _) = s.compress(original, 20);
        let expanded = s.expand(&compressed);
        assert_eq!(expanded, original);
    }

    #[test]
    fn expand_unknown_marker_passthrough() {
        let s = CcrStore::new();
        let out = s.expand("hello {ccr:99} world");
        assert_eq!(out, "hello {ccr:99} world");
    }

    #[test]
    fn clear_resets_state() {
        let s = CcrStore::new();
        s.intern("test_long_token");
        assert_eq!(s.len(), 1);
        s.clear();
        assert_eq!(s.len(), 0);
        let m = s.intern("test_long_token");
        assert_eq!(m, "{ccr:0}"); // 다시 0번
    }

    #[test]
    fn multiple_markers_in_one_text() {
        let s = CcrStore::new();
        let input = "anthropic_claude_sonnet_4_6 and google_gemini_2_5_pro are models";
        let (compressed, stats) = s.compress(input, 20);
        assert_eq!(stats.markers, 2);
        let expanded = s.expand(&compressed);
        assert_eq!(expanded, input);
    }

    #[test]
    fn savings_ratio_correct() {
        let s = CcrStore::new();
        let (_, stats) = s.compress("aaaaaaaaaaaaaaaaaaaaaa", 5);
        // 22 'a' char, marker {ccr:0} = 7 chars → savings = 1 - 7/22 ≈ 0.68
        // BUT: is_compressable_token 이 "aaaa.." 같은 단일 char 반복을 skip
        // 따라서 markers=0, savings=0
        assert_eq!(stats.markers, 0);
    }
}
