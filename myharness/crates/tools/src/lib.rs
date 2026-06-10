//! myharness-tools — Read/Write/Edit/Bash/Grep/Glob (5종 기본 tool)
//!
//! v1 MVP (TASK-005-1 W3).

pub mod adapter;
pub mod bash;
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
pub use error::ToolError;
pub use permission::{PermissionDecision, PermissionGuard};
pub use registry::ToolRegistry;
pub use sanitizer::{BashSanitizer, SanitizerMode, SanitizerViolation};
pub use schema::{ProviderCompat, ToolSchema, ToolSchemaRegistry};
pub use tool::{PermissionMode, Tool, ToolContext, ToolResult};

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
}
