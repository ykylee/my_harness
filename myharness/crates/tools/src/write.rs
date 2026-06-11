use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::error::ToolError;
use crate::permission::{PermissionDecision, PermissionGuard};
use crate::tool::{Tool, ToolContext, ToolResult};

pub struct WriteTool;

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &'static str {
        "Write"
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

        let content = input
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing content".into()))?;

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

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(ToolError::IoError)?;
        }

        fs::write(&path, content)
            .await
            .map_err(ToolError::IoError)?;

        Ok(ToolResult {
            output: format!("wrote {} bytes to {}", content.len(), path.display()),
            is_error: false,
            metadata: Some(serde_json::json!({
                "path": path.to_string_lossy(),
                "size": content.len(),
            })),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::*;
    use crate::sanitizer::SanitizerMode;
    use crate::tool::{PermissionMode, ToolContext};

    #[tokio::test]
    async fn test_write_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("written.txt");

        let tool = WriteTool;
        let ctx = ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: true,
            sanitizer_mode: SanitizerMode::default(),
        };
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "content": "test content",
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error);

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "test content");
    }
}
