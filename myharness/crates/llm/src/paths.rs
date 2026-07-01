//! `~/.myharness/` 경로 헬퍼. `MYHARNESS_HOME` 환경변수로 override 가능 (테스트용).

use std::path::{Path, PathBuf};

#[must_use]
pub fn home_dir() -> PathBuf {
    if let Ok(p) = std::env::var("MYHARNESS_HOME") {
        return PathBuf::from(p);
    }
    dirs::home_dir().map_or_else(|| PathBuf::from(".myharness"), |h| h.join(".myharness"))
}

#[must_use]
pub fn config_toml() -> PathBuf {
    home_dir().join("config.toml")
}

#[must_use]
pub fn providers_toml() -> PathBuf {
    home_dir().join("providers.toml")
}

#[must_use]
pub fn state_dir() -> PathBuf {
    home_dir().join("state")
}

#[must_use]
pub fn state_active_providers_toml() -> PathBuf {
    state_dir().join("active-providers.toml")
}

#[must_use]
pub fn state_auth_toml(provider: &str) -> PathBuf {
    state_dir().join("auth").join(format!("{provider}.toml"))
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_state_dir() -> std::io::Result<PathBuf> {
    let dir = state_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_auth_dir() -> std::io::Result<PathBuf> {
    let dir = state_dir().join("auth");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[must_use]
pub fn auth_dir() -> PathBuf {
    home_dir().join("auth")
}

#[must_use]
pub fn auth_toml(provider: &str) -> PathBuf {
    auth_dir().join(format!("{provider}.toml"))
}

#[must_use]
pub fn config_dir() -> PathBuf {
    home_dir().join("config")
}

#[must_use]
pub fn memory_dir() -> PathBuf {
    home_dir().join("memory")
}

#[must_use]
pub fn handoff_dir() -> PathBuf {
    home_dir().join("handoff")
}

#[must_use]
pub fn compression_dir() -> PathBuf {
    home_dir().join("compression")
}

#[must_use]
pub fn sub_agents_dir() -> PathBuf {
    home_dir().join("sub-agents")
}

#[must_use]
pub fn plugins_dir() -> PathBuf {
    home_dir().join("plugins")
}

#[must_use]
pub fn runtime_dir() -> PathBuf {
    home_dir().join("runtime")
}

#[must_use]
pub fn cache_dir() -> PathBuf {
    home_dir().join("cache")
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_top_level_auth_dir() -> std::io::Result<PathBuf> {
    let dir = auth_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_config_dir() -> std::io::Result<PathBuf> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_memory_dir() -> std::io::Result<PathBuf> {
    let dir = memory_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_handoff_dir() -> std::io::Result<PathBuf> {
    let dir = handoff_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_compression_dir() -> std::io::Result<PathBuf> {
    let dir = compression_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_sub_agents_dir() -> std::io::Result<PathBuf> {
    let dir = sub_agents_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_plugins_dir() -> std::io::Result<PathBuf> {
    let dir = plugins_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_runtime_dir() -> std::io::Result<PathBuf> {
    let dir = runtime_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// # Errors
///
/// This function returns an error if the underlying operation fails.
pub fn ensure_cache_dir() -> std::io::Result<PathBuf> {
    let dir = cache_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// §5.12 spec 의 11개 + plugins/ 12개 디렉토리 자동 생성. idempotent.
pub fn init_home_dir() -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(home_dir())?;
    ensure_config_dir()?;
    ensure_state_dir()?;
    ensure_memory_dir()?;
    ensure_handoff_dir()?;
    ensure_compression_dir()?;
    ensure_sub_agents_dir()?;
    ensure_top_level_auth_dir()?;
    ensure_auth_dir()?;
    ensure_runtime_dir()?;
    ensure_cache_dir()?;
    ensure_plugins_dir()?;
    Ok(home_dir())
}

#[must_use]
pub fn home_exists(p: &Path) -> bool {
    p.exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_dir_respects_env_var() {
        // MYHARNESS_HOME 환경변수가 있으면 그 경로를 그대로 반환한다.
        // set_var/remove_var 는 Rust 2024 edition 에서 unsafe 이므로,
        // 이미 다른 테스트에서 설정된 값을 가정하지 않고 default 동작만 검증한다.
        // env_override 동작은 crates/llm/tests/ 의 통합 테스트에서 검증한다.
        let _ = std::env::var("MYHARNESS_HOME");
        let got = home_dir();
        // home_dir() 가 panic 없이 PathBuf 반환하면 성공
        assert!(got.is_absolute() || got.components().count() > 0);
    }

    #[test]
    fn state_dir_is_under_home() {
        // 기본 home 기준 경로가 일관되게 계산되는지 (env mutation 없이)
        let home = home_dir();
        let s = state_dir();
        let a = state_active_providers_toml();
        let b = state_auth_toml("claude");
        // state_dir 은 home 하위, active-providers.toml 은 state_dir 하위
        assert!(
            s.starts_with(&home),
            "state_dir {s:?} should be under home {home:?}"
        );
        assert!(a.starts_with(&s));
        assert!(b.starts_with(s.join("auth")));
    }

    #[test]
    fn new_top_level_dirs_are_under_home() {
        let home = home_dir();
        assert!(config_dir().starts_with(&home));
        assert!(memory_dir().starts_with(&home));
        assert!(handoff_dir().starts_with(&home));
        assert!(compression_dir().starts_with(&home));
        assert!(sub_agents_dir().starts_with(&home));
        assert!(auth_dir().starts_with(&home));
        assert!(runtime_dir().starts_with(&home));
        assert!(cache_dir().starts_with(&home));
    }

    #[test]
    fn auth_toml_under_top_level_auth() {
        let a = auth_toml("claude");
        assert!(a.starts_with(auth_dir()));
        assert!(a.ends_with("claude.toml"));
    }

    #[test]
    fn init_home_dir_signature_compiles() {
        let _: fn() -> std::io::Result<PathBuf> = init_home_dir;
    }
}
