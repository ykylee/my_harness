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
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn check(
        tool_name: &str,
        mode: PermissionMode,
        confirm_override: bool,
        detail: Option<&str>,
    ) -> Result<PermissionDecision, ToolError> {
        if mode == PermissionMode::BypassPermissions {
            return Ok(PermissionDecision::Allow);
        }

        if matches!(tool_name, "Read" | "Grep" | "Glob") {
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
                if tool_name == "Bash" {
                    if confirm_override {
                        return Ok(PermissionDecision::Allow);
                    }
                    Self::prompt(tool_name, detail)
                } else {
                    Ok(PermissionDecision::Allow)
                }
            }
            PermissionMode::Plan => Ok(PermissionDecision::Deny(format!(
                "{tool_name} blocked in plan mode (read-only)"
            ))),
            PermissionMode::BypassPermissions => unreachable!(),
        }
    }

    fn prompt(tool_name: &str, detail: Option<&str>) -> Result<PermissionDecision, ToolError> {
        let msg = match detail {
            Some(d) => format!("myharness: {tool_name} 실행하시겠습니까? [y/N] ({d}) "),
            None => format!("myharness: {tool_name} 실행하시겠습니까? [y/N] "),
        };
        print!("{msg}");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let answer = input.trim().to_lowercase();

        if answer == "y" || answer == "yes" {
            Ok(PermissionDecision::Allow)
        } else {
            Ok(PermissionDecision::Deny(format!(
                "user declined: {tool_name}"
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
            PermissionGuard::check("Bash", PermissionMode::BypassPermissions, false, None).unwrap(),
            PermissionDecision::Allow
        );
    }

    #[test]
    #[allow(clippy::enum_variant_names)] // matches wildcards future variants of PermissionMode
    fn plan_denies_writes() {
        match PermissionGuard::check("Write", PermissionMode::Plan, false, None).unwrap() {
            PermissionDecision::Deny(_) => {}
            PermissionDecision::Allow => panic!("expected deny"),
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
                PermissionGuard::check("Read", mode, false, None).unwrap(),
                PermissionDecision::Allow
            );
            assert_eq!(
                PermissionGuard::check("Grep", mode, false, None).unwrap(),
                PermissionDecision::Allow
            );
            assert_eq!(
                PermissionGuard::check("Glob", mode, false, None).unwrap(),
                PermissionDecision::Allow
            );
        }
    }

    #[test]
    fn accept_edits_auto_allows_write_with_confirm_override() {
        assert_eq!(
            PermissionGuard::check("Write", PermissionMode::AcceptEdits, true, None).unwrap(),
            PermissionDecision::Allow
        );
    }

    #[test]
    fn confirm_override_skips_prompt_in_default() {
        assert_eq!(
            PermissionGuard::check("Write", PermissionMode::Default, true, None).unwrap(),
            PermissionDecision::Allow
        );
    }
}
