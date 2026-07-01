use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::ToolError;
use crate::sanitizer::SanitizerMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PermissionMode {
    #[serde(rename = "default")]
    #[default]
    Default,
    #[serde(rename = "acceptEdits")]
    AcceptEdits,
    #[serde(rename = "plan")]
    Plan,
    #[serde(rename = "bypassPermissions")]
    BypassPermissions,
}

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub cwd: PathBuf,
    pub permission_mode: PermissionMode,
    pub confirm_override: bool,
    pub sanitizer_mode: SanitizerMode,
}

impl ToolContext {
    #[must_use] 
    pub fn new(cwd: PathBuf, permission_mode: PermissionMode) -> Self {
        Self {
            cwd,
            permission_mode,
            confirm_override: false,
            sanitizer_mode: SanitizerMode::default(),
        }
    }

    #[must_use] 
    pub fn with_confirm_override(mut self, override_: bool) -> Self {
        self.confirm_override = override_;
        self
    }

    #[must_use] 
    pub fn with_sanitizer_mode(mut self, mode: SanitizerMode) -> Self {
        self.sanitizer_mode = mode;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub output: String,
    pub is_error: bool,
    pub metadata: Option<serde_json::Value>,
}

impl ToolResult {
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            output: msg.into(),
            is_error: true,
            metadata: None,
        }
    }
}

#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;

    /// A-proper native tool calling (D-108 follow-up, D-109): human
    /// description of the tool, surfaced as the OpenAI `description`
    /// field. Default is the empty string (minimal-spec behaviour
    /// from D-108 v1.5). Override to give the LLM a clear hint.
    fn description(&self) -> &'static str {
        ""
    }

    /// A-proper native tool calling (D-108 follow-up, D-109): JSON
    /// Schema object describing the tool's input arguments. Default
    /// is an empty `{"type":"object","properties":{}}` (i.e. no
    /// declared parameters). Override to declare per-tool fields.
    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        input: serde_json::Value,
    ) -> Result<ToolResult, ToolError>;
}
