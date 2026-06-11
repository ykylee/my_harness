//! Auto memory — `~/.myharness/memory/auto/` 에 JSON append-only.
//!
//! 각 record 는 1줄 JSON (NDJSON). v1 simple: timestamp + kind + payload.

use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryKind {
    Tool,
    Agent,
    Command,
    Note,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub timestamp: DateTime<Utc>,
    pub kind: MemoryKind,
    pub payload: serde_json::Value,
}

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("home dir unavailable")]
    NoHome,
}

#[derive(Debug, Clone)]
pub struct AutoMemory {
    pub base_dir: PathBuf,
}

impl AutoMemory {
    #[must_use = "AutoMemory 인스턴스는 stateful — drop 하면 event log flush 가 누락될 수 있음"]
    /// 기본 경로 `~/.myharness/memory/auto/`. `MYHARNESS_HOME` env 로 override 가능.
    ///
    /// # Errors
    /// `dirs::home_dir()` 를 찾을 수 없으면 `MemoryError::NoHome` 에러 반환.
    pub fn new() -> Result<Self, MemoryError> {
        let base_dir = if let Ok(p) = std::env::var("MYHARNESS_HOME") {
            PathBuf::from(p).join("memory").join("auto")
        } else {
            dirs::home_dir()
                .ok_or(MemoryError::NoHome)?
                .join(".myharness")
                .join("memory")
                .join("auto")
        };
        Ok(Self { base_dir })
    }

    #[must_use]
    pub fn with_base(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn log_path(&self) -> PathBuf {
        self.base_dir.join("memory.ndjson")
    }

    ///
    /// # Errors
    /// 디렉토리 생성 실패 시 `MemoryError` 반환.
    pub fn ensure_dir(&self) -> Result<(), MemoryError> {
        std::fs::create_dir_all(&self.base_dir)?;
        Ok(())
    }

    /// record append. 한 줄 = 1 JSON.
    ///
    /// # Errors
    /// 파일 열기/쓰기 실패, 또는 JSON 직렬화 실패 시 `MemoryError` 반환.
    pub fn append(&self, record: &MemoryRecord) -> Result<(), MemoryError> {
        self.ensure_dir()?;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())?;
        let line = serde_json::to_string(record)?;
        writeln!(f, "{line}")?;
        Ok(())
    }

    ///
    /// # Errors
    /// 파일 저장 실패 시 `MemoryError` 반환.
    #[allow(clippy::needless_pass_by_value)]
    pub fn append_tool(&self, tool: &str, args: serde_json::Value) -> Result<(), MemoryError> {
        self.append(&MemoryRecord {
            timestamp: Utc::now(),
            kind: MemoryKind::Tool,
            payload: serde_json::json!({ "tool": tool, "args": args }),
        })
    }

    ///
    /// # Errors
    /// 파일 저장 실패 시 `MemoryError` 반환.
    pub fn append_note(&self, text: &str) -> Result<(), MemoryError> {
        self.append(&MemoryRecord {
            timestamp: Utc::now(),
            kind: MemoryKind::Note,
            payload: serde_json::json!({ "text": text }),
        })
    }

    ///
    /// # Errors
    /// 파일 저장 실패 시 `MemoryError` 반환.
    pub fn append_error(&self, err: &str) -> Result<(), MemoryError> {
        self.append(&MemoryRecord {
            timestamp: Utc::now(),
            kind: MemoryKind::Error,
            payload: serde_json::json!({ "error": err }),
        })
    }

