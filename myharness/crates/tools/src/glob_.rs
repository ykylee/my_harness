use std::path::PathBuf;

use async_trait::async_trait;
use walkdir::WalkDir;

use crate::error::ToolError;
use crate::tool::{Tool, ToolContext, ToolResult};

pub struct GlobTool;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &'static str {
        "Glob"
    }

    fn description(&self) -> &'static str {
        "Find files whose path matches a glob pattern. Returns matching paths,          sorted, relative to the optional base path."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Glob pattern (e.g. '**/*.rs', 'src/*.toml')."
                },
                "path": {
                    "type": "string",
                    "description": "Optional base directory; defaults to working directory."
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        input: serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        let pattern_str = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing pattern".into()))?;

        let search_path = input
            .get("path")
            .and_then(|v| v.as_str()).map_or_else(|| ctx.cwd.clone(), |p| {
                if PathBuf::from(p).is_absolute() {
                    PathBuf::from(p)
                } else {
                    ctx.cwd.join(p)
                }
            });

        let glob_pattern = glob::Pattern::new(pattern_str)
            .map_err(|e| ToolError::InvalidInput(format!("invalid glob pattern: {e}")))?;

        let max_results: usize = 1000;
        let mut matches = Vec::new();

        for entry in WalkDir::new(&search_path)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if matches.len() >= max_results {
                break;
            }
            if glob_pattern.matches_path(entry.path()) {
                matches.push(entry.path().to_string_lossy().to_string());
            }
        }

        let output = serde_json::to_string_pretty(&matches)
            .map_err(|e| ToolError::Other(format!("serialization error: {e}")))?;

        Ok(ToolResult {
            output,
            is_error: false,
            metadata: Some(serde_json::json!({
                "matches": matches.len(),
                "capped": matches.len() >= max_results,
            })),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;
    use tokio::fs;

    use super::*;
    use crate::sanitizer::SanitizerMode;
    use crate::tool::{PermissionMode, ToolContext};

    #[tokio::test]
    async fn test_glob_find() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("test.txt"), "content")
            .await
            .unwrap();
        fs::write(dir.path().join("other.rs"), "fn main() {}")
            .await
            .unwrap();

        let tool = GlobTool;
        let ctx = ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: false,
            sanitizer_mode: SanitizerMode::default(),
        };
        let input = serde_json::json!({
            "pattern": "*.txt",
            "path": dir.path().to_string_lossy(),
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error);

        let parsed: Vec<String> = serde_json::from_str(&result.output).unwrap();
        assert!(!parsed.is_empty());
        assert!(parsed[0].ends_with("test.txt"));
    }
}
