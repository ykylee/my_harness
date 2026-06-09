use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::error::ToolError;
use crate::permission::{PermissionDecision, PermissionGuard};
use crate::tool::{Tool, ToolContext, ToolResult};

pub struct EditTool;

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &'static str {
        "edit"
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        input: serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        let file_path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing file_path".into()))?;

        let old_string = input
            .get("old_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing old_string".into()))?;

        let new_string = input
            .get("new_string")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing new_string".into()))?;

        let replace_all = input
            .get("replace_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let decision =
            PermissionGuard::check(self.name(), ctx.permission_mode, ctx.confirm_override, None)?;
        match decision {
            PermissionDecision::Allow => {}
            PermissionDecision::Deny(reason) => {
                return Ok(ToolResult::error(reason));
            }
        }

        let path = if PathBuf::from(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            ctx.cwd.join(file_path)
        };

        let content = fs::read_to_string(&path)
            .await
            .map_err(ToolError::IoError)?;

        let count = content.matches(old_string).count();
        if count == 0 {
            return Err(ToolError::InvalidInput(format!(
                "old_string not found in {}",
                path.display()
            )));
        }
        if !replace_all && count > 1 {
            return Err(ToolError::InvalidInput(format!(
                "found {} matches for old_string in {}. provide more context or set replace_all=true",
                count,
                path.display()
            )));
        }

        let new_content = if replace_all {
            content.replace(old_string, new_string)
        } else {
            content.replacen(old_string, new_string, 1)
        };

        fs::write(&path, &new_content)
            .await
            .map_err(ToolError::IoError)?;

        Ok(ToolResult {
            output: format!("replaced {count} occurrence(s) in {}", path.display()),
            is_error: false,
            metadata: Some(serde_json::json!({
                "path": path.to_string_lossy(),
                "replacements": count,
            })),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;
    use crate::tool::{PermissionMode, ToolContext};

    #[tokio::test]
    async fn test_edit_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("edit.txt");
        fs::write(&file_path, "hello world").await.unwrap();

        let tool = EditTool;
        let ctx = ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: true,
            sanitizer_mode: Default::default(),
        };
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "old_string": "world",
            "new_string": "there",
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error);

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "hello there");
    }
}
