//! `auth add-local` subcommand 의 `register_local_provider` API (W16, D-59).
//!
//! 흐름:
//! 1. `probe_local_models` — `OpenAI` 호환 GET `{base_url}/models` 호출
//! 2. cli 측 inquire UI — 모델 선택
//! 3. `register_local_provider` — `KeyringAuthStore` set + `ProviderRegistry` 의 `LocalLlm` entry 갱신
//!
//! # 설계 의도 (DD-AddLocal §3)
//!
//! - `register_local_provider` 는 **순수 등록 함수** (inquire 미사용) → unit test 가능
//! - cli 측 (`myharness-cli`) 에서 inquire 통합 + `is_terminal()` 분기 + 한국어 출력
//! - `ProviderRegistry` 갱신 시 atomic write (tmp + rename) 로 손상 방지
//! - `KeyringAuthStore` backend=None (Linux libsecret 미설치) → in-memory fallback (W7.2 정책)

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

/// `OpenAI` 호환 `/v1/models` 응답 의 한 model entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    /// 서버가 제공한 추가 메타 (e.g., `owned_by`). 표시용, persistence 안 함.
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

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// 로컬 LLM 서버 probe (D-63 cascade: Ollama native → `OpenAI` 호환).
///
/// # Cascade 순서
/// 1. `GET {base}/api/tags` (Ollama native) — Ollama default 모드 커버
/// 2. 실패 시 `GET {base}/v1/models` (`OpenAI` 호환) — vLLM / LM Studio / llama.cpp / Ollama `OpenAI` compat
///
/// # 인자
/// - `base_url`: 사용자가 입력한 base URL (`http://localhost:11434` 또는 `http://localhost:11434/v1`)
/// - `token`: 옵션 Bearer token (Ollama 는 None, vLLM/LM Studio/llama.cpp 는 Some)
///
/// # Returns
/// - 성공 시 `Vec<ModelInfo>` (cascade 첫 번째 성공 endpoint)
/// - 둘 다 실패 시 cascade 의 마지막 에러 (`OpenAI` compat 4xx/5xx or connection refused)
pub async fn probe_local_models(
    base_url: &str,
    token: Option<&str>,
) -> Result<Vec<ModelInfo>, RegisterError> {
    let base = base_url.trim_end_matches('/');

    // Stage 1: Ollama native `/api/tags` 시도
    match probe_ollama_tags(base, token).await {
        Ok(Some(models)) if !models.is_empty() => return Ok(models),
        Ok(_) => {
            tracing::debug!("ollama /api/tags 비어있거나 응답 없음, /v1/models fallback");
        }
        Err(e) => {
            tracing::debug!("ollama /api/tags 실패: {e}, /v1/models fallback");
        }
    }

    // Stage 2: OpenAI 호환 `/v1/models` fallback
    probe_openai_models(base, token).await
}

/// Ollama native `/api/tags` probe. 성공 시 `Some(Vec<ModelInfo>)`, endpoint 없으면 `Ok(None)`.
async fn probe_ollama_tags(
    base: &str,
    token: Option<&str>,
) -> Result<Option<Vec<ModelInfo>>, RegisterError> {
    let url = format!("{base}/api/tags");
    match fetch_json_body(&url, token, Duration::from_secs(2)).await {
        Ok(body) => {
            let models = parse_ollama_tags(&body);
            Ok(Some(models))
        }
        Err(RegisterError::HttpError { status: 404, .. } | RegisterError::HttpError {
status: 405, .. }) => Ok(None),
        Err(e) => Err(e),
    }
}

/// `OpenAI` 호환 `/v1/models` probe. W16 의 원래 동작.
///
/// `base` 가 `/v1` 으로 끝나면 `format!("{base}/models")` (back-compat W16 caller),
/// 아니면 `format!("{base}/v1/models")` (caller 가 bare host 만 전달한 경우).
async fn probe_openai_models(
    base: &str,
    token: Option<&str>,
) -> Result<Vec<ModelInfo>, RegisterError> {
    let url = if base.ends_with("/v1") {
        format!("{base}/models")
    } else {
        format!("{base}/v1/models")
    };
    let body = fetch_json_body(&url, token, Duration::from_secs(3)).await?;
    let models = parse_openai_models(&body);
    if models.is_empty() {
        return Err(RegisterError::NoModels { url });
    }
    Ok(models)
}

