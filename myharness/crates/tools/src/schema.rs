//! Tool schemas for Read/Write/Edit/Bash/Grep/Glob.
//!
//! ## Provider Compat Matrix (TASK-005-1 W6.5.5, librarian 검증)
//!
//! ### Request side (tools definition)
//!
//! | Field | Anthropic | `OpenAI` strict | `DeepSeek` | Ollama (OpenAI-compat) | llama.cpp | `LiteLlm` |
//! |---|---|---|---|---|---|---|
//! | `name` | ✅ top-level | ✅ `function.name` | ✅ `function.name` | ✅ `function.name` | ✅ `function.name` | ✅ `function.name` |
//! | `description` | ✅ top-level | ✅ `function.description` | ✅ `function.description` | ✅ `function.description` | ✅ `function.description` | ✅ `function.description` |
//! | `parameters` | ✅ `input_schema` | ✅ `function.parameters` | ✅ `function.parameters` | ✅ `function.parameters` | ✅ `function.parameters` | ✅ `function.parameters` |
//! | `strict: true` | ❌ | ✅ (2024-08+) | ✅ (**Beta URL** `api.deepseek.com/beta`) | ❌ | ❌ | ✅ (pass-through) |
//! | `parameters.additionalProperties: false` | ❌ | ✅ (strict 필수) | ✅ (strict 필수) | ❌ | ❌ | ✅ (pass-through) |
//! | `parameters.required` (all fields) | ❌ (in `input_schema`) | ✅ (strict 필수) | ✅ (strict 필수) | ✅ (basic) | ✅ (basic) | ✅ (pass-through) |
//! | `parameters.$schema` | ❌ | ⚠️ strip 권장 (draft-07 거부 가능) | ⚠️ strip 권장 | ❌ (strip) | ❌ (strip) | ⚠️ strip 권장 (pass-through) |
//!
//! ### Response side (`tool_calls` array)
//!
//! | Field | `OpenAI` | `DeepSeek` | Ollama (OpenAI-compat) | llama.cpp | `LiteLlm` |
//! |---|---|---|---|---|---|
//! | `tool_calls[].id` | ✅ `call_abc` | ✅ `call_00_...` | ❌ **없음** | ❌ **없음** | ✅ (upstream) |
//! | `tool_calls[].type: "function"` | ✅ | ✅ | ❌ | ❌ | ✅ (upstream) |
//! | `tool_calls[].function.arguments` | ✅ **string** (JSON) | ✅ **string** | ✅ **object** (raw, native API) | ✅ **string** | ✅ (upstream) |
//! | `finish_reason` | `"tool_calls"` | `"tool_calls"` | `"tool_calls"` | `"tool"` ⚠️ | ✅ (upstream) |
//! | `reasoning_content` (extra) | ❌ | ✅ thinking mode | ✅ | ✅ `<think>` | ✅ (upstream) |
//!
//! 출처: librarian 조사 (5 provider 공식 docs, 2026-06 시점)
//! Response 처리는 W7+ (llm crate 진입 시점).
//!
//! **중요 정정 (W6.5.5)**:
//! - `DeepSeek` strict mode = **Beta URL** `https://api.deepseek.com/beta` 필요 (W6.5 가 "❌" 으로 표기했으나 librarian 가 "✅" 확인)
//! - schemars 1.2 의 `$schema` 출력 = `http://json-schema.org/draft-07/schema#` (W6.5 가 "draft/2020-12" 로 임의 강제했으나 실제는 draft-07)
//! - `OpenAI` structured outputs 는 `$schema` field 거부 가능 → strict mode wire format 에서 strip
//! - Ollama/llama.cpp 는 response 의 `id`/`type`/`arguments` (object) 가 `OpenAI` 와 다름 → W7+ 에서 response 처리 시 분기
//! - llama.cpp `finish_reason: "tool"` (단수) → `OpenAI` `"tool_calls"` 와 다름

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct ReadInput {
    pub file_path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct WriteInput {
    pub file_path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct EditInput {
    pub file_path: String,
    pub old_string: String,
    pub new_string: String,
    pub replace_all: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct BashInput {
    pub command: String,
    pub timeout_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct GrepInput {
    pub pattern: String,
    pub path: Option<String>,
    pub include: Option<String>,
    pub case_insensitive: Option<bool>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct GlobInput {
    pub pattern: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct ToolSchemaRegistry {
    schemas: HashMap<String, ToolSchema>,
}

impl Default for ToolSchemaRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolSchemaRegistry {
    #[must_use] 
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn register(&mut self, schema: ToolSchema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    #[must_use] 
    pub fn default_schemas() -> Self {
        let mut reg = Self::new();
        reg.register(ToolSchema {
            name: "Read".to_string(),
            description: "Read the contents of a file".to_string(),
            input_schema: serde_json::to_value(schema_for!(ReadInput)).unwrap(),
        });
        reg.register(ToolSchema {
            name: "Write".to_string(),
            description: "Write content to a file (creates parent directories if needed)"
                .to_string(),
            input_schema: serde_json::to_value(schema_for!(WriteInput)).unwrap(),
        });
        reg.register(ToolSchema {
            name: "Edit".to_string(),
            description: "Replace strings in a file (single or all occurrences)".to_string(),
            input_schema: serde_json::to_value(schema_for!(EditInput)).unwrap(),
        });
        reg.register(ToolSchema {
            name: "Bash".to_string(),
            description: "Execute a shell command".to_string(),
            input_schema: serde_json::to_value(schema_for!(BashInput)).unwrap(),
        });
        reg.register(ToolSchema {
            name: "Grep".to_string(),
            description: "Search for a regex pattern in files".to_string(),
            input_schema: serde_json::to_value(schema_for!(GrepInput)).unwrap(),
        });
        reg.register(ToolSchema {
            name: "Glob".to_string(),
            description: "Find files matching a glob pattern".to_string(),
            input_schema: serde_json::to_value(schema_for!(GlobInput)).unwrap(),
        });
        reg
    }

    #[must_use] 
    pub fn get(&self, name: &str) -> Option<&ToolSchema> {
        self.schemas.get(name)
    }

    #[must_use] 
    pub fn names(&self) -> Vec<String> {
        let mut n: Vec<_> = self.schemas.keys().cloned().collect();
        n.sort();
        n
    }

    #[must_use] 
    pub fn to_anthropic_tools(&self) -> Vec<serde_json::Value> {
        self.schemas
            .values()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "description": s.description,
                    "input_schema": s.input_schema,
                })
            })
            .collect()
    }

    #[must_use] 
    pub fn to_openai_tools(&self) -> Vec<serde_json::Value> {
        self.schemas
            .values()
            .map(|s| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": s.name,
                        "description": s.description,
                        "parameters": s.input_schema,
                    }
                })
            })
            .collect()
    }

    /// `OpenAI` strict mode: `strict=true`, `additionalProperties=false`,
    /// `$schema` stripped (`OpenAI` 거부 가능), `required` array from schemars.
    #[must_use] 
    pub fn to_openai_tools_strict(&self) -> Vec<serde_json::Value> {
        self.schemas
            .values()
            .map(|s| {
                let mut params = s.input_schema.clone();
                if let Some(obj) = params.as_object_mut() {
                    obj.insert(
                        "additionalProperties".to_string(),
                        serde_json::Value::Bool(false),
                    );
                }
                let params = strip_dollar_schema(params);
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": s.name,
                        "description": s.description,
                        "parameters": params,
                        "strict": true,
                    }
                })
            })
            .collect()
    }

    /// `DeepSeek`: OpenAI-compatible wire format.
    /// Strict mode 는 Beta URL (`https://api.deepseek.com/beta`) 에서 지원.
    /// Non-Beta URL 에서는 `to_openai_tools()` 사용.
    #[must_use] 
    pub fn to_deepseek_tools(&self) -> Vec<serde_json::Value> {
        self.schemas
            .values()
            .map(|s| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": s.name,
                        "description": s.description,
                        "parameters": s.input_schema,
                    }
                })
            })
            .collect()
    }

    /// Ollama 0.5.x: OpenAI-compatible minus `$schema` and `additionalProperties`.
    #[must_use] 
    pub fn to_ollama_tools(&self) -> Vec<serde_json::Value> {
        self.schemas
            .values()
            .map(|s| {
                let mut params = s.input_schema.clone();
                if let Some(obj) = params.as_object_mut() {
                    obj.remove("$schema");
                    obj.remove("additionalProperties");
                }
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": s.name,
                        "description": s.description,
                        "parameters": params,
                    }
                })
            })
            .collect()
    }

    /// `LiteLlm`: 1:1 alias → [`ToolSchemaRegistry::to_openai_tools_strict`].
    /// `$schema` strip 포함 (pass-through 시 거부 회피).
    #[must_use] 
    pub fn to_litellm_tools(&self) -> Vec<serde_json::Value> {
        self.to_openai_tools_strict()
    }
}

