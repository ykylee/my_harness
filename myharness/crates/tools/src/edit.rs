use std::path::PathBuf;

use tree_sitter_rust as _ts_rust;

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

/// Hashline v2 (D-106) block-anchored edit payload.
///
/// `start_line` is 1-indexed and must point at the line that OPENS the
/// syntactic construct (the `fn`/`struct`/`impl`/`if`/`for`/etc. line, or
/// the first line of a leading decorator/doc-comment for languages that
/// parse them as part of the same node). tree-sitter resolves the
/// matching closing line.
#[derive(Debug, Deserialize)]
struct BlockAnchoredEdit {
    start_line: usize,
    expected_hash: String,
    replacement: String,
}

/// Hashline v2 (D-107) pure insert/delete payload.
///
/// `insertions` and `deletions` are both optional. If both are empty
/// the patch is a no-op (and is rejected — see execute_pure below).
/// Lines refer to the ORIGINAL file (pre-patch); all ops are applied
/// in line-descending order so a deletion or insertion at a higher
/// line number never shifts the anchor of a lower one.
///
/// `insert_after_block` requires tree-sitter (D-106 Rust only) and
/// uses the same `resolve_block_span` as `block_anchored`.
#[derive(Debug, Deserialize, Default)]
struct PureEdit {
    #[serde(default)]
    expected_hash: Option<String>,
    #[serde(default)]
    insertions: Vec<PureInsertion>,
    #[serde(default)]
    deletions: Vec<PureDeletion>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op")]
enum PureInsertion {
    /// Insert `content` immediately before the line with the given
    /// 1-indexed line number. `line` is the anchor — existing line
    /// shifts down by `content.lines().count()` lines.
    #[serde(rename = "insert_before")]
    Before { line: usize, content: String },
    /// Insert `content` immediately after the line with the given
    /// 1-indexed line number. The anchor line stays put; the new
    /// rows land right below it.
    #[serde(rename = "insert_after")]
    After { line: usize, content: String },
    /// Insert `content` at the very start of the file (before line 1).
    #[serde(rename = "insert_head")]
    Head { content: String },
    /// Insert `content` at the very end of the file (after the last
    /// line; if the file has no trailing newline, the new content
    /// is glued onto the last line — callers should end `content`
    /// with `\n` when targeting a line-oriented file).
    #[serde(rename = "insert_tail")]
    Tail { content: String },
    /// Insert `content` immediately after the END of the syntactic
    /// block that BEGINS on the given 1-indexed line (tree-sitter
    /// resolution — see D-106). Rust only in v1.5.
    #[serde(rename = "insert_after_block")]
    AfterBlock { line: usize, content: String },
    /// D-123: Replace the entire syntactic block that BEGINS on the
    /// given 1-indexed line with `content`. tree-sitter resolves the
    /// closing line (D-106 의 `resolve_block_span` 와 동일 path).
    /// Rust only in v1.5.
    #[serde(rename = "replace_block")]
    ReplaceBlock { line: usize, content: String },
}

#[derive(Debug, Deserialize)]
struct PureDeletion {
    start_line: usize,
    end_line: usize,
}

/// Internal op representation for `pure_edit` mode (D-107).
///
/// `anchor` is a line number in the ORIGINAL file (pre-patch). For
/// `InsertBefore` it is the line the new content lands in front of;
/// for `InsertAfter` it is the line the new content lands after; for
/// `Delete` it is the start of the deleted range; `InsertHead` uses
/// `0` and `InsertTail` uses `total_lines + 1` as anchor values that
/// sort to the edges.
struct PendingOp<'a> {
    anchor: usize,
    kind: OpKind<'a>,
}