/// 공통 HTTP fetch + status check + JSON parse helper (D-63).
async fn fetch_json_body(
    url: &str,
    token: Option<&str>,
    timeout: Duration,
) -> Result<serde_json::Value, RegisterError> {
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| RegisterError::HttpError {
            url: url.to_string(),
            status: 0,
            body: e.to_string(),
        })?;

    let mut req = client.get(url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }

    let resp = req.send().await.map_err(|e| {
        if e.is_connect() || e.is_timeout() {
            RegisterError::ConnectionRefused { url: url.to_string() }
        } else {
            RegisterError::HttpError {
                url: url.to_string(),
                status: 0,
                body: e.to_string(),
            }
        }
    })?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(RegisterError::HttpError {
            url: url.to_string(),
            status: status.as_u16(),
            body,
        });
    }

    resp.json().await.map_err(|e| RegisterError::HttpError {
        url: url.to_string(),
        status: status.as_u16(),
        body: e.to_string(),
    })
}

/// Ollama native `/api/tags` 응답 → `Vec<ModelInfo>`.
///
/// 응답 형식: `{"models": [{"name": "llama3.1:8b", "details": {"family": "llama"}, ...}]}`
pub(crate) fn parse_ollama_tags(body: &serde_json::Value) -> Vec<ModelInfo> {
    body.get("models")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| ModelInfo {
                            id: s.to_string(),
                            owned_by: m
                                .get("details")
                                .and_then(|d| d.get("family"))
                                .and_then(|f| f.as_str())
                                .map(String::from),
                        })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `OpenAI` 호환 `/v1/models` 응답 → `Vec<ModelInfo>`.
