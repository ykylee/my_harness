//! Content-addressable hash for line-anchored edits (Hashline v2 spec, D-104).
//!
//! 4-hex fingerprint of the full normalized file text. Used by `Read` to mint a
//! per-file `content_hash` so a follow-up Edit can reject stale anchors before
//! it corrupts the file. Mirrors the oh-my-pi `@oh-my-pi/hashline` spec shape
//! (`HL_FILE_HASH_LENGTH = 4`, uppercase hex), but uses `sha2` already in the
//! workspace rather than pulling a new `xxhash-rust` dep — 16 bits is plenty
//! for session-scope collision avoidance (single file, ≤ 1 MB).
//!
//! References: `ai-workflow/memory/hashline_v2_spec.md` §3 (content hash decision).

use sha2::{Digest, Sha256};

/// Number of hex characters in a content-derived file-hash tag.
pub const HASH_TAG_LENGTH: usize = 4;

/// Normalize text before hashing: trim trailing `[ \t\r]` from every line
/// (including the final line) so display-trimmed lines and CRLF endings do not
/// invalidate a tag. Matches `@oh-my-pi/hashline` `normalizeFileHashText`.
fn normalize_for_hash(text: &str) -> String {
    // Split on `\n`, trim trailing whitespace from each segment, rejoin.
    // An empty trailing segment (from text ending in `\n`) is preserved as an
    // empty line so the line count + final newline shape round-trip.
    let trailing_nl = text.ends_with('\n');
    let mut lines: Vec<&str> = text.split('\n').collect();
    if trailing_nl {
        // Pop the empty phantom segment so we don't double-rewrite it.
        lines.pop();
    }
    let mut out = String::with_capacity(text.len());
    for (i, raw) in lines.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        // Trim trailing space/tab/CR from this segment.
        let trimmed = raw.trim_end_matches([' ', '\t', '\r']);
        out.push_str(trimmed);
    }
    if trailing_nl {
        out.push('\n');
    }
    out
}

/// Compute the content-derived hash tag carried by a hashline section header.
/// The tag is a 4-hex fingerprint of the full normalized file text: any read
/// of byte-identical content mints the same tag, and a follow-up edit anchored
/// at any line validates whenever the live file still hashes to it.
#[must_use]
pub fn compute_content_hash(text: &str) -> String {
    let normalized = normalize_for_hash(text);
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let digest = hasher.finalize();
    // Take the low 16 bits (2 bytes = 4 hex chars) — session-scope dedup is
    // sufficient at this width, and we avoid pulling in `xxhash-rust` for now.
    let low16 = u16::from(digest[0]) | (u16::from(digest[1]) << 8);
    format!("{low16:0>4X}")
}

/// Format file text in Hashline-style `LINE:TEXT` form. Lines are 1-indexed.
/// Uses `str::lines()` so a trailing newline does NOT mint a phantom
/// trailing row — `text.lines()` already collapses `"a\n".lines()` to
/// `["a"]`. The total line count the caller reports in metadata should match
/// `text.lines().count()`.
#[must_use]
pub fn format_line_anchored(text: &str, start_line: usize) -> String {
    let mut out = String::with_capacity(text.len() + text.lines().count() * 4);
    for (i, line) in text.lines().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let n = start_line + i;
        let _ = std::fmt::Write::write_fmt(&mut out, format_args!("{n}:{line}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let h1 = compute_content_hash("hello world");
        let h2 = compute_content_hash("hello world");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 4);
    }

    #[test]
    fn hash_changes_with_content() {
        let a = compute_content_hash("hello world");
        let b = compute_content_hash("hello WORLD");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_normalizes_trailing_whitespace() {
        let a = compute_content_hash("hello world\n");
        let b = compute_content_hash("hello world   \n");
        // Trailing whitespace on each line should be normalized away.
        assert_eq!(a, b);
    }

    #[test]
    fn hash_normalizes_per_line_trailing_whitespace() {
        let a = compute_content_hash("foo\nbar\nbaz");
        let b = compute_content_hash("foo  \nbar\t\nbaz \r");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_is_uppercase_hex() {
        let h = compute_content_hash("anything");
        assert!(
            h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_lowercase()),
            "hash must be uppercase hex, got: {h}"
        );
    }

    #[test]
    fn format_line_anchored_basic() {
        let text = "fn main() {\n    println!(\"hi\");\n}\n";
        let out = format_line_anchored(text, 1);
        assert_eq!(out, "1:fn main() {\n2:    println!(\"hi\");\n3:}");
    }

    #[test]
    fn format_line_anchored_with_offset() {
        let text = "line1\nline2\nline3\n";
        let out = format_line_anchored(text, 10);
        assert_eq!(out, "10:line1\n11:line2\n12:line3");
    }

    #[test]
    fn format_line_anchored_no_trailing_newline() {
        let text = "only";
        let out = format_line_anchored(text, 1);
        assert_eq!(out, "1:only");
    }
}
