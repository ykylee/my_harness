use crate::error::ToolError;
use crate::tool::PermissionMode;
use std::io::{self, Write};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny(String),
}

pub struct PermissionGuard;

impl PermissionGuard {
    pub fn check(
        tool_name: &str,
        mode: PermissionMode,
        confirm_override: bool,
        detail: Option<&str>,
    ) -> Result<PermissionDecision, ToolError> {
        if mode == PermissionMode::BypassPermissions {
            return Ok(PermissionDecision::Allow);
        }

        if matches!(tool_name, "read" | "grep" | "glob") {
            return Ok(PermissionDecision::Allow);
        }

        match mode {
            PermissionMode::Default => {
                if confirm_override {
                    return Ok(PermissionDecision::Allow);
                }
                Self::prompt(tool_name, detail)
            }
            PermissionMode::AcceptEdits => {
                if tool_name == "bash" {
                    if confirm_override {
                        return Ok(PermissionDecision::Allow);
                    }
                    Self::prompt(tool_name, detail)
                } else {
                    Ok(PermissionDecision::Allow)
                }
            }
            PermissionMode::Plan => Ok(PermissionDecision::Deny(format!(
                "{} blocked in plan mode (read-only)",
                tool_name
            ))),
            PermissionMode::BypassPermissions => unreachable!(),
        }
    }

    fn prompt(tool_name: &str, detail: Option<&str>) -> Result<PermissionDecision, ToolError> {
        let msg = match detail {
            Some(d) => format!("myharness: {} 실행하시겠습니까? [y/N] ({}) ", tool_name, d),
            None => format!("myharness: {} 실행하시겠습니까? [y/N] ", tool_name),
        };
        print!("{}", msg);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();

        if answer == "y" || answer == "yes" {
            Ok(PermissionDecision::Allow)
        } else {
            Ok(PermissionDecision::Deny(format!(
                "user declined: {}",
                tool_name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bypass_allows_everything() {
        assert_eq!(
            PermissionGuard::check("bash", PermissionMode::BypassPermissions, false, None).unwrap(),
            PermissionDecision::Allow
        );
    }

    #[test]
    fn plan_denies_writes() {
        match PermissionGuard::check("write", PermissionMode::Plan, false, None).unwrap() {
            PermissionDecision::Deny(_) => {}
            _ => panic!("expected deny"),
        }
    }

    #[test]
    fn read_only_always_allowed() {
        for mode in [
            PermissionMode::Default,
            PermissionMode::AcceptEdits,
            PermissionMode::Plan,
            PermissionMode::BypassPermissions,
        ] {
            assert_eq!(
                PermissionGuard::check("read", mode, false, None).unwrap(),
                PermissionDecision::Allow
            );
            assert_eq!(
                PermissionGuard::check("grep", mode, false, None).unwrap(),
                PermissionDecision::Allow
            );
            assert_eq!(
                PermissionGuard::check("glob", mode, false, None).unwrap(),
                PermissionDecision::Allow
            );
        }
    }

    #[test]
    fn accept_edits_auto_allows_write_with_confirm_override() {
        assert_eq!(
            PermissionGuard::check("write", PermissionMode::AcceptEdits, true, None).unwrap(),
            PermissionDecision::Allow
        );
    }

    #[test]
    fn confirm_override_skips_prompt_in_default() {
        assert_eq!(
            PermissionGuard::check("write", PermissionMode::Default, true, None).unwrap(),
            PermissionDecision::Allow
        );
    }
}