/// Recursively remove `$schema` fields from a JSON schema value.
/// `OpenAI` strict mode / `LiteLlm` pass-through 시 `$schema` 거부 회피.
fn strip_dollar_schema(mut params: serde_json::Value) -> serde_json::Value {
    if let Some(obj) = params.as_object_mut() {
        obj.remove("$schema");
        for (_key, v) in obj.iter_mut() {
            if v.is_object() {
                *v = strip_dollar_schema(v.clone());
            } else if let Some(arr) = v.as_array_mut() {
                for item in arr.iter_mut() {
                    if item.is_object() {
                        *item = strip_dollar_schema(item.clone());
                    }
                }
            }
        }
    }
    params
}

/// Provider-agnostic dispatch for wire-format tool generation.
///
/// Maps each provider to its corresponding `ToolSchemaRegistry` method.
/// Supported providers: Anthropic, `OpenAI` (strict), `DeepSeek`, Ollama, llama.cpp, `LiteLlm`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderCompat {
    Anthropic,
    OpenAI,
    DeepSeek,
    Ollama,
    LlamaCpp,
    LiteLlm,
}

impl ProviderCompat {
    /// Dispatch to the correct wire-format method for this provider.
    #[must_use] 
    pub fn wire_format_tools(&self, reg: &ToolSchemaRegistry) -> Vec<serde_json::Value> {
        match self {
            Self::Anthropic => reg.to_anthropic_tools(),
            Self::OpenAI => reg.to_openai_tools_strict(),
            Self::DeepSeek => reg.to_deepseek_tools(),
            Self::Ollama | Self::LlamaCpp => reg.to_ollama_tools(),
            Self::LiteLlm => reg.to_litellm_tools(),
        }
    }

