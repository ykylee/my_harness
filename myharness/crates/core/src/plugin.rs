//! Plugin 시스템 (CONCEPT §5.7 + claude-code 13.3 plugin 패턴).
//!
//! v2.0 진입의 첫 commit (D-71) — manifest + discovery 만. commands/agents/skills/hooks
//! 로딩은 D-72~D-75 에서 단계적.
//!
//! ## 구조
//!
//! ```text
//! ~/.myharness/plugins/<name>/
//! ├── plugin.json           # manifest
//! ├── commands/             # slash commands (D-72)
//! ├── agents/               # specialized sub-agents (D-73)
//! ├── skills/               # auto-invoke knowledge (D-74)
//! └── hooks/                # event handlers (D-75)
//! ```
//!
//! ## plugin.json 형식
//!
//! ```json
//! {
//!   "name": "git-workflow",
//!   "version": "0.1.0",
//!   "description": "Git operations and PR workflow helpers",
//!   "author": "yklee",
//!   "entrypoints": {
//!     "commands": "commands/",
//!     "agents": "agents/",
//!     "skills": "skills/",
//!     "hooks": "hooks/"
//!   },
//!   "requires": {
//!     "myharness": ">=0.1.0"
//!   }
//! }
//! ```
//!
//! ## D-71 scope
//!
//! - `PluginManifest` schema (serde + JSON)
//! - `PluginLocation` (path 정보)
//! - `PluginRegistry::discover(plugins_dir)` (root scan, manifest parse)
//! - `PluginError` (parse, io, version, missing field)
//! - `~/.myharness/plugins/` 자동 생성 (paths.rs)
//! - Unit test + integration test

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Plugin manifest. `plugin.json` 의 스키마.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    /// plugin 이름 (디렉토리 이름과 동일 권장).
    #[serde(default)]
    pub name: String,
    /// semver.
    #[serde(default)]
    pub version: String,
    /// 한 줄 설명.
    #[serde(default)]
    pub description: String,
    /// 작성자.
    #[serde(default)]
    pub author: String,
    /// entrypoint 디렉토리 (기본값 모두 표준 이름).
    #[serde(default)]
    pub entrypoints: Entrypoints,
    /// 의존성 제약.
    #[serde(default)]
    pub requires: BTreeMap<String, String>,
}

impl PluginManifest {
    /// `name@version` 형식.
    #[must_use] 
    pub fn id(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// entrypoint 디렉토리. 표준 경로 사용 시 비워두면 됨.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entrypoints {
    #[serde(default = "default_commands")]
    pub commands: String,
    #[serde(default = "default_agents")]
    pub agents: String,
    #[serde(default = "default_skills")]
    pub skills: String,
    #[serde(default = "default_hooks")]
    pub hooks: String,
}

impl Default for Entrypoints {
    fn default() -> Self {
        Self {
            commands: default_commands(),
            agents: default_agents(),
            skills: default_skills(),
            hooks: default_hooks(),
        }
    }
}

fn default_commands() -> String {
    "commands".into()
}
fn default_agents() -> String {
    "agents".into()
}
fn default_skills() -> String {
    "skills".into()
}
fn default_hooks() -> String {
    "hooks".into()
}

/// 발견된 plugin 의 location 정보. manifest + path 결합.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginLocation {
    pub manifest: PluginManifest,
    pub root: PathBuf,
}

impl PluginLocation {
    #[must_use] 
    pub fn id(&self) -> String {
        self.manifest.id()
    }
}

/// Plugin registry. `~/.myharness/plugins/` root 에서 발견된 plugins.
#[derive(Debug, Clone, Default)]
pub struct PluginRegistry {
    plugins: BTreeMap<String, PluginLocation>,
}

