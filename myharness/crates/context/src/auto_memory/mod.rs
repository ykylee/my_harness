//! Auto memory facade — `AutoMemory` 가 `MemoryStore` trait object 를 감싸고,
//! 기존 caller (config.rs::ContextOrchestrator 등) 가 기대하는 **sync API** 를
//! 그대로 노출한다.
//!
//! ## 모듈 구성
//! - [`types`]: `MemoryKind` / `MemoryRecord` / `MemoryQuery` / `MemoryHit` / `MemoryError`.
//! - [`store`]: `MemoryStore` async trait + `NdjsonMemoryStore` adapter.
//! - [`query`]: `MemoryQuery` builder + `bm25_normalize`.
//!
//! ## back-compat (Commit A 핵심)
//! - `AutoMemory::new() / with_base() / append() / append_tool() / append_note() /
//!   append_error() / recent() / recent_by_kind() / to_system_prompt_section()`
//!   시그니처/동작 100% 동일 — 기존 `config.rs` caller 변경 0.
//! - sync wrapper 내부에서 [`block_on`] tokio bridge 로 async trait 호출.
//!
//! ## Commit B 대비
//! - `MemoryBackend::Sqlite` variant (현재는 `#[allow(dead_code)]`).
//! - `AutoMemory::open()` async — Sqlite store 동적 선택.
//! - `query()` / `compact()` async 신규 — Sqlite FTS5 활용 진입점.

use std::path::PathBuf;
use std::sync::Arc;

// 서브모듈 선언 — `crate::auto_memory::types::MemoryError` 등의 절대 경로 활성화.
mod query;
mod sqlite_store;
mod store;
mod types;

// Sibling re-exports: `NdjsonMemoryStore` / `SqliteMemoryStore` 는 facade 내부 전용.
pub use store::MemoryStore;
pub use types::{MemoryError, MemoryHit, MemoryKind, MemoryQuery, MemoryRecord};
use sqlite_store::SqliteMemoryStore;
use store::NdjsonMemoryStore;

/// Backend 선택. `MYHARNESS_MEMORY_BACKEND` env 로 override.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryBackend {
    Ndjson,
    /// Commit B 에서 활성화 (rusqlite FTS5 + BM25).
    #[allow(dead_code)]
    Sqlite,
}

/// `AutoMemory` 의 backend + 경로 설정.
///
/// `Default::default()` 가 `MYHARNESS_MEMORY_BACKEND` (값: `"sqlite"` 면 Sqlite, 그 외 Ndjson)
/// + `MYHARNESS_HOME` (없으면 `~/.myharness/memory/auto`) env 를 읽는다.
#[derive(Debug, Clone)]
pub struct AutoMemoryConfig {
    pub backend: MemoryBackend,
    pub base_dir: PathBuf,
}

impl Default for AutoMemoryConfig {
    fn default() -> Self {
        let backend = match std::env::var("MYHARNESS_MEMORY_BACKEND").as_deref() {
            Ok("sqlite") => MemoryBackend::Sqlite,
            _ => MemoryBackend::Ndjson,
        };
        let base_dir = if let Ok(p) = std::env::var("MYHARNESS_HOME") {
            PathBuf::from(p).join("memory").join("auto")
        } else {
            dirs::home_dir()
                .expect("home dir (overridable via MYHARNESS_HOME)")
                .join(".myharness")
                .join("memory")
                .join("auto")
        };
        Self { backend, base_dir }
    }
}

/// Auto memory facade. 기존 caller (config.rs) 가 `AutoMemory` 인스턴스를
/// 그대로 쓰는 back-compat 시그니처를 유지한다.
///
/// # Examples
/// ```no_run
/// use myharness_context::AutoMemory;
/// let m = AutoMemory::new().unwrap();
/// m.append_note("hello").unwrap();
/// let recs = m.recent(5).unwrap();
/// ```
pub struct AutoMemory {
    inner: Arc<dyn MemoryStore>,
    #[allow(dead_code)]
    config: AutoMemoryConfig,
}

impl AutoMemory {
    /// 기본 경로 `~/.myharness/memory/auto/`. `MYHARNESS_HOME` env 로 override.
    ///
    /// # Errors
    /// `dirs::home_dir()` 가 없고 `MYHARNESS_HOME` 도 미설정이면
    /// [`MemoryError::NoHome`].
    pub fn new() -> Result<Self, MemoryError> {
        // `open` 이 async 라서 `block_on` 필요. 단, Config 초기화는 sync
        // 가능하므로 여기서는 backend=Ndjson 으로 직접 shortcut.
        let config = AutoMemoryConfig::default();
        let inner: Arc<dyn MemoryStore> = match config.backend {
            MemoryBackend::Ndjson => Arc::new(NdjsonMemoryStore::new(config.base_dir.clone())?),
            MemoryBackend::Sqlite => {
                return Err(MemoryError::BackendInit(
                    "sqlite backend not yet implemented (Commit B)".to_string(),
                ));
            }
        };
        Ok(Self { inner, config })
    }

    /// 특정 `base_dir` 로 강제 (NDJSON backend). infallible.
    /// 테스트에서 env-independent 한 격리 디렉토리 주입에 사용.
    #[must_use]
    pub fn with_base(base_dir: PathBuf) -> Self {
        let config = AutoMemoryConfig {
            backend: MemoryBackend::Ndjson,
            base_dir,
        };
        let inner: Arc<dyn MemoryStore> = Arc::new(
            NdjsonMemoryStore::new(config.base_dir.clone())
                .expect("ndjson init never fails"),
        );
        Self { inner, config }
    }

