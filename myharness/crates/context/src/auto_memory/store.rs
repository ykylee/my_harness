//! `MemoryStore` trait + NDJSON adapter (`NdjsonMemoryStore`).
//!
//! TASK-005-2 v2.0 Plugin, Layer 1. 기존 `auto_memory.rs` 의 293 lines NDJSON
//! append-only 로직을 그대로 async trait impl 로 옮겼다.
//!
//! ## back-compat
//! - on-disk format (`<base_dir>/memory.ndjson`, 1 line = 1 JSON) 그대로.
//! - `MemoryRecord::summary` / `tags` 는 `skip_serializing_if` 로 optional.
//!
//! ## Commit B 대비
//! - `query()` / `compact()` 메서드는 v1 NDJSON 에서는 no-op (빈 결과 / `Ok(0)`).
//! - `SqliteMemoryStore` 가 BM25 query + VACUUM compact 를 구현할 예정.

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use async_trait::async_trait;

use crate::auto_memory::types::{
    MemoryError, MemoryHit, MemoryKind, MemoryQuery, MemoryRecord,
};

/// Auto memory backend 추상화. NDJSON / Sqlite (Commit B) 모두 이 trait 구현.
///
/// `Send + Sync` 필수 — facade `AutoMemory` 가 `Arc<dyn MemoryStore>` 로 보관.
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// record 한 건 append. idempotent 보장 X (중복 append 가능).
    async fn append(&self, record: MemoryRecord) -> Result<(), MemoryError>;

    /// 검색 질의 실행. NDJSON backend 는 BM25 미지원 → 빈 결과 반환.
    /// Commit B 의 `SqliteMemoryStore` 가 sqlite FTS5 bm25 구현.
    async fn query(&self, _q: MemoryQuery) -> Result<Vec<MemoryHit>, MemoryError>;

    /// 최근 N 개 read. file 없으면 empty.
    async fn recent(&self, n: usize) -> Result<Vec<MemoryRecord>, MemoryError>;

    /// kind 필터 + 최근 N.
    async fn recent_by_kind(
        &self,
        kind: MemoryKind,
        n: usize,
    ) -> Result<Vec<MemoryRecord>, MemoryError>;

    /// compaction. NDJSON 은 append-only → no-op (`Ok(0)`).
    /// Commit B 의 `SqliteMemoryStore` 가 `VACUUM` + 중복 제거.
    async fn compact(&self) -> Result<usize, MemoryError>;

    /// system prompt injection 텍스트. 최근 N (default 20) 를 kind 별 요약.
    async fn to_system_prompt_section(&self, n: usize) -> Result<String, MemoryError>;
}

/// NDJSON append-only adapter. file 위치: `<base_dir>/memory.ndjson`.
pub struct NdjsonMemoryStore {
    base_dir: PathBuf,
}

impl NdjsonMemoryStore {
    /// base_dir 은 caller 책임 (존재하지 않아도 됨 — `ensure_dir` 이 생성).
    ///
    /// # Errors
    /// 현재는 infallible (구조체만 초기화). 추후 validation 추가 시 에러 가능.
    pub fn new(base_dir: PathBuf) -> Result<Self, MemoryError> {
        Ok(Self { base_dir })
    }

    fn log_path(&self) -> PathBuf {
        self.base_dir.join("memory.ndjson")
    }

    fn ensure_dir(&self) -> Result<(), MemoryError> {
        std::fs::create_dir_all(&self.base_dir)?;
        Ok(())
    }
}

#[async_trait]
impl MemoryStore for NdjsonMemoryStore {
    async fn append(&self, record: MemoryRecord) -> Result<(), MemoryError> {
        self.ensure_dir()?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())?;
        let line = serde_json::to_string(&record)?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    async fn query(&self, _q: MemoryQuery) -> Result<Vec<MemoryHit>, MemoryError> {
        // NDJSON backend 는 full-text search 미지원 — 빈 결과.
        Ok(Vec::new())
    }