impl PluginRegistry {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// `plugins_dir` (=`~/.myharness/plugins/`) 에서 모든 plugin 발견.
    /// 각 subdir 의 `plugin.json` parse. 실패한 plugin 은 skip + `tracing::warn`.
    pub fn discover(plugins_dir: &Path) -> Self {
        let mut reg = Self::new();
        let entries = match std::fs::read_dir(plugins_dir) {
            Ok(e) => e,
            Err(e) => {
                tracing::debug!(dir = %plugins_dir.display(), error = %e, "plugin dir not readable");
                return reg;
            }
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let manifest_path = path.join("plugin.json");
            if !manifest_path.is_file() {
                tracing::debug!(path = %path.display(), "skip: no plugin.json");
                continue;
            }
            match Self::load_one(&path, &manifest_path) {
                Ok(loc) => {
                    reg.plugins.insert(loc.manifest.name.clone(), loc);
                }
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "plugin load failed; skipping");
                }
            }
        }
        reg
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// 단일 plugin 디렉토리 load.
    pub fn load_one(root: &Path, manifest_path: &Path) -> Result<PluginLocation, PluginError> {
        let bytes = std::fs::read(manifest_path).map_err(|e| PluginError::Io {
            path: manifest_path.to_path_buf(),
            source: e,
        })?;
        let manifest: PluginManifest = serde_json::from_slice(&bytes).map_err(|e| {
            PluginError::Parse {
                path: manifest_path.to_path_buf(),
                source: e,
            }
        })?;
        if manifest.name.is_empty() {
            return Err(PluginError::MissingField {
                path: manifest_path.to_path_buf(),
                field: "name".into(),
            });
        }
        if manifest.version.is_empty() {
            return Err(PluginError::MissingField {
                path: manifest_path.to_path_buf(),
                field: "version".into(),
            });
        }
        Ok(PluginLocation {
            manifest,
            root: root.to_path_buf(),
        })
    }

    #[must_use] 
    pub fn get(&self, name: &str) -> Option<&PluginLocation> {
        self.plugins.get(name)
    }

    #[must_use] 
    pub fn names(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    #[must_use] 
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    /// v1 first run 시 `~/.myharness/plugins/` 자동 생성.
    /// `init_home_dir()` 와 함께 호출 (D-69 §5.12 통합).
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn ensure_plugins_dir(plugins_dir: &Path) -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(plugins_dir)?;
        Ok(plugins_dir.to_path_buf())
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// 수동 등록 (test/외부 주입 용).
    pub fn register(&mut self, loc: PluginLocation) {
        self.plugins.insert(loc.manifest.name.clone(), loc);
    }
}

