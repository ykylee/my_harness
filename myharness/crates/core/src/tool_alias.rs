//! tool name alias table — sub-agent 의 도구 이름 (`PascalCase`) ↔ tools crate (`snake_case`).
//!
//! sub-agent (W10.2) 가 기대하는 도구 이름: Read, Write, Edit, Bash, Grep, Glob
//! tools crate (W3) 의 실제 이름: read, write, edit, bash, grep, glob_
//!
//! v1 simple: 양방향 매핑 + lookup helper.

use std::collections::HashMap;

/// 양방향 alias 매핑.
#[must_use]
pub fn known_aliases() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    // sub-agent PascalCase → tools crate snake_case
    m.insert("Read", "read");
    m.insert("Write", "write");
    m.insert("Edit", "edit");
    m.insert("Bash", "bash");
    m.insert("Grep", "grep");
    m.insert("Glob", "glob_");
    // tools crate → sub-agent PascalCase
    m.insert("read", "Read");
    m.insert("write", "Write");
    m.insert("edit", "Edit");
    m.insert("bash", "Bash");
    m.insert("grep", "Grep");
    m.insert("glob_", "Glob");
    m
}

/// alias table.
pub static KNOWN_TOOL_ALIASES: std::sync::LazyLock<HashMap<&'static str, &'static str>> =
    std::sync::LazyLock::new(known_aliases);

/// sub-agent name → tools crate name. 없으면 그대로 반환 (passthrough).
pub fn resolve_tool_alias(name: &str) -> String {
    KNOWN_TOOL_ALIASES
        .get(name)
        .map_or_else(|| name.to_string(), std::string::ToString::to_string)
}

/// 여러 alias 일괄 변환.
#[must_use]
pub fn resolve_all_aliases(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| resolve_tool_alias(n)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pascal_to_snake() {
        assert_eq!(resolve_tool_alias("Read"), "read");
        assert_eq!(resolve_tool_alias("Write"), "write");
        assert_eq!(resolve_tool_alias("Bash"), "bash");
        assert_eq!(resolve_tool_alias("Glob"), "glob_");
    }

    #[test]
    fn snake_to_pascal() {
        assert_eq!(resolve_tool_alias("read"), "Read");
        assert_eq!(resolve_tool_alias("grep"), "Grep");
        assert_eq!(resolve_tool_alias("glob_"), "Glob");
    }

    #[test]
    fn unknown_passthrough() {
        assert_eq!(resolve_tool_alias("Custom"), "Custom");
        assert_eq!(resolve_tool_alias("mystery"), "mystery");
    }

    #[test]
    fn batch_resolve() {
        let r = resolve_all_aliases(&["Read", "Write", "Unknown"]);
        assert_eq!(r, vec!["read", "write", "Unknown"]);
    }

    #[test]
    fn aliases_table_has_six_pairs() {
        assert_eq!(KNOWN_TOOL_ALIASES.len(), 12); // 6 양방향
    }
}
