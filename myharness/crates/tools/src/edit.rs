use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::fs;

use crate::content_hash::compute_content_hash;
use crate::error::ToolError;
use crate::permission::{PermissionDecision, PermissionGuard};
use crate::tool::{Tool, ToolContext, ToolResult};

pub struct EditTool;

/// Hashline v2 (D-105) line-anchored edit payload.
///
/// Lines are 1-indexed and inclusive on both ends (`start_line..=end_line`).
/// `expected_hash` is the 4-hex content hash minted by the prior `Read` call;
/// if the live file's hash no longer matches, the edit is rejected before any
/// bytes are written.
#[derive(Debug, Deserialize)]
struct LineAnchoredEdit {
    start_line: usize,
    end_line: usize,
    expected_hash: String,
    replacement: String,
}

/// Replace a 1-indexed inclusive line range with the given replacement text.
///
/// Semantics:
/// - `start_line_1` and `end_line_1` are both 1-indexed and inclusive.
/// - `end_line_1` must be `<=` the line count as reported by `content.lines().count()`.
/// - An empty `replacement` deletes the targeted range.
/// - A trailing newline on the original content is preserved on the output.
/// - Other lines are passed through verbatim (no normalization).
///
/// Returns `Err(msg)` on any validation failure with an actionable message.
fn apply_line_replacement(
    content: &str,
    start_line_1: usize,
    end_line_1: usize,
    replacement: &str,
) -> Result<String, String> {
    if start_line_1 == 0 {
        return Err("start_line must be >= 1 (1-indexed)".to_string());
    }
    if end_line_1 < start_line_1 {
        return Err(format!(
            "end_line ({end_line_1}) must be >= start_line ({start_line_1})"
        ));
    }

    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();
    if end_line_1 > total {
        return Err(format!(
            "end_line ({end_line_1}) out of range: file has {total} line(s)"
        ));
    }

    let start_idx = start_line_1 - 1;
    let end_idx_excl = end_line_1; // exclusive upper bound for slicing
    let pre = &lines[..start_idx];
    let post = &lines[end_idx_excl..];

    let repl_lines: Vec<&str> = if replacement.is_empty() {
        Vec::new()
    } else {
        replacement.split('\n').collect()
    };

    let mut combined: Vec<&str> =
        Vec::with_capacity(pre.len() + repl_lines.len() + post.len());
    combined.extend_from_slice(pre);
    combined.extend(repl_lines);
    combined.extend_from_slice(post);

    let mut out = combined.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &'static str {
        "Edit"
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        input: serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        let file_path = input
            .get("file_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing file_path".into()))?
            .to_string();

        // Hashline v2 (D-105): opt-in `line_anchored` mode. Dispatch FIRST so
        // the existing `old_string`/`new_string`/`replace_all` path stays
        // byte-identical for callers that have not opted in.
        if input.get("line_anchored").is_some() {
            return self.execute_line_anchored(ctx, &file_path, input).await;
        }

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
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let decision =
            PermissionGuard::check(self.name(), ctx.permission_mode, ctx.confirm_override, None)?;
        match decision {
            PermissionDecision::Allow => {}
            PermissionDecision::Deny(reason) => {
                return Ok(ToolResult::error(reason));
            }
        }

        let path = if PathBuf::from(&file_path).is_absolute() {
            PathBuf::from(&file_path)
        } else {
            ctx.cwd.join(&file_path)
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

impl EditTool {
    /// Hashline v2 (D-105) `line_anchored` mode — validate the anchor's hash,
    /// swap the targeted line range, then write and report.
    async fn execute_line_anchored(
        &self,
        ctx: &ToolContext,
        file_path: &str,
        input: serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        // Parse the nested payload. `expected_value` extraction mirrors how the
        // sibling `old_string`/`new_string` keys are read.
        let la: LineAnchoredEdit = serde_json::from_value(
            input
                .get("line_anchored")
                .cloned()
                .ok_or_else(|| ToolError::InvalidInput("missing line_anchored".into()))?,
        )
        .map_err(|e| ToolError::InvalidInput(format!("invalid line_anchored: {e}")))?;

        // Same permission contract as the legacy path: check first, then read.
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

        // Stale-anchor gate (spec §5.2 step 3). Fail BEFORE any line math so
        // a malformed LLM payload can never silently corrupt a moved file.
        let current_hash = compute_content_hash(&content);
        if current_hash != la.expected_hash {
            return Err(ToolError::InvalidInput(format!(
                "stale anchor: file modified; re-read with `Read` tool (current hash {current_hash}, expected {})",
                la.expected_hash
            )));
        }

        // Range + apply (spec §5.2 steps 4-5). `apply_line_replacement`
        // returns user-actionable error strings; surface them as InvalidInput.
        let new_content = apply_line_replacement(
            &content,
            la.start_line,
            la.end_line,
            &la.replacement,
        )
        .map_err(ToolError::InvalidInput)?;

        fs::write(&path, &new_content)
            .await
            .map_err(ToolError::IoError)?;

        let new_hash = compute_content_hash(&new_content);
        let replaced_lines = la.end_line - la.start_line + 1;

        Ok(ToolResult {
            output: format!(
                "replaced {replaced_lines} line(s) ({}..={}) in {}",
                la.start_line,
                la.end_line,
                path.display()
            ),
            is_error: false,
            metadata: Some(serde_json::json!({
                "path": path.to_string_lossy(),
                "mode": "line_anchored",
                "start_line": la.start_line,
                "end_line": la.end_line,
                "replaced_lines": replaced_lines,
                "old_hash": la.expected_hash,
                "new_hash": new_hash,
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

    fn make_ctx() -> ToolContext {
        ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: true,
            sanitizer_mode: SanitizerMode::default(),
        }
    }

    #[tokio::test]
    async fn test_edit_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("edit.txt");
        fs::write(&file_path, "hello world").await.unwrap();

        let tool = EditTool;
        let ctx = make_ctx();
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

    fn make_tool() -> EditTool {
        EditTool
    }

    // --- line_anchored mode (Hashline v2 / D-105) -----------------------------

    #[tokio::test]
    async fn test_edit_line_anchored_happy_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("anchored.txt");
        let original = "line1\nline2\nline3\nline4\nline5\nline6\n";
        fs::write(&file_path, original).await.unwrap();

        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "line_anchored": {
                "start_line": 2,
                "end_line": 4,
                "expected_hash": expected_hash,
                "replacement": "REPLACED-A\nREPLACED-B\nREPLACED-C",
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "result: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(
            read_back,
            "line1\nREPLACED-A\nREPLACED-B\nREPLACED-C\nline5\nline6\n"
        );

        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["mode"], "line_anchored");
        assert_eq!(meta["start_line"], 2);
        assert_eq!(meta["end_line"], 4);
        assert_eq!(meta["replaced_lines"], 3);
        assert_eq!(meta["old_hash"], expected_hash);
        assert!(
            meta["new_hash"].as_str().is_some(),
            "new_hash must be present"
        );
        // New hash must differ from old hash (we actually changed bytes).
        assert_ne!(meta["new_hash"], expected_hash);
    }

    #[tokio::test]
    async fn test_edit_line_anchored_stale_anchor() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("stale.txt");
        let original = "alpha\nbeta\ngamma\ndelta\n";
        fs::write(&file_path, original).await.unwrap();

        // Step 1: read hash as the LLM would.
        let initial = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&initial);

        // Step 2: file mutated externally (e.g. another tool / git pull).
        fs::write(&file_path, "alpha\nBETA-MUTATED\ngamma\ndelta\n")
            .await
            .unwrap();

        let tool = make_tool();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "line_anchored": {
                "start_line": 2,
                "end_line": 2,
                "expected_hash": expected_hash,
                "replacement": "REPLACED",
            }
        });
        let err = tool
            .execute(&ctx, input)
            .await
            .expect_err("stale anchor must surface as an error");
        let msg = err.to_string();
        assert!(
            msg.contains("stale anchor"),
            "expected 'stale anchor' in error, got: {msg}"
        );

        // File must NOT have been touched.
        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "alpha\nBETA-MUTATED\ngamma\ndelta\n");
    }

    #[tokio::test]
    async fn test_edit_line_anchored_out_of_range() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("range.txt");
        let body = "one\ntwo\nthree\n";
        fs::write(&file_path, body).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_ctx();
        // 3-line file, end_line = 99 must be rejected.
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "line_anchored": {
                "start_line": 1,
                "end_line": 99,
                "expected_hash": expected_hash,
                "replacement": "X",
            }
        });
        let err = tool
            .execute(&ctx, input)
            .await
            .expect_err("out-of-range must error");
        let msg = err.to_string();
        assert!(
            msg.contains("out of range") || msg.contains("99"),
            "expected out-of-range message, got: {msg}"
        );

        // File untouched.
        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, body);
    }

    #[tokio::test]
    async fn test_edit_line_anchored_invalid_range() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("invalid.txt");
        let body = "a\nb\nc\nd\n";
        fs::write(&file_path, body).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_ctx();
        // start > end: invalid range.
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "line_anchored": {
                "start_line": 5,
                "end_line": 2,
                "expected_hash": expected_hash,
                "replacement": "X",
            }
        });
        let err = tool
            .execute(&ctx, input)
            .await
            .expect_err("invalid range must error");
        let msg = err.to_string();
        assert!(
            msg.contains("end_line") && msg.contains("start_line"),
            "expected range-validation message, got: {msg}"
        );

        // File untouched.
        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, body);
    }

    #[tokio::test]
    async fn test_edit_line_anchored_single_line() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("single.txt");
        let body = "first\nsecond\nthird\nfourth\n";
        fs::write(&file_path, body).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "line_anchored": {
                "start_line": 3,
                "end_line": 3,
                "expected_hash": expected_hash,
                "replacement": "THIRD-NEW",
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "result: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "first\nsecond\nTHIRD-NEW\nfourth\n");

        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["replaced_lines"], 1);
    }

    #[tokio::test]
    async fn test_edit_line_anchored_preserve_trailing_newline() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("trailing.txt");
        let body = "x\ny\nz\n"; // trailing newline
        fs::write(&file_path, body).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "line_anchored": {
                "start_line": 2,
                "end_line": 2,
                "expected_hash": expected_hash,
                "replacement": "Y-NEW",
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "result: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert!(
            read_back.ends_with('\n'),
            "trailing newline must be preserved, got: {read_back:?}"
        );
        assert_eq!(read_back, "x\nY-NEW\nz\n");
    }

    #[tokio::test]
    async fn test_edit_line_anchored_entire_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("entire.txt");
        let body = "old-line1\nold-line2\nold-line3\n";
        fs::write(&file_path, body).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_ctx();
        let new_body = "new-A\nnew-B\nnew-C\n";
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "line_anchored": {
                "start_line": 1,
                "end_line": 3,
                "expected_hash": expected_hash,
                "replacement": "new-A\nnew-B\nnew-C",
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "result: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, new_body);

        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["replaced_lines"], 3);
        assert_eq!(meta["start_line"], 1);
        assert_eq!(meta["end_line"], 3);
    }

    #[tokio::test]
    async fn test_edit_line_anchored_multiline_replacement() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("multi.txt");
        let body = "header\nold-body\nfooter\n";
        fs::write(&file_path, body).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "line_anchored": {
                "start_line": 2,
                "end_line": 2,
                "expected_hash": expected_hash,
                "replacement": "X\nY\nZ",
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "result: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        // "X\nY\nZ" splits into 3 lines, so single-line replacement expands to 3.
        assert_eq!(read_back, "header\nX\nY\nZ\nfooter\n");
    }

    #[tokio::test]
    async fn test_edit_old_mode_still_works() {
        // Regression: the legacy old_string/new_string/replace_all path must be
        // untouched after D-105 wiring.
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("legacy.txt");
        fs::write(&file_path, "foo bar foo baz").await.unwrap();

        let tool = make_tool();
        let ctx = make_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "old_string": "foo",
            "new_string": "FOO",
            "replace_all": true,
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "result: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "FOO bar FOO baz");

        // Legacy path does NOT emit the line_anchored metadata shape.
        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["replacements"], 2);
    }

    // --- apply_line_replacement direct unit tests ----------------------------

    #[test]
    fn test_apply_line_replacement_unit() {
        // Happy: single line replaced, trailing \n preserved.
        let out = apply_line_replacement("a\nb\nc\n", 2, 2, "B").unwrap();
        assert_eq!(out, "a\nB\nc\n");

        // Multi-line replacement text expands the range.
        let out = apply_line_replacement("a\nb\nc\n", 2, 2, "X\nY\nZ").unwrap();
        assert_eq!(out, "a\nX\nY\nZ\nc\n");

        // Range covers multiple lines.
        let out = apply_line_replacement("a\nb\nc\nd\ne\n", 2, 4, "B\nC\nD").unwrap();
        assert_eq!(out, "a\nB\nC\nD\ne\n");

        // Entire file replacement: start=1, end=total.
        let out = apply_line_replacement("a\nb\nc", 1, 3, "X\nY\nZ").unwrap();
        assert_eq!(out, "X\nY\nZ");

        // No trailing newline: output has no trailing newline.
        let out = apply_line_replacement("a\nb\nc", 2, 2, "B").unwrap();
        assert_eq!(out, "a\nB\nc");

        // Empty replacement deletes the range.
        let out = apply_line_replacement("a\nb\nc\nd\n", 2, 3, "").unwrap();
        assert_eq!(out, "a\nd\n");

        // start_line = 0 is rejected.
        let err = apply_line_replacement("a\nb\nc", 0, 1, "X").unwrap_err();
        assert!(err.contains("start_line"));

        // start > end is rejected.
        let err = apply_line_replacement("a\nb\nc", 3, 2, "X").unwrap_err();
        assert!(err.contains("end_line") && err.contains("start_line"));

        // end_line > total is rejected.
        let err = apply_line_replacement("a\nb\nc", 1, 99, "X").unwrap_err();
        assert!(err.contains("out of range") || err.contains("99"));
    }
}