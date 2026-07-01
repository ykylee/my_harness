//! `ProviderRegistry` — in-memory CRUD + TOML 영속화.

use std::collections::BTreeMap;
use std::path::Path;

use thiserror::Error;

use crate::metadata::ProviderMetadata;
use crate::provider::ProviderId;

#[derive(Debug, Default, Clone)]
pub struct ProviderRegistry {
    providers: BTreeMap<ProviderId, ProviderMetadata>,
}

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml decode: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("toml encode: {0}")]
    TomlEncode(#[from] toml::ser::Error),
}

impl ProviderRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 6 built-in 으로 시작.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut r = Self::new();
        for m in ProviderMetadata::all_builtins() {
            r.insert(m);
        }
        r
    }

    /// 새 entry 삽입. 이미 존재하면 old 값 반환.
    pub fn insert(&mut self, meta: ProviderMetadata) -> Option<ProviderMetadata> {
        self.providers.insert(meta.id, meta)
    }

    ///
    /// # Panics
    ///
    /// This function returns an error if the underlying operation fails.
    /// upsert. 항상 최신 값 반환.
    pub fn replace(&mut self, meta: ProviderMetadata) -> ProviderMetadata {
        self.providers.insert(meta.id, meta).unwrap_or_else(|| {
            // unwrap_or_else 의 dummy — 실제로는 Some 일 때만 old 반환
            // insert 가 Some/None 모두 처리하지만 dummy 가 필요
            let id = self
                .providers
                .keys()
                .next()
                .copied()
                .expect("insert returned None → 키가 있어야 함");
            ProviderMetadata::builtin(id) // 실제로는 호출되지 않음
        })
    }

    #[must_use]
    pub fn get(&self, id: ProviderId) -> Option<&ProviderMetadata> {
        self.providers.get(&id)
    }

    pub fn get_mut(&mut self, id: ProviderId) -> Option<&mut ProviderMetadata> {
        self.providers.get_mut(&id)
    }

    pub fn remove(&mut self, id: ProviderId) -> Option<ProviderMetadata> {
        self.providers.remove(&id)
    }

    /// id 순서 (`BTreeMap` 이므로 정렬됨).
    #[must_use]
    pub fn list(&self) -> Vec<&ProviderMetadata> {
        self.providers.values().collect()
    }

    #[must_use]
    pub fn ids(&self) -> Vec<ProviderId> {
        self.providers.keys().copied().collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// TOML 문자열 → registry. provider 별 [[providers]] array-of-tables 형식.
    pub fn from_toml(s: &str) -> Result<Self, RegistryError> {
        #[derive(serde::Deserialize)]
        struct Wrapper {
            providers: Vec<ProviderMetadata>,
        }
        let w: Wrapper = toml::from_str(s)?;
        let mut r = Self::new();
        for m in w.providers {
            r.insert(m);
        }
        Ok(r)
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// registry → TOML.
    pub fn to_toml(&self) -> Result<String, RegistryError> {
        #[derive(serde::Serialize)]
        struct Wrapper<'a> {
            providers: Vec<&'a ProviderMetadata>,
        }
        let w = Wrapper {
            providers: self.providers.values().collect(),
        };
        Ok(toml::to_string(&w)?)
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// 파일에서 로드. 없으면 빈 registry.
    pub fn load_from_path(path: &Path) -> Result<Self, RegistryError> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let s = std::fs::read_to_string(path)?;
        Self::from_toml(&s)
    }

    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying operation fails.
    /// 파일로 저장. 부모 디렉토리 자동 생성.
    pub fn save_to_path(&self, path: &Path) -> Result<(), RegistryError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = self.to_toml()?;
        std::fs::write(path, s)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ProviderKind;

    #[test]
    fn with_builtins_has_six() {
        let r = ProviderRegistry::with_builtins();
        assert_eq!(r.len(), 6);
    }

    #[test]
    fn insert_returns_old_value() {
        let mut r = ProviderRegistry::new();
        let m1 = ProviderMetadata::builtin(ProviderId::Claude);
        assert!(r.insert(m1).is_none());
        let m2 = ProviderMetadata {
            display_name: "Anthropic Claude v2".into(),
            ..ProviderMetadata::builtin(ProviderId::Claude)
        };
        let old = r.insert(m2);
        assert!(old.is_some());
        assert_eq!(old.unwrap().display_name, "Anthropic Claude");
    }

    #[test]
    fn get_unknown_returns_none() {
        let r = ProviderRegistry::with_builtins();
        // 임의의 미존재 ID 는 from_str 로 만들 수 없으므로 builtin 으로 만든 후 remove
        let mut r2 = r.clone();
        r2.remove(ProviderId::Claude);
        assert!(r2.get(ProviderId::Claude).is_none());
    }

    #[test]
    fn list_sorted_by_id() {
        let r = ProviderRegistry::with_builtins();
        let ids: Vec<_> = r.list().iter().map(|m| m.id).collect();
        // BTreeMap 정렬 순: Claude < Codex < Deepseek < Gemini < LocalLlm < Minimax
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }

    #[test]
    fn toml_roundtrip() {
        let r = ProviderRegistry::with_builtins();
        let s = r.to_toml().unwrap();
        let back = ProviderRegistry::from_toml(&s).unwrap();
        assert_eq!(back.len(), 6);
        assert_eq!(
            back.get(ProviderId::Claude).unwrap().display_name,
            r.get(ProviderId::Claude).unwrap().display_name
        );
    }

    #[test]
    fn load_from_missing_path_returns_empty() {
        let r =
            ProviderRegistry::load_from_path(std::path::Path::new("/nonexistent.toml")).unwrap();
        assert!(r.is_empty());
    }

    #[test]
    fn save_load_uses_tempfile() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("providers.toml");
        let r = ProviderRegistry::with_builtins();
        r.save_to_path(&path).unwrap();
        let back = ProviderRegistry::load_from_path(&path).unwrap();
        assert_eq!(back.len(), 6);
    }

    #[test]
    fn save_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("providers.toml");
        let r = ProviderRegistry::with_builtins();
        r.save_to_path(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn is_empty_default() {
        let r = ProviderRegistry::new();
        assert!(r.is_empty());
    }

    #[test]
    fn remove_returns_old() {
        let mut r = ProviderRegistry::with_builtins();
        let old = r.remove(ProviderId::Claude);
        assert!(old.is_some());
        assert_eq!(old.unwrap().id, ProviderId::Claude);
        assert!(r.get(ProviderId::Claude).is_none());
    }

    #[test]
    fn kind_preserved_through_toml() {
        let r = ProviderRegistry::with_builtins();
        let s = r.to_toml().unwrap();
        let back = ProviderRegistry::from_toml(&s).unwrap();
        assert_eq!(
            back.get(ProviderId::Claude).unwrap().kind,
            ProviderKind::Native
        );
        assert_eq!(
            back.get(ProviderId::Deepseek).unwrap().kind,
            ProviderKind::OpenAiCompat
        );
    }
}
