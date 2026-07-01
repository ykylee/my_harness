//! Commit B integration tests — Sqlite backend (FTS5 + BM25) + back-compat 검증.
//!
//! ## gate (Commit B 완료 기준, §검증 gate 매핑)
//! 1. `cargo build --manifest-path myharness/Cargo.toml --workspace` → 0 errors
//! 2. `cargo clippy --manifest-path myharness/Cargo.toml --workspace --all-targets -- -D warnings` → clean
//! 3. `cargo test --manifest-path myharness/Cargo.toml --workspace` → 447 prior + 7 Commit A + 6 Commit B
//! 4. `MYHARNESS_MEMORY_BACKEND=sqlite cargo test -p myharness-context --test auto_memory` → 6/6 pass
//! 5. NDJSON default path unchanged → 7/7 still pass

use myharness_context::{AutoMemory, AutoMemoryConfig, MemoryBackend, MemoryKind, MemoryQuery};
use std::path::Path;

fn make_config(dir: &Path, backend: MemoryBackend) -> AutoMemoryConfig {
    AutoMemoryConfig {
        backend,
        base_dir: dir.to_path_buf(),
    }
}

#[tokio::test]
async fn back_compat_ndjson_default() {
    let dir = tempfile::tempdir().unwrap();
    // unset MYHARNESS_MEMORY_BACKEND → backend field drives dispatch (not env).
    let cfg = make_config(dir.path(), MemoryBackend::Ndjson);
    let m = AutoMemory::open(cfg).await.unwrap();
    m.append_tool("Read", serde_json::json!({"path": "/tmp/a"}))
        .unwrap();
    let log = dir.path().join("memory.ndjson");
    assert!(log.exists(), "NDJSON backend must produce memory.ndjson");
}

#[tokio::test]
async fn back_compat_legacy_records_load() {
    let dir = tempfile::tempdir().unwrap();
    let legacy = dir.path().join("memory.ndjson");
    std::fs::write(
        &legacy,
        r#"{"timestamp":"2026-06-01T00:00:00Z","kind":"tool","payload":{"tool":"Read"}}"#,
    )
    .unwrap();
    let cfg = make_config(dir.path(), MemoryBackend::Ndjson);
    let m = AutoMemory::open(cfg).await.unwrap();
    let recs = m.recent(10).unwrap();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].kind, MemoryKind::Tool);
}

#[tokio::test]
async fn sqlite_append_and_query_match() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = make_config(dir.path(), MemoryBackend::Sqlite);
    let m = AutoMemory::open(cfg).await.unwrap();
    m.append_note("hello world").unwrap();
    m.append_note("rust async runtime").unwrap();
    m.append_note("unrelated content").unwrap();
    let hits = m
        .query(MemoryQuery::default().with_keyword("rust"))
        .await
        .unwrap();
    assert!(!hits.is_empty(), "expected at least one hit for 'rust'");
    assert_eq!(hits[0].record.kind, MemoryKind::Note);
}

#[tokio::test]
async fn sqlite_cross_session_recall() {
    let dir = tempfile::tempdir().unwrap();
    {
        let cfg = make_config(dir.path(), MemoryBackend::Sqlite);
        let m = AutoMemory::open(cfg).await.unwrap();
        m.append_note("session 1 wrote this").unwrap();
    }
    let cfg = make_config(dir.path(), MemoryBackend::Sqlite);
    let m = AutoMemory::open(cfg).await.unwrap();
    let recs = m.recent(10).unwrap();
    assert_eq!(recs.len(), 1);
    assert!(recs[0].payload.to_string().contains("session 1"));
}

#[tokio::test]
async fn sqlite_bm25_ranking() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = make_config(dir.path(), MemoryBackend::Sqlite);
    let m = AutoMemory::open(cfg).await.unwrap();
    m.append_note("rust rust rust programming language")
        .unwrap();
    m.append_note("rust async tokio").unwrap();
    m.append_note("unrelated python java").unwrap();
    let hits = m
        .query(MemoryQuery::default().with_keyword("rust"))
        .await
        .unwrap();
    assert_eq!(hits.len(), 2);
    assert!(
        hits[0].score >= hits[1].score,
        "BM25 normalization must order higher-relevance first ({} >= {})",
        hits[0].score,
        hits[1].score
    );
}

#[tokio::test]
async fn sqlite_kind_filter() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = make_config(dir.path(), MemoryBackend::Sqlite);
    let m = AutoMemory::open(cfg).await.unwrap();
    m.append_tool("Read", serde_json::json!({"path": "x.rs"}))
        .unwrap();
    m.append_note("readme text").unwrap();
    m.append_error("oops").unwrap();
    let hits = m
        .query(
            MemoryQuery::default()
                .with_keyword("read")
                .with_kinds(vec![MemoryKind::Tool]),
        )
        .await
        .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].record.kind, MemoryKind::Tool);
}
