//! Auto memory — domain types (TASK-005-2 v2.0 Plugin, Layer 1).
//!
//! 이 모듈은 **순수 타입** 만을 정의한다. I/O, trait, facade 는 sibling 모듈에 있다.
//!
//! - [`MemoryKind`]: record 분류 (Tool / Agent / Command / Note / Error).
//! - [`MemoryRecord`]: 저장 단위 (timestamp + kind + payload + optional summary/tags).
//! - [`MemoryQuery`]: 검색 질의 (builder pattern, [`super::query`]).
//! - [`MemoryHit`]: 검색 결과 (record + score + matched terms).
//! - [`MemoryError`]: 모든 backend 공통 에러.
//!
//! ## on-disk format 안정성
//! `MemoryRecord::summary` 와 `tags` 는 `skip_serializing_if` 로 optional 이라
//! 기존 NDJSON file (summary/tags 없음) 은 그대로 읽힌다 — backward compatible.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Record 분류. `kebab-case` 직렬화 (`"tool"`, `"agent"`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryKind {
    Tool,
    Agent,
    Command,
    Note,
    Error,
}

/// 저장 단위. `payload` 는 임의 JSON (tool args / note text / error message 등).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub timestamp: DateTime<Utc>,
    pub kind: MemoryKind,
    pub payload: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// 검색 질의. 모든 필드 optional — builder pattern 으로 채운다
/// ([`super::query`] 의 `with_*` 메서드 참조).
#[derive(Debug, Clone, Default)]
pub struct MemoryQuery {
    pub keyword: Option<String>,
    pub kinds: Option<Vec<MemoryKind>>,
    pub since: Option<i64>,
    pub until: Option<i64>,
    pub limit: Option<usize>,
}

/// 검색 결과. `score` 는 높을수록 좋음 (sqlite BM25 는 음수 → [`super::query::bm25_normalize`]).
#[derive(Debug, Clone)]
pub struct MemoryHit {
    pub record: MemoryRecord,
    pub score: f64,
    pub matched_terms: Vec<String>,
}

/// 모든 backend 공통 에러.
///
/// Commit B: `Sqlite` variant 가 `#[from] rusqlite::Error` 로 tighten —
/// `?` operator 한 줄로 sqlite 에러 → `MemoryError::Sqlite` 변환 가능.
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("home dir unavailable")]
    NoHome,
    #[error("invalid query: {0}")]
    InvalidQuery(String),
    #[error("backend init: {0}")]
    BackendInit(String),
}

impl MemoryKind {
    /// kebab-case 표기 (역직렬화 형식과 일치).
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            MemoryKind::Tool => "tool",
            MemoryKind::Agent => "agent",
            MemoryKind::Command => "command",
            MemoryKind::Note => "note",
            MemoryKind::Error => "error",
        }
    }
}

impl MemoryRecord {
    /// `self.kind.label()` 위임 — 기존 caller (config.rs::ContextOrchestrator)
    /// back-compat 보존.
    #[must_use]
    pub fn kind_label(&self) -> &'static str {
        self.kind.label()
    }
}
