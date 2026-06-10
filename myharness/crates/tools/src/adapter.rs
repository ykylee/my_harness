use crate::error::ToolError;
use crate::registry::ToolRegistry;
use crate::tool::{ToolContext, ToolResult};

pub async fn execute_by_name(
    name: &str,
    input: serde_json::Value,
    ctx: &ToolContext,
) -> Result<ToolResult, ToolError> {
    let registry = ToolRegistry::default_tools();
    let tool = registry
        .get(name)
        .ok_or_else(|| ToolError::Other(format!("unknown tool: {}", name)))?;
    tool.execute(ctx, input).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::PermissionMode;

    #[tokio::test]
    async fn test_execute_read() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test.txt");
        tokio::fs::write(&file, "hello world").await.unwrap();

        let ctx = ToolContext::new(dir.path().to_path_buf(), PermissionMode::BypassPermissions);
        let input = serde_json::json!({ "file_path": file.to_str().unwrap() });
        let result = execute_by_name("Read", input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("hello world"));
    }

    #[tokio::test]
    async fn test_execute_unknown_tool() {
        let ctx = ToolContext::new("/tmp".into(), PermissionMode::BypassPermissions);
        let result = execute_by_name("Nonexistent", serde_json::json!({}), &ctx).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_bash_safe() {
        let ctx = ToolContext::new("/tmp".into(), PermissionMode::BypassPermissions);
        let input = serde_json::json!({ "command": "echo hello", "timeout_ms": 5000 });
        let result = execute_by_name("Bash", input, &ctx).await.unwrap();
        assert!(!result.is_error);
    }

    #[tokio::test]
    async fn test_execute_write_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test_write.txt");

        let ctx = ToolContext::new(dir.path().to_path_buf(), PermissionMode::BypassPermissions);
        let write_input = serde_json::json!({ "file_path": file.to_str().unwrap(), "content": "written content" });
        let write_result = execute_by_name("Write", write_input, &ctx).await.unwrap();
        assert!(!write_result.is_error);

        let read_input = serde_json::json!({ "file_path": file.to_str().unwrap() });
        let read_result = execute_by_name("Read", read_input, &ctx).await.unwrap();
        assert!(!read_result.is_error);
        assert!(read_result.output.contains("written content"));
    }

    #[tokio::test]
    async fn test_execute_edit() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("test_edit.txt");
        tokio::fs::write(&file, "old content").await.unwrap();

        let ctx = ToolContext::new(dir.path().to_path_buf(), PermissionMode::BypassPermissions);
        let input = serde_json::json!({
            "file_path": file.to_str().unwrap(),
            "old_string": "old",
            "new_string": "new",
        });
        let result = execute_by_name("Edit", input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.output.contains("replaced"));
    }
}