    /// Config 기반 backend 선택 (async — Commit B 의 Sqlite store init 위함).
    ///
    /// # Errors
    /// backend init 실패 시 (Commit B 의 sqlite open 실패 등).
    pub async fn open(config: AutoMemoryConfig) -> Result<Self, MemoryError> {
        let inner: Arc<dyn MemoryStore> = match config.backend {
            MemoryBackend::Ndjson => Arc::new(NdjsonMemoryStore::new(config.base_dir.clone())?),
            MemoryBackend::Sqlite => Arc::new(SqliteMemoryStore::open(&config.base_dir)?),
        };
        Ok(Self { inner, config })
    }

    // ── sync wrappers (back-compat) ───────────────────────────────────────

    /// record 한 건 append (sync — async trait 호출).
    ///
    /// # Errors
    /// 파일 I/O 또는 직렬화 실패 시 [`MemoryError`].
    pub fn append(&self, record: &MemoryRecord) -> Result<(), MemoryError> {
        let inner = std::sync::Arc::clone(&self.inner);
        let record = record.clone();
        block_on(async move { inner.append(record).await })
    }

    /// tool 호출 event 기록.
    ///
    /// # Errors
    /// 파일 저장 실패 시.
    #[allow(clippy::needless_pass_by_value)]
    pub fn append_tool(&self, tool: &str, args: serde_json::Value) -> Result<(), MemoryError> {
        self.append(&MemoryRecord {
            timestamp: chrono::Utc::now(),
            kind: MemoryKind::Tool,
            payload: serde_json::json!({ "tool": tool, "args": args }),
            summary: None,
            tags: vec![],
        })
    }

    /// 자유 형식 note 기록.
    ///
    /// # Errors
    /// 파일 저장 실패 시.
    pub fn append_note(&self, text: &str) -> Result<(), MemoryError> {
        self.append(&MemoryRecord {
            timestamp: chrono::Utc::now(),
            kind: MemoryKind::Note,
            payload: serde_json::json!({ "text": text }),
            summary: None,
            tags: vec![],
        })
    }

    /// 에러 event 기록.
    ///
    /// # Errors
    /// 파일 저장 실패 시.
    pub fn append_error(&self, err: &str) -> Result<(), MemoryError> {
        self.append(&MemoryRecord {
            timestamp: chrono::Utc::now(),
            kind: MemoryKind::Error,
            payload: serde_json::json!({ "error": err }),
            summary: None,
            tags: vec![],
        })
    }

    /// 최근 N 개 read. file 없으면 empty.
    ///
    /// # Errors
    /// 파일 읽기 또는 역직렬화 실패 시.
    pub fn recent(&self, n: usize) -> Result<Vec<MemoryRecord>, MemoryError> {
        let inner = std::sync::Arc::clone(&self.inner);
        block_on(async move { inner.recent(n).await })
    }

    /// kind 필터 + 최근 N.
    ///
    /// # Errors
    /// 파일 읽기 실패 시.
    pub fn recent_by_kind(
        &self,
        kind: MemoryKind,
        n: usize,
    ) -> Result<Vec<MemoryRecord>, MemoryError> {
        let inner = std::sync::Arc::clone(&self.inner);
        block_on(async move { inner.recent_by_kind(kind, n).await })
    }

    /// system prompt injection 텍스트. 최근 N (default 20) 를 kind 별 요약.
    ///
    /// # Errors
    /// 파일 읽기 실패 시.
    pub fn to_system_prompt_section(&self, n: usize) -> Result<String, MemoryError> {
        let inner = std::sync::Arc::clone(&self.inner);
        block_on(async move { inner.to_system_prompt_section(n).await })
    }

    // ── async new methods (Commit B 사용 예정) ──────────────────────────

    /// 검색 질의 실행 (async). NDJSON backend 는 빈 결과, Commit B 의
    /// SqliteMemoryStore 가 BM25 구현.
    ///
    /// # Errors
    /// backend query 실패 시.
    pub async fn query(&self, q: MemoryQuery) -> Result<Vec<MemoryHit>, MemoryError> {
        self.inner.query(q).await
    }

    /// compaction. NDJSON 은 no-op, Sqlite 는 `VACUUM` + 중복 제거.
    /// 반환값: 제거/정리된 record 수.
    ///
    /// # Errors
    /// backend compact 실패 시.
    pub async fn compact(&self) -> Result<usize, MemoryError> {
        self.inner.compact().await
    }

    /// 현재 backend (introspection / logging 용).
    #[must_use]
    pub fn backend(&self) -> MemoryBackend {
        self.config.backend
    }
}

/// sync → async bridge. tokio runtime 이 scope 에 있으면 별도 OS thread + 자체
/// current_thread runtime 으로 escape (current_thread 의 executor 점유 회피).
/// runtime 없으면 직접 `Runtime::block_on`.
///
/// ## 제약
/// - caller 가 sync wrapper 에서 `Arc::clone(&self.inner)` 후 `async move` block
///   으로 호출해야 함 (E0521 방지 — self borrow 가 'static 이 아닌 문제 회피).
/// - F: `Send + 'static` 필수 (std::thread::spawn closure bound).
fn block_on<F>(f: F) -> F::Output
where
    F: std::future::Future + Send + 'static,
    F::Output: Send,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        return std::thread::spawn(move || {
            let mut builder = tokio::runtime::Builder::new_current_thread();
            builder.enable_all();
            let rt = builder
                .build()
                .expect("create escape runtime");
            rt.block_on(f)
        })
        .join()
        .expect("join escape thread");
    }
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    let rt = builder
        .build()
        .expect("create tokio runtime for sync bridge");
    rt.block_on(f)
}
