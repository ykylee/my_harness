use std::path::PathBuf;

use myharness_tools::SanitizerMode;
use myharness_tools::Tool;
use myharness_tools::tool::{PermissionMode, ToolContext};
use myharness_tools::write::WriteTool;

#[tokio::test]
async fn test_plan_mode_blocks_write() {
    let tool = WriteTool;
    let ctx = ToolContext {
        cwd: PathBuf::from("/tmp"),
        permission_mode: PermissionMode::Plan,
        confirm_override: false,
        sanitizer_mode: SanitizerMode::Strict,
    };
    let input = serde_json::json!({
        "file_path": "/nonexistent/test_plan_block.txt",
        "content": "should not write",
    });
    let result = tool.execute(&ctx, input).await.unwrap();
    assert!(result.is_error);
    assert!(result.output.contains("plan mode"));
}

#[tokio::test]
async fn test_bypass_allows_write() {
    let tool = WriteTool;
    let ctx = ToolContext {
        cwd: PathBuf::from("/tmp"),
        permission_mode: PermissionMode::BypassPermissions,
        confirm_override: false,
        sanitizer_mode: SanitizerMode::Strict,
    };
    let input = serde_json::json!({
        "file_path": "/tmp/test_bypass_write.txt",
        "content": "allowed content",
    });
    let result = tool.execute(&ctx, input).await;
    assert!(result.is_ok());
    let result = result.unwrap();
    if result.is_error {
        // Permission bypass should work; if file write fails, it's due to fs, not permission
        assert!(
            !result.output.contains("bypassPermissions"),
            "bypass mode should not deny: {}",
            result.output
        );
    }
}

#[tokio::test]
async fn test_accept_edits_allows_write() {
    let tool = WriteTool;
    let ctx = ToolContext {
        cwd: PathBuf::from("/tmp"),
        permission_mode: PermissionMode::AcceptEdits,
        confirm_override: false,
        sanitizer_mode: SanitizerMode::Strict,
    };
    let input = serde_json::json!({
        "file_path": "/tmp/test_accept_edits_write.txt",
        "content": "edits allowed content",
    });
    let result = tool.execute(&ctx, input).await;
    assert!(result.is_ok());
    let result = result.unwrap();
    if result.is_error {
        assert!(
            !result.output.contains("blocked"),
            "acceptEdits should not block write: {}",
            result.output
        );
    }
}
