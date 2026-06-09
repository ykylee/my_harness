use myharness_tools::SanitizerMode;
use myharness_tools::Tool;
use myharness_tools::bash::BashTool;
use myharness_tools::tool::{PermissionMode, ToolContext};

#[tokio::test]
async fn test_bash_strict_blocks_dangerous_command() {
    let tool = BashTool;
    let input = serde_json::json!({ "command": "rm -rf /", "timeout_ms": 1000 });
    let ctx = ToolContext::new("/tmp".into(), PermissionMode::BypassPermissions)
        .with_sanitizer_mode(SanitizerMode::Strict);
    let result = tool.execute(&ctx, input).await.unwrap();
    assert!(result.is_error);
    assert!(result.output.contains("blocked by sanitizer"));
}

#[tokio::test]
async fn test_bash_off_allows_safe_command() {
    let tool = BashTool;
    let input = serde_json::json!({ "command": "echo hello", "timeout_ms": 5000 });
    let ctx = ToolContext::new("/tmp".into(), PermissionMode::BypassPermissions)
        .with_sanitizer_mode(SanitizerMode::Off);
    let result = tool.execute(&ctx, input).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("hello"));
}

#[tokio::test]
async fn test_bash_safe_command_passes_strict() {
    let tool = BashTool;
    let input = serde_json::json!({ "command": "echo hello", "timeout_ms": 5000 });
    let ctx = ToolContext::new("/tmp".into(), PermissionMode::BypassPermissions)
        .with_sanitizer_mode(SanitizerMode::Strict);
    let result = tool.execute(&ctx, input).await.unwrap();
    assert!(!result.is_error);
    assert!(result.output.contains("hello"));
}
