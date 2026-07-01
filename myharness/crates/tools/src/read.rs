use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::content_hash::{compute_content_hash, format_line_anchored, HASH_TAG_LENGTH};
use crate::error::ToolError;
use crate::tool::{Tool, ToolContext, ToolResult};

pub struct ReadTool;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &'static str {
        "Read"
    }

    fn description(&self) -> &'static str {
        "Read a file from the filesystem. Returns LINE:TEXT prefixed content          for hashline addressing, with a final content hash line."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute or working-directory-relative path to the file to read."
                },
                "offset": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Optional line offset to start reading from (0-indexed)."
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "Optional maximum number of lines to read."
                },
                "format": {
                    "type": "string",
                    "enum": ["line_text", "raw"],
                    "description": "Output format: line_text (LINE:TEXT prefix, default) or raw (no prefix)."
                }
            },
            "required": ["file_path"]
        })
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

        // Hashline v2 (D-104): mint a 4-hex content fingerprint so a follow-up
        // Edit can reject stale anchors. Computed against the FULL file content
        // — chunked reads share the same tag, so D-103 chunked Read still
        // produces anchors that Edit can validate.
        let content_hash = compute_content_hash(&content);

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

        // Compute total line count once (cheap — borrowed reference; no extra
        // allocation beyond the iterator).
        let total_lines = content.lines().count();

        // Always emit Hashline-style `LINE:TEXT` output (1-indexed absolute line
        // numbers so D-105 Edit can anchor by `start_line`/`end_line`). For
        // chunked reads (`offset`/`limit`), we still number from the ORIGINAL
        // file — i.e. line N in the output corresponds to line N in the file.
        let start_line = offset + 1;
        let output = if let Some(limit) = limit {
            let sliced: String = content
                .lines()
                .skip(offset)
                .take(limit)
                .collect::<Vec<_>>()
                .join("\n");
            // The trailing newline (or its absence) in `sliced` decides whether
            // the helper will emit a phantom trailing `N:` row. We rebuild via
            // `format_line_anchored` so chunked output is exactly the same
            // shape as full output would be.
            let with_nl = if content.ends_with('\n') && !sliced.ends_with('\n') {
                format!("{sliced}\n")
            } else {
                sliced
            };
            format_line_anchored(&with_nl, start_line)
        } else {
            format_line_anchored(&content, start_line)
        };

        // Compute the inclusive end_line for metadata — D-105 Edit's
        // `line_anchored` validator can sanity-check against this.
        let emitted = if let Some(limit) = limit {
            limit.min(total_lines.saturating_sub(offset))
        } else {
            total_lines
        };
        let end_line = start_line.saturating_add(emitted).saturating_sub(1);

        Ok(ToolResult {
            output,
            is_error: false,
            metadata: Some(serde_json::json!({
                "path": path.to_string_lossy(),
                "size": meta.len(),
                "line_count": total_lines,
                "format": "line_text",
                "content_hash": content_hash,
                "hash_length": HASH_TAG_LENGTH,
                "start_line": start_line,
                "end_line": end_line,
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
        // LINE:TEXT format (D-104)
        assert_eq!(result.output, "1:hello\n2:world");
    }

    #[tokio::test]
    async fn test_read_emits_content_hash_in_metadata() {
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
        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["format"], "line_text");
        assert_eq!(meta["hash_length"], 4);
        let hash = meta["content_hash"].as_str().expect("content_hash string");
        assert_eq!(hash.len(), 4);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()));
    }

    #[tokio::test]
    async fn test_read_chunked_preserves_absolute_line_numbers() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("big.txt");
        let body: String = (1..=10)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file_path, &body).await.unwrap();

        let tool = ReadTool;
        let ctx = ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: false,
            sanitizer_mode: SanitizerMode::default(),
        };
        // Read lines 3..=5 (offset=2, limit=3)
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "offset": 2,
            "limit": 3,
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        // Absolute line numbers preserved
        assert_eq!(result.output, "3:line3\n4:line4\n5:line5");
        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["start_line"], 3);
        assert_eq!(meta["end_line"], 5);
        assert_eq!(meta["line_count"], 10);
    }

    #[tokio::test]
    async fn test_read_full_file_matches_chunked_hash() {
        // Contract: D-103 chunked Read + D-104 hash must agree — same file
        // produces the same hash whether you read 1 chunk or 100.
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("big.txt");
        let body: String = (1..=20)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file_path, &body).await.unwrap();

        let tool = ReadTool;
        let ctx = ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: false,
            sanitizer_mode: SanitizerMode::default(),
        };

        let full_input = serde_json::json!({ "file_path": file_path.to_string_lossy() });
        let chunk_input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "offset": 5,
            "limit": 5,
        });

        let full_hash = tool
            .execute(&ctx, full_input)
            .await
            .unwrap()
            .metadata
            .unwrap()["content_hash"]
            .as_str()
            .unwrap()
            .to_string();
        let chunk_hash = tool
            .execute(&ctx, chunk_input)
            .await
            .unwrap()
            .metadata
            .unwrap()["content_hash"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(full_hash, chunk_hash);
    }
}
