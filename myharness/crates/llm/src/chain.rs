//! `ActiveProviderChain` — 우선순위 기반 활성 provider 목록.
//!
//! load/save: `~/.myharness/state/active-providers.toml`

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::auth_state::AuthState;
use crate::provider::ProviderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChainSource {
    Discover,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChainEntry {
    pub provider: ProviderId,
    pub priority: u32,
    pub auth_state: AuthState,
    pub default_model: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActiveProviderChain {
    pub version: u32,
    pub generated_at: DateTime<Utc>,
    pub source: ChainSource,
    pub entries: Vec<ChainEntry>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredProvider {
    pub provider: ProviderId,
    pub auth_state: AuthState,
    pub default_model: String,
}

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml decode: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("toml encode: {0}")]
    TomlEncode(#[from] toml::ser::Error),
}

impl ActiveProviderChain {
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: 1,
            generated_at: Utc::now(),
            source: ChainSource::Discover,
            entries: Vec::new(),
        }
    }

    /// discovered 목록을 우선순위별로 정렬해 chain 생성.
    /// 우선순위: `env_var=0..99`, keychain=100..199, manual=200..299, `local_detected=300..399`.
    #[must_use]
    pub fn from_discovered(discovered: Vec<DiscoveredProvider>) -> Self {
        let mut sorted = discovered;
        sorted.sort_by_key(|d| match d.auth_state {
            AuthState::EnvVar => 0,
            AuthState::Keychain => 100,
            AuthState::Manual => 200,
            AuthState::LocalDetected => 300,
            AuthState::Unset => 400,
            AuthState::Error => 500,
        });
        let entries: Vec<ChainEntry> = sorted
            .into_iter()
            .enumerate()
            .map(|(i, d)| ChainEntry {
                provider: d.provider,
                priority: i as u32,
                auth_state: d.auth_state,
                default_model: d.default_model,
            })
            .collect();
        Self {
            version: 1,
            generated_at: Utc::now(),
            source: ChainSource::Discover,
            entries,
        }
    }

    pub fn push(&mut self, entry: ChainEntry) {
        self.entries.push(entry);
    }

    pub fn remove(&mut self, provider: ProviderId) {
        self.entries.retain(|e| e.provider != provider);
    }

    pub fn iter(&self) -> impl Iterator<Item = &ChainEntry> {
        self.entries.iter()
    }

    #[must_use]
    pub fn primary(&self) -> Option<&ChainEntry> {
        self.entries.iter().min_by_key(|e| e.priority)
    }

    pub fn sort_by_priority(&mut self) {
        self.entries.sort_by_key(|e| e.priority);
    }

    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn load(path: &Path) -> Result<Self, ChainError> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let s = std::fs::read_to_string(path)?;
        let chain: ActiveProviderChain = toml::from_str(&s)?;
        Ok(chain)
    }

    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    pub fn save(&self, path: &Path) -> Result<(), ChainError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string(self)?;
        std::fs::write(path, s)?;
        Ok(())
    }
}

impl Default for ActiveProviderChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let c = ActiveProviderChain::new();
        assert_eq!(c.version, 1);
        assert!(c.entries.is_empty());
    }

    #[test]
    fn push_and_iter() {
        let mut c = ActiveProviderChain::new();
        c.push(ChainEntry {
            provider: ProviderId::Claude,
            priority: 0,
            auth_state: AuthState::EnvVar,
            default_model: "claude-sonnet-4-6".into(),
        });
        assert_eq!(c.iter().count(), 1);
    }

    #[test]
    fn from_discovered_priority_ordering() {
        let d = vec![
            DiscoveredProvider {
                provider: ProviderId::LocalLlm,
                auth_state: AuthState::LocalDetected,
                default_model: "llama3.1".into(),
            },
            DiscoveredProvider {
                provider: ProviderId::Claude,
                auth_state: AuthState::EnvVar,
                default_model: "claude-sonnet-4-6".into(),
            },
            DiscoveredProvider {
                provider: ProviderId::Codex,
                auth_state: AuthState::Keychain,
                default_model: "gpt-4o".into(),
            },
        ];
        let c = ActiveProviderChain::from_discovered(d);
        let providers: Vec<_> = c.iter().map(|e| e.provider).collect();
        assert_eq!(
            providers,
            vec![ProviderId::Claude, ProviderId::Codex, ProviderId::LocalLlm,]
        );
    }

    #[test]
    fn primary_returns_lowest_priority() {
        let mut c = ActiveProviderChain::new();
        c.push(ChainEntry {
            provider: ProviderId::Codex,
            priority: 1,
            auth_state: AuthState::Keychain,
            default_model: "gpt-4o".into(),
        });
        c.push(ChainEntry {
            provider: ProviderId::Claude,
            priority: 0,
            auth_state: AuthState::EnvVar,
            default_model: "claude-sonnet-4-6".into(),
        });
        assert_eq!(c.primary().unwrap().provider, ProviderId::Claude);
    }

    #[test]
    fn remove_drops_provider() {
        let mut c = ActiveProviderChain::new();
        c.push(ChainEntry {
            provider: ProviderId::Claude,
            priority: 0,
            auth_state: AuthState::EnvVar,
            default_model: "claude-sonnet-4-6".into(),
        });
        c.remove(ProviderId::Claude);
        assert!(c.iter().next().is_none());
    }

    #[test]
    fn version_field_set_to_one() {
        let c = ActiveProviderChain::new();
        assert_eq!(c.version, 1);
    }

    #[test]
    fn save_load_roundtrip_via_tempfile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("active-providers.toml");
        let mut c = ActiveProviderChain::new();
        c.push(ChainEntry {
            provider: ProviderId::Claude,
            priority: 0,
            auth_state: AuthState::EnvVar,
            default_model: "claude-sonnet-4-6".into(),
        });
        c.save(&path).unwrap();
        let back = ActiveProviderChain::load(&path).unwrap();
        assert_eq!(back.entries.len(), 1);
        assert_eq!(back.entries[0].provider, ProviderId::Claude);
    }

    #[test]
    fn load_missing_file_returns_empty_chain() {
        let c =
            ActiveProviderChain::load(std::path::Path::new("/nonexistent/active-providers.toml"))
                .unwrap();
        assert!(c.entries.is_empty());
    }

    #[test]
    fn sort_by_priority_works() {
        let mut c = ActiveProviderChain::new();
        c.push(ChainEntry {
            provider: ProviderId::LocalLlm,
            priority: 5,
            auth_state: AuthState::LocalDetected,
            default_model: "llama3.1".into(),
        });
        c.push(ChainEntry {
            provider: ProviderId::Claude,
            priority: 0,
            auth_state: AuthState::EnvVar,
            default_model: "claude-sonnet-4-6".into(),
        });
        c.sort_by_priority();
        assert_eq!(c.iter().next().unwrap().provider, ProviderId::Claude);
    }
}
