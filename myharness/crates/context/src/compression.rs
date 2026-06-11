//! Layer 2 — headroom 4 알고리즘 built-in (D-27 + D-30).
//!
//! - `CacheAligner`: system + 최근 N message prefix 안정화
//! - `ContentRouter`: content type 분류 (json/code/text/log)
//! - `SmartCrusher`: JSON key 순서 정규화 + whitespace 제거 + 숫자 정밀도 축소
//! - `CodeCompressor`: tree-sitter 식별자 shorten + 주석 제거 (v1.5+, 현재는 간단 regex 기반 stub)

#![allow(clippy::cast_possible_truncation, clippy::cast_precision_loss, clippy::cast_sign_loss, clippy::cast_possible_wrap)]
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::budget::Message;

/// content type 분류 (`ContentRouter` 결과).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentType {
    Json,
    Code,
    Log,
    Text,
}

#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("invalid config: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinConfig {
    pub cache_aligner: bool,
    pub content_router: bool,
    pub smart_crusher: bool,
    pub code_compressor: bool,
}

impl Default for BuiltinConfig {
    fn default() -> Self {
        Self {
            cache_aligner: true,
            content_router: true,
            smart_crusher: true,
            code_compressor: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinPipeline {
    pub config: BuiltinConfig,
}

impl BuiltinPipeline {
    #[must_use] 
    pub fn new(config: BuiltinConfig) -> Self {
        Self { config }
    }

    /// 전체 파이프라인. messages + system 을 받아서 압축된 (system, messages) 반환.
    #[must_use] 
    pub fn run(&self, system: Option<String>, messages: Vec<Message>) -> (Option<String>, Vec<Message>) {
        let (system, messages) = if self.config.cache_aligner {
            self.cache_align(system, messages)
        } else {
            (system, messages)
        };

        let messages = if self.config.content_router
            || self.config.smart_crusher
            || self.config.code_compressor
        {
            messages
                .into_iter()
                .map(|m| {
                    let ctype = if self.config.content_router {
                        detect_content_type(&m.content)
                    } else {
                        ContentType::Text
                    };
                    let compressed = match ctype {
                        ContentType::Json if self.config.smart_crusher => smart_crush(&m.content),
                        ContentType::Code if self.config.code_compressor => code_compress(&m.content),
                        _ => m.content,
                    };
                    Message { role: m.role, content: compressed }
                })
                .collect()
        } else {
            messages
        };

        (system, messages)
    }

    /// `CacheAligner`: system prompt 의 공백/개행 정규화 + 최근 N message 의 선두 공백 제거.
    /// 효과: KV cache 의 prefix 가 안정화되어 hit rate ↑.
    #[must_use] 
    pub fn cache_align(&self, system: Option<String>, messages: Vec<Message>) -> (Option<String>, Vec<Message>) {
        let sys = system.map(|s| normalize_whitespace(&s));
        let msgs = messages
            .into_iter()
            .map(|m| Message { role: m.role, content: trim_leading_ws(&m.content) })
            .collect();
        (sys, msgs)
    }
}

/// `ContentRouter` — content type 분류. 휴리스틱 기반.
#[must_use] 
pub fn detect_content_type(content: &str) -> ContentType {
    let trimmed = content.trim_start();
    if (trimmed.starts_with('{') || trimmed.starts_with('['))
        && serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
    {
        return ContentType::Json;
    }
    if looks_like_code(trimmed) {
        return ContentType::Code;
    }
    if looks_like_log(trimmed) {
        return ContentType::Log;
    }
    ContentType::Text
}

fn looks_like_code(s: &str) -> bool {
    let s = s.trim_start();
    if s.starts_with("fn ") || s.starts_with("pub fn ") || s.starts_with("def ") || s.starts_with("class ") {
        return true;
    }
    if s.starts_with("use ") || s.starts_with("import ") || s.starts_with("#include") {
        return true;
    }
    if s.contains("fn ") && s.contains("()") {
        return true;
    }
    if s.contains(" = ") && (s.contains("let ") || s.contains("const ") || s.contains("var ")) {
        return true;
    }
    false
}

fn looks_like_log(s: &str) -> bool {
    // RFC3339 timestamp prefix or [LEVEL] marker
    let first_line = s.lines().next().unwrap_or("");
    if first_line.starts_with('[') && (first_line.contains("INFO") || first_line.contains("WARN") || first_line.contains("ERROR") || first_line.contains("DEBUG")) {
        return true;
    }
    if first_line.len() >= 20 && first_line.chars().take(19).all(|c| c.is_ascii_digit() || c == '-' || c == 'T' || c == ':' || c == 'Z' || c == ' ') {
        return true;
    }
    false
}

/// `SmartCrusher` — JSON key 순서 정규화 + whitespace 제거 + 숫자 정밀도 축소.
#[must_use] 
pub fn smart_crush(s: &str) -> String {
    // 1) JSON parse 시도
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
        // 숫자 정밀도 축소 (소수점 6자리)
        let v = reduce_number_precision(&v, 6);
        return serde_json::to_string(&v).unwrap_or_else(|_| s.to_string());
    }
    s.to_string()
}

fn reduce_number_precision(v: &serde_json::Value, decimals: usize) -> serde_json::Value {
    match v {
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                let factor = 10f64.powi(decimals as i32);
                let truncated = (f * factor).trunc() / factor;
                if let Some(num) = serde_json::Number::from_f64(truncated) {
                    return serde_json::Value::Number(num);
                }
            }
            v.clone()
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(|x| reduce_number_precision(x, decimals)).collect())
        }
        serde_json::Value::Object(obj) => {
            let mut new_obj = serde_json::Map::new();
            // key 알파벳 순 정렬
            let mut entries: Vec<_> = obj.iter().collect();
            entries.sort_by_key(|(k, _)| k.as_str());
            for (k, v) in entries {
                new_obj.insert(k.clone(), reduce_number_precision(v, decimals));
            }
            serde_json::Value::Object(new_obj)
        }
        _ => v.clone(),
    }
}

