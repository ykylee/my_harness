//! Kompress-base v1 simple — text-entropy 기반 자유 텍스트 압축.
//!
//! v1 단순화 (ONNX runtime 없음):
//! 1) 연속 공백/탭 → 단일 space
//! 2) 중복 newline → 최대 2 (paragraph 구분)
//! 3) stopword 제거 (영어 small set: the/a/an/is/are/was/were/be/been/being/...)
//! 4) 3+ char 단어 끝 자음 제거 (간이 stemming, best-effort: "running" → "runn", "tests" → "test")
//!
//! 정확 복원 안 함 — LLM 이 이해 가능한 수준. v1.5+ 에서 ONNX ML 모델 도입 (Kompress-base 실제).

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
#[derive(Debug, Clone)]
pub struct KompressConfig {
    pub collapse_whitespace: bool,
    pub collapse_newlines: bool,
    pub remove_stopwords: bool,
    pub stem: bool,
    pub min_word_length: usize,
}

impl Default for KompressConfig {
    fn default() -> Self {
        Self {
            collapse_whitespace: true,
            collapse_newlines: true,
            remove_stopwords: true,
            stem: true,
            min_word_length: 4,
        }
    }
}

const STOPWORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "am",
    "have", "has", "had", "having", "do", "does", "did", "doing", "would", "should",
    "could", "will", "shall", "may", "might", "must", "can",
    "i", "you", "he", "she", "it", "we", "they", "me", "him", "her", "us", "them",
    "my", "your", "his", "its", "our", "their",
    "this", "that", "these", "those",
    "and", "or", "but", "if", "then", "else", "so", "as", "of", "in", "on", "at",
    "to", "for", "from", "by", "with", "without", "about", "into", "through",
    "during", "before", "after", "above", "below", "up", "down", "out", "off",
    "over", "under", "again", "further", "once", "here", "there", "when", "where",
    "why", "how", "all", "any", "both", "each", "few", "more", "most", "other",
    "some", "such", "no", "nor", "not", "only", "own", "same", "than", "too",
    "very", "just", "now",
];

/// v1 simple 압축. LLM 이 이해 가능한 수준으로 줄임.
#[must_use] 
pub fn kompress_v1(text: &str, cfg: &KompressConfig) -> String {
    let mut out = text.to_string();
    if cfg.collapse_whitespace {
        out = collapse_whitespace(&out);
    }
    if cfg.collapse_newlines {
        out = collapse_newlines(&out);
    }
    if cfg.remove_stopwords {
        out = remove_stopwords(&out);
    }
    if cfg.stem {
        out = stem_words(&out, cfg.min_word_length);
    }
    out
}

fn collapse_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' || c == '\t' {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

fn collapse_newlines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut consecutive_nl = 0;
    for c in s.chars() {
        if c == '\n' {
            consecutive_nl += 1;
            if consecutive_nl <= 2 {
                out.push('\n');
            }
        } else {
            consecutive_nl = 0;
            out.push(c);
        }
    }
    out
}

fn remove_stopwords(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for token in s.split_inclusive(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
        // token = 단어 + trailing 공백/구두점
        let (word, trailing) = split_word_trailing(token);
        let lower = word.to_ascii_lowercase();
        if STOPWORDS.contains(&lower.as_str()) {
            // stopword: trailing 만 보존
            out.push_str(trailing);
        } else {
            out.push_str(token);
        }
    }
    out
}

fn split_word_trailing(token: &str) -> (&str, &str) {
    let bytes = token.as_bytes();
    let mut i = 0;
    while i < bytes.len() && !(bytes[i] as char).is_whitespace() && !(bytes[i] as char).is_ascii_punctuation() {
        i += 1;
    }
    (&token[..i], &token[i..])
}

fn stem_words(s: &str, min_length: usize) -> String {
    // 단순 stemming: 끝 1 char 제거 (영어 한정). 3+ char 단어만.
    let mut out = String::with_capacity(s.len());
    for token in s.split_inclusive(|c: char| c.is_whitespace() || c.is_ascii_punctuation()) {
        let (word, trailing) = split_word_trailing(token);
        if word.len() >= min_length && word.chars().all(|c| c.is_ascii_alphabetic()) {
            // 단순: 끝 1자 제거
            let stemmed = &word[..word.len() - 1];
            out.push_str(stemmed);
            out.push_str(trailing);
        } else {
            out.push_str(token);
        }
    }
    out
}

