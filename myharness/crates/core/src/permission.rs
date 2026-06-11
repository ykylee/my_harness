//! 4 permission mode (CONCEPT §5.4, claude-code 패턴).
//!
//! - `default` — 매 destructive tool 실행 전 prompt
//! - `acceptEdits` — Edit/Write 자동 승인, Bash 는 prompt
//! - `plan` — 모든 tool 실행 차단 (읽기만), plan 승인 후 실행
//! - `bypassPermissions` — 모든 tool 자동 실행 (CI / 비대화형)

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionMode {
    Default,
    AcceptEdits,
    Plan,
    BypassPermissions,
}

impl PermissionMode {
    pub const ALL: &'static [PermissionMode] = &[
        PermissionMode::Default,
        PermissionMode::AcceptEdits,
        PermissionMode::Plan,
        PermissionMode::BypassPermissions,
    ];

    #[must_use] 
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionMode::Default => "default",
            PermissionMode::AcceptEdits => "accept-edits",
            PermissionMode::Plan => "plan",
            PermissionMode::BypassPermissions => "bypass-permissions",
        }
    }

    #[allow(clippy::should_implement_trait)]
    #[must_use] 
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "default" => Some(Self::Default),
            "accept-edits" | "acceptEdits" => Some(Self::AcceptEdits),
            "plan" => Some(Self::Plan),
            "bypass-permissions" | "bypassPermissions" => Some(Self::BypassPermissions),
            _ => None,
        }
    }

    #[must_use] 
    pub fn label(&self) -> &'static str {
        match self {
            PermissionMode::Default => "Default (prompt for each destructive action)",
            PermissionMode::AcceptEdits => "Accept Edits (auto-approve Edit/Write, prompt for Bash)",
            PermissionMode::Plan => "Plan (read-only, requires explicit plan approval)",
            PermissionMode::BypassPermissions => "Bypass Permissions (auto-approve all, CI/non-interactive)",
        }
    }
}

/// tool 실행 결과에 대한 decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionDecision {
    Allow,
    Deny,
    Prompt,
}

/// tool 분류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ToolCategory {
    Read,
    Edit,
    Write,
    Bash,
    Grep,
    Glob,
}

impl ToolCategory {
    #[must_use] 
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "read" => Some(Self::Read),
            "edit" => Some(Self::Edit),
            "write" => Some(Self::Write),
            "bash" => Some(Self::Bash),
            "grep" => Some(Self::Grep),
            "glob" | "glob_" => Some(Self::Glob),
            _ => None,
        }
    }

    #[must_use] 
    pub fn is_destructive(&self) -> bool {
        matches!(self, Self::Edit | Self::Write | Self::Bash)
    }
}

/// permission policy — mode + tool category → decision.
#[derive(Debug, Clone)]
pub struct PermissionPolicy {
    pub mode: PermissionMode,
    /// 비대화형 (CI) — Prompt → Allow 자동 fallback. CLI 의 --yes 와 동일.
    pub auto_approve: bool,
}

impl Default for PermissionPolicy {
    fn default() -> Self {
        Self { mode: PermissionMode::Default, auto_approve: false }
    }
}

impl PermissionPolicy {
    #[must_use] 
    pub fn new(mode: PermissionMode) -> Self {
        Self { mode, auto_approve: false }
    }

    #[must_use] 
    pub fn with_auto_approve(mut self, yes: bool) -> Self {
        self.auto_approve = yes;
        self
    }

