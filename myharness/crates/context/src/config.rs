//! ContextConfig — `~/.myharness/config.toml` 의 [context] 섹션 통합.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::auto_memory::AutoMemory;
use crate::budget::{BudgetConfig, BudgetReport, ContextManager};
use crate::claude_md::{ContextLoader, DiscoveredContext};
use crate::compression::{BuiltinConfig, BuiltinPipeline};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ContextConfig {
    #[serde(default)]
    pub budget: BudgetConfig,
    #[serde(default)]
    pub builtin: BuiltinLayerConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuiltinLayerConfig {
    pub enabled: bool,
    #[serde(default)]
    pub algorithms: BuiltinConfig,
}

impl Default for BuiltinLayerConfig {
    fn default() -> Self {
        Self {
            enabled: false, // D-30: 기본 OFF
            algorithms: BuiltinConfig::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml decode: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("toml encode: {0}")]
    TomlEncode(#[from] toml::ser::Error),
}

impl ContextConfig {
    /// TOML 파일에서 load. 없거나 [context] 섹션이 없으면 default 반환.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(path)?;
        // 전체 config.toml 에서 [context] 섹션만 발취
        let value: toml::Value = toml::from_str(&s)?;
        if let Some(ctx) = value.get("context") {
            let cfg: ContextConfig = ctx.clone().try_into()?;
            Ok(cfg)
        } else {
            Ok(Self::default())
        }
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        Ok(toml::to_string(self)?)
    }
}

/// ContextManager + Builtin pipeline + Auto memory 통합.
pub struct ContextOrchestrator {
    pub config: ContextConfig,
    pub manager: ContextManager,
    pub builtin: BuiltinPipeline,
    pub auto_memory: Option<AutoMemory>,
    pub claude_contexts: Vec<DiscoveredContext>,
}

impl ContextOrchestrator {
    pub fn from_config(config: ContextConfig, cwd: &Path) -> Result<Self, crate::budget::BudgetError> {
        let manager = ContextManager::new(config.budget.clone())?;
        let builtin = BuiltinPipeline::new(config.builtin.algorithms.clone());
        let auto_memory = AutoMemory::new().ok();
        let mut loader = ContextLoader::new();
        if !config.builtin.enabled {
            // global fallback 도 사용 안 함 — builtin OFF 일 때 injection 안 함
            loader = loader.without_global();
        }
        let claude_contexts = loader.discover(cwd);
        Ok(Self { config, manager, builtin, auto_memory, claude_contexts })
    }

    /// LLM 호출 직전 호출: 1) builtin 압축, 2) auto memory inject, 3) claude context inject.
    /// returns (system_prompt, messages).
    pub fn prepare_request(
        &self,
        user_system: Option<String>,
        messages: Vec<crate::budget::Message>,
    ) -> (Option<String>, Vec<crate::budget::Message>) {
        // 1) builtin 압축
        let (sys, msgs) = if self.config.builtin.enabled {
            self.builtin.run(user_system, messages)
        } else {
            (user_system, messages)
        };

        // 2) system prompt 에 CLAUDE.md context + auto memory 주입
        let mut parts: Vec<String> = Vec::new();
        if let Some(s) = sys {
            parts.push(s);
        }
        let ctx_prompt = ContextLoader::merge_to_system_prompt(&self.claude_contexts);
        if !ctx_prompt.is_empty() {
            parts.push(ctx_prompt);
        }
        if let Some(mem) = &self.auto_memory {
            if let Ok(s) = mem.to_system_prompt_section(20) {
                if !s.is_empty() {
                    parts.push(s);
                }
            }
        }
        let combined = if parts.is_empty() { None } else { Some(parts.join("\n\n")) };

        (combined, msgs)
    }

    pub fn push(&mut self, msg: crate::budget::Message) -> BudgetReport {
        self.manager.push(msg);
        self.manager.maybe_auto_compact();
        self.manager.budget_report()
    }

    /// /compact slash command. user-callable 수동 압축.
    pub fn compact(&mut self) {
        self.manager.compact();
    }

    pub fn record_tool(&self, tool: &str, args: serde_json::Value) {
        if let Some(mem) = &self.auto_memory {
            let _ = mem.append_tool(tool, args);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::budget::CompactStrategy;

    #[test]
    fn default_config_has_builtin_disabled() {
        let c = ContextConfig::default();
        assert!(!c.builtin.enabled);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let c = ContextConfig::load(std::path::Path::new("/nonexistent/config.toml")).unwrap();
        assert!(!c.builtin.enabled);
    }

    #[test]
    fn load_with_context_section() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, r#"
[context]
[context.budget]
max_tokens = 100000
warn_ratio = 0.7
keep_recent = 10
strategy = "truncate"
chars_per_token = 4

[context.builtin]
enabled = true
[context.builtin.algorithms]
cache_aligner = true
content_router = true
smart_crusher = false
code_compressor = false
"#).unwrap();
        let c = ContextConfig::load(&path).unwrap();
        assert!(c.builtin.enabled);
        assert_eq!(c.budget.max_tokens, 100_000);
        assert_eq!(c.budget.keep_recent, 10);
        assert!(!c.builtin.algorithms.smart_crusher);
    }

    #[test]
    fn orchestrator_prepare_request_includes_claude_context() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("CLAUDE.md"), "test rule").unwrap();
        let config = ContextConfig::default();
        let orch = ContextOrchestrator::from_config(config, dir.path()).unwrap();
        let (sys, _) = orch.prepare_request(None, vec![crate::budget::Message::user("hi")]);
        let s = sys.unwrap();
        assert!(s.contains("test rule"));
    }

    #[test]
    fn orchestrator_builtin_disabled_passes_through() {
        let dir = tempfile::tempdir().unwrap();
        let config = ContextConfig {
            builtin: BuiltinLayerConfig { enabled: false, ..Default::default() },
            ..Default::default()
        };
        let orch = ContextOrchestrator::from_config(config, dir.path()).unwrap();
        let input = vec![crate::budget::Message::user(r#"{"b":2,"a":1}"#)];
        let (sys, msgs) = orch.prepare_request(None, input);
        // builtin off → JSON 압축 안 됨
        assert!(msgs[0].content.contains("\"b\""));
        assert!(sys.is_none() || !sys.as_ref().unwrap().contains("\"a\""));
    }

    #[test]
    fn orchestrator_builtin_enabled_compresses_json() {
        let dir = tempfile::tempdir().unwrap();
        let config = ContextConfig {
            builtin: BuiltinLayerConfig {
                enabled: true,
                algorithms: BuiltinConfig {
                    cache_aligner: false,
                    content_router: true,
                    smart_crusher: true,
                    code_compressor: false,
                },
            },
            ..Default::default()
        };
        let orch = ContextOrchestrator::from_config(config, dir.path()).unwrap();
        let (sys, msgs) = orch.prepare_request(None, vec![crate::budget::Message::user(r#"{"b":2,"a":1}"#)]);
        // smart_crush 가 key 정렬
        assert!(msgs[0].content.find("\"a\"").unwrap() < msgs[0].content.find("\"b\"").unwrap());
        assert!(sys.is_none());
    }

    #[test]
    fn record_tool_writes_to_memory() {
        let dir = tempfile::tempdir().unwrap();
        // AutoMemory::new() 는 MYHARNESS_HOME env 를 존중. Rust 2024 의
        // env::set_var/remove_var unsafe 회피 — with_base 로 직접 override.
        // ContextOrchestrator 의 auto_memory 는 AutoMemory::new() 호출이지만
        // 테스트에서는 .with_base() 로 교체한다.
        let config = ContextConfig::default();
        let mut orch = ContextOrchestrator::from_config(config, dir.path()).unwrap();
        orch.auto_memory = Some(AutoMemory::with_base(
            dir.path().join("memory").join("auto"),
        ));
        orch.record_tool("Read", serde_json::json!({"path": "/x"}));
        let log = dir.path().join("memory").join("auto").join("memory.ndjson");
        assert!(log.exists());
    }

    #[test]
    fn strategy_default_is_hybrid() {
        assert_eq!(BudgetConfig::default().strategy, CompactStrategy::Hybrid);
    }
}