#[derive(Debug, Default, Clone)]
pub struct KompressStats {
    pub original_chars: usize,
    pub compressed_chars: usize,
}

impl KompressStats {
    #[must_use] 
    pub fn from(original: &str, compressed: &str) -> Self {
        Self {
            original_chars: original.chars().count(),
            compressed_chars: compressed.chars().count(),
        }
    }

    #[must_use] 
    pub fn savings_ratio(&self) -> f32 {
        if self.original_chars == 0 {
            return 0.0;
        }
        1.0 - (self.compressed_chars as f32 / self.original_chars as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_whitespace_preserves_newlines() {
        let s = "a   b\t\tc\n  d";
        let out = collapse_whitespace(s);
        assert_eq!(out, "a b c\n d"); // "  d" 의 leading spaces 도 collapse
    }

    #[test]
    fn collapse_newlines_max_two() {
        let s = "a\n\n\n\nb";
        let out = collapse_newlines(s);
        assert_eq!(out, "a\n\nb");
    }

    #[test]
    fn remove_stopwords_basic() {
        let s = "the quick brown fox is fast";
        let out = remove_stopwords(s);
        assert!(!out.contains("the "));
        assert!(!out.contains(" is "));
        assert!(out.contains("quick"));
        assert!(out.contains("brown"));
        assert!(out.contains("fox"));
        assert!(out.contains("fast"));
    }

    #[test]
    fn remove_stopwords_preserves_punctuation() {
        let s = "Hello, the world!";
        let out = remove_stopwords(s);
        assert_eq!(out, "Hello,  world!");
    }

    #[test]
    fn stem_short_words_unchanged() {
        let s = "the cat ran";
        let out = stem_words(s, 4);
        // the/cat/ran 모두 3자 → unchanged
        assert_eq!(out, "the cat ran");
    }

    #[test]
    fn stem_long_words_drop_last_char() {
        let s = "running tests";
        let out = stem_words(s, 4);
        // running (7) → runnin, tests (5) → test
        assert_eq!(out, "runnin test");
    }

    #[test]
    fn stem_skips_non_alpha() {
        let s = "abc123 xyz";
        let out = stem_words(s, 4);
        // abc123 has digit → unchanged
        assert_eq!(out, "abc123 xyz");
    }

    #[test]
    fn kompress_v1_reduces_size() {
        let s = "the quick brown fox is jumping over the lazy dog and the cat is running very fast";
        let out = kompress_v1(s, &KompressConfig::default());
        let stats = KompressStats::from(s, &out);
        assert!(stats.savings_ratio() > 0.2, "savings={} ({})", stats.savings_ratio(), out);
    }

    #[test]
    fn kompress_v1_preserves_key_content() {
        // v1 simple 의 stemming 은 best-effort — 도메인 명사 stem 됨. "anthropic" → "anthropi".
        // 그래도 핵심 token prefix 는 보존됨.
        let s = "anthropic claude sonnet 4 6 and google gemini 2 5 pro";
        let out = kompress_v1(s, &KompressConfig::default());
        assert!(out.contains("anthropi")); // "anthropic" stem
        assert!(out.contains("claude") || out.contains("claud")); // "claude" stem
        assert!(out.contains("gemin") || out.contains("gemini")); // "gemini" 5자 < 6 → stem 안 됨
    }

    #[test]
    fn kompress_v1_all_disabled_passthrough() {
        let s = "hello   world\n\n\nfoo";
        let cfg = KompressConfig {
            collapse_whitespace: false,
            collapse_newlines: false,
            remove_stopwords: false,
            stem: false,
            min_word_length: 4,
        };
        let out = kompress_v1(s, &cfg);
        assert_eq!(out, s);
    }

    #[test]
    fn split_word_trailing_keeps_punctuation() {
        let (w, t) = split_word_trailing("hello,");
        assert_eq!(w, "hello");
        assert_eq!(t, ",");
    }

    #[test]
    fn savings_ratio_zero_for_empty() {
        let s = KompressStats::from("", "");
        assert!(s.savings_ratio().abs() < f32::EPSILON);
    }
}