    /// tool name + mode → decision.
    #[must_use] 
    pub fn decide(&self, tool_name: &str) -> PermissionDecision {
        let Some(cat) = ToolCategory::from_name(tool_name) else {
            // unknown tool — default 에서 prompt
            return match self.mode {
                PermissionMode::BypassPermissions => PermissionDecision::Allow,
                PermissionMode::Plan => PermissionDecision::Deny,
                _ => PermissionDecision::Prompt,
            };
        };
        match self.mode {
            PermissionMode::Default => {
                if cat.is_destructive() {
                    if self.auto_approve {
                        PermissionDecision::Allow
                    } else {
                        PermissionDecision::Prompt
                    }
                } else {
                    PermissionDecision::Allow
                }
            }
            PermissionMode::AcceptEdits => match cat {
                ToolCategory::Edit
                | ToolCategory::Write
                | ToolCategory::Read
                | ToolCategory::Grep
                | ToolCategory::Glob => PermissionDecision::Allow,
                ToolCategory::Bash => {
                    if self.auto_approve {
                        PermissionDecision::Allow
                    } else {
                        PermissionDecision::Prompt
                    }
                }
            },
            PermissionMode::Plan => PermissionDecision::Deny,
            PermissionMode::BypassPermissions => PermissionDecision::Allow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_four_modes() {
        assert_eq!(PermissionMode::ALL.len(), 4);
    }

    #[test]
    fn as_str_roundtrip() {
        for m in PermissionMode::ALL {
            assert_eq!(PermissionMode::from_str(m.as_str()), Some(*m));
        }
    }

    #[test]
    fn from_str_aliases() {
        assert_eq!(PermissionMode::from_str("acceptEdits"), Some(PermissionMode::AcceptEdits));
        assert_eq!(PermissionMode::from_str("bypassPermissions"), Some(PermissionMode::BypassPermissions));
    }

    #[test]
    fn unknown_returns_none() {
        assert!(PermissionMode::from_str("nonexistent").is_none());
    }

    #[test]
    fn tool_category_destructive() {
        assert!(ToolCategory::Bash.is_destructive());
        assert!(ToolCategory::Edit.is_destructive());
        assert!(ToolCategory::Write.is_destructive());
        assert!(!ToolCategory::Read.is_destructive());
        assert!(!ToolCategory::Grep.is_destructive());
        assert!(!ToolCategory::Glob.is_destructive());
    }

    #[test]
    fn tool_category_from_name_case_insensitive() {
        assert_eq!(ToolCategory::from_name("Bash"), Some(ToolCategory::Bash));
        assert_eq!(ToolCategory::from_name("bash"), Some(ToolCategory::Bash));
        assert_eq!(ToolCategory::from_name("BASH"), Some(ToolCategory::Bash));
        assert_eq!(ToolCategory::from_name("glob_"), Some(ToolCategory::Glob));
    }

    #[test]
    fn default_mode_destructive_prompts() {
        let p = PermissionPolicy::default();
        assert_eq!(p.decide("Bash"), PermissionDecision::Prompt);
        assert_eq!(p.decide("Edit"), PermissionDecision::Prompt);
    }

    #[test]
    fn default_mode_read_allows() {
        let p = PermissionPolicy::default();
        assert_eq!(p.decide("Read"), PermissionDecision::Allow);
        assert_eq!(p.decide("Grep"), PermissionDecision::Allow);
    }

    #[test]
    fn default_mode_with_auto_approve_allows() {
        let p = PermissionPolicy::default().with_auto_approve(true);
        assert_eq!(p.decide("Bash"), PermissionDecision::Allow);
    }

    #[test]
    fn accept_edits_allows_edit_write() {
        let p = PermissionPolicy::new(PermissionMode::AcceptEdits);
        assert_eq!(p.decide("Edit"), PermissionDecision::Allow);
        assert_eq!(p.decide("Write"), PermissionDecision::Allow);
        assert_eq!(p.decide("Bash"), PermissionDecision::Prompt);
    }

    #[test]
    fn plan_denies_all() {
        let p = PermissionPolicy::new(PermissionMode::Plan);
        assert_eq!(p.decide("Read"), PermissionDecision::Deny);
        assert_eq!(p.decide("Bash"), PermissionDecision::Deny);
        assert_eq!(p.decide("Edit"), PermissionDecision::Deny);
    }

    #[test]
    fn bypass_allows_all() {
        let p = PermissionPolicy::new(PermissionMode::BypassPermissions);
        assert_eq!(p.decide("Bash"), PermissionDecision::Allow);
        assert_eq!(p.decide("Edit"), PermissionDecision::Allow);
        assert_eq!(p.decide("Read"), PermissionDecision::Allow);
    }

    #[test]
    fn unknown_tool_default_mode_prompts() {
        let p = PermissionPolicy::default();
        assert_eq!(p.decide("mystery-tool"), PermissionDecision::Prompt);
    }

    #[test]
    fn unknown_tool_plan_denies() {
        let p = PermissionPolicy::new(PermissionMode::Plan);
        assert_eq!(p.decide("mystery-tool"), PermissionDecision::Deny);
    }

    #[test]
    fn unknown_tool_bypass_allows() {
        let p = PermissionPolicy::new(PermissionMode::BypassPermissions);
        assert_eq!(p.decide("mystery-tool"), PermissionDecision::Allow);
    }
}
