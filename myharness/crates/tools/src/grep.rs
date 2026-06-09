use std::path::PathBuf;

use async_trait::async_trait;
use regex::Regex;
use tokio::fs;
use walkdir::WalkDir;

use crate::error::ToolError;
use crate::tool::{Tool, ToolContext, ToolResult};

pub struct GrepTool;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &'static str {
        "grep"
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        input: serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        let pattern = input
            .get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing pattern".into()))?;

        let search_path = input
            .get("path")
            .and_then(|v| v.as_str())
            .map(|p| {
                if PathBuf::from(p).is_absolute() {
                    PathBuf::from(p)
                } else {
                    ctx.cwd.join(p)
                }
            })
            .unwrap_or_else(|| ctx.cwd.clone());

        let include_filter = input.get("include").and_then(|v| v.as_str());

        let case_insensitive = input
            .get("case_insensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let re = if case_insensitive {
            Regex::new(&format!("(?i){}", pattern))
                .map_err(|e| ToolError::InvalidInput(format!("invalid regex: {}", e)))?
        } else {
            Regex::new(pattern)
                .map_err(|e| ToolError::InvalidInput(format!("invalid regex: {}", e)))?
        };

        let max_results: usize = 100;
        let mut results = Vec::new();

        for entry in WalkDir::new(&search_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            if let Some(include) = include_filter {
                let pat = glob::Pattern::new(include).map_err(|e| {
                    ToolError::InvalidInput(format!("invalid include pattern: {}", e))
                })?;
                if !pat.matches_path(entry.path()) {
                    continue;
                }
            }

            if results.len() >= max_results {
                break;
            }

            if let Ok(content) = fs::read_to_string(entry.path()).await {
                for (line_num, line) in content.lines().enumerate() {
                    if results.len() >= max_results {
                        break;
                    }
                    if re.is_match(line) {
                        results.push(serde_json::json!({
                            "file": entry.path().to_string_lossy(),
                            "line": line_num + 1,
                            "content": line,
                        }));
                    }
                }
            }
        }

        let output = serde_json::to_string_pretty(&results)
            .map_err(|e| ToolError::Other(format!("serialization error: {}", e)))?;

        Ok(ToolResult {
            output,
            is_error: false,
            metadata: Some(serde_json::json!({
                "matches": results.len(),
                "capped": results.len() >= max_results,
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
    async fn test_grep_find() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("search.txt");
        fs::write(&file_path, "needle\nhaystack\nneedle2")
            .await
            .unwrap();

        let tool = GrepTool;
        let ctx = ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: false,
            sanitizer_mode: Default::default(),
        };
        let input = serde_json::json!({
            "pattern": "needle",
            "path": dir.path().to_string_lossy(),
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error);

        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result.output).unwrap();
        assert!(parsed.len() >= 1);
    }
}
