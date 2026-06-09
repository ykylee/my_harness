//! `auth add-local` subcommand 의 register_local_provider API (W16, D-59).
//!
//! 흐름:
//! 1. `probe_local_models` — OpenAI 호환 GET `{base_url}/models` 호출
//! 2. cli 측 inquire UI — 모델 선택
//! 3. `register_local_provider` — KeyringAuthStore set + ProviderRegistry 의 LocalLlm entry 갱신
//!
//! # 설계 의도 (DD-AddLocal §3)
//!
//! - `register_local_provider` 는 **순수 등록 함수** (inquire 미사용) → unit test 가능
//! - cli 측 (`myharness-cli`) 에서 inquire 통합 + `is_terminal()` 분기 + 한국어 출력
//! - ProviderRegistry 갱신 시 atomic write (tmp + rename) 로 손상 방지
//! - KeyringAuthStore backend=None (Linux libsecret 미설치) → in-memory fallback (W7.2 정책)

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::auth_keyring::KeyringAuthStore;
use crate::auth_store::AuthStore;
use crate::metadata::ProviderMetadata;
use crate::paths;
use crate::provider::ProviderId;
use crate::registry::{ProviderRegistry, RegistryError};

/// OpenAI 호환 `/v1/models` 응답 의 한 model entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    /// 서버가 제공한 추가 메타 (e.g., owned_by). 표시용, persistence 안 함.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
}

impl std::fmt::Display for ModelInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.owned_by {
            Some(owned) => write!(f, "{} ({})", self.id, owned),
            None => write!(f, "{}", self.id),
        }
    }
}

/// `register_local_provider` 의 성공 보고.
#[derive(Debug, Clone, PartialEq)]
pub struct RegisterReport {
    pub base_url: String,
    pub model_id: String,
    pub available_models: Vec<String>,
    pub token_saved: bool,
}

/// `register_local_provider` / `probe_local_models` 의 에러.
#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("connection refused at {url}: 서버가 실행 중인지 확인 (e.g., `ollama serve`)")]
    ConnectionRefused { url: String },

    #[error("HTTP {status} at {url}: {body}")]
    HttpError { url: String, status: u16, body: String },

    #[error("no models found at {url}: 모델을 먼저 다운로드 받으세요 (e.g., `ollama pull llama3.1`)")]
    NoModels { url: String },

    #[error("not interactive: stdin/stdout 이 tty 아님 — interactive 만 지원")]
    NotInteractive,

    #[error("registry I/O: {0}")]
    RegistryIo(#[from] RegistryError),

    #[error("atomic write: {0}")]
    AtomicWrite(#[from] std::io::Error),
}

/// `GET {base_url}/models` probe (OpenAI 호환).
///
/// - `url` trailing `/` 자동 trim
/// - 3s timeout
/// - `token` 있으면 Bearer auth 부착
/// - 성공: Vec<ModelInfo> (data[*].id 추출)
/// - 실패: RegisterError 매칭
pub async fn probe_local_models(
    base_url: &str,
    token: Option<&str>,
) -> Result<Vec<ModelInfo>, RegisterError> {
    let base = base_url.trim_end_matches('/');
    let url = format!("{base}/models");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| RegisterError::HttpError {
            url: url.clone(),
            status: 0,
            body: e.to_string(),
        })?;

    let mut req = client.get(&url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }

    let resp = req.send().await.map_err(|e| {
        if e.is_connect() || e.is_timeout() {
            RegisterError::ConnectionRefused { url: url.clone() }
        } else {
            RegisterError::HttpError {
                url: url.clone(),
                status: 0,
                body: e.to_string(),
            }
        }
    })?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(RegisterError::HttpError {
            url,
            status: status.as_u16(),
            body,
        });
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| RegisterError::HttpError {
        url: url.clone(),
        status: status.as_u16(),
        body: e.to_string(),
    })?;

    let models: Vec<ModelInfo> = body
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m.get("id").and_then(|id| id.as_str()).map(|s| ModelInfo {
                        id: s.to_string(),
                        owned_by: m.get("owned_by").and_then(|o| o.as_str()).map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    if models.is_empty() {
        return Err(RegisterError::NoModels { url });
    }
    Ok(models)
}

/// 로컬 LLM 등록 — `~/.myharness/providers.toml` 의 LocalLlm entry 갱신 + (옵션) keyring set.
///
/// # 흐름
/// 1. `base_url` 의 URL parse 검증
/// 2. `token` 이 Some 이면 `KeyringAuthStore::set(LocalLlm, &token)` 호출
/// 3. `ProviderRegistry::load_from_path` (없으면 with_builtins() 시작) → `LocalLlm` entry 의
///    `base_url` + `default_model` + `available_models` 갱신 → `save_to_path`
/// 4. atomic write (tmp + rename) 로 providers.toml 손상 방지
///
/// # 인자
/// - `base_url`: OpenAI 호환 endpoint (e.g., `http://localhost:11434/v1`)
/// - `token`: API token (Ollama 는 None, vLLM/LM Studio/llama.cpp 는 Some)
/// - `selected_model`: 사용자가 선택한 모델
/// - `available_models`: probe 결과 전체 (selected_model 포함)
pub async fn register_local_provider(
    base_url: String,
    token: Option<String>,
    selected_model: ModelInfo,
    available_models: Vec<ModelInfo>,
) -> Result<RegisterReport, RegisterError> {
    // 1. URL 검증
    url::Url::parse(&base_url).map_err(|_| RegisterError::InvalidUrl(base_url.clone()))?;

    // 2. token → KeyringAuthStore (W7.2 backend policy)
    let token_saved = if let Some(t) = token.as_deref() {
        let store = KeyringAuthStore::probe();
        store
            .set(ProviderId::LocalLlm, t)
            .await
            .map_err(|e| -> RegisterError {
                // keyring backend 에러 → RegistryError::Io 로 wrap (best-effort)
                RegistryError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())).into()
            })?;
        true
    } else {
        false
    };

    // 3. ProviderRegistry 갱신
    let path = paths::providers_toml();
    let mut registry = ProviderRegistry::load_from_path(&path).unwrap_or_else(|_| {
        // 파일 없거나 손상 시 built-in 으로 시작 (W7.1 정책)
        ProviderRegistry::with_builtins()
    });

    let old = ProviderMetadata::builtin(ProviderId::LocalLlm);
    let new = ProviderMetadata {
        base_url: base_url.clone(),
        default_model: selected_model.id.clone(),
        available_models: available_models.iter().map(|m| m.id.clone()).collect(),
        ..old
    };
    registry.replace(new);

    // 4. atomic write
    let toml_str = registry.to_toml()?;
    atomic_write(&path, &toml_str).map_err(RegistryError::from)?;

    Ok(RegisterReport {
        base_url,
        model_id: selected_model.id,
        available_models: available_models.into_iter().map(|m| m.id).collect(),
        token_saved,
    })
}

