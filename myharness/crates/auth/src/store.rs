//! OAuth token store. `~/.myharness/oauth/{provider}.toml` (chmod 600).
//!
//! disk 영구 저장 + KeyringAuthStore 의 in-memory cache 와 연동 (set 시 양쪽 update).

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::flow::OAuthToken;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml: {0}")]
    Toml(#[from] toml::ser::Error),
    #[error("toml de: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("no home directory")]
    NoHome,
    #[error("not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub provider: String,
    pub token: OAuthToken,
    pub stored_at: chrono::DateTime<chrono::Utc>,
}

pub struct TokenStore {
    pub base_dir: PathBuf,
}

impl TokenStore {
    pub fn new() -> Result<Self, StoreError> {
        let base_dir = if let Ok(p) = std::env::var("MYHARNESS_HOME") {
            PathBuf::from(p).join("oauth")
        } else {
            dirs::home_dir()
                .ok_or(StoreError::NoHome)?
                .join(".myharness")
                .join("oauth")
        };
        Ok(Self { base_dir })
    }

    pub fn with_base(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn path(&self, provider: &str) -> PathBuf {
        self.base_dir.join(format!("{provider}.toml"))
    }

    pub fn ensure_dir(&self) -> Result<(), StoreError> {
        std::fs::create_dir_all(&self.base_dir)?;
        Ok(())
    }

    pub fn save(&self, provider: &str, token: &OAuthToken) -> Result<(), StoreError> {
        self.ensure_dir()?;
        let stored = StoredToken {
            provider: provider.to_string(),
            token: token.clone(),
            stored_at: chrono::Utc::now(),
        };
        let s = toml::to_string(&stored)?;
        let path = self.path(provider);
        std::fs::write(&path, s)?;
        // chmod 600 (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = std::fs::metadata(&path)?.permissions();
            perm.set_mode(0o600);
            std::fs::set_permissions(&path, perm)?;
        }
        Ok(())
    }

    pub fn load(&self, provider: &str) -> Result<StoredToken, StoreError> {
        let path = self.path(provider);
        if !path.exists() {
            return Err(StoreError::NotFound);
        }
        let s = std::fs::read_to_string(&path)?;
        let stored: StoredToken = toml::from_str(&s)?;
        Ok(stored)
    }

    pub fn delete(&self, provider: &str) -> Result<(), StoreError> {
        let path = self.path(provider);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<String>, StoreError> {
        if !self.base_dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(stem) = name.strip_suffix(".toml") {
                out.push(stem.to_string());
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_token() -> OAuthToken {
        OAuthToken {
            access_token: "at".into(),
            refresh_token: Some("rt".into()),
            expires_at: Some(Utc::now() + chrono::Duration::seconds(3600)),
            scope: Some("read".into()),
            token_type: "Bearer".into(),
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        let token = make_token();
        store.save("minimax", &token).unwrap();
        let loaded = store.load("minimax").unwrap();
        assert_eq!(loaded.provider, "minimax");
        assert_eq!(loaded.token.access_token, "at");
    }

    #[test]
    fn load_missing_returns_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        let r = store.load("nonexistent");
        assert!(matches!(r, Err(StoreError::NotFound)));
    }

    #[test]
    fn delete_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        let token = make_token();
        store.save("minimax", &token).unwrap();
        store.delete("minimax").unwrap();
        assert!(matches!(store.load("minimax"), Err(StoreError::NotFound)));
    }

    #[test]
    fn delete_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        store.delete("nonexistent").unwrap();
    }

    #[test]
    fn list_returns_provider_ids() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        store.save("minimax", &make_token()).unwrap();
        store.save("openai", &make_token()).unwrap();
        let mut list = store.list().unwrap();
        list.sort();
        assert_eq!(list, vec!["minimax", "openai"]);
    }

    #[test]
    fn list_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        assert_eq!(store.list().unwrap(), Vec::<String>::new());
    }

    #[cfg(unix)]
    #[test]
    fn save_sets_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let store = TokenStore::with_base(dir.path().to_path_buf());
        store.save("minimax", &make_token()).unwrap();
        let meta = std::fs::metadata(dir.path().join("minimax.toml")).unwrap();
        let mode = meta.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}