    /// 최근 N 개 read. file 없으면 empty.
    ///
    /// # Errors
    /// 파일 읽기 또는 JSON 역직렬화 실패 시 `MemoryError` 반환.
    pub fn recent(&self, n: usize) -> Result<Vec<MemoryRecord>, MemoryError> {
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

    /// kind 필터 + 최근 N
    ///
    /// # Errors
    /// 파일 읽기 실패 시 `MemoryError` 반환.
    pub fn recent_by_kind(&self, kind: MemoryKind, n: usize) -> Result<Vec<MemoryRecord>, MemoryError> {
        Ok(self.recent(usize::MAX)?.into_iter().filter(|r| r.kind == kind).rev().take(n).collect::<Vec<_>>().into_iter().rev().collect())
    }

    /// system prompt injection 텍스트. 최근 N (default 20) 를 kind 별 요약.
    ///
    /// # Errors
    /// 파일 읽기 실패 시 `MemoryError` 반환.
    pub fn to_system_prompt_section(&self, max_records: usize) -> Result<String, MemoryError> {
        let recs = self.recent(max_records)?;
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

impl MemoryKind {
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
    #[must_use]
    pub fn kind_label(&self) -> &'static str {
        self.kind.label()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_recent_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let m = AutoMemory::with_base(dir.path().to_path_buf());
        m.append_tool("Read", serde_json::json!({"path": "/tmp/a"})).unwrap();
        m.append_note("manual note").unwrap();
        m.append_error("auth missing").unwrap();
        let recs = m.recent(10).unwrap();
        assert_eq!(recs.len(), 3);
        assert_eq!(recs[0].kind, MemoryKind::Tool);
        assert_eq!(recs[1].kind, MemoryKind::Note);
        assert_eq!(recs[2].kind, MemoryKind::Error);
    }

    #[test]
    fn recent_returns_last_n() {
        let dir = tempfile::tempdir().unwrap();
        let m = AutoMemory::with_base(dir.path().to_path_buf());
        for i in 0..5 {
            m.append_note(&format!("n{i}")).unwrap();
        }
        let recs = m.recent(3).unwrap();
        assert_eq!(recs.len(), 3);
        assert!(recs[0].payload["text"].as_str().unwrap().contains("n2"));
        assert!(recs[2].payload["text"].as_str().unwrap().contains("n4"));
    }

    #[test]
    fn recent_by_kind_filters() {
        let dir = tempfile::tempdir().unwrap();
        let m = AutoMemory::with_base(dir.path().to_path_buf());
        m.append_tool("Read", serde_json::json!({})).unwrap();
        m.append_note("a").unwrap();
        m.append_tool("Write", serde_json::json!({})).unwrap();
        m.append_note("b").unwrap();
        let tools = m.recent_by_kind(MemoryKind::Tool, 10).unwrap();
        assert_eq!(tools.len(), 2);
        assert!(tools.iter().all(|r| r.kind == MemoryKind::Tool));
    }

    #[test]
    fn recent_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let m = AutoMemory::with_base(dir.path().to_path_buf());
        let recs = m.recent(10).unwrap();
        assert!(recs.is_empty());
    }

    #[test]
    fn to_system_prompt_section_empty() {
        let dir = tempfile::tempdir().unwrap();
        let m = AutoMemory::with_base(dir.path().to_path_buf());
        let s = m.to_system_prompt_section(20).unwrap();
        assert!(s.is_empty());
    }

    #[test]
    fn to_system_prompt_section_includes_records() {
        let dir = tempfile::tempdir().unwrap();
        let m = AutoMemory::with_base(dir.path().to_path_buf());
        m.append_tool("Read", serde_json::json!({"path": "x.rs"})).unwrap();
        let s = m.to_system_prompt_section(20).unwrap();
        assert!(s.contains("Auto memory"));
        assert!(s.contains("tool"));
        assert!(s.contains("x.rs"));
    }

    #[test]
    fn append_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("deeply").join("nested");
        let m = AutoMemory::with_base(nested.clone());
        m.append_note("hello").unwrap();
        assert!(nested.join("memory.ndjson").exists());
    }

    #[test]
    fn kind_label() {
        assert_eq!(MemoryKind::Tool.label(), "tool");
        assert_eq!(MemoryKind::Agent.label(), "agent");
        assert_eq!(MemoryKind::Error.label(), "error");
    }
}
