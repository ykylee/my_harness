//! myharness-tools — Read/Write/Edit/Bash/Grep/Glob (5종 기본 tool)
//!
//! v1 MVP (TASK-005-1 W3).

pub mod adapter;
pub mod bash;
pub mod content_hash;
pub mod edit;
pub mod error;
pub mod glob_;
pub mod grep;
pub mod permission;
pub mod read;
pub mod registry;
pub mod sanitizer;
pub mod schema;
pub mod tool;
pub mod write;

pub use adapter::execute_by_name;
pub use content_hash::{compute_content_hash, format_line_anchored, HASH_TAG_LENGTH};
pub use error::ToolError;
pub use permission::{PermissionDecision, PermissionGuard};
pub use registry::ToolRegistry;
pub use sanitizer::{BashSanitizer, SanitizerMode, SanitizerViolation};
pub use schema::{ProviderCompat, ToolSchema, ToolSchemaRegistry};
pub use tool::{PermissionMode, Tool, ToolContext, ToolResult};

#[must_use] 
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub mod prelude {
    pub use crate::adapter::execute_by_name;
    pub use crate::error::ToolError;
    pub use crate::permission::{PermissionDecision, PermissionGuard};
    pub use crate::registry::ToolRegistry;
    pub use crate::sanitizer::{BashSanitizer, SanitizerMode, SanitizerViolation};
    pub use crate::schema::{ProviderCompat, ToolSchema, ToolSchemaRegistry};
    pub use crate::tool::{PermissionMode, Tool, ToolContext, ToolResult};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_set() {
        assert!(!version().is_empty());
    }

    #[test]
    fn registry_default_roundtrip() {
        let reg = registry::ToolRegistry::default_tools();
        let names = reg.names();
        assert_eq!(names.len(), 6);
        for name in &["Bash", "Edit", "Glob", "Grep", "Read", "Write"] {
            assert!(names.contains(&name.to_string()), "missing tool: {name}");
        }
    }

    #[test]
    fn schema_default_has_6() {
        let reg = schema::ToolSchemaRegistry::default_schemas();
        assert_eq!(reg.names().len(), 6);
        assert!(reg.get("Read").is_some());
        assert!(reg.get("Glob").is_some());
    }

    // D-109 (2026-07-01) — Tool trait description + input_schema
    // override tests. Every default tool must declare a non-empty
    // description and a JSON Schema object that lists at least one
    // required field. The Edit tool may declare additional optional
    // modes (line_anchored / block_anchored / pure_edit) but its
    // top-level `required` is still only `file_path`.
    #[test]
    fn d109_all_default_tools_declare_description_and_schema() {
        let reg = registry::ToolRegistry::default_tools();
        for name in reg.names() {
            let tool = reg.get(&name).expect("tool must exist");
            assert!(!tool.description().is_empty(), "{name}: empty description");
            let schema = tool.input_schema();
            assert_eq!(
                schema.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "{name}: schema.type != object"
            );
            let required = schema
                .get("required")
                .and_then(|v| v.as_array())
                .unwrap_or_else(|| panic!("{name}: schema.required is not an array"));
            assert!(!required.is_empty(), "{name}: schema.required is empty");
        }
    }

    #[test]
    fn d109_read_tool_schema_is_well_formed() {
        let reg = registry::ToolRegistry::default_tools();
        let tool = reg.get("Read").unwrap();
        let schema = tool.input_schema();
        let required = schema["required"].as_array().unwrap();
        assert_eq!(required, &vec![serde_json::Value::String("file_path".to_string())]);
        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("file_path"));
        assert!(props.contains_key("offset"));
        assert!(props.contains_key("limit"));
        assert!(props.contains_key("format"));
    }

    #[test]
    fn d109_write_tool_schema_requires_content() {
        let reg = registry::ToolRegistry::default_tools();
        let tool = reg.get("Write").unwrap();
        let schema = tool.input_schema();
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert!(required.contains(&"file_path"));
        assert!(required.contains(&"content"));
    }

    #[test]
    fn d109_edit_tool_schema_includes_modes() {
        let reg = registry::ToolRegistry::default_tools();
        let tool = reg.get("Edit").unwrap();
        let schema = tool.input_schema();
        let props = schema["properties"].as_object().unwrap();
        // D-109 surface: 3 hashline modes + classic old_string/new_string.
        assert!(props.contains_key("old_string"));
        assert!(props.contains_key("new_string"));
        assert!(props.contains_key("line_anchored"));
        assert!(props.contains_key("block_anchored"));
        assert!(props.contains_key("pure_edit"));
    }

    #[test]
    fn d109_bash_tool_schema_requires_command() {
        let reg = registry::ToolRegistry::default_tools();
        let tool = reg.get("Bash").unwrap();
        let schema = tool.input_schema();
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(required, vec!["command"]);
        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("timeout_ms"));
    }

    #[test]
    fn d109_glob_tool_schema_requires_pattern() {
        let reg = registry::ToolRegistry::default_tools();
        let tool = reg.get("Glob").unwrap();
        let schema = tool.input_schema();
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(required, vec!["pattern"]);
    }

    #[test]
    fn d109_grep_tool_schema_requires_pattern_and_supports_include() {
        let reg = registry::ToolRegistry::default_tools();
        let tool = reg.get("Grep").unwrap();
        let schema = tool.input_schema();
        let required: Vec<&str> = schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(required, vec!["pattern"]);
        let props = schema["properties"].as_object().unwrap();
        assert!(props.contains_key("include"));
    }
}
