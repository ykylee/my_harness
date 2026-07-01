//! `SqliteMemoryStore` — sqlite FTS5 + BM25 backend.
//!
//! TASK-005-2 v2.0 Plugin Sub-task 1 Commit B. 기본값은 NDJSON
//! (`AutoMemory::new()` / `with_base()`) — sqlite 는
//! `AutoMemory::open(MemoryBackend::Sqlite)` 또는 `MYHARNESS_MEMORY_BACKEND=sqlite`
//! 로 명시 opt-in.
//!
//! ## schema (verbatim, contract)
//! - `memory`: rowid 자동증가, payload/summary/tags JSON serialized.
//! - `memory_fts`: fts5 가상 테이블 (porter + unicode61 tokenizer), content=memory mirror.
//! - 3 triggers (`memory_ai/ad/au`): insert/delete/update 시 fts index 자동 동기화.
//!
//! ## thread safety
//! `Connection` 는 `!Sync` 이므로 `Arc<Mutex<Connection>>` 로 wrap. 모든 호출은
//! `tokio::task::spawn_blocking` 으로 off-thread 실행 (sqlite 는 blocking).
//!
//! ## ranking
//! `bm25(memory_fts)` 는 음수 반환 (낮을수록 좋음) — `bm25_normalize()` 로 flip.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{Connection, params};

use crate::auto_memory::query::bm25_normalize;
use crate::auto_memory::store::MemoryStore;
use crate::auto_memory::types::{MemoryError, MemoryHit, MemoryKind, MemoryQuery, MemoryRecord};

const SCHEMA_SQL: &str = r"
CREATE TABLE IF NOT EXISTS memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    kind TEXT NOT NULL,
    payload TEXT NOT NULL,
    summary TEXT,
    tags TEXT
);

CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
    kind UNINDEXED,
    payload_text,
    summary,
    tags,
    content='memory',
    content_rowid='id',
    tokenize='porter unicode61'
);

CREATE TRIGGER IF NOT EXISTS memory_ai AFTER INSERT ON memory BEGIN
    INSERT INTO memory_fts(rowid, kind, payload_text, summary, tags)
    VALUES (new.id, new.kind, new.payload, new.summary, new.tags);
END;
CREATE TRIGGER IF NOT EXISTS memory_ad AFTER DELETE ON memory BEGIN
    INSERT INTO memory_fts(memory_fts, rowid, kind, payload_text, summary, tags)
    VALUES ('delete', old.id, old.kind, old.payload, old.summary, old.tags);
END;
CREATE TRIGGER IF NOT EXISTS memory_au AFTER UPDATE ON memory BEGIN
    INSERT INTO memory_fts(memory_fts, rowid, kind, payload_text, summary, tags)
    VALUES ('delete', old.id, old.kind, old.payload, old.summary, old.tags);
    INSERT INTO memory_fts(rowid, kind, payload_text, summary, tags)
    VALUES (new.id, new.kind, new.payload, new.summary, new.tags);
END;
";

pub struct SqliteMemoryStore {
    conn: Arc<Mutex<Connection>>,
    #[allow(dead_code)]
    db_path: PathBuf,
}

impl SqliteMemoryStore {
    /// `base_dir` 생성 + `<base_dir>/index.db` sqlite open + schema migration.
    ///
    /// # Errors
    /// dir 생성 / sqlite open / schema execute 실패 시 [`MemoryError`].
    pub fn open(base_dir: &Path) -> Result<Self, MemoryError> {
        std::fs::create_dir_all(base_dir)?;
        let db_path = base_dir.join("index.db");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }
}

fn parse_kind(s: &str) -> Result<MemoryKind, MemoryError> {
    match s {
        "tool" => Ok(MemoryKind::Tool),
        "agent" => Ok(MemoryKind::Agent),
        "command" => Ok(MemoryKind::Command),
        "note" => Ok(MemoryKind::Note),
        "error" => Ok(MemoryKind::Error),
        other => Err(MemoryError::BackendInit(format!("unknown kind: {other}"))),
    }
}

fn decode_row(row: &rusqlite::Row<'_>) -> Result<MemoryRecord, MemoryError> {
    let ts: i64 = row.get(0)?;
    let kind_str: String = row.get(1)?;
    let payload_str: String = row.get(2)?;
    let summary: Option<String> = row.get(3)?;
    let tags_str: Option<String> = row.get(4)?;
    let kind = parse_kind(&kind_str)?;
    let payload: serde_json::Value = serde_json::from_str(&payload_str)?;
    let tags: Vec<String> = match tags_str {
        Some(s) => serde_json::from_str(&s).unwrap_or_default(),
        None => Vec::new(),
    };
    let timestamp: DateTime<Utc> = Utc.timestamp_opt(ts, 0).single().unwrap_or_else(Utc::now);
    Ok(MemoryRecord {
        timestamp,
        kind,
        payload,
        summary,
        tags,
    })
}

fn lock_conn(
    conn: &Arc<Mutex<Connection>>,
) -> Result<std::sync::MutexGuard<'_, Connection>, MemoryError> {
    conn.lock()
        .map_err(|e| MemoryError::BackendInit(format!("poisoned: {e}")))
}