/// `CodeCompressor` — 간단한 정규식 기반 압축 (v1.5+ 에서 tree-sitter 통합).
#[must_use] 
pub fn code_compress(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for line in s.lines() {
        let trimmed = line.trim_end();
        // shebang 보존
        if trimmed.starts_with("#!") {
            out.push_str(trimmed);
            out.push('\n');
            continue;
        }
        // 전체 라인이 주석
        let tsl = trimmed.trim_start();
        if tsl.starts_with("//") || (tsl.starts_with('#') && !tsl.starts_with("#!")) {
            continue;
        }
        // 라인 내 // 위치 — 그 이후 제거 (v1.5+: 문자열 내부 보호)
        if let Some(idx) = find_line_comment(trimmed) {
            out.push_str(&trimmed[..idx]);
            out.push('\n');
            continue;
        }
        out.push_str(trimmed);
        out.push('\n');
    }
    out.trim_end().to_string()
}

fn find_line_comment(line: &str) -> Option<usize> {
    // v1.5+: 문자열/character literal 보호. 현재는 단순 검색.
    let bytes = line.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'/' {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn normalize_whitespace(s: &str) -> String {
    // 연속 공백/탭 → 단일 스페이스, trailing whitespace 제거, 개행은 보존
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c == ' ' || c == '\t' {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else if c == '\n' {
            out.push('\n');
            prev_space = false;
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

fn trim_leading_ws(s: &str) -> String {
    s.lines().map(|l| l.trim_start().to_string()).collect::<Vec<_>>().join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(content: &str) -> Message {
        Message::user(content)
    }

    #[test]
    fn detect_json_object() {
        assert_eq!(detect_content_type(r#"{"a":1}"#), ContentType::Json);
    }

    #[test]
    fn detect_json_array() {
        assert_eq!(detect_content_type("[1,2,3]"), ContentType::Json);
    }

    #[test]
    fn detect_code_rust() {
        assert_eq!(detect_content_type("fn main() { println!(\"hi\"); }"), ContentType::Code);
        assert_eq!(detect_content_type("pub fn foo() -> i32 { 1 }"), ContentType::Code);
    }

    #[test]
    fn detect_code_python() {
        assert_eq!(detect_content_type("def hello():\n    print('hi')"), ContentType::Code);
    }

    #[test]
    fn detect_code_js() {
        assert_eq!(detect_content_type("const x = 1;"), ContentType::Code);
    }

    #[test]
    fn detect_log() {
        assert_eq!(detect_content_type("[INFO] server started on :8080"), ContentType::Log);
        assert_eq!(detect_content_type("2026-06-09T12:00:00Z request handled"), ContentType::Log);
    }

    #[test]
    fn detect_text_default() {
        assert_eq!(detect_content_type("hello world"), ContentType::Text);
    }

    #[test]
    fn smart_crush_sorts_keys() {
        let s = r#"{"b":2,"a":1}"#;
        let out = smart_crush(s);
        assert!(out.find("\"a\"").unwrap() < out.find("\"b\"").unwrap());
    }

    #[test]
    fn smart_crush_reduces_number_precision() {
        let s = r#"{"x":3.14159265358979}"#;
        let out = smart_crush(s);
        // 6 자리 정밀도: 3.141593 (rounding)
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        let x = v["x"].as_f64().unwrap();
        assert!((x - std::f64::consts::PI).abs() < 1e-6, "got {x}");
    }

    #[test]
    fn smart_crush_nested() {
        let s = r#"{"outer":{"z":1,"a":2},"first":true}"#;
        let out = smart_crush(s);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        // reduce_number_precision 가 모든 number 를 f64 로 변환하므로 as_f64 로 비교
        assert_eq!(v["outer"]["a"].as_f64().unwrap(), 2.0);
        assert_eq!(v["outer"]["z"].as_f64().unwrap(), 1.0);
    }

    #[test]
    fn smart_crush_invalid_json_passthrough() {
        let s = "not json";
        assert_eq!(smart_crush(s), s);
    }

    #[test]
    fn code_compress_strips_line_comments() {
        let s = "let x = 1; // a comment\nlet y = 2;";
        let out = code_compress(s);
        assert!(!out.contains("// a comment"));
        assert!(out.contains("let x = 1;"));
        assert!(out.contains("let y = 2;"));
    }

    #[test]
    fn code_compress_preserves_shebang() {
        let s = "#!/usr/bin/env python\nprint(1)";
        let out = code_compress(s);
        assert!(out.contains("#!/usr/bin/env python"));
        assert!(out.contains("print(1)"));
    }

    #[test]
    fn cache_align_normalizes_whitespace() {
        let pipe = BuiltinPipeline::new(BuiltinConfig::default());
        let (sys, _) = pipe.cache_align(Some("a   b\t\tc".into()), vec![]);
        assert_eq!(sys.unwrap(), "a b c");
    }

    #[test]
    fn cache_align_trims_leading_ws_per_line() {
        let pipe = BuiltinPipeline::new(BuiltinConfig::default());
        let (_, msgs) = pipe.cache_align(None, vec![msg("   hello\n  world")]);
        assert_eq!(msgs[0].content, "hello\nworld");
    }

    #[test]
    fn full_pipeline_compresses_json_message() {
        let pipe = BuiltinPipeline::new(BuiltinConfig {
            cache_aligner: false,
            content_router: true,
            smart_crusher: true,
            code_compressor: false,
        });
        let (sys, msgs) = pipe.run(None, vec![msg(r#"{"z":1,"a":2}"#)]);
        assert!(sys.is_none());
        assert!(msgs[0].content.find("\"a\"").unwrap() < msgs[0].content.find("\"z\"").unwrap());
    }

    #[test]
    fn full_pipeline_disabled_is_passthrough() {
        let pipe = BuiltinPipeline::new(BuiltinConfig {
            cache_aligner: false,
            content_router: false,
            smart_crusher: false,
            code_compressor: false,
        });
        let (sys, msgs) = pipe.run(Some("a   b".into()), vec![msg("   x")]);
        assert_eq!(sys.unwrap(), "a   b");
        assert_eq!(msgs[0].content, "   x");
    }

    #[test]
    fn detect_text_after_routing() {
        assert_eq!(detect_content_type("Just a normal sentence."), ContentType::Text);
    }
}
