//! `MemoryQuery` builder + 검색 score 정규화 helper.
//!
//! - `MemoryQuery::with_*` 메서드: fluent builder.
//! - `effective_limit`: 기본 20, 상한 1000 (과도한 결과 방지).
//! - `bm25_normalize`: sqlite FTS5 `bm25()` 가 음수를 반환 (낮을수록 좋음) —
//!   우리 컨벤션 (높을수록 좋음) 으로 부호 반전.

use crate::auto_memory::types::{MemoryKind, MemoryQuery};

impl MemoryQuery {
    /// keyword (LIKE / FTS5 match) 추가.
    #[must_use]
    pub fn with_keyword(mut self, kw: impl Into<String>) -> Self {
        self.keyword = Some(kw.into());
        self
    }

    /// kinds 필터 (Vec 전체 매치).
    #[must_use]
    pub fn with_kinds(mut self, kinds: Vec<MemoryKind>) -> Self {
        self.kinds = Some(kinds);
        self
    }

    /// since timestamp (epoch seconds).
    #[must_use]
    pub fn with_since(mut self, since: i64) -> Self {
        self.since = Some(since);
        self
    }

    /// until timestamp (epoch seconds).
    #[must_use]
    pub fn with_until(mut self, until: i64) -> Self {
        self.until = Some(until);
        self
    }

    /// limit (raw — `effective_limit` 가 default + cap 적용).
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// 실제 limit. 미설정이면 20, 1000 초과는 1000 으로 cap.
    #[must_use]
    pub fn effective_limit(&self) -> usize {
        self.limit.unwrap_or(20).min(1000)
    }
}

/// BM25 raw score (sqlite fts5 bm25 함수가 음수 반환) 를
/// 우리 컨벤션 (높을수록 좋음) 으로 정규화.
#[allow(dead_code)]
#[must_use]
pub fn bm25_normalize(raw_bm25: f64) -> f64 {
    -raw_bm25
}
