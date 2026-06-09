//! myharness-core — Harness 5 components 공통 trait/struct + standard_ai_workflow 6 원칙
//!
//! 모듈:
//! - [`workflow`]: standard_ai_workflow 6 원칙 native (W11.1)
//!   - TaskStartReport / TaskEndReport (한국어 보고)
//!   - EventLog (이벤트 소싱, append-only NDJSON)
//!   - HandoffDoc (비참조, handoff)
//! - [`permission`]: 4 permission mode (W11.2)
//! - [`tool_alias`]: sub-agent ↔ tools crate name alias (W11.3)

pub mod permission;
pub mod tool_alias;
pub mod workflow;

pub use permission::{PermissionDecision, PermissionMode, PermissionPolicy, ToolCategory};
pub use tool_alias::{resolve_all_aliases, resolve_tool_alias, KNOWN_TOOL_ALIASES};
pub use workflow::{
    EventKind, EventLog, EventLogEntry, FollowUpEntry, HandoffDoc, RiskEntry, RiskKind,
    TaskEndReport, TaskStartReport, TaskStatus,
};

/// Crate 버전.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }

    #[test]
    fn public_api_exports() {
        let _mode = PermissionMode::Default;
        let _kind = TaskStatus::InProgress;
        let _kind = RiskKind::Environment;
        let _kind = EventKind::Decision;
    }
}
