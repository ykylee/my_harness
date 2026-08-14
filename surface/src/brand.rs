//! User-visible brand. Engine chrome must not leak onto our pixels.

pub const WORDMARK: &str = "myharness";
pub const OSC_TITLE: &str = "myharness";
pub const MODEL_ALIAS: &str = "MiniMax-M3";

const GROK_NAMESPACES: &[&str] = &[
    "GrokBuildHashline:",
    "GrokBuildConcise:",
    "GrokBuild:",
    "grokbuildhashline:",
    "grokbuildconcise:",
    "grokbuild:",
];

/// Drop leading `Grok*` tool namespaces. Keep Codex: / OpenCode: / MCP:.
pub fn remap_tool(name: &str) -> String {
    let mut s = name.to_string();
    for prefix in GROK_NAMESPACES {
        if s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix) {
            s = s[prefix.len()..].to_string();
            break;
        }
    }
    s
}

pub fn strip_chrome(text: &str) -> String {
    let mut out = String::new();
    let mut hide = false;
    for line in text.lines() {
        let lower = line.to_ascii_lowercase();
        if lower.contains("<think>") {
            hide = true;
        }
        if !hide
            && !lower.contains("grok build")
            && !lower.contains("groknight")
            && !lower.contains("xai-grok-pager")
        {
            out.push_str(line);
            out.push('\n');
        }
        if lower.contains("</think>") {
            hide = false;
        }
    }
    out.trim().to_string()
}

pub fn leaks_vendor_chrome(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("grok build")
        || lower.contains("groknight")
        || lower.contains("xai-grok-pager")
        || lower.contains("grokbuild:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaps_all_grok_namespaces() {
        assert_eq!(remap_tool("GrokBuild:bash"), "bash");
        assert_eq!(remap_tool("GrokBuildConcise:read_file"), "read_file");
        assert_eq!(remap_tool("GrokBuildHashline:grep"), "grep");
        assert_eq!(remap_tool("Codex:ApplyPatch"), "Codex:ApplyPatch");
        assert_eq!(remap_tool("OpenCode:bash"), "OpenCode:bash");
        assert_eq!(remap_tool("MCP:foo"), "MCP:foo");
    }

    #[test]
    fn wordmark_is_ours() {
        assert_eq!(WORDMARK, "myharness");
        assert!(!WORDMARK.to_ascii_lowercase().contains("grok"));
    }
}