///
/// 응답 형식: `{"data": [{"id": "llama3.1:8b", "owned_by": "ollama"}, ...]}`
pub(crate) fn parse_openai_models(body: &serde_json::Value) -> Vec<ModelInfo> {
    body.get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    m.get("id").and_then(|id| id.as_str()).map(|s| ModelInfo {
                        id: s.to_string(),
                        owned_by: m
                            .get("owned_by")
                            .and_then(|o| o.as_str())
                            .map(String::from),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// 로컬 LLM 등록 — `~/.myharness/providers.toml` 의 `LocalLlm` entry 갱신 + (옵션) keyring set.
///
/// # 흐름
/// 1. `base_url` 의 URL parse 검증
/// 2. `token` 이 Some 이면 `KeyringAuthStore::set(LocalLlm, &token)` 호출
/// 3. `ProviderRegistry::load_from_path` (없으면 `with_builtins()` 시작) → `LocalLlm` entry 의
///    `base_url` + `default_model` + `available_models` 갱신 → `save_to_path`
/// 4. atomic write (tmp + rename) 로 providers.toml 손상 방지
///
/// # 인자
/// - `base_url`: `OpenAI` 호환 endpoint (e.g., `http://localhost:11434/v1`)
/// - `token`: API token (Ollama 는 None, vLLM/LM Studio/llama.cpp 는 Some)
/// - `selected_model`: 사용자가 선택한 모델
/// - `available_models`: probe 결과 전체 (`selected_model` 포함)
///
/// # 비고 (W19-1, D-62 follow-up)
/// - thin wrapper: `KeyringAuthStore::probe()` 1회 → [`register_local_provider_with_store`] 위임
/// - cli caller 가 store lifecycle 제어하려면 `with_store` 직접 호출 (test 등)
pub async fn register_local_provider(
    base_url: String,
    token: Option<String>,
    selected_model: ModelInfo,
    available_models: Vec<ModelInfo>,
) -> Result<RegisterReport, RegisterError> {
    let store = KeyringAuthStore::probe();
    register_local_provider_with_store(&base_url, token.as_deref(), &selected_model, &available_models, &store).await
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// `AuthStore` 트레이트 객체 주입 변형 (W19-1, D-62 follow-up).
///
/// ## WHY
/// - v1.5 `register_local_provider` 는 내부에서 `KeyringAuthStore::probe()` 1회 — 호출자(store A)
///   와 함수 내부(store B) 가 **별개 instance** 라 in-memory cache 공유 안 됨
/// - TC-W17-002 가 libsecret 부재 환경에서 store `A.get()` 시 `BackendUnavailable` 로 fail
/// - 해결: caller 가 만든 store 1개를 명시적으로 전달 → cache lifecycle 단일화
///
/// ## 인자
/// - `base_url`: `OpenAI` 호환 endpoint
/// - `token`: API token (optional)
/// - `selected_model`: 사용자가 선택한 모델
/// - `available_models`: probe 결과 전체
/// - `store`: `AuthStore` 트레이트 객체 (caller lifecycle)
pub async fn register_local_provider_with_store(
    base_url: &str,
    token: Option<&str>,
    selected_model: &ModelInfo,
    available_models: &[ModelInfo],
    store: &dyn AuthStore,
) -> Result<RegisterReport, RegisterError> {
    // 1. URL 검증
    url::Url::parse(base_url).map_err(|_| RegisterError::InvalidUrl(base_url.to_string()))?;

    // 2. token → store (W7.2 backend policy)
    let token_saved = if let Some(t) = token {
        store
            .set(ProviderId::LocalLlm, t)
            .await
            .map_err(|e| -> RegisterError {
                // keyring backend 에러 → RegistryError::Io 로 wrap (best-effort)
                RegistryError::Io(std::io::Error::other(e.to_string())).into()
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
        base_url: base_url.to_string(),
        default_model: selected_model.id.clone(),
        available_models: available_models.iter().map(|m| m.id.clone()).collect(),
        ..old
    };
    registry.replace(new);

    // 4. atomic write (with auto-backup, W18 R-4 대응)
    //    backup 은 silent — 실패 시 register 계속 진행 (R-4 graceful)
    let _ = backup_providers_toml(&path, 5);
    let toml_str = registry.to_toml()?;
    atomic_write(&path, &toml_str).map_err(RegistryError::from)?;

    Ok(RegisterReport {
        base_url: base_url.to_string(),
        model_id: selected_model.id.clone(),
        available_models: available_models.iter().map(|m| m.id.clone()).collect(),
        token_saved,
    })
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// W17 (v1.5 OI-1) — `register_local_provider` 의 비대화형 변형.
///
/// `probe_local_models` 호출 없이 (HTTP round-trip 생략) `register_local_provider` 를 직접 호출.
/// CI/스크립트 환경 (stdin/stdout non-tty) 에서 사용.
///
/// # 인자
/// - `base_url`: `OpenAI` 호환 endpoint (e.g., `http://localhost:11434/v1`)
/// - `token`: API token (optional)
/// - `model_id`: 사용자가 직접 지정한 모델 id (probe 없음 → 모델 검증 ❌, user 책임)
///
/// # 비고
/// - `available_models = vec![model_id]` 1개로 hardcode (probe 없으므로)
/// - `selected_model.owned_by = None` (probe 안 했으니 서버 메타 모름)
/// - URL 검증 + `KeyringAuthStore` set + `ProviderRegistry` 갱신 + atomic write = interactive 와 동일
/// - thin wrapper: [`register_local_provider_non_interactive_with_store`] 에 위임 (W19-1, D-62 follow-up)
pub async fn register_local_provider_non_interactive(
    base_url: String,
    token: Option<String>,
    model_id: String,
) -> Result<RegisterReport, RegisterError> {
    let store = KeyringAuthStore::probe();
    register_local_provider_non_interactive_with_store(&base_url, token.as_deref(), &model_id, &store).await
}

///
/// # Errors
///
/// This function returns an error if the underlying operation fails.
/// W19-1 (D-62 follow-up) — `register_local_provider_non_interactive` 의 store 주입 변형.
///
/// caller 가 store lifecycle 명시적 제어. test / 멀티 store 환경 / mock store 격리에 사용.
pub async fn register_local_provider_non_interactive_with_store(
    base_url: &str,
    token: Option<&str>,
    model_id: &str,
    store: &dyn AuthStore,
) -> Result<RegisterReport, RegisterError> {
    let selected = ModelInfo {
        id: model_id.to_string(),
        owned_by: None,
    };
    let available = vec![selected.clone()];
    register_local_provider_with_store(base_url, token, &selected, &available, store).await
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

/// W18 (v1.5 R-4 대응) + W21 (D-64 F-1+F-2) — providers.toml 덮어쓰기 직전 자동 backup.
///
/// # 동작
/// 1. `path` 가 존재하지 않으면 `Some(path)` 반환 (신규 write case, backup 불요)
/// 2. `path` 가 존재하면 content 읽기 → SHA-256 hash8 계산 → `path.backup.<ts>.<hash8>` 으로 copy
/// 3. **실패 시 warn 만, `register_local_provider` 는 계속 진행** (graceful, R-4 fail-soft)
/// 4. `max_backups` 개수 초과 시 가장 오래된 것부터 삭제 (default 5)
///
/// # W21 변경 (D-64 F-1+F-2)
/// - filename 에 content hash 8-char 추가: `<ts>.<sha256_8>`
/// - **WHY**: 동일 second 내 rapid register (R-5-A) 시 hash 가 달라 filename 충돌 방지
/// - 동시에 backup file 식별 (R-5-B) — content snapshot fingerprint 역할
/// - sha2 = 0.10 (auth crate 에 이미 존재, 새 dep 추가 불요)
///
/// # WHY silent fail
/// - 사용자가 R-4 사고에도 register 가 성공해야 LLM 사용 가능
/// - backup 실패는 `eprintln!` 로 stderr 에 알리고, 사용자가 수동 `cp` 가능
/// - 명시적 `--backup` flag ❌ (사용자 부담 + default = ON 이 안전)
///
/// # Returns
/// - `Some(backup_path)`: backup 성공 (또는 skip)
/// - `None`: backup 시도했으나 실패 (warn 만, register 계속)
#[must_use] 
pub fn backup_providers_toml(
    path: &Path,
    max_backups: usize,
) -> Option<std::path::PathBuf> {
    if !path.exists() {
        return Some(path.to_path_buf());
    }
    let content = match std::fs::read(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("⚠  providers.toml 읽기 실패 ({e}). backup 스킵.");
            return None;
        }
    };
    let hash8 = crate::hash8::content_hash_8(&content);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup = with_backup_suffix(path, ts, &hash8);
    if let Err(e) = std::fs::copy(path, &backup) {
        eprintln!(
            "⚠  providers.toml backup 실패 ({e}). register 는 계속 진행 — 수동으로 `cp {} {{}}.backup` 권고.",
            path.display()
        );
        return None;
    }
    if let Err(e) = cleanup_old_backups(path, max_backups) {
        eprintln!("⚠  backup cleanup 실패 ({e}). 수동 정리 권고.");
    }
    Some(backup)
}

/// `providers.toml` → `providers.toml.backup.<ts>.<hash8>` 경로 생성 (W21, D-64).
fn with_backup_suffix(path: &Path, ts: u64, hash8: &str) -> std::path::PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(format!(".backup.{ts}.{hash8}"));
    std::path::PathBuf::from(s)
}

/// backup 파일들 중 가장 오래된 것부터 삭제하여 `max_backups` 개 이하로 유지.
///
/// W21 (D-64) sort fix: filename string sort 는 `backup.999` 와 `backup.10000` 비교 시
/// '9' > '1' 으로 retention 거꾸로 동작. **numeric parse** 로 `unix_ts` 추출 후 정렬.
/// hash suffix 는 tie-breaker (동일 ts 시 stable sort) — 동일 ts 내 여러 backup 보존.
fn cleanup_old_backups(path: &Path, max_backups: usize) -> std::io::Result<()> {
    let Some(parent) = path.parent() else { return Ok(()) };
    let prefix = format!(
        "{}.backup.",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("providers.toml")
    );
    let mut backups: Vec<_> = std::fs::read_dir(parent)?
        .filter_map(std::result::Result::ok)
        .filter(|e| {
            e.file_name()
                .to_str()
                .is_some_and(|n| n.starts_with(&prefix))
        })
        .collect();
    // numeric sort on unix_ts (D-64 fix): "backup.999.a1b2" < "backup.10000.c3d4"
    backups.sort_by_key(|e| {
        e.file_name()
            .to_str()
            .and_then(|n| n.strip_prefix(&prefix))
            .and_then(|s| s.split('.').next())
            .and_then(|s| s.parse::<u64>().ok())
    });
    let excess = backups.len().saturating_sub(max_backups);
    for e in backups.iter().take(excess) {
        let _ = std::fs::remove_file(e.path());
    }
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
    async fn tc_w17_004_non_interactive_empty_model_id() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        // 빈 model_id 도 register 자체는 성공 (사용자 책임) — 단 available_models 에 빈 string 1개
        let report = register_local_provider_non_interactive(
            "http://localhost:11434/v1".into(),
            None,
            String::new(),
        )
        .await
        .unwrap();

        assert_eq!(report.model_id, "");
        assert_eq!(report.available_models, vec![String::new()]);
        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    // ── W18 (v1.5 R-4 대응) backup TC ─────────────────────────────────────────

    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w18_001_backup_created_before_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        // 1) 첫 register — providers.toml 신규 생성 (backup ❌)
        let r1 = register_local_provider(
            "http://localhost:11434/v1".into(),
            None,
            ModelInfo { id: "llama3.1".into(), owned_by: None },
            vec![ModelInfo { id: "llama3.1".into(), owned_by: None }],
        )
        .await
        .unwrap();
        assert_eq!(r1.model_id, "llama3.1");
        let toml_path = tmp.path().join("providers.toml");
        assert!(toml_path.exists());

        // backup 파일 없음 (신규 write)
        let backups_after_first: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(".backup."))
            .collect();
        assert_eq!(backups_after_first.len(), 0, "신규 write 시 backup ❌");

        // 2) 두 번째 register — backup 생성 확인
        //    (ts 가 동일할 수 있으니 살짝 sleep)
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let _r2 = register_local_provider(
            "http://localhost:11434/v1".into(),
            None,
            ModelInfo { id: "qwen2.5".into(), owned_by: None },
            vec![ModelInfo { id: "qwen2.5".into(), owned_by: None }],
        )
        .await
        .unwrap();

        let backups_after_second: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(".backup."))
            .collect();
        assert_eq!(backups_after_second.len(), 1, "두 번째 write 시 backup 1개");

        // backup 내용 = 첫 번째 write (llama3.1)
        let backup_content = std::fs::read_to_string(backups_after_second[0].path()).unwrap();
        assert!(backup_content.contains("llama3.1"));
        assert!(!backup_content.contains("qwen2.5"), "backup 은 덮어쓰기 전 상태");

        // 현재 providers.toml = 두 번째 write (qwen2.5)
        let current = std::fs::read_to_string(&toml_path).unwrap();
        assert!(current.contains("qwen2.5"));

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w18_002_backup_max_retention_5() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        // 7번 연속 register → backup 6개 생성 시도 → max_backups=5 로 oldest 1개 삭제
        for i in 0..7 {
            // ts 가 같으면 filename 동일 → sleep 으로 분기
            if i > 0 {
                std::thread::sleep(std::time::Duration::from_millis(1100));
            }
            let _ = register_local_provider(
                "http://localhost:11434/v1".into(),
                None,
                ModelInfo { id: format!("m{i}"), owned_by: None },
                vec![ModelInfo { id: format!("m{i}"), owned_by: None }],
            )
            .await
            .unwrap();
        }

        let backups: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(".backup."))
            .collect();
        // 6번 backup (1~6번 write 각각) + cleanup → 5개 이하
        // 첫 번째 write 는 backup ❌, 두 번째부터 backup
        // → 6번 backup 시도 → max 5개 유지 → 5개
        assert!(
            backups.len() <= 5,
            "max 5개 유지, 실제 {}개",
            backups.len()
        );

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    #[test]
    fn tc_w18_003_backup_helper_unit_no_file() {
        // providers.toml 없는 상태에서 backup → Some(path) (신규 write case)
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("providers.toml");
        assert!(!path.exists());

        let result = backup_providers_toml(&path, 5);
        assert!(result.is_some(), "no-file case 도 Some 반환 (신규 write)");
        assert!(!path.exists(), "신규 write case → 실제 파일 생성 ❌");
    }

    // ── W17 (v1.5 OI-1) 비대화형 TC ─────────────────────────────────────────

    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w17_001_non_interactive_no_token() {
        let tmp = tempfile::tempdir().unwrap();
        // SAFETY: serial_test::serial(env) 로 다른 env-mutating test 와 직렬화
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let report = register_local_provider_non_interactive(
            "http://localhost:11434/v1".into(),
            None,
            "llama3.1:8b".into(),
        )
        .await
        .unwrap();

        assert_eq!(report.base_url, "http://localhost:11434/v1");
        assert_eq!(report.model_id, "llama3.1:8b");
        // probe 안 했으므로 available_models 는 [model_id] 1개
        assert_eq!(report.available_models, vec!["llama3.1:8b".to_string()]);
        assert!(!report.token_saved);

        // providers.toml 검증
        let toml_path = tmp.path().join("providers.toml");
        assert!(toml_path.exists());
        let content = std::fs::read_to_string(&toml_path).unwrap();
        assert!(
            content.contains("base_url = \"http://localhost:11434/v1\"")
                || content.contains("base-url = \"http://localhost:11434/v1\""),
            "base_url not found in: {content}"
        );
        assert!(content.contains("llama3.1:8b"));

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w17_002_non_interactive_with_token() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        // W19-1 (D-62 follow-up): store 1개 만들어서 with_store 에 전달 — cache lifecycle 단일화
        let store = KeyringAuthStore::probe();
        let report = register_local_provider_non_interactive_with_store(
            "http://localhost:8000/v1",
            Some("ci-token-xyz"),
            "qwen2.5:14b",
            &store,
        )
        .await
        .unwrap();

        assert!(report.token_saved);
        assert_eq!(report.model_id, "qwen2.5:14b");

        // backend=None 환경 (CI Linux) → in-memory cache 검증
        // W19-1: 같은 store instance 라 set → get cache hit
        if store.backend() == crate::KeyringBackend::None {
            let cached = store.get(ProviderId::LocalLlm).await.unwrap();
            assert_eq!(cached.as_deref(), Some("ci-token-xyz"));
        } else {
            assert!(report.token_saved);
        }
        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    #[tokio::test]
    async fn tc_w17_003_non_interactive_invalid_url() {
        let err = register_local_provider_non_interactive(
            "not a url at all".into(),
            None,
            "x".into(),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, RegisterError::InvalidUrl(_)));
        assert!(err.to_string().contains("invalid URL"));
    }

    // ===== W19-1 (D-62 follow-up): store 주입 변형 검증 =====

    /// TC-W19-001: `register_local_provider_with_store` 에 caller 제공 store 전달 시
    /// set → get 이 같은 instance 라 cache hit 보장.
    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w19_001_with_store_same_instance_shares_cache() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let store = KeyringAuthStore::probe();
        let selected = ModelInfo {
            id: "llama3.1:8b".into(),
            owned_by: None,
        };
        let report = register_local_provider_with_store(
            "http://localhost:11434/v1",
            Some("w19-token-abc"),
            &selected,
            std::slice::from_ref(&selected),
            &store,
        )
        .await
        .unwrap();

        assert!(report.token_saved);
        let cached = store.get(ProviderId::LocalLlm).await.unwrap();
        assert_eq!(cached.as_deref(), Some("w19-token-abc"));

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    /// TC-W19-002: store 주입 변형 + `token = None` 일 때 `token_saved = false` + cache 무변경.
    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w19_002_with_store_none_token_no_save() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let store = KeyringAuthStore::probe();
        let selected = ModelInfo {
            id: "llama3.1:8b".into(),
            owned_by: None,
        };
        let report = register_local_provider_with_store(
            "http://localhost:11434/v1",
            None,
            &selected,
            std::slice::from_ref(&selected),
            &store,
        )
        .await
        .unwrap();

        assert!(!report.token_saved);
        // Ollama 처럼 token 없는 경우 — cache 에 아무것도 안 들어감
        // backend=None 환경: cache miss + backend unavailable → Err (의도된 동작)
        // backend 있는 환경: cache miss + 영구 저장소 미스 → Ok(None)
        if store.backend() == crate::KeyringBackend::None {
            let r = store.get(ProviderId::LocalLlm).await;
            assert!(r.is_err(), "token=None + backend=None → BackendUnavailable, got {r:?}");
        } else {
            let cached = store.get(ProviderId::LocalLlm).await.unwrap();
            assert_eq!(cached, None, "token=None 시 store cache 도 비어있어야");
        }

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    /// TC-W19-003: store A (caller) 와 별개 store B (wrapper 내부) 는 in-memory cache 공유 안 함.
    /// `register_local_provider` (`with_store` 미사용) 의 thin wrapper 동작 검증.
    /// W19-1 이전의 TC-W17-002 가 같은 이유로 fail 했었음. 이 test 가 회귀 방지.
    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w19_003_thin_wrapper_creates_separate_store() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let store_outer = KeyringAuthStore::probe();
        let report = register_local_provider(
            "http://localhost:11434/v1".into(),
            Some("w19-wrapper-token".into()),
            ModelInfo {
                id: "llama3.1:8b".into(),
                owned_by: None,
            },
            vec![ModelInfo {
                id: "llama3.1:8b".into(),
                owned_by: None,
            }],
        )
        .await
        .unwrap();

        assert!(report.token_saved);
        if store_outer.backend() == crate::KeyringBackend::None {
            let r = store_outer.get(ProviderId::LocalLlm).await;
            assert!(r.is_err(), "outer store 에는 token 이 없어야 (별개 instance), got {r:?}");
        }

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    // ── W21 (v1.5 D-64 F-1+F-2) backup filename + sort fix ─────────────────────

    /// TC-W21-001 — backup filename 이 `.<ts>.<hash8>` 형식 검증 (D-64)
    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w21_001_backup_filename_includes_hash8() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let toml_path = tmp.path().join("providers.toml");
        std::fs::write(&toml_path, "[placeholder]\n").unwrap();

        let backup = backup_providers_toml(&toml_path, 5).unwrap();

        let name = backup.file_name().unwrap().to_str().unwrap();
        let prefix = "providers.toml.backup.";
        assert!(name.starts_with(prefix), "expected prefix, got {name}");
        let suffix = &name[prefix.len()..];
        let parts: Vec<_> = suffix.split('.').collect();
        assert_eq!(parts.len(), 2, "expected <ts>.<hash8>, got {suffix}");
        assert!(parts[0].parse::<u64>().is_ok(), "ts not numeric: {}", parts[0]);
        assert_eq!(parts[1].len(), 8, "hash8 must be 8-char, got {}", parts[1]);
        assert!(parts[1].chars().all(|c| c.is_ascii_hexdigit()), "hash8 not hex: {}", parts[1]);

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    /// TC-W21-002 — `cleanup_old_backups` sort 정확도 (D-64 fix)
    ///
    /// **WHY**: 기존 string sort 는 `backup.999` < `backup.10000` 거꾸로 동작.
    /// 수동으로 8개 backup 파일 생성 후 `max_backups=5` → 가장 오래된 ts 3개 삭제.
    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w21_002_cleanup_old_backups_uses_numeric_sort() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        // 8개 backup 파일 (ts=100~107) — string sort 와 numeric sort 가 다른 순서를 보이는 구간
        for ts in 100u64..108 {
            let name = format!("providers.toml.backup.{ts}.aabbccdd");
            std::fs::write(tmp.path().join(&name), format!("content-{ts}")).unwrap();
        }

        let toml_path = tmp.path().join("providers.toml");
        std::fs::write(&toml_path, "current\n").unwrap();

        let _ = backup_providers_toml(&toml_path, 5);

        let mut remaining: Vec<u64> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("providers.toml.backup.")
            })
            .filter_map(|e| {
                e.file_name()
                    .to_str()
                    .and_then(|n| n.strip_prefix("providers.toml.backup."))
                    .and_then(|s| s.split('.').next())
                    .and_then(|s| s.parse::<u64>().ok())
            })
            .collect();
        remaining.sort_unstable();

        // 8 + 1(current backup) = 9 → max=5 → 가장 오래된 4개 삭제
        // 100, 101, 102, 103 삭제 → 104~107 + current backup (ts >= 108) 남아야
        assert_eq!(remaining.len(), 5, "remaining 5개여야, got {remaining:?}");
        assert_eq!(remaining[0], 104, "104 가 가장 오래된 남은 ts 여야");
        assert_eq!(remaining[3], 107, "107 가 두번째 신선한 ts 여야");
        assert!(remaining[4] >= 108, "마지막은 새 backup (ts={}, >= 108)", remaining[4]);

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    /// TC-W21-003 — sub-second 연속 backup (sleep 없이) → 모두 다른 filename 보존 (D-64 F-1+F-2 핵심)
    ///
    /// **WHY**: 기존 W18 의 `as_secs()` 만으로는 동일 second 내 rapid register 시 backup
    /// filename 충돌 → 앞 backup 덮어쓰기. W21 의 hash8 으로 동일 second 내 다른 content
    /// → 다른 hash → 다른 filename 보장.
    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w21_003_subsecond_backups_all_preserved_with_distinct_hashes() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let toml_path = tmp.path().join("providers.toml");
        std::fs::write(&toml_path, "v1\n").unwrap();
        let _ = backup_providers_toml(&toml_path, 10);
        std::fs::write(&toml_path, "v2\n").unwrap();
        let _ = backup_providers_toml(&toml_path, 10);
        std::fs::write(&toml_path, "v3\n").unwrap();
        let _ = backup_providers_toml(&toml_path, 10);

        let backups: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("providers.toml.backup.")
            })
            .collect();
        assert_eq!(backups.len(), 3, "3개 backup 모두 보존되어야, got {backups:?}");

        let mut contents: Vec<String> = backups
            .iter()
            .map(|e| std::fs::read_to_string(e.path()).unwrap())
            .collect();
        contents.sort();
        assert!(contents.iter().any(|c| c == "v1\n"));
        assert!(contents.iter().any(|c| c == "v2\n"));
        assert!(contents.iter().any(|c| c == "v3\n"));

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }

    /// TC-W21-004 — 동일 content 백업 시 hash 동일 (deterministic)
    #[tokio::test]
    #[serial_test::serial(env)]
    async fn tc_w21_004_same_content_same_hash_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("MYHARNESS_HOME", tmp.path()); }

        let toml_path = tmp.path().join("providers.toml");
        std::fs::write(&toml_path, "identical-content\n").unwrap();
        let b1 = backup_providers_toml(&toml_path, 10).unwrap();
        std::fs::write(&toml_path, "identical-content\n").unwrap();
        let b2 = backup_providers_toml(&toml_path, 10).unwrap();

        let n1 = b1.file_name().unwrap().to_str().unwrap();
        let n2 = b2.file_name().unwrap().to_str().unwrap();
        let h1 = n1.rsplit('.').next().unwrap();
        let h2 = n2.rsplit('.').next().unwrap();
        assert_eq!(h1, h2, "동일 content → 동일 hash8: {n1} vs {n2}");

        unsafe { std::env::remove_var("MYHARNESS_HOME"); }
    }
}

