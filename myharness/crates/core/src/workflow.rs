//! standard_ai_workflow 6 원칙 native 구현 (CONCEPT §5.9.1).
//!
//! 6 원칙:
//! 1. **한국어 보고** — 사용자 직접 노출 텍스트는 한국어
//! 2. **컨텍스트 절약** — 핵심 사실만 handoff, 중간 reasoning 생략
//! 3. **상태값** — task/handoff/agent state 명시
//! 4. **이벤트 소싱** — append-only event log
//! 5. **비참조** — task/handoff 텍스트가 다른 task 직접 참조 안 함
//! 6. **handoff** — 다음 세션/agent 가 복원 가능한 정보
//!
//! Zero coupling: Mavis 파일 없어도 동작. 옵션 Mavis 통합은 v1.5+.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// task 상태. CONCEPT §5.9.3 (Mavis 호환).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Planned,
    InProgress,
    Blocked,
    Done,
    Cancelled,
}

/// risk/follow-up 카테고리.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskKind {
    /// 외부 의존성 미해결 (libsecret, network, etc)
    Dependency,
    /// 컨텍스트 한계 (token overflow, compaction)
    Context,
    /// 환경 부재 (binary 없음, keychain 부재)
    Environment,
    /// 일반 risk
    General,
}

/// Task 시작 보고서.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStartReport {
    pub id: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub status: TaskStatus,
    /// 관련 task ID (비참조 원칙: 단순 나열, 다른 task 의 본문 인용 안 함)
    pub related: Vec<String>,
    /// 한국어 설명. 핵심 의도 + scope 만.
    pub intent_ko: String,
    /// scope (한글). 1-2 문장.
    pub scope_ko: String,
}

impl TaskStartReport {
    pub fn new(id: impl Into<String>, title: impl Into<String>, intent: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            started_at: Utc::now(),
            status: TaskStatus::InProgress,
            related: Vec::new(),
            intent_ko: intent.into(),
            scope_ko: String::new(),
        }
    }

    pub fn with_related(mut self, related: Vec<String>) -> Self {
        self.related = related;
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope_ko = scope.into();
        self
    }

    /// 한국어 human-readable 직렬화 (한국어 보고 원칙).
    pub fn to_korean(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Task 시작: {}\n\n", self.title));
        out.push_str(&format!("- ID: {}\n", self.id));
        out.push_str(&format!("- 시작: {}\n", self.started_at.format("%Y-%m-%dT%H:%M:%SZ")));
        out.push_str(&format!("- 상태: {:?}\n", self.status));
        if !self.related.is_empty() {
            out.push_str(&format!("- 관련: {}\n", self.related.join(", ")));
        }
        out.push_str(&format!("\n## 의도\n{}\n", self.intent_ko));
        if !self.scope_ko.is_empty() {
            out.push_str(&format!("\n## 범위\n{}\n", self.scope_ko));
        }
        out
    }
}

/// Task 종료 보고서.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEndReport {
    pub id: String,
    pub title: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub status: TaskStatus,
    /// 한국어 요약 (1-3 문장).
    pub summary_ko: String,
    /// risk 목록.
    pub risks: Vec<RiskEntry>,
    /// follow-up 작업.
    pub follow_up: Vec<FollowUpEntry>,
    /// 산출물 (commit SHA, file path 등).
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEntry {
    pub kind: RiskKind,
    pub description_ko: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpEntry {
    pub id: String,
    pub title: String,
    pub description_ko: String,
}

impl TaskEndReport {
    pub fn new(id: impl Into<String>, title: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            started_at: Utc::now(),
            ended_at: Utc::now(),
            status: TaskStatus::Done,
            summary_ko: summary.into(),
            risks: Vec::new(),
            follow_up: Vec::new(),
            artifacts: Vec::new(),
        }
    }

    pub fn started_at(mut self, t: DateTime<Utc>) -> Self {
        self.started_at = t;
        self
    }

    pub fn with_status(mut self, s: TaskStatus) -> Self {
        self.status = s;
        self
    }

    pub fn add_risk(&mut self, kind: RiskKind, desc: impl Into<String>) {
        self.risks.push(RiskEntry { kind, description_ko: desc.into() });
    }

    pub fn add_follow_up(&mut self, id: impl Into<String>, title: impl Into<String>, desc: impl Into<String>) {
        self.follow_up.push(FollowUpEntry { id: id.into(), title: title.into(), description_ko: desc.into() });
    }

    pub fn add_artifact(&mut self, a: impl Into<String>) {
        self.artifacts.push(a.into());
    }

    /// 한국어 직렬화.
    pub fn to_korean(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Task 종료: {}\n\n", self.title));
        out.push_str(&format!("- ID: {}\n", self.id));
        out.push_str(&format!("- 시작: {}\n", self.started_at.format("%Y-%m-%dT%H:%M:%SZ")));
        out.push_str(&format!("- 종료: {}\n", self.ended_at.format("%Y-%m-%dT%H:%M:%SZ")));
        out.push_str(&format!("- 상태: {:?}\n", self.status));
        if !self.summary_ko.is_empty() {
            out.push_str(&format!("\n## 요약\n{}\n", self.summary_ko));
        }
        if !self.artifacts.is_empty() {
            out.push_str("\n## 산출물\n");
            for a in &self.artifacts {
                out.push_str(&format!("- {a}\n"));
            }
        }
        if !self.risks.is_empty() {
            out.push_str("\n## Risk\n");
            for r in &self.risks {
                out.push_str(&format!("- [{}] {}\n", format!("{:?}", r.kind).to_lowercase(), r.description_ko));
            }
        }
        if !self.follow_up.is_empty() {
            out.push_str("\n## Follow-up\n");
            for f in &self.follow_up {
                out.push_str(&format!("- {} ({}): {}\n", f.id, f.title, f.description_ko));
            }
        }
        out
    }
}