#[async_trait]
impl MemoryStore for SqliteMemoryStore {
    async fn append(&self, record: MemoryRecord) -> Result<(), MemoryError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> Result<(), MemoryError> {
            let conn = lock_conn(&conn)?;
            let payload = serde_json::to_string(&record.payload)?;
            let tags = serde_json::to_string(&record.tags)?;
            conn.execute(
                "INSERT INTO memory (timestamp, kind, payload, summary, tags) VALUES (?, ?, ?, ?, ?)",
                params![
                    record.timestamp.timestamp(),
                    record.kind.label(),
                    payload,
                    record.summary,
                    tags,
                ],
            )?;
            Ok(())
        })
        .await
        .map_err(|e| MemoryError::BackendInit(format!("spawn_blocking: {e}")))?
    }

    #[allow(clippy::collapsible_if)]
    async fn query(&self, q: MemoryQuery) -> Result<Vec<MemoryHit>, MemoryError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> Result<Vec<MemoryHit>, MemoryError> {
            let conn = lock_conn(&conn)?;
            let limit = q.effective_limit() as i64;
            let match_expr = q.to_fts5_match();

            let mut sql = String::from(
                "SELECT m.timestamp, m.kind, m.payload, m.summary, m.tags, \
                 bm25(memory_fts) AS score \
                 FROM memory_fts f INNER JOIN memory m ON f.rowid = m.id",
            );
            let mut where_clauses: Vec<String> = Vec::new();
            let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

            if let Some(ref expr) = match_expr {
                where_clauses.push("memory_fts MATCH ?".to_string());
                bind_values.push(Box::new(expr.clone()));
            }
            if let Some(ref kinds) = q.kinds {
                if !kinds.is_empty() {
                    let placeholders: Vec<String> =
                        (0..kinds.len()).map(|_| "?".to_string()).collect();
                    where_clauses.push(format!("m.kind IN ({})", placeholders.join(",")));
                    for k in kinds {
                        bind_values.push(Box::new(k.label().to_string()));
                    }
                }
            } // allow: collapsible_if (kinds 가 Some + non-empty 의미상 분리, let-chain 회피)
            if let Some(since) = q.since {
                where_clauses.push("m.timestamp >= ?".to_string());
                bind_values.push(Box::new(since));
            }
            if let Some(until) = q.until {
                where_clauses.push("m.timestamp <= ?".to_string());
                bind_values.push(Box::new(until));
            }
            if !where_clauses.is_empty() {
                sql.push_str(" WHERE ");
                sql.push_str(&where_clauses.join(" AND "));
            }
            sql.push_str(" ORDER BY score LIMIT ?");
            bind_values.push(Box::new(limit));

            let mut stmt = conn.prepare(&sql)?;
            let bind_refs: Vec<&dyn rusqlite::ToSql> =
                bind_values.iter().map(|b| b.as_ref()).collect();
            let matched_kw = q.keyword.clone();
            let rows = stmt.query_and_then(
                &bind_refs[..],
                |row| -> Result<(MemoryRecord, f64), MemoryError> {
                    let raw_score: f64 = row.get(5)?;
                    let record = decode_row(row)?;
                    Ok((record, raw_score))
                },
            )?;

            let mut hits = Vec::new();
            for row in rows {
                let (record, raw) = row?;
                let matched_terms: Vec<String> = matched_kw
                    .as_ref()
                    .map(|kw| kw.split_whitespace().map(String::from).collect())
                    .unwrap_or_default();
                hits.push(MemoryHit {
                    record,
                    score: bm25_normalize(raw),
                    matched_terms,
                });
            }
            Ok(hits)
        })
        .await
        .map_err(|e| MemoryError::BackendInit(format!("spawn_blocking: {e}")))?
    }

    async fn recent(&self, n: usize) -> Result<Vec<MemoryRecord>, MemoryError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> Result<Vec<MemoryRecord>, MemoryError> {
            let conn = lock_conn(&conn)?;
            let mut stmt = conn.prepare(
                "SELECT timestamp, kind, payload, summary, tags FROM memory ORDER BY timestamp DESC LIMIT ?",
            )?;
            let rows = stmt.query_and_then(params![n as i64], decode_row)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
        .await
        .map_err(|e| MemoryError::BackendInit(format!("spawn_blocking: {e}")))?
    }

    async fn recent_by_kind(
        &self,
        kind: MemoryKind,
        n: usize,
    ) -> Result<Vec<MemoryRecord>, MemoryError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> Result<Vec<MemoryRecord>, MemoryError> {
            let conn = lock_conn(&conn)?;
            let mut stmt = conn.prepare(
                "SELECT timestamp, kind, payload, summary, tags FROM memory WHERE kind = ? ORDER BY timestamp DESC LIMIT ?",
            )?;
            let rows = stmt.query_and_then(params![kind.label(), n as i64], decode_row)?;
            let mut out = Vec::new();
            for row in rows {
                out.push(row?);
            }
            Ok(out)
        })
        .await
        .map_err(|e| MemoryError::BackendInit(format!("spawn_blocking: {e}")))?
    }

    async fn compact(&self) -> Result<usize, MemoryError> {
        let conn = Arc::clone(&self.conn);
        tokio::task::spawn_blocking(move || -> Result<usize, MemoryError> {
            let conn = lock_conn(&conn)?;
            let before: i64 = conn.query_row("SELECT COUNT(*) FROM memory", [], |r| r.get(0))?;
            conn.execute_batch("VACUUM;")?;
            let after: i64 = conn.query_row("SELECT COUNT(*) FROM memory", [], |r| r.get(0))?;
            Ok((before - after).max(0) as usize)
        })
        .await
        .map_err(|e| MemoryError::BackendInit(format!("spawn_blocking: {e}")))?
    }

    async fn to_system_prompt_section(&self, n: usize) -> Result<String, MemoryError> {
        let recs = self.recent(n).await?;
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
