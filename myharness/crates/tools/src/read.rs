use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::error::ToolError;
use crate::tool::{Tool, ToolContext, ToolResult};

pub struct ReadTool;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &'static str {
        "Read"
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

        let path = if PathBuf::from(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            ctx.cwd.join(file_path)
        };

        let meta = fs::metadata(&path).await.map_err(ToolError::IoError)?;
        if meta.len() > 1_048_576 {
            return Err(ToolError::InvalidInput(format!(
                "file too large: {} bytes (max 1MB)",
                meta.len()
            )));
        }

        let content = fs::read_to_string(&path)
            .await
            .map_err(ToolError::IoError)?;

        let offset = input
            .get("offset")
            .and_then(serde_json::Value::as_u64)
            .map(|v| {
                #[allow(clippy::cast_possible_truncation)]
                let v = v as usize;
                v
            });
        let offset = offset.unwrap_or(0);

        let limit = input
            .get("limit")
            .and_then(serde_json::Value::as_u64)
            .map(|v| {
                #[allow(clippy::cast_possible_truncation)]
                let v = v as usize;
                v
            });
        let output = if let Some(limit) = limit {
            content
                .lines()
                .skip(offset)
                .take(limit)
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            content.lines().skip(offset).collect::<Vec<_>>().join("\n")
        };

        Ok(ToolResult {
            output,
            is_error: false,
            metadata: Some(serde_json::json!({
                "path": path.to_string_lossy(),
                "size": meta.len(),
                "line_count": content.lines().count(),
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
    async fn test_read_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello\nworld").await.unwrap();

        let tool = ReadTool;
        let ctx = ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: false,
            sanitizer_mode: SanitizerMode::default(),
        };
        let input = serde_json::json!({ "file_path": file_path.to_string_lossy() });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error);
        assert_eq!(result.output, "hello\nworld");
    }
}
