use std::path::PathBuf;

use async_trait::async_trait;
use tokio::fs;

use crate::content_hash::{HASH_TAG_LENGTH, compute_content_hash, format_line_anchored};
use crate::error::ToolError;
use crate::tool::{Tool, ToolContext, ToolResult};

/// Default line cap when caller does not specify `limit`.
/// Calibrated against D-103 prompt guidance ("~200 chunks for >500 lines")
/// and a typical LLM context window — 500 LINE:TEXT lines stay well under
/// the per-result budget even for ~120-char lines.
const DEFAULT_READ_LINE_LIMIT: usize = 500;

/// Hard upper bound on `limit`. Defends the tool against a caller asking
/// for an unbounded slice — anything beyond this is clamped. Picked to
/// fit comfortably inside the 1MB file-size cap above.
const MAX_READ_LINE_LIMIT: usize = 5_000;

pub struct ReadTool;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &'static str {
        "Read"
    }

    fn description(&self) -> &'static str {
        "Read a file from the filesystem. Returns LINE:TEXT prefixed content for hashline addressing. When the file is larger than the read limit, the response is automatically truncated and metadata signals `has_more` + `next_offset` so the caller can fetch the next chunk."
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
                    "description": "Optional maximum number of lines to read. Defaults to 500 when omitted. Capped at 5000; larger values are clamped and the response is marked truncated."
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
        // Apply default + max clamp (D-112). Caller-supplied limits above
        // MAX_READ_LINE_LIMIT are silently clamped — the resulting metadata
        // surfaces `has_more: true` if the file was actually truncated, so
        // the caller can still discover there's more to read. A `limit: 0`
        // is treated as "not specified" to match the schema's minimum:1 hint.
        let limit = match limit {
            Some(0) | None => DEFAULT_READ_LINE_LIMIT,
            Some(n) => n.min(MAX_READ_LINE_LIMIT),
        };

        // Compute total line count once (cheap — borrowed reference; no extra
        // allocation beyond the iterator).
        let total_lines = content.lines().count();

        // Always emit Hashline-style `LINE:TEXT` output (1-indexed absolute line
        // numbers so D-105 Edit can anchor by `start_line`/`end_line`). For
        // chunked reads (`offset`/`limit`), we still number from the ORIGINAL
        // file — i.e. line N in the output corresponds to line N in the file.
        let start_line = offset + 1;
        let output = if limit > 0 {
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

        // `emitted` reflects the lines actually returned. If the file has
        // fewer lines than `limit` requests, `emitted` shrinks to what's
        // available — `has_more` then correctly reports false.
        let emitted = limit.min(total_lines.saturating_sub(offset));
        let end_line = start_line.saturating_add(emitted).saturating_sub(1);

        // Build the metadata payload. D-112 adds truncation hints so the
        // caller (LLM) can discover there's more to read without guessing.
        let has_more = (offset + emitted) < total_lines;
        let mut meta = serde_json::json!({
            "path": path.to_string_lossy(),
            "size": meta.len(),
            "line_count": total_lines,
            "format": "line_text",
            "content_hash": content_hash,
            "hash_length": HASH_TAG_LENGTH,
            "start_line": start_line,
            "end_line": end_line,
            "limit": limit,
            "has_more": has_more,
        });
        if has_more {
            meta["next_offset"] = serde_json::json!(offset + emitted);
        }

        Ok(ToolResult {
            output,
            is_error: false,
            metadata: Some(meta),
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
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase())
        );
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

    // --- D-112: large file auto-truncation + has_more / next_offset hints ---

    fn ctx_default() -> ToolContext {
        ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: false,
            sanitizer_mode: SanitizerMode::default(),
        }
    }

    /// 1000-line file read with no `limit` returns the first 500 lines, marks
    /// `has_more: true`, and surfaces `next_offset = 500` for the caller to
    /// fetch the next chunk. This is the headline D-112 behavior — the LLM
    /// no longer has to remember the prompt-level "use limit+offset" rule.
    #[tokio::test]
    async fn test_d112_auto_truncates_large_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("big.txt");
        let body: String = (1..=1000)
            .map(|i| format!("line{i:04}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file_path, &body).await.unwrap();

        let result = ReadTool
            .execute(
                &ctx_default(),
                serde_json::json!({ "file_path": file_path.to_string_lossy() }),
            )
            .await
            .unwrap();
        assert!(!result.is_error);
        // Output should end with line 500 (1-indexed absolute).
        assert!(
            result.output.lines().last().unwrap().starts_with("500:"),
            "expected last emitted line to be 500:, got: {}",
            result.output.lines().last().unwrap()
        );
        // Output must NOT contain line 501.
        assert!(!result.output.contains("\n501:"));
        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["has_more"], serde_json::json!(true));
        assert_eq!(meta["next_offset"], serde_json::json!(500));
        assert_eq!(meta["limit"], serde_json::json!(500));
        assert_eq!(meta["line_count"], serde_json::json!(1000));
        assert_eq!(meta["end_line"], serde_json::json!(500));
        assert_eq!(meta["start_line"], serde_json::json!(1));
    }

    /// Caller can request an explicit `limit` above the MAX_READ_LINE_LIMIT
    /// (5000) — we clamp it to the max and surface `has_more: true` for any
    /// file larger than the clamp. This protects the tool from unbounded
    /// output while still being useful for very large explicit scans.
    #[tokio::test]
    async fn test_d112_clamps_excessive_limit() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("big.txt");
        let body: String = (1..=7000)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file_path, &body).await.unwrap();

        let result = ReadTool
            .execute(
                &ctx_default(),
                serde_json::json!({
                    "file_path": file_path.to_string_lossy(),
                    "limit": 10_000,
                }),
            )
            .await
            .unwrap();
        let meta = result.metadata.expect("metadata required");
        // 10000 was clamped down to 5000.
        assert_eq!(meta["limit"], serde_json::json!(5_000));
        assert_eq!(meta["has_more"], serde_json::json!(true));
        assert_eq!(meta["next_offset"], serde_json::json!(5_000));
        assert_eq!(meta["end_line"], serde_json::json!(5_000));
    }

    /// When the file is smaller than the (auto or explicit) limit, `has_more`
    /// must be `false` and `next_offset` must be absent — otherwise the LLM
    /// would loop forever fetching "more" that doesn't exist.
    #[tokio::test]
    async fn test_d112_no_truncation_signals_correctly() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("small.txt");
        let body: String = (1..=10)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file_path, &body).await.unwrap();

        let result = ReadTool
            .execute(
                &ctx_default(),
                serde_json::json!({ "file_path": file_path.to_string_lossy() }),
            )
            .await
            .unwrap();
        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["has_more"], serde_json::json!(false));
        assert!(
            meta.get("next_offset").is_none(),
            "next_offset must be absent when has_more is false, got: {}",
            meta
        );
        assert_eq!(meta["limit"], serde_json::json!(500));
        assert_eq!(meta["end_line"], serde_json::json!(10));
    }

    /// `limit: 0` is treated as "not specified" (defensive — the schema says
    /// minimum:1, but real callers sometimes pass 0). Result must equal the
    /// no-limit path.
    #[tokio::test]
    async fn test_d112_limit_zero_uses_default() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("big.txt");
        let body: String = (1..=1000)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file_path, &body).await.unwrap();

        let result = ReadTool
            .execute(
                &ctx_default(),
                serde_json::json!({
                    "file_path": file_path.to_string_lossy(),
                    "limit": 0,
                }),
            )
            .await
            .unwrap();
        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["limit"], serde_json::json!(500));
        assert_eq!(meta["has_more"], serde_json::json!(true));
        assert_eq!(meta["end_line"], serde_json::json!(500));
    }

    /// Walking a 1500-line file with two Read calls (default 500, then
    /// `offset: 500`, then `offset: 1000`) — the second call sees
    /// `has_more: true` and the third sees `has_more: false`. This is the
    /// end-to-end contract the LLM relies on for large-file code review.
    #[tokio::test]
    async fn test_d112_paginated_walk_through_large_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("walk.txt");
        let body: String = (1..=1500)
            .map(|i| format!("line{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&file_path, &body).await.unwrap();

        // First chunk: lines 1..=500, has_more → next_offset 500.
        let r1 = ReadTool
            .execute(
                &ctx_default(),
                serde_json::json!({ "file_path": file_path.to_string_lossy() }),
            )
            .await
            .unwrap();
        let m1 = r1.metadata.unwrap();
        assert_eq!(m1["start_line"], 1);
        assert_eq!(m1["end_line"], 500);
        assert_eq!(m1["has_more"], serde_json::json!(true));
        assert_eq!(m1["next_offset"], serde_json::json!(500));

        // Second chunk: offset=500, limit=500 → lines 501..=1000, has_more → 1000.
        let r2 = ReadTool
            .execute(
                &ctx_default(),
                serde_json::json!({
                    "file_path": file_path.to_string_lossy(),
                    "offset": 500,
                    "limit": 500,
                }),
            )
            .await
            .unwrap();
        let m2 = r2.metadata.unwrap();
        assert_eq!(m2["start_line"], 501);
        assert_eq!(m2["end_line"], 1000);
        assert_eq!(m2["has_more"], serde_json::json!(true));
        assert_eq!(m2["next_offset"], serde_json::json!(1000));

        // Third chunk: offset=1000, limit=500 → lines 1001..=1500, has_more=false.
        let r3 = ReadTool
            .execute(
                &ctx_default(),
                serde_json::json!({
                    "file_path": file_path.to_string_lossy(),
                    "offset": 1000,
                    "limit": 500,
                }),
            )
            .await
            .unwrap();
        let m3 = r3.metadata.unwrap();
        assert_eq!(m3["start_line"], 1001);
        assert_eq!(m3["end_line"], 1500);
        assert_eq!(m3["has_more"], serde_json::json!(false));
        assert!(m3.get("next_offset").is_none());
    }
}
