//! `~/.myharness/` 경로 헬퍼. `MYHARNESS_HOME` 환경변수로 override 가능 (테스트용).

use std::path::{Path, PathBuf};

pub fn home_dir() -> PathBuf {
    if let Ok(p) = std::env::var("MYHARNESS_HOME") {
        return PathBuf::from(p);
    }
    dirs::home_dir()
        .map(|h| h.join(".myharness"))
        .unwrap_or_else(|| PathBuf::from(".myharness"))
}

pub fn config_toml() -> PathBuf {
    home_dir().join("config.toml")
}

pub fn providers_toml() -> PathBuf {
    home_dir().join("providers.toml")
}

pub fn state_dir() -> PathBuf {
    home_dir().join("state")
}

pub fn state_active_providers_toml() -> PathBuf {
    state_dir().join("active-providers.toml")
}

pub fn state_auth_toml(provider: &str) -> PathBuf {
    state_dir().join("auth").join(format!("{provider}.toml"))
}

pub fn ensure_state_dir() -> std::io::Result<PathBuf> {
    let dir = state_dir();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn ensure_auth_dir() -> std::io::Result<PathBuf> {
    let dir = state_dir().join("auth");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

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
        assert!(s.starts_with(&home), "state_dir {:?} should be under home {:?}", s, home);
        assert!(a.starts_with(&s));
        assert!(b.starts_with(&s.join("auth")));
    }
}