    /// Canonical provider name string.
    #[must_use] 
    pub fn name(&self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAI => "openai",
            Self::DeepSeek => "deepseek",
            Self::Ollama => "ollama",
            Self::LlamaCpp => "llama.cpp",
            Self::LiteLlm => "litellm",
        }
    }

    /// Response format 의 주요 차이점 1-2줄. W7+ 에서 response 처리 시 활용.
    #[must_use] 
    pub fn response_notes(&self) -> &'static str {
        match self {
            Self::Anthropic => {
                "Anthropic: tool_use content block 별도. id 자동 생성. finish_reason=tool_use."
            }
            Self::OpenAI => {
                "OpenAI: tool_calls[].id='call_abc', arguments=string(JSON). finish_reason='tool_calls'."
            }
            Self::DeepSeek => {
                "DeepSeek: OpenAI 호환. reasoning_content 별도 (thinking mode). Beta URL 로 strict 지원."
            }
            Self::Ollama => {
                "Ollama: tool_calls[].id 없음. arguments=object(native). tool_choice 미지원."
            }
            Self::LlamaCpp => {
                "llama.cpp: tool_calls[].id 없음. finish_reason='tool' (단수). parallel_tool_calls 명시 필요."
            }
            Self::LiteLlm => "litellm: pass-through. upstream provider 응답 그대로.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schemas_has_6() {
        let reg = ToolSchemaRegistry::default_schemas();
        assert_eq!(reg.names().len(), 6);
        assert!(reg.get("Read").is_some());
        assert!(reg.get("Bash").is_some());
        assert!(reg.get("Glob").is_some());
    }

    #[test]
    fn test_anthropic_wire_format() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_anthropic_tools();
        assert_eq!(tools.len(), 6);
        for tool in &tools {
            assert!(tool.get("name").is_some());
            assert!(tool.get("description").is_some());
            assert!(tool.get("input_schema").is_some());
        }
    }

    #[test]
    fn test_openai_wire_format() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_openai_tools();
        assert_eq!(tools.len(), 6);
        for tool in &tools {
            assert_eq!(tool.get("type").and_then(|v| v.as_str()), Some("function"));
            let func = tool.get("function").unwrap();
            assert!(func.get("name").is_some());
            assert!(func.get("parameters").is_some());
        }
    }

    #[test]
    fn test_schema_has_object_type() {
        let reg = ToolSchemaRegistry::default_schemas();
        for name in &["Read", "Write", "Edit", "Bash", "Grep", "Glob"] {
            let schema = reg.get(name).expect("missing schema");
            let input = &schema.input_schema;
            assert_eq!(
                input.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "{name} input_schema should have type=object"
            );
            assert!(
                input.get("properties").is_some(),
                "{name} input_schema should have properties"
            );
        }
    }

    #[test]
    fn test_schema_roundtrip() {
        let reg = ToolSchemaRegistry::default_schemas();
        let anthropic = reg.to_anthropic_tools();
        let openai = reg.to_openai_tools();
        assert_eq!(anthropic.len(), openai.len());
        for (a, o) in anthropic.iter().zip(openai.iter()) {
            assert_eq!(
                a.get("name").and_then(|v| v.as_str()),
                o["function"].get("name").and_then(|v| v.as_str())
            );
        }
    }

    // ── W6.5: Provider-specific wire format tests ──

    #[test]
    fn test_openai_strict_has_required() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_openai_tools_strict();
        for tool in &tools {
            let params = &tool["function"]["parameters"];
            assert!(
                params.get("required").and_then(|v| v.as_array()).is_some(),
                "OpenAI strict should have parameters.required"
            );
        }
    }

    #[test]
    fn test_openai_strict_has_additional_properties_false() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_openai_tools_strict();
        for tool in &tools {
            let params = &tool["function"]["parameters"];
            assert_eq!(
                params.get("additionalProperties"),
                Some(&serde_json::Value::Bool(false))
            );
        }
    }

    #[test]
    fn test_openai_strict_has_strict_true() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_openai_tools_strict();
        for tool in &tools {
            assert_eq!(
                tool["function"].get("strict"),
                Some(&serde_json::Value::Bool(true))
            );
        }
    }

    #[test]
    fn test_openai_strict_strips_dollar_schema() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_openai_tools_strict();
        for tool in &tools {
            assert!(
                tool["function"]["parameters"].get("$schema").is_none(),
                "OpenAI strict 는 $schema field 없어야 함"
            );
        }
    }

    #[test]
    fn test_litellm_strips_dollar_schema() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_litellm_tools();
        for tool in &tools {
            assert!(tool["function"]["parameters"].get("$schema").is_none());
        }
    }

    #[test]
    fn test_deepseek_does_not_strip_dollar_schema() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_deepseek_tools();
        for tool in &tools {
            assert!(
                tool["function"]["parameters"].get("$schema").is_some(),
                "DeepSeek 는 draft-07 $schema 허용, strip 불필요"
            );
        }
    }

    #[test]
    fn test_ollama_strips_dollar_schema() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_ollama_tools();
        for tool in &tools {
            assert!(tool["function"]["parameters"].get("$schema").is_none());
        }
    }

    #[test]
    fn test_provider_response_notes() {
        assert!(
            ProviderCompat::Ollama
                .response_notes()
                .contains("arguments=object")
        );
        assert!(ProviderCompat::LlamaCpp.response_notes().contains("tool"));
        assert!(
            ProviderCompat::DeepSeek
                .response_notes()
                .contains("reasoning_content")
        );
        assert!(
            ProviderCompat::Anthropic
                .response_notes()
                .contains("tool_use")
        );
        assert!(ProviderCompat::OpenAI.response_notes().contains("call_abc"));
    }

    #[test]
    fn test_deepseek_no_strict_field() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_deepseek_tools();
        for tool in &tools {
            let strict = tool["function"].get("strict");
            assert!(
                strict.is_none() || strict == Some(&serde_json::Value::Null),
                "DeepSeek should not have strict field"
            );
        }
    }

    #[test]
    fn test_ollama_no_dollar_schema() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = reg.to_ollama_tools();
        for tool in &tools {
            let dollar_schema = tool["function"]["parameters"].get("$schema");
            assert!(
                dollar_schema.is_none(),
                "Ollama should strip $schema from parameters"
            );
        }
    }

    #[test]
    fn test_litellm_equals_openai_strict() {
        let reg = ToolSchemaRegistry::default_schemas();
        let openai = reg.to_openai_tools_strict();
        let litellm = reg.to_litellm_tools();
        assert_eq!(
            openai, litellm,
            "LiteLlm should produce OpenAI strict format"
        );
    }

    // ── W6.5: ProviderCompat dispatch tests ──

    #[test]
    fn test_provider_compat_dispatch_anthropic() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = ProviderCompat::Anthropic.wire_format_tools(&reg);
        assert_eq!(tools.len(), 6);
        assert!(tools[0].get("input_schema").is_some());
    }

    #[test]
    fn test_provider_compat_dispatch_openai() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = ProviderCompat::OpenAI.wire_format_tools(&reg);
        assert_eq!(tools.len(), 6);
        let func = &tools[0]["function"];
        assert_eq!(func["strict"], serde_json::Value::Bool(true));
    }

    #[test]
    fn test_provider_compat_dispatch_deepseek() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = ProviderCompat::DeepSeek.wire_format_tools(&reg);
        assert_eq!(tools.len(), 6);
        let func = &tools[0]["function"];
        assert!(func.get("strict").is_none() || func["strict"].is_null());
    }

    #[test]
    fn test_provider_compat_dispatch_ollama() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = ProviderCompat::Ollama.wire_format_tools(&reg);
        assert_eq!(tools.len(), 6);
        assert!(tools[0]["function"]["parameters"].get("$schema").is_none());
    }

    #[test]
    fn test_provider_compat_dispatch_llamacpp() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = ProviderCompat::LlamaCpp.wire_format_tools(&reg);
        assert_eq!(tools.len(), 6);
        assert!(tools[0]["function"]["parameters"].get("$schema").is_none());
    }

    #[test]
    fn test_provider_compat_dispatch_litellm() {
        let reg = ToolSchemaRegistry::default_schemas();
        let tools = ProviderCompat::LiteLlm.wire_format_tools(&reg);
        assert_eq!(tools.len(), 6);
        let func = &tools[0]["function"];
        assert_eq!(func["strict"], serde_json::Value::Bool(true));
    }

    #[test]
    fn test_provider_compat_name() {
        assert_eq!(ProviderCompat::Anthropic.name(), "anthropic");
        assert_eq!(ProviderCompat::OpenAI.name(), "openai");
        assert_eq!(ProviderCompat::DeepSeek.name(), "deepseek");
        assert_eq!(ProviderCompat::Ollama.name(), "ollama");
        assert_eq!(ProviderCompat::LlamaCpp.name(), "llama.cpp");
        assert_eq!(ProviderCompat::LiteLlm.name(), "litellm");
    }
}