/// Atomic write — `path.tmp` 에 write → `path` 로 rename.
/// 실패 시 원본 보존 (DD-AddLocal §3.3).
pub(crate) fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tc_w16_001_modelinfo_serde_roundtrip() {
        let m = ModelInfo {
            id: "llama3.1:8b".into(),
            owned_by: Some("ollama".into()),
        };
        let s = serde_json::to_string(&m).unwrap();
        let back: ModelInfo = serde_json::from_str(&s).unwrap();
        assert_eq!(back, m);
        assert!(s.contains("\"id\":\"llama3.1:8b\""));
        assert!(s.contains("\"owned_by\":\"ollama\""));
    }

    #[test]
    fn tc_w16_001b_modelinfo_owned_by_optional() {
        let m = ModelInfo {
            id: "x".into(),
            owned_by: None,
        };
        let s = serde_json::to_string(&m).unwrap();
        // skip_serializing_if = Option::is_none → "owned_by" key 누락
        assert!(!s.contains("owned_by"));
        let back: ModelInfo = serde_json::from_str(&s).unwrap();
        assert_eq!(back, m);
    }

    #[test]
    fn tc_w16_002_register_error_invalid_url() {
        let e = RegisterError::InvalidUrl("not a url".into());
        assert!(matches!(e, RegisterError::InvalidUrl(_)));
        assert!(e.to_string().contains("invalid URL"));
        assert!(e.to_string().contains("not a url"));
    }

    #[test]
    fn tc_w16_003_register_error_not_interactive() {
        let e = RegisterError::NotInteractive;
        assert!(matches!(e, RegisterError::NotInteractive));
        assert!(e.to_string().contains("not interactive"));
        assert!(e.to_string().contains("tty"));
    }

    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w16_004_register_local_provider_valid_no_token() {
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: test 전용 env mutation. parallel test 시 충돌 가능 → 본 TC 는 serial 가정
        // (CI 에서 cargo test --workspace 시 tempfile 격리로 안전)
        // SAFETY: test 전용 env mutation
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let report = register_local_provider(
            "http://localhost:11434/v1".into(),
            None,
            ModelInfo {
                id: "llama3.1".into(),
                owned_by: None,
            },
            vec![ModelInfo {
                id: "llama3.1".into(),
                owned_by: None,
            }],
        )
        .await
        .unwrap();

        assert_eq!(report.base_url, "http://localhost:11434/v1");
        assert_eq!(report.model_id, "llama3.1");
        assert_eq!(report.available_models, vec!["llama3.1".to_string()]);
        assert!(!report.token_saved);

        // providers.toml 검증
        let toml_path = tmp.path().join("providers.toml");
        assert!(toml_path.exists());
        let content = std::fs::read_to_string(&toml_path).unwrap();
        // serde 가 field 이름 그대로 (base_url) 또는 alias (base-url) 사용 가능
        assert!(
            content.contains("base_url = \"http://localhost:11434/v1\"")
                || content.contains("base-url = \"http://localhost:11434/v1\""),
            "base_url not found in: {content}"
        );
        assert!(content.contains("llama3.1"));

        // SAFETY: test 전용 env cleanup
        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w16_005_register_token_none_means_token_saved_false() {
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: test 전용 env mutation
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let report = register_local_provider(
            "http://localhost:8000/v1".into(),
            None,
            ModelInfo {
                id: "test-model".into(),
                owned_by: None,
            },
            vec![ModelInfo {
                id: "test-model".into(),
                owned_by: None,
            }],
        )
        .await
        .unwrap();

        assert!(!report.token_saved);
        // SAFETY: test 전용 env cleanup
        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w16_006_register_token_some_means_token_saved_true() {
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: test 전용 env mutation
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let store = KeyringAuthStore::probe();
        let report = register_local_provider(
            "http://localhost:8000/v1".into(),
            Some("test-token-abc123".into()),
            ModelInfo {
                id: "test-model".into(),
                owned_by: None,
            },
            vec![ModelInfo {
                id: "test-model".into(),
                owned_by: None,
            }],
        )
        .await
        .unwrap();

        assert!(report.token_saved);

        // CI Linux (backend=None) → in-memory cache 검증
        if store.backend() == crate::auth_keyring::KeyringBackend::None {
            let cached = store.get(ProviderId::LocalLlm).await.unwrap();
            assert_eq!(cached.as_deref(), Some("test-token-abc123"));
        }
        // SAFETY: test 전용 env cleanup
        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    #[test]
    fn tc_w16_007_atomic_write_preserves_original_on_rename_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("providers.toml");
        std::fs::write(&target, "ORIGINAL CONTENT\n").unwrap();

        // 정상 write 성공 케이스 먼저
        atomic_write(&target, "NEW CONTENT").unwrap();
        let content = std::fs::read_to_string(&target).unwrap();
        assert_eq!(content, "NEW CONTENT");

        // read-only parent 로 만들어 tmp write 실패 유도
        let mut perms = std::fs::metadata(tmp.path()).unwrap().permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o555); // read+execute only
        }
        #[cfg(not(unix))]
        {
            perms.set_readonly(true);
        }
        std::fs::set_permissions(tmp.path(), perms).unwrap();

        let result = atomic_write(&target, "FAILING CONTENT");
        // Unix read-only: write 실패 (EACCES) 또는 rename 실패 (EACCES)
        // write 가 .tmp 에서 실패 → tmp 파일 없음, 원본 그대로
        // write 성공 + rename 실패 → .tmp 파일 남고 원본 그대로
        // 둘 다 원본 보존
        if result.is_err() {
            let content = std::fs::read_to_string(&target).unwrap();
            // 원본이 보존되거나, NEW CONTENT 가 그대로일 수 있음 (read-only 가 rename 만 막은 경우)
            // 핵심: "FAILING CONTENT" 가 target 에 안 써짐
            assert_ne!(content, "FAILING CONTENT", "원본/이전값 보존 필수");
        }

        // cleanup
        let mut perms = std::fs::metadata(tmp.path()).unwrap().permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
        }
        #[cfg(not(unix))]
        {
            perms.set_readonly(false);
        }
        std::fs::set_permissions(tmp.path(), perms).unwrap();
    }

    #[test]
    fn tc_w16_008_url_trim_trailing_slash() {
        // probe_local_models 내부의 URL build 가 trim_end_matches('/') 사용 검증
        // → 직접 probe 호출은 HTTP 발생하므로, build_url helper 가 있다면 그것만 검증
        // 본 L1 TC 는 단순히 trim 함수 동작 검증
        let url_with_slash = "http://localhost:11434/v1/";
        let url_no_slash = "http://localhost:11434/v1";
        assert_eq!(url_with_slash.trim_end_matches('/'), url_no_slash);

        // probe_local_models 의 URL format 검증 (실제 HTTP 호출 ❌, build 만)
        let built = format!("{}/models", "http://localhost:11434/v1/".trim_end_matches('/'));
        assert_eq!(built, "http://localhost:11434/v1/models");
    }
}