/// Plugin 시스템 error. CONCEPT §5.7 의 spec 검증.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("io error at {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("parse error at {path:?}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("missing required field `{field}` at {path:?}")]
    MissingField { path: PathBuf, field: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_manifest(dir: &Path, name: &str, body: &str) -> PathBuf {
        let root = dir.join(name);
        std::fs::create_dir_all(&root).unwrap();
        let p = root.join("plugin.json");
        std::fs::write(&p, body).unwrap();
        root
    }

    #[test]
    fn manifest_minimal_parses() {
        let body = r#"{"name": "git-workflow", "version": "0.1.0"}"#;
        let m: PluginManifest = serde_json::from_str(body).unwrap();
        assert_eq!(m.name, "git-workflow");
        assert_eq!(m.version, "0.1.0");
        assert_eq!(m.entrypoints.commands, "commands");
        assert_eq!(m.entrypoints.agents, "agents");
        assert_eq!(m.entrypoints.skills, "skills");
        assert_eq!(m.entrypoints.hooks, "hooks");
        assert_eq!(m.id(), "git-workflow@0.1.0");
    }

    #[test]
    fn manifest_full_parses() {
        let body = r#"{
            "name": "frontend-design",
            "version": "1.2.3",
            "description": "Bold design",
            "author": "yklee",
            "entrypoints": {
                "commands": "cmds",
                "agents": "agents",
                "skills": "skls",
                "hooks": "events"
            },
            "requires": {"myharness": ">=0.1.0"}
        }"#;
        let m: PluginManifest = serde_json::from_str(body).unwrap();
        assert_eq!(m.entrypoints.commands, "cmds");
        assert_eq!(m.entrypoints.hooks, "events");
        assert_eq!(m.requires.get("myharness").map(String::as_str), Some(">=0.1.0"));
    }

    #[test]
    fn load_one_valid() {
        let dir = tempfile::tempdir().unwrap();
        let root = write_manifest(dir.path(), "p1", r#"{"name":"p1","version":"0.1.0"}"#);
        let loc = PluginRegistry::load_one(&root, &root.join("plugin.json")).unwrap();
        assert_eq!(loc.manifest.name, "p1");
        assert_eq!(loc.root, root);
    }

    #[test]
    fn load_one_missing_name_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = write_manifest(dir.path(), "bad", r#"{"version":"0.1.0"}"#);
        let err = PluginRegistry::load_one(&root, &root.join("plugin.json")).unwrap_err();
        assert!(matches!(err, PluginError::MissingField { field, .. } if field == "name"));
    }

    #[test]
    fn load_one_missing_version_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = write_manifest(dir.path(), "bad", r#"{"name":"bad"}"#);
        let err = PluginRegistry::load_one(&root, &root.join("plugin.json")).unwrap_err();
        assert!(matches!(err, PluginError::MissingField { field, .. } if field == "version"));
    }

    #[test]
    fn load_one_invalid_json_errors() {
        let dir = tempfile::tempdir().unwrap();
        let root = write_manifest(dir.path(), "bad", "not json");
        let err = PluginRegistry::load_one(&root, &root.join("plugin.json")).unwrap_err();
        assert!(matches!(err, PluginError::Parse { .. }));
    }

    #[test]
    fn discover_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let reg = PluginRegistry::discover(dir.path());
        assert!(reg.is_empty());
    }

    #[test]
    fn discover_finds_multiple_plugins() {
        let dir = tempfile::tempdir().unwrap();
        write_manifest(dir.path(), "a", r#"{"name":"a","version":"1.0.0"}"#);
        write_manifest(dir.path(), "b", r#"{"name":"b","version":"0.2.1"}"#);
        write_manifest(dir.path(), "c", r#"{"name":"c","version":"2.0.0"}"#);
        let reg = PluginRegistry::discover(dir.path());
        assert_eq!(reg.len(), 3);
        assert_eq!(reg.names(), vec!["a", "b", "c"]);
        assert!(reg.get("a").is_some());
        assert_eq!(reg.get("a").unwrap().manifest.version, "1.0.0");
    }

    #[test]
    fn discover_skips_invalid_plugin() {
        let dir = tempfile::tempdir().unwrap();
        write_manifest(dir.path(), "good", r#"{"name":"good","version":"0.1.0"}"#);
        write_manifest(dir.path(), "bad", r#"{"name":"bad"}"#);
        let reg = PluginRegistry::discover(dir.path());
        assert_eq!(reg.len(), 1);
        assert!(reg.get("good").is_some());
    }

    #[test]
    fn discover_skips_dir_without_manifest() {
        let dir = tempfile::tempdir().unwrap();
        write_manifest(dir.path(), "good", r#"{"name":"good","version":"0.1.0"}"#);
        std::fs::create_dir_all(dir.path().join("not-a-plugin")).unwrap();
        let reg = PluginRegistry::discover(dir.path());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn register_manual() {
        let dir = tempfile::tempdir().unwrap();
        let root = write_manifest(dir.path(), "x", r#"{"name":"x","version":"0.1.0"}"#);
        let mut reg = PluginRegistry::new();
        reg.register(PluginRegistry::load_one(&root, &root.join("plugin.json")).unwrap());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn ensure_plugins_dir_creates() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("plugins");
        assert!(!target.exists());
        let p = PluginRegistry::ensure_plugins_dir(&target).unwrap();
        assert!(p.is_dir());
    }
}