#[derive(Debug)]
enum OpKind<'a> {
    Head(&'a str),
    Tail(&'a str),
    Before(&'a str),
    After(&'a str),
    Delete { start: usize, end: usize },
    /// D-123: block-aware replace. `start`..=`end` (1-indexed, inclusive)
    /// span → `content` 로 교체. `apply_line_replacement` 와 동일 path.
    Replace { start: usize, end: usize, content: &'a str },
}

impl<'a> OpKind<'a> {
    /// Sort priority within a single anchor line (lower runs first).
    /// Delete before InsertAfter on the same line so the inserted
    /// content lands after the (now-shorter) original block.
    fn priority(&self) -> u8 {
        match self {
            OpKind::Delete { .. } | OpKind::Replace { .. } => 0,
            OpKind::After(_) => 1,
            OpKind::Before(_) => 2,
            OpKind::Head(_) | OpKind::Tail(_) => 3,
        }
    }
}

/// Insert `content` immediately before the given 1-indexed line of
/// `src`. Returns the new string. Empty `content` is a no-op.
fn apply_insert_before(src: &str, line_1: usize, content: &str) -> String {
    if content.is_empty() {
        return src.to_string();
    }
    let lines: Vec<&str> = src.split_inclusive('\n').collect();
    let idx = line_1.saturating_sub(1).min(lines.len());
    let mut out = String::with_capacity(src.len() + content.len());
    for l in &lines[..idx] {
        out.push_str(l);
    }
    out.push_str(content);
    // Note: if  ends with a newline and  happens
    // to be empty, the surrounding lines keep their own newlines, so
    // no double-newline is introduced.
    for l in &lines[idx..] {
        out.push_str(l);
    }
    out
}

/// Insert `content` immediately after the given 1-indexed line of
/// `src`. Returns the new string. Empty `content` is a no-op.
fn apply_insert_after(src: &str, line_1: usize, content: &str) -> String {
    if content.is_empty() {
        return src.to_string();
    }
    let lines: Vec<&str> = src.split_inclusive('\n').collect();
    // `line_1` 1-indexed into lines. If line_1 == lines.len(), we
    // append after the final line.
    let idx = line_1.min(lines.len());
    let mut out = String::with_capacity(src.len() + content.len());
    for l in &lines[..idx] {
        out.push_str(l);
    }
    out.push_str(content);
    for l in &lines[idx..] {
        out.push_str(l);
    }
    out
}

/// Resolve the smallest syntactic block that *begins* on the given
/// 1-indexed line of a Rust source string, returning its 1-indexed
/// inclusive end line plus a human-readable node kind ("function_item",
/// "struct_item", "impl_item", etc.).
///
/// "Smallest block that begins on line N" means: walk from the AST root
/// and pick the deepest named node whose start line == N AND whose
/// end line is `> N`. We deliberately do NOT include nodes whose start
/// is before N (those cover the line but don't open on it) and we do
/// NOT include nodes whose end equals N (zero-width).
///
/// Returns `Err(msg)` with an actionable error when:
/// - the parser fails to consume the source,
/// - the file has fewer lines than `start_line_1`,
/// - no construct on `start_line_1` owns at least one additional line
///   (the line is blank, a comment, a closing brace, etc.).
fn resolve_block_span(content: &str, start_line_1: usize) -> Result<(usize, String), String> {
    if start_line_1 == 0 {
        return Err("start_line must be >= 1 (1-indexed)".to_string());
    }

    let total_lines = content.lines().count();
    if start_line_1 > total_lines {
        return Err(format!(
            "start_line ({start_line_1}) out of range (file has {total_lines} line(s))"
        ));
    }

    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter::Language::new(_ts_rust::LANGUAGE);
    parser
        .set_language(&language)
        .map_err(|e| format!("tree-sitter set_language failed: {e}"))?;

    let tree = parser
        .parse(content, None)
        .ok_or_else(|| "tree-sitter parse returned None".to_string())?;

    let root = tree.root_node();
    if root.has_error() {
        return Err(format!(
            "tree-sitter parse has syntax errors (root kind={})",
            root.kind()
        ));
    }

    // tree-sitter rows are 0-indexed.
    let target_row = start_line_1 - 1;

    fn collect_candidates(
        node: tree_sitter::Node<'_>,
        target_row: usize,
        depth: usize,
        candidates: &mut Vec<(usize, usize, String)>, // (end_row, depth, kind)
    ) {
        let start_row = node.start_position().row;
        let end_row = node.end_position().row;

        if start_row == target_row && end_row >= start_row {
            // Require start column == 0: a line that is blank or
            // whitespace-only is still a "row" in tree-sitter terms
            // (any non-newline byte starts a row), but no syntactic
            // construct can begin on it. This protects against
            // `replace block 3` succeeding on a blank line that
            // happens to share its row with a following single-line
            // construct.
            if node.start_position().column == 0 {
                candidates.push((end_row, depth, node.kind().to_string()));
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_candidates(child, target_row, depth + 1, candidates);
        }
    }

    let mut candidates: Vec<(usize, usize, String)> = Vec::new();
    collect_candidates(root, target_row, 0, &mut candidates);

    // Filter out nodes that don't carry useful edit semantics:
    //  - `source_file`: the root, would swallow the whole file
    //  - inner brace blocks: `block`, `field_declaration_list`,
    //    `declaration_list`, `token_tree`, `where_clause`, etc.
    //    These are CHILDREN of the construct the user actually meant
    //    to anchor on (e.g. `block` is the `{ ... }` of a `fn`).
    //  - anonymous tokens (`{`, `}`, `(`, `)`, `[`, `]`) — they have
    //    no edit semantic, just punctuation.
    //  - any node whose depth is 1 (direct child of `source_file`)
    //    that is NOT itself an `item` — those are `use_list`, `attr`,
    //    `vis`, etc., which are siblings of the real construct.
    //
    // We rely on the grammar's own naming: any node ending in
    // `_item` is a real construct. Anonymous tokens are short (1-3
    // chars) and never end in `_item`. This makes the filter
    // grammar-agnostic without a hardcoded kind allowlist.
    fn is_meaningful(kind: &str, _depth: usize) -> bool {
        // Only accept `*_item` nodes. The Rust grammar names every
        // top-level construct with the `_item` suffix (function_item,
        // struct_item, impl_item, trait_item, mod_item, enum_item,
        // type_item, const_item, static_item, etc.). This rules out:
        //  - `source_file` (root)
        //  - `block` / `field_declaration_list` / `declaration_list`
        //    / `token_tree` (inner brace bodies)
        //  - `struct` / `fn` / `impl` / `let` / `if` (keyword nodes)
        //  - `use_list` / `attr` / `vis` (siblings of items)
        //  - `{` / `(` / `;` (anonymous tokens)
        kind.ends_with("_item")
    }

    let filtered: Vec<(usize, usize, String)> = candidates
        .into_iter()
        .filter(|(_, depth, kind)| {
            if !is_meaningful(kind, *depth) {
                return false;
            }
            true
        })
        .collect();

    // Pick the DEEPEST remaining candidate (smallest specific construct)
    // and use its end_row as the resolved span. If multiple candidates
    // sit at the same depth, prefer the one with the smallest end_row
    // (most local construct — though this is a tie-breaker, not the
    // common case).
    let best = filtered.into_iter().min_by(|a, b| {
        // deeper = larger depth value; sort by depth descending first
        b.1.cmp(&a.1).then(a.0.cmp(&b.0))
    });

    match best {
        Some((end_row_0idx, _, kind)) => Ok((end_row_0idx + 1, kind)),
        None => Err(format!(
            "no syntactic block opens on line {start_line_1} (line is blank, a comment, a closing brace, or a non-block-brace construct)"
        )),
    }
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

    let mut combined: Vec<&str> = Vec::with_capacity(pre.len() + repl_lines.len() + post.len());
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

    fn description(&self) -> &'static str {
        "Edit a file using one of three modes: (1) old_string/new_string with          optional replace_all, (2) line_anchored hashline replace (D-105),          (3) block_anchored hashline replace (D-106), (4) pure_edit multi-section          atomic insert/replace/delete (D-107). Returns the final content hash."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Absolute or working-directory-relative path to the file to edit."
                },
                "old_string": {
                    "type": "string",
                    "description": "Literal text to find (mode 1)."
                },
                "new_string": {
                    "type": "string",
                    "description": "Replacement text (mode 1)."
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "If true, replace every occurrence of old_string (mode 1)."
                },
                "line_anchored": {
                    "type": "object",
                    "description": "Hashline v2 (D-105) line-anchored replace. Schema: {anchor, content_hash, replacement, replace?}",
                    "properties": {
                        "anchor": {"type": "string"},
                        "content_hash": {"type": "string"},
                        "replacement": {"type": "string"},
                        "replace": {"type": "string", "enum": ["this", "below", "above", "block"]}
                    },
                    "required": ["anchor", "content_hash", "replacement"]
                },
                "block_anchored": {
                    "type": "object",
                    "description": "Hashline v2 (D-106) block-anchored replace. Schema: {start_anchor, end_anchor, content_hash, replacement}",
                    "properties": {
                        "start_anchor": {"type": "string"},
                        "end_anchor": {"type": "string"},
                        "content_hash": {"type": "string"},
                        "replacement": {"type": "string"}
                    },
                    "required": ["start_anchor", "end_anchor", "content_hash", "replacement"]
                },
                "pure_edit": {
                    "type": "object",
                    "description": "Hashline v2 (D-107) multi-section atomic edit. Schema: {content_hash, operations: [{op, ...}]}",
                    "properties": {
                        "content_hash": {"type": "string"},
                        "operations": {"type": "array"}
                    },
                    "required": ["content_hash", "operations"]
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
            .ok_or_else(|| ToolError::InvalidInput("missing file_path".into()))?
            .to_string();

        // Hashline v2 (D-105): opt-in `line_anchored` mode. Dispatch FIRST so
        // the existing `old_string`/`new_string`/`replace_all` path stays
        // byte-identical for callers that have not opted in.
        if input.get("line_anchored").is_some() {
            return self.execute_line_anchored(ctx, &file_path, input).await;
        }

        // Hashline v2 (D-106): opt-in `block_anchored` mode — resolve the
        // syntactic block that begins on the given line via tree-sitter, so
        // a long body cannot be mis-counted and a stale end cannot clip it
        // mid-block. Same stale-anchor gate as `line_anchored`.
        if input.get("block_anchored").is_some() {
            return self.execute_block_anchored(ctx, &file_path, &input).await;
        }

        // Hashline v2 (D-107): opt-in `pure_edit` mode — multi-section
        // insert/delete ops applied line-descending so anchors never
        // shift. Single op = `insertions: [{...}]` or `deletions: [{...}]`
        // with one entry; multi-section = both arrays populated.
        if input.get("pure_edit").is_some() {
            return self.execute_pure(ctx, &file_path, &input).await;
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
        let new_content =
            apply_line_replacement(&content, la.start_line, la.end_line, &la.replacement)
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

    /// Hashline v2 (D-106) `block_anchored` mode — resolve the syntactic
    /// block beginning on the given line via tree-sitter, so the closing
    /// line is the actual end of the construct (function / `if` / loop /
    /// struct / enum / impl / trait / fn / mod / closure) regardless of
    /// hand-counting. Stale-anchor gate is identical to `line_anchored`.
    async fn execute_block_anchored(
        &self,
        ctx: &ToolContext,
        file_path: &str,
        input: &serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        let path = if PathBuf::from(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            ctx.cwd.join(file_path)
        };

        let content = fs::read_to_string(&path)
            .await
            .map_err(ToolError::IoError)?;

        let ba: BlockAnchoredEdit = serde_json::from_value(
            input
                .get("block_anchored")
                .cloned()
                .ok_or_else(|| ToolError::InvalidInput("missing block_anchored".into()))?,
        )
        .map_err(|e| ToolError::InvalidInput(format!("invalid block_anchored: {e}")))?;

        // Stale-anchor gate (spec §5.2 step 3 / D-106 same). Fail BEFORE
        // any tree-sitter work so a stale payload can never silently
        // corrupt a moved file.
        let current_hash = compute_content_hash(&content);
        if current_hash != ba.expected_hash {
            return Err(ToolError::InvalidInput(format!(
                "stale anchor: file modified; re-read with `Read` tool (current hash {current_hash}, expected {})",
                ba.expected_hash
            )));
        }

        // Resolve the block: Rust only in v1.5 (D-106 scope). Other
        // extensions are an explicit error so callers learn to use
        // `line_anchored` for non-Rust files instead of silently getting
        // a wrong span.
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if extension != "rs" {
            return Err(ToolError::InvalidInput(format!(
                "block_anchored: tree-sitter grammar for extension .{extension} not yet supported; use line_anchored for non-Rust files"
            )));
        }

        let (resolved_end_line, node_kind) = match resolve_block_span(&content, ba.start_line) {
            Ok(span) => span,
            Err(msg) => return Err(ToolError::InvalidInput(msg)),
        };

        // range check (start_line must be within the file).
        let total_lines = content.lines().count();
        if ba.start_line == 0 || ba.start_line > total_lines {
            return Err(ToolError::InvalidInput(format!(
                "start_line ({}) out of range (file has {total_lines} line(s))",
                ba.start_line
            )));
        }

        let new_content =
            apply_line_replacement(&content, ba.start_line, resolved_end_line, &ba.replacement)
                .map_err(ToolError::InvalidInput)?;

        fs::write(&path, &new_content)
            .await
            .map_err(ToolError::IoError)?;

        let new_hash = compute_content_hash(&new_content);
        let replaced_lines = resolved_end_line - ba.start_line + 1;

        Ok(ToolResult {
            output: format!(
                "replaced block ({}..={}) [tree-sitter: {node_kind}] in {}",
                ba.start_line,
                resolved_end_line,
                path.display()
            ),
            is_error: false,
            metadata: Some(serde_json::json!({
                "path": path.to_string_lossy(),
                "mode": "block_anchored",
                "start_line": ba.start_line,
                "end_line": resolved_end_line,
                "replaced_lines": replaced_lines,
                "node_kind": node_kind,
                "old_hash": ba.expected_hash,
                "new_hash": new_hash,
            })),
        })
    }

    /// Hashline v2 (D-107) `pure_edit` mode — multi-section
    /// insert/delete with stale-anchor gate, applied in
    /// line-descending order so each op's anchor is computed against
    /// the ORIGINAL file lines and cannot be shifted by a sibling op
    /// at a higher line number.
    async fn execute_pure(
        &self,
        ctx: &ToolContext,
        file_path: &str,
        input: &serde_json::Value,
    ) -> Result<ToolResult, ToolError> {
        let path = if PathBuf::from(file_path).is_absolute() {
            PathBuf::from(file_path)
        } else {
            ctx.cwd.join(file_path)
        };

        let content = fs::read_to_string(&path)
            .await
            .map_err(ToolError::IoError)?;

        let pe: PureEdit = serde_json::from_value(
            input
                .get("pure_edit")
                .cloned()
                .ok_or_else(|| ToolError::InvalidInput("missing pure_edit".into()))?,
        )
        .map_err(|e| ToolError::InvalidInput(format!("invalid pure_edit: {e}")))?;

        // Stale-anchor gate. If `expected_hash` is provided we enforce
        // it; if not, this is an opt-in lenient mode for callers that
        // do not want the gate (e.g. a batch that re-reads inside the
        // same tool call). For safety we REQUIRE the gate for v1.5.
        let expected_hash = pe.expected_hash.as_deref().ok_or_else(|| {
            ToolError::InvalidInput("pure_edit requires expected_hash (stale-anchor gate)".into())
        })?;
        let current_hash = compute_content_hash(&content);
        if current_hash != expected_hash {
            return Err(ToolError::InvalidInput(format!(
                "stale anchor: file modified; re-read with `Read` tool (current hash {current_hash}, expected {expected_hash})"
            )));
        }

        if pe.insertions.is_empty() && pe.deletions.is_empty() {
            return Err(ToolError::InvalidInput(
                "pure_edit: at least one of `insertions` or `deletions` must be non-empty".into(),
            ));
        }

        // Validate deletions first (cheaper) and resolve
        // `insert_after_block` ops to concrete (insert_after) targets.
        let total_lines = content.lines().count();
        for d in &pe.deletions {
            if d.start_line == 0 || d.end_line < d.start_line {
                return Err(ToolError::InvalidInput(format!(
                    "deletion: invalid range {}..={}",
                    d.start_line, d.end_line
                )));
            }
            if d.start_line > total_lines {
                return Err(ToolError::InvalidInput(format!(
                    "deletion: start_line {} out of range (file has {total_lines} line(s))",
                    d.start_line
                )));
            }
        }

        // For `insert_after_block`, only Rust is supported in v1.5.
        // We resolve each op to a concrete insertion point now so the
        // line-descending sort below can mix insert/deletion ops
        // uniformly.
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        // D-123: block-aware ops (AfterBlock, ReplaceBlock) 둘 다 tree-sitter
        // resolve 가 필요하므로 함께 gather. 둘 다 Rust 한정 (v1.5 grammar).
        let block_ops: Vec<(usize, &PureInsertion)> = pe
            .insertions
            .iter()
            .enumerate()
            .filter(|(_, ins)| {
                matches!(ins, PureInsertion::AfterBlock { .. } | PureInsertion::ReplaceBlock { .. })
            })
            .collect();
        if !block_ops.is_empty() && extension != "rs" {
            return Err(ToolError::InvalidInput(format!(
                "pure_edit.insert_after_block / replace_block: tree-sitter grammar for extension .{extension} not yet supported; use line_anchored / block_anchored for non-Rust files"
            )));
        }

        // Build a concrete (anchor_line, op) list. For deletions the
        // anchor is `start_line - 1` (insert the deletion marker
        // before that line, conceptually); for insertions the anchor
        // is the line number the spec defines.
        //
        // We'll apply them line-DESCENDING, which means: process
        // highest anchor first. For each op:
        //   - deletion [s..=e]: remove lines s..=e (inclusive)
        //   - insert_before N: insert content immediately before N
        //   - insert_after  N: insert content immediately after N
        //   - insert_head: insert at byte 0
        //   - insert_tail: insert at byte len
        //
        // We process them sorted by (anchor_line DESC, op_kind
        // priority), where priority ensures a deletion at line N is
        // applied before a same-line insert_after (so the inserted
        // content is preserved).
        let mut ops: Vec<PendingOp<'_>> = Vec::new();
        for ins in &pe.insertions {
            match ins {
                PureInsertion::Before { line, content } => {
                    if *line == 0 || *line > total_lines + 1 {
                        return Err(ToolError::InvalidInput(format!(
                            "insert_before: line {line} out of range (file has {total_lines} line(s))"
                        )));
                    }
                    ops.push(PendingOp {
                        anchor: *line,
                        kind: OpKind::Before(content.as_str()),
                    });
                }
                PureInsertion::After { line, content } => {
                    if *line == 0 || *line > total_lines {
                        return Err(ToolError::InvalidInput(format!(
                            "insert_after: line {line} out of range (file has {total_lines} line(s))"
                        )));
                    }
                    ops.push(PendingOp {
                        anchor: *line,
                        kind: OpKind::After(content.as_str()),
                    });
                }
                PureInsertion::Head { content } => {
                    ops.push(PendingOp {
                        anchor: 0,
                        kind: OpKind::Head(content.as_str()),
                    });
                }
                PureInsertion::Tail { content } => {
                    ops.push(PendingOp {
                        anchor: total_lines + 1,
                        kind: OpKind::Tail(content.as_str()),
                    });
                }
                PureInsertion::AfterBlock { line, content } => {
                    let (resolved_end_line, _kind) =
                        resolve_block_span(content, *line).map_err(ToolError::InvalidInput)?;
                    ops.push(PendingOp {
                        anchor: resolved_end_line,
                        kind: OpKind::After(content.as_str()),
                    });
                }
                // D-123: block-aware replace. tree-sitter 가 block 의
                // closing line 을 resolve → start..=end span 을
                // replacement 로 교체. *원본* file content 에서
                // resolve 해야 함 (replacement 가 아닌) — AfterBlock 의
                // line 887 호출과 구분.
                PureInsertion::ReplaceBlock { line, content: replacement } => {
                    // D-123: arm pattern 의 `content` 를 `replacement` 로 rename 해서
                    // outer `content` (file string) 와 구분. block END 는 원본 file 에서
                    // resolve.
                    let (resolved_end_line, _kind) = resolve_block_span(&content, *line)
                        .map_err(ToolError::InvalidInput)?;
                    ops.push(PendingOp {
                        anchor: resolved_end_line,
                        kind: OpKind::Replace {
                            start: *line,
                            end: resolved_end_line,
                            content: replacement.as_str(),
                        },
                    });
                }
            }
        }
        for d in &pe.deletions {
            // Represent the deletion as an "insert" of an empty span
            // at the boundary that, when sorted descending, lands
            // first. We use the same OpKind::Delete variant and
            // resolve it below.
            ops.push(PendingOp {
                anchor: d.start_line,
                kind: OpKind::Delete {
                    start: d.start_line,
                    end: d.end_line,
                },
            });
        }

        // Sort: line-DESCENDING so higher anchors are applied first
        // and never shift lower ones. Tie-breaker: among ops on the
        // same line, deletion first (it removes a line that an
        // insert_after on the same line should still apply AFTER),
        // then InsertAfter, then InsertBefore, then Head/Tail.
        ops.sort_by(|a, b| {
            b.anchor
                .cmp(&a.anchor)
                .then_with(|| a.kind.priority().cmp(&b.kind.priority()))
        });

        // Apply.
        let mut new_content = content.clone();
        let mut applied: Vec<serde_json::Value> = Vec::new();
        for op in &ops {
            match op.kind {
                OpKind::Before(s) => {
                    let line_count = s.lines().count();
                    new_content = apply_insert_before(&new_content, op.anchor, s);
                    applied.push(serde_json::json!({"op": "insert_before", "line": op.anchor, "lines_added": line_count}));
                }
                OpKind::After(s) => {
                    let line_count = s.lines().count();
                    new_content = apply_insert_after(&new_content, op.anchor, s);
                    applied.push(serde_json::json!({"op": "insert_after", "line": op.anchor, "lines_added": line_count}));
                }
                OpKind::Head(s) => {
                    let line_count = s.lines().count();
                    new_content = format!("{s}{new_content}");
                    applied
                        .push(serde_json::json!({"op": "insert_head", "lines_added": line_count}));
                }
                OpKind::Tail(s) => {
                    let line_count = s.lines().count();
                    new_content.push_str(s);
                    applied
                        .push(serde_json::json!({"op": "insert_tail", "lines_added": line_count}));
                }
                OpKind::Delete { start, end } => {
                    let removed = end - start + 1;
                    new_content = apply_line_replacement(&new_content, start, end, "")
                        .map_err(ToolError::InvalidInput)?;
                    applied.push(serde_json::json!({"op": "delete", "start_line": start, "end_line": end, "lines_removed": removed}));
                }
                // D-123: block span 을 `content` 로 교체.
                OpKind::Replace { start, end, content } => {
                    let replaced = end - start + 1;
                    new_content = apply_line_replacement(&new_content, start, end, content)
                        .map_err(ToolError::InvalidInput)?;
                    applied.push(serde_json::json!({"op": "replace_block", "start_line": start, "end_line": end, "lines_replaced": replaced}));
                }
            }
        }

        fs::write(&path, &new_content)
            .await
            .map_err(ToolError::IoError)?;

        let new_hash = compute_content_hash(&new_content);

        Ok(ToolResult {
            output: format!(
                "applied {} op(s) to {} (deletions={}, insertions={})",
                applied.len(),
                path.display(),
                pe.deletions.len(),
                pe.insertions.len(),
            ),
            is_error: false,
            metadata: Some(serde_json::json!({
                "path": path.to_string_lossy(),
                "mode": "pure_edit",
                "applied": applied,
                "old_hash": expected_hash,
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

    // --- block_anchored mode (Hashline v2 / D-106) ---------------------------

    fn make_block_ctx() -> ToolContext {
        ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: true,
            sanitizer_mode: SanitizerMode::default(),
        }
    }

    #[test]
    fn test_resolve_block_span_fn() {
        // A small Rust file with two `fn`s; line 1 opens `alpha`, line 7 opens `beta`.
        let content =
            "fn alpha(x: u32) -> u32 {\n    x + 1\n}\n\nfn beta(y: u32) -> u32 {\n    y * 2\n}\n";
        let (end, kind) = resolve_block_span(content, 1).expect("alpha should resolve");
        assert_eq!(kind, "function_item");
        assert_eq!(end, 3, "alpha body is lines 1..=3");

        let (end, kind) = resolve_block_span(content, 5).expect("beta should resolve");
        assert_eq!(kind, "function_item");
        assert_eq!(end, 7, "beta body is lines 5..=7");
    }

    #[test]
    fn test_resolve_block_span_struct() {
        let content = "struct Point {\n    x: i32,\n    y: i32,\n}\n";
        let (end, kind) = resolve_block_span(content, 1).unwrap();
        assert_eq!(kind, "struct_item");
        assert_eq!(end, 4);
    }

    #[test]
    fn test_resolve_block_span_impl() {
        let content = "impl Foo {\n    fn bar(&self) -> i32 { 42 }\n}\n";
        let (end, kind) = resolve_block_span(content, 1).unwrap();
        assert_eq!(kind, "impl_item");
        assert_eq!(end, 3);
    }

    #[test]
    fn test_resolve_block_span_not_at_block_head() {
        // Line 1 = `fn alpha`, line 2 = blank, line 3 = `fn beta`.
        // Line 2 is blank — no construct opens on it.
        let content = "fn alpha() {}\n\nfn beta() {}\n";
        let err = resolve_block_span(content, 2).unwrap_err();
        assert!(err.contains("no syntactic block opens"), "err was: {err}");
    }

    #[test]
    fn test_resolve_block_span_start_out_of_range() {
        let content = "fn alpha() {}\n";
        let err = resolve_block_span(content, 99).unwrap_err();
        assert!(err.contains("out of range"), "err was: {err}");
    }

    #[test]
    fn test_resolve_block_span_start_zero() {
        let err = resolve_block_span("fn alpha() {}\n", 0).unwrap_err();
        assert!(err.contains("must be >= 1"), "err was: {err}");
    }

    #[tokio::test]
    async fn test_edit_block_anchored_happy_path_fn() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.rs");
        let original =
            "fn alpha(x: u32) -> u32 {\n    x + 1\n}\n\nfn beta(y: u32) -> u32 {\n    y * 2\n}\n";
        fs::write(&file_path, original).await.unwrap();

        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_block_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "block_anchored": {
                "start_line": 1,
                "expected_hash": expected_hash,
                "replacement": "fn alpha(x: u32) -> u32 { x + 99 }",
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "result: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(
            read_back,
            "fn alpha(x: u32) -> u32 { x + 99 }\n\nfn beta(y: u32) -> u32 {\n    y * 2\n}\n"
        );

        let meta = result.metadata.expect("metadata required");
        assert_eq!(meta["mode"], "block_anchored");
        assert_eq!(meta["start_line"], 1);
        assert_eq!(meta["end_line"], 3);
        assert_eq!(meta["replaced_lines"], 3);
        assert_eq!(meta["node_kind"], "function_item");
        assert_eq!(meta["old_hash"], expected_hash);
        assert!(meta["new_hash"].as_str().is_some());
        assert_ne!(meta["new_hash"], expected_hash);
    }

    #[tokio::test]
    async fn test_edit_block_anchored_keeps_sibling() {
        // Replacing the first `fn` must leave the second `fn` intact.
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.rs");
        let original = "fn alpha() -> i32 { 1 }\nfn beta() -> i32 { 2 }\n";
        fs::write(&file_path, original).await.unwrap();

        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_block_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "block_anchored": {
                "start_line": 1,
                "expected_hash": expected_hash,
                "replacement": "fn alpha() -> i32 { 42 }",
            }
        });
        tool.execute(&ctx, input).await.unwrap();

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        // beta must be byte-identical to its original.
        assert!(read_back.contains("fn beta() -> i32 { 2 }"));
    }

    #[tokio::test]
    async fn test_edit_block_anchored_stale_anchor() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.rs");
        let original = "fn alpha() -> i32 { 1 }\nfn beta() -> i32 { 2 }\n";
        fs::write(&file_path, original).await.unwrap();
        let expected_hash = compute_content_hash(original);

        // External write changes the file between Read and Edit.
        fs::write(
            &file_path,
            "fn alpha() -> i32 { 1 }\n// someone added a comment\nfn beta() -> i32 { 2 }\n",
        )
        .await
        .unwrap();

        let tool = make_tool();
        let ctx = make_block_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "block_anchored": {
                "start_line": 1,
                "expected_hash": expected_hash,
                "replacement": "fn alpha() -> i32 { 99 }",
            }
        });
        let err = tool.execute(&ctx, input).await.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("stale anchor"), "msg was: {msg}");
    }

    #[tokio::test]
    async fn test_edit_block_anchored_non_rust_rejected() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("notes.txt");
        let original = "fn alpha() -> i32 { 1 }\n";
        fs::write(&file_path, original).await.unwrap();
        let expected_hash = compute_content_hash(original);

        let tool = make_tool();
        let ctx = make_block_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "block_anchored": {
                "start_line": 1,
                "expected_hash": expected_hash,
                "replacement": "anything",
            }
        });
        let err = tool.execute(&ctx, input).await.unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("not yet supported") && msg.contains(".txt"),
            "msg was: {msg}"
        );
    }

    #[tokio::test]
    async fn test_edit_block_anchored_not_at_block_head() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.rs");
        let original = "fn alpha() -> i32 { 1 }\n\nfn beta() -> i32 { 2 }\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_block_ctx();
        // Line 2 is the blank between the two fns.
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "block_anchored": {
                "start_line": 2,
                "expected_hash": expected_hash,
                "replacement": "x",
            }
        });
        let err = tool.execute(&ctx, input).await.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("no syntactic block"), "msg was: {msg}");
    }

    // --- pure_edit mode (Hashline v2 / D-107) --------------------------------

    fn make_pure_ctx() -> ToolContext {
        ToolContext {
            cwd: PathBuf::from("/"),
            permission_mode: PermissionMode::Default,
            confirm_override: true,
            sanitizer_mode: SanitizerMode::default(),
        }
    }

    #[test]
    fn test_apply_insert_before_basic() {
        let src = "a\nb\nc\n";
        assert_eq!(apply_insert_before(src, 2, "X\n"), "a\nX\nb\nc\n");
        assert_eq!(apply_insert_before(src, 1, "HEAD\n"), "HEAD\na\nb\nc\n");
        assert_eq!(apply_insert_before(src, 99, "TAIL\n"), "a\nb\nc\nTAIL\n");
        assert_eq!(apply_insert_before(src, 2, ""), "a\nb\nc\n"); // empty = no-op
    }

    #[test]
    fn test_apply_insert_after_basic() {
        let src = "a\nb\nc\n";
        assert_eq!(apply_insert_after(src, 1, "X\n"), "a\nX\nb\nc\n");
        assert_eq!(apply_insert_after(src, 3, "X\n"), "a\nb\nc\nX\n");
        assert_eq!(apply_insert_after(src, 99, "X\n"), "a\nb\nc\nX\n");
        assert_eq!(apply_insert_after(src, 2, ""), "a\nb\nc\n"); // empty = no-op
    }

    #[tokio::test]
    async fn test_pure_edit_insert_before() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.txt");
        let original = "alpha\nbeta\ngamma\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_pure_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "insertions": [
                    {"op": "insert_before", "line": 2, "content": "BETA-INSERT\n"}
                ]
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "result: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "alpha\nBETA-INSERT\nbeta\ngamma\n");

        let meta = result.metadata.expect("metadata");
        assert_eq!(meta["mode"], "pure_edit");
        assert_eq!(meta["applied"][0]["op"], "insert_before");
        assert_eq!(meta["applied"][0]["line"], 2);
        assert_eq!(meta["applied"][0]["lines_added"], 1);
    }

    #[tokio::test]
    async fn test_pure_edit_insert_after_head_tail() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.txt");
        let original = "middle\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_pure_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "insertions": [
                    {"op": "insert_head", "content": "HEAD\n"},
                    {"op": "insert_tail", "content": "\nTAIL"}
                ]
            }
        });
        tool.execute(&ctx, input).await.unwrap();

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "HEAD\nmiddle\n\nTAIL");
    }

    #[tokio::test]
    async fn test_pure_edit_delete_single_line() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.txt");
        let original = "a\nb\nc\nd\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_pure_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "deletions": [{"start_line": 2, "end_line": 2}]
            }
        });
        tool.execute(&ctx, input).await.unwrap();

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "a\nc\nd\n");
    }

    #[tokio::test]
    async fn test_pure_edit_delete_range() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.txt");
        let original = "a\nb\nc\nd\ne\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_pure_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "deletions": [{"start_line": 2, "end_line": 4}]
            }
        });
        tool.execute(&ctx, input).await.unwrap();

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "a\ne\n");
    }

    #[tokio::test]
    async fn test_pure_edit_multi_section_atomic() {
        // Two non-adjacent ops applied in the same patch: insert
        // before line 2 AND delete line 4. With line-descending sort,
        // the deletion runs first (anchor 4 > 2), so the insert's
        // anchor stays valid.
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.txt");
        let original = "a\nb\nc\nd\ne\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_pure_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "insertions": [
                    {"op": "insert_before", "line": 2, "content": "INSERTED\n"}
                ],
                "deletions": [{"start_line": 4, "end_line": 4}]
            }
        });
        tool.execute(&ctx, input).await.unwrap();

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        assert_eq!(read_back, "a\nINSERTED\nb\nc\ne\n");
    }

    #[tokio::test]
    async fn test_pure_edit_stale_anchor() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.txt");
        let original = "a\nb\nc\n";
        fs::write(&file_path, original).await.unwrap();
        let expected_hash = compute_content_hash(original);

        // External write changes the file before Edit.
        fs::write(&file_path, "a\n// comment\nb\nc\n")
            .await
            .unwrap();

        let tool = make_tool();
        let ctx = make_pure_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "insertions": [{"op": "insert_after", "line": 1, "content": "X\n"}]
            }
        });
        let err = tool.execute(&ctx, input).await.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("stale anchor"), "msg was: {msg}");
    }

    #[tokio::test]
    async fn test_pure_edit_empty_rejected() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.txt");
        let original = "a\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_pure_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "insertions": [],
                "deletions": []
            }
        });
        let err = tool.execute(&ctx, input).await.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("at least one"), "msg was: {msg}");
    }

    #[tokio::test]
    async fn test_pure_edit_missing_hash_rejected() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.txt");
        let original = "a\n";
        fs::write(&file_path, original).await.unwrap();

        let tool = make_tool();
        let ctx = make_pure_ctx();
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "insertions": [{"op": "insert_after", "line": 1, "content": "X\n"}]
            }
        });
        let err = tool.execute(&ctx, input).await.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("expected_hash"), "msg was: {msg}");
    }

    // --- D-123: pure_edit.replace_block op (옵션 j) ---

    /// D-123: `fn foo` 전체를 새 body 로 교체. tree-sitter 가
    /// closing brace 까지 resolve → span 전체 replacement.
    #[tokio::test]
    async fn d123_replace_block_replaces_entire_fn() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.rs");
        let original = "fn foo() -> i32 {\n    let x = 1;\n    let y = 2;\n    x + y\n}\nfn bar() -> i32 {\n    42\n}\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_pure_ctx();
        // foo() 의 `fn` 키워드 line (1) 을 anchor 로 → tree-sitter 가
        // closing `}` (line 5) 까지 resolve.
        let new_body = "fn foo() -> i32 {\n    100\n}\n";
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "insertions": [{
                    "op": "replace_block",
                    "line": 1,
                    "content": new_body
                }]
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "execute failed: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        // foo() 전체가 new_body 로 교체, bar() 는 그대로
        assert_eq!(read_back, format!("{new_body}\nfn bar() -> i32 {{\n    42\n}}\n"));
    }

    /// D-123: `pure_edit` multi-section atomic context 에서
    /// `replace_block` + `insert_before` 동시 사용. 다른 op 들과
    /// line-descending sort 로 충돌 없이 적용되어야 함.
    #[tokio::test]
    async fn d123_replace_block_in_pure_edit_multi_op() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("lib.rs");
        let original = "use std::io;\n\nfn greet() -> String {\n    \"hello\".to_string()\n}\n";
        fs::write(&file_path, original).await.unwrap();
        let content = fs::read_to_string(&file_path).await.unwrap();
        let expected_hash = compute_content_hash(&content);

        let tool = make_tool();
        let ctx = make_pure_ctx();
        // 1) line 1 (use std::io;) 앞에 use std::fmt; 추가
        // 2) fn greet 전체를 새 body 로 교체
        let new_greet = "fn greet() -> String {\n    \"hi\".to_string()\n}";
        let input = serde_json::json!({
            "file_path": file_path.to_string_lossy(),
            "pure_edit": {
                "expected_hash": expected_hash,
                "insertions": [
                    {"op": "insert_before", "line": 1, "content": "use std::fmt;\n"},
                    {"op": "replace_block", "line": 3, "content": new_greet}
                ]
            }
        });
        let result = tool.execute(&ctx, input).await.unwrap();
        assert!(!result.is_error, "execute failed: {result:?}");

        let read_back = fs::read_to_string(&file_path).await.unwrap();
        let expected = "use std::fmt;\nuse std::io;\n\nfn greet() -> String {\n    \"hi\".to_string()\n}\n";
        assert_eq!(read_back, expected);
    }
}