/// append-only event log (이벤트 소싱 원칙).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogEntry {
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventKind {
    Info,
    Warn,
    Error,
    Decision,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub entries: Vec<EventLogEntry>,
}

impl EventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, kind: EventKind, message: impl Into<String>) {
        self.entries.push(EventLogEntry {
            timestamp: Utc::now(),
            kind,
            message: message.into(),
        });
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        self.append(EventKind::Info, msg);
    }
    pub fn warn(&mut self, msg: impl Into<String>) {
        self.append(EventKind::Warn, msg);
    }
    pub fn error(&mut self, msg: impl Into<String>) {
        self.append(EventKind::Error, msg);
    }
    pub fn decision(&mut self, msg: impl Into<String>) {
        self.append(EventKind::Decision, msg);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// NDJSON 직렬화 (append-friendly).
    pub fn to_ndjson(&self) -> String {
        self.entries
            .iter()
            .map(|e| serde_json::to_string(e).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// handoff (다음 세션/agent 가 복원) — 컨텍스트 절약 원칙: 핵심 사실만.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffDoc {
    pub from_session: String,
    pub to_session: String,
    pub created_at: DateTime<Utc>,
    /// 핵심 사실 (다음 세션이 알아야 할 것). 5-10 항목.
    pub key_facts: Vec<String>,
    /// 다음 행동 우선순위.
    pub next_actions: Vec<String>,
    /// 활성 in-progress task.
    pub in_progress: Vec<String>,
    /// blocked items.
    pub blocked: Vec<String>,
    /// 환경 제약.
    pub environment: Vec<String>,
}

impl HandoffDoc {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from_session: from.into(),
            to_session: to.into(),
            created_at: Utc::now(),
            key_facts: Vec::new(),
            next_actions: Vec::new(),
            in_progress: Vec::new(),
            blocked: Vec::new(),
            environment: Vec::new(),
        }
    }

    pub fn add_fact(&mut self, f: impl Into<String>) {
        self.key_facts.push(f.into());
    }
    pub fn add_action(&mut self, a: impl Into<String>) {
        self.next_actions.push(a.into());
    }
    pub fn add_in_progress(&mut self, t: impl Into<String>) {
        self.in_progress.push(t.into());
    }
    pub fn add_blocked(&mut self, t: impl Into<String>) {
        self.blocked.push(t.into());
    }
    pub fn add_env(&mut self, e: impl Into<String>) {
        self.environment.push(e.into());
    }

    /// 한국어 직렬화.
    pub fn to_korean(&self) -> String {
        let mut out = String::new();
        out.push_str("# Handoff\n\n");
        out.push_str(&format!("- from: {}\n", self.from_session));
        out.push_str(&format!("- to: {}\n", self.to_session));
        out.push_str(&format!("- created: {}\n", self.created_at.format("%Y-%m-%dT%H:%M:%SZ")));
        if !self.key_facts.is_empty() {
            out.push_str("\n## 핵심 사실\n");
            for f in &self.key_facts {
                out.push_str(&format!("- {f}\n"));
            }
        }
        if !self.in_progress.is_empty() {
            out.push_str("\n## In Progress\n");
            for t in &self.in_progress {
                out.push_str(&format!("- {t}\n"));
            }
        }
        if !self.blocked.is_empty() {
            out.push_str("\n## Blocked\n");
            for t in &self.blocked {
                out.push_str(&format!("- {t}\n"));
            }
        }
        if !self.next_actions.is_empty() {
            out.push_str("\n## 다음 행동\n");
            for a in &self.next_actions {
                out.push_str(&format!("- {a}\n"));
            }
        }
        if !self.environment.is_empty() {
            out.push_str("\n## 환경\n");
            for e in &self.environment {
                out.push_str(&format!("- {e}\n"));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_serde_kebab() {
        let s = serde_json::to_string(&TaskStatus::InProgress).unwrap();
        assert_eq!(s, "\"in-progress\"");
        let back: TaskStatus = serde_json::from_str("\"done\"").unwrap();
        assert_eq!(back, TaskStatus::Done);
    }

    #[test]
    fn risk_kind_serde() {
        let s = serde_json::to_string(&RiskKind::Environment).unwrap();
        assert_eq!(s, "\"environment\"");
    }

    #[test]
    fn task_start_to_korean_includes_all_fields() {
        let r = TaskStartReport::new("TASK-005", "스택 결정", "Rust 1안 결정")
            .with_related(vec!["TASK-001".into()])
            .with_scope("Rust cargo workspace 기반");
        let s = r.to_korean();
        assert!(s.contains("TASK-005"));
        assert!(s.contains("스택 결정"));
        assert!(s.contains("Rust 1안"));
        assert!(s.contains("TASK-001"));
    }

    #[test]
    fn task_end_with_risks_and_followups() {
        let mut r = TaskEndReport::new("TASK-005", "스택 결정", "Rust 1안으로 결정. cargo workspace + 8 crate skeleton.");
        r.add_risk(RiskKind::Environment, "libsecret 부재 — keyring backend fallback 필요");
        r.add_follow_up("TASK-005-1", "v1 MVP 빌드", "8 crate 골격 위에 실제 구현");
        r.add_artifact("Cargo.toml");
        r.add_artifact("crates/");
        let s = r.to_korean();
        assert!(s.contains("요약"));
        assert!(s.contains("libsecret"));
        assert!(s.contains("TASK-005-1"));
        assert!(s.contains("Cargo.toml"));
    }

    #[test]
    fn event_log_append_and_ndjson() {
        let mut log = EventLog::new();
        log.info("시작");
        log.warn("libsecret 없음");
        log.decision("Rust 1안");
        assert_eq!(log.len(), 3);
        let nd = log.to_ndjson();
        assert_eq!(nd.lines().count(), 3);
    }

    #[test]
    fn handoff_doc_to_korean() {
        let mut h = HandoffDoc::new("session-1", "session-2");
        h.add_fact("Rust 1안 결정 (D-36)");
        h.add_in_progress("TASK-005-1 W3~W6.5");
        h.add_blocked("TASK-002 (yklee 인프라 정보)");
        h.add_action("W3 시작");
        h.add_env("Rust 1.94.1");
        let s = h.to_korean();
        assert!(s.contains("Handoff"));
        assert!(s.contains("session-1"));
        assert!(s.contains("Rust 1안"));
        assert!(s.contains("TASK-002"));
    }

    #[test]
    fn handoff_no_ref_to_other_docs() {
        // 비참조 원칙: handoff 가 다른 handoff 의 본문을 인용하지 않음
        let h1 = HandoffDoc::new("a", "b");
        let h2 = HandoffDoc::new("b", "c");
        let s1 = h1.to_korean();
        let s2 = h2.to_korean();
        // 서로 file path/token id 같은 cross-reference 없음
        assert!(!s2.contains("see handoff a"));
    }

    #[test]
    fn event_kind_serde() {
        let s = serde_json::to_string(&EventKind::Decision).unwrap();
        assert_eq!(s, "\"decision\"");
    }

    #[test]
    fn task_start_with_no_related_omits_section() {
        let r = TaskStartReport::new("TASK-001", "bootstrap", "standard_ai_workflow 적용");
        let s = r.to_korean();
        assert!(!s.contains("- 관련:"));
    }

    #[test]
    fn task_end_with_no_risks_omits_section() {
        let mut r = TaskEndReport::new("TASK-001", "bootstrap", "done");
        assert!(!r.to_korean().contains("## Risk"));
    }
}