    async fn recent(&self, n: usize) -> Result<Vec<MemoryRecord>, MemoryError> {
        if !self.log_path().exists() {
            return Ok(Vec::new());
        }
        let f = std::fs::File::open(self.log_path())?;
        let reader = BufReader::new(f);
        let mut all: Vec<MemoryRecord> = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let rec: MemoryRecord = serde_json::from_str(&line)?;
            all.push(rec);
        }
        // 최근 N 개 (파일 끝 기준)
        let start = all.len().saturating_sub(n);
        Ok(all.split_off(start))
    }

    async fn recent_by_kind(
        &self,
        kind: MemoryKind,
        n: usize,
    ) -> Result<Vec<MemoryRecord>, MemoryError> {
        Ok(self
            .recent(usize::MAX)
            .await?
            .into_iter()
            .filter(|r| r.kind == kind)
            .rev()
            .take(n)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect())
    }

    async fn compact(&self) -> Result<usize, MemoryError> {
        // NDJSON 은 append-only — no-op.
        Ok(0)
    }

    async fn to_system_prompt_section(&self, max_records: usize) -> Result<String, MemoryError> {
        let recs = self.recent(max_records).await?;
        if recs.is_empty() {
            return Ok(String::new());
        }
        let mut out = String::new();
        out.push_str("## Auto memory (recent activity)\n\n");
        for r in &recs {
            use std::fmt::Write;
            let _ = writeln!(
                out,
                "- [{}] {}: {}",
                r.timestamp.format("%Y-%m-%dT%H:%M:%SZ"),
                r.kind_label(),
                r.payload
            );
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    /// sync 테스트 → async trait 호출용. 매 테스트마다 새 runtime (격리).
    fn rt() -> Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    fn note(text: &str) -> MemoryRecord {
        MemoryRecord {
            timestamp: chrono::Utc::now(),
            kind: MemoryKind::Note,
            payload: serde_json::json!({ "text": text }),
            summary: None,
            tags: vec![],
        }
    }

    fn tool(name: &str, args: serde_json::Value) -> MemoryRecord {
        MemoryRecord {
            timestamp: chrono::Utc::now(),
            kind: MemoryKind::Tool,
            payload: serde_json::json!({ "tool": name, "args": args }),
            summary: None,
            tags: vec![],
        }
    }

    #[test]
    fn ndjson_kind_label() {
        assert_eq!(MemoryKind::Tool.label(), "tool");
        assert_eq!(MemoryKind::Agent.label(), "agent");
        assert_eq!(MemoryKind::Error.label(), "error");
    }

    #[test]
    fn ndjson_append_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("deeply").join("nested");
        let s = NdjsonMemoryStore::new(nested.clone()).unwrap();
        rt().block_on(s.append(note("hello"))).unwrap();
        assert!(nested.join("memory.ndjson").exists());
    }

    #[test]
    fn ndjson_recent_returns_last_n() {
        let dir = tempfile::tempdir().unwrap();
        let s = NdjsonMemoryStore::new(dir.path().to_path_buf()).unwrap();
        rt().block_on(async {
            for i in 0..5 {
                s.append(note(&format!("n{i}"))).await.unwrap();
            }
        });
        let recs = rt().block_on(s.recent(3)).unwrap();
        assert_eq!(recs.len(), 3);
        assert!(recs[0].payload["text"].as_str().unwrap().contains("n2"));
        assert!(recs[2].payload["text"].as_str().unwrap().contains("n4"));
    }

    #[test]
    fn ndjson_recent_by_kind_filters() {
        let dir = tempfile::tempdir().unwrap();
        let s = NdjsonMemoryStore::new(dir.path().to_path_buf()).unwrap();
        rt().block_on(async {
            s.append(tool("Read", serde_json::json!({}))).await.unwrap();
            s.append(note("a")).await.unwrap();
            s.append(tool("Write", serde_json::json!({}))).await.unwrap();
            s.append(note("b")).await.unwrap();
        });
        let tools = rt().block_on(s.recent_by_kind(MemoryKind::Tool, 10)).unwrap();
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().all(|r| r.kind == MemoryKind::Tool));
    }

    #[test]
    fn ndjson_recent_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let s = NdjsonMemoryStore::new(dir.path().to_path_buf()).unwrap();
        let recs = rt().block_on(s.recent(10)).unwrap();
        assert!(recs.is_empty());
    }

    #[test]
    fn ndjson_to_system_prompt_section_empty() {
        let dir = tempfile::tempdir().unwrap();
        let s = NdjsonMemoryStore::new(dir.path().to_path_buf()).unwrap();
        let out = rt().block_on(s.to_system_prompt_section(20)).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn ndjson_to_system_prompt_section_includes_records() {
        let dir = tempfile::tempdir().unwrap();
        let s = NdjsonMemoryStore::new(dir.path().to_path_buf()).unwrap();
        rt().block_on(async {
            s.append(tool("Read", serde_json::json!({ "path": "x.rs" })))
                .await
                .unwrap();
        });
        let out = rt().block_on(s.to_system_prompt_section(20)).unwrap();
        assert!(out.contains("Auto memory"));
        assert!(out.contains("tool"));
        assert!(out.contains("x.rs"));
    }
}
