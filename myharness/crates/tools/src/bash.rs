use async_trait::async_trait;
use tokio::process::Command;
use tokio::time::{Duration, timeout};

use crate::error::ToolError;
use crate::permission::{PermissionDecision, PermissionGuard};
use crate::sanitizer::BashSanitizer;
use crate::tool::{Tool, ToolContext, ToolResult};

pub struct BashTool;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &'static str {
        "Bash"
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        input: serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        let command = input
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing command".into()))?;

        let timeout_ms = input
            .get("timeout_ms")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(30_000);

        let decision = PermissionGuard::check(
            self.name(),
            ctx.permission_mode,
            ctx.confirm_override,
            Some(command),
        )?;
        match decision {
            PermissionDecision::Allow => {}
            PermissionDecision::Deny(reason) => {
                return Ok(ToolResult::error(reason));
            }
        }

        if let Err(violation) = BashSanitizer::check(command, ctx.sanitizer_mode) {
            return Ok(ToolResult::error(format!(
                "blocked by sanitizer: {} (pattern={})",
                violation.reason, violation.pattern
            )));
        }

        let output = timeout(Duration::from_millis(timeout_ms), async {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&ctx.cwd)
                .output()
                .await
        })
        .await
        .map_err(|_| {
            ToolError::ExecutionFailed(format!("command timed out after {timeout_ms}ms"))
        })?
        .map_err(ToolError::IoError)?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = if stderr.is_empty() {
            stdout.clone()
        } else {
            format!("{stdout}{stderr}")
        };

        Ok(ToolResult {
            output: combined,
            is_error: !output.status.success(),
            metadata: Some(serde_json::json!({
                "exit_code": output.status.code().unwrap_or(-1),
                "stdout": stdout,
                "stderr": stderr,
            })),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::sanitizer::SanitizerMode;
    use crate::tool::{PermissionMode, ToolContext};

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool;
        let ctx = ToolContext {
            cwd: PathBuf::from("/tmp"),
            permission_mode: PermissionMode::Default,
            confirm_override: true,
            sanitizer_mode: SanitizerMode::Strict,
        };
        let input = serde_json::json!({
            "command": "echo hello",
            "timeout_ms": 5000,
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("hello"));
    }
}
