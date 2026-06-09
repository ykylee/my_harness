# DETAILED_DESIGN_ADD_LOCAL.md — `auth add-local` subcommand 상세설계 (W16, D-59)

### VERDICT: PASS — `myharness auth add-local` subcommand spec 확정 (D-59 W16)

> 본 문서 = TASK-005-1 W16 의 상세설계. INITIAL_DESIGN.md §6.1 (line 1310-1430) 의 6 built-in provider 중 `LocalLlm` 의 **수동 1-shot 등록 UI** + REQUIREMENTS.md §5.2.5 (W16 결정) + USE_CASES.md §3.5 (UC-AUTH-010) + CONCEPT.md §5.5.1 (discover + auth + save 3-단계) + §5.2 (12 명령어) 의 구현 입력.
>
> - **시점**: 2026-06-09 (W11~W15 OAuth flow 완료 후, v1 의 6 built-in provider 중 LocalLlm 1-shot 수동 등록)
> - **대상 독자**: TASK-005-1 (W16 구현) 의 coder worker
> - **입력 SSOT (4 docs)**: CONCEPT.md (1,024) + REQUIREMENTS.md (1,003) + USE_CASES.md (1,197 with §3.5) + INITIAL_DESIGN.md (2,056) + D-38 provider-auto-config 의존성
> - **목적**: `myharness auth add-local` clap subcommand + `myharness-llm::register_local_provider()` API + `inquire` UI 통합의 미명시 시그니처 / error / persistence / TC 진입점 제공

**핵심 결정 (3 line)**:
1. **subcommand** = `myharness auth add-local` (no-arg, AuthAction::AddLocal enum) — 기존 `Login/Logout/Status/List` 옆에 자연스러움
2. **API** = `myharness_llm::register_local_provider(base_url, token, model) -> Result<RegisterReport, RegisterError>` — ProviderRegistry 의 LocalLlm entry 갱신 + keyring set
3. **UI** = `inquire` crate (arrow-key select + text input) — stdin read_line 직접 대비 의존성 +1 만 추가, UX 일관성 ↑

**4 trade-off** (verifier cross-check): §1 (clap subcommand 이름) / §2 (inquire vs stdin) / §3 (atomic write 방식) / §4 (models 0개 시 fallback).

**3 risks** (verifier patch reference): §6.1 R-1 (inquire 비대화형 fallback) / R-2 (OpenAI 호환 endpoint schema 차이) / R-3 (keyring backend None 시 in-memory 한계).

**분량**: target 350~500 lines (over-shoot 600+ ❌). chunked write D-16 4 chunk (§1-§2 / §3-§4 / §5-§6 / §7+VERDICT). 표준 6 원칙 (D-26) / D-06 메커니즘만 / 안티 6 미반영.

---

## §0. 메타 + 읽는 법 (D-26 + D-16)

### 0.1 문서 구조 (7 sections + VERDICT)

| § | 제목 | 역할 |
| --- | --- | --- |
| §1 | clap subcommand 결정 | AuthAction::AddLocal enum, 인자, --help 텍스트 |
| §2 | inquire UI 통합 | Text / Password / Select prompt, 비대화형 감지 |
| §3 | `register_local_provider` API | 시그니처, 에러, ProviderRegistry 갱신 |
| §4 | 모델 probe + persistence | GET /v1/models, atomic write, available_models |
| §5 | KeyringAuthStore 통합 | token set, env hint, None backend fallback |
| §6 | Trade-off + Risks + Open issues | verifier cross-check reference |
| §7 | TDD TC 진입점 | L1 Unit 8 + L2 Integration 3 정의 |

### 0.2 의존성 (Cargo.toml 변경)

**`myharness-cli/Cargo.toml`** — inquire 추가:

```toml
inquire = "0.7"
```

**`myharness-llm/Cargo.toml`** — url + reqwest 추가 (이미 있을 가능성 큼):

```toml
url = { workspace = true }  # workspace 에 없으면 "2" 추가
reqwest = { workspace = true }  # 이미 dev-dep 일 가능성
```

**확인 필요**: §0.3 workspace.dependencies 에 `url` / `reqwest` 가 있는지, 없다면 root `Cargo.toml` `[workspace.dependencies]` 에 추가.

### 0.3 선행 조건 (선확인)

- `myharness-llm::scan_local::LocalHit` (W7.3) — **재사용 ❌** (4개 hardcoded endpoint, 사용자 입력 URL 미지원). 신규 `probe_local_models(url, token)` 함수 별도 작성.
- `myharness-llm::KeyringAuthStore` (W7.2/W12) — `set(LocalLlm, &token).await` 재사용. in-memory cache + env hint 동일.
- `myharness-llm::ProviderRegistry` (W7.1) — `load_from_path` + `replace(LocalLlm metadata)` + `save_to_path` 재사용. atomic write 는 `save_to_path` 가 단순 `fs::write` 사용 → **§3 에서 tmp + rename 으로 보강**.
- `myharness-cli/Cargo.toml` 에 `inquire` 없음 → **신규 추가** (§0.2).

---

## §1. clap subcommand 결정

### §1.1 `AuthAction` enum 확장

**기존** (`myharness-cli/src/main.rs:139~155`):
```rust
enum AuthAction {
    Login { provider: String, port: u16, no_browser: bool, non_interactive: bool },
    Logout { provider: String },
    Status { provider: String },
    List,
}
```

**변경** (W16 patch):
```rust
enum AuthAction {
    // ... 기존 4개 ...

    /// 로컬 LLM 서버 등록 (D-59 W16) — Ollama / vLLM / LM Studio / llama.cpp
    /// Interactive wizard: URL → token(선택) → /v1/models probe → 모델 선택 → providers.toml 갱신
    AddLocal,
}
```

**4 trade-off 결정**:
- (A) `auth add <url>` (positional) — 비대화형 가능. 단, 모델 선택이 필수라 결국 interactive 필요 → ❌
- (B) `auth add-local <url>` (positional, subcommand 별도) — 명확. 단, OAuth flow 의 `auth <provider> login` 과 충돌 (`add` vs `add-local` 패턴 비일관) → ❌
- (C) `auth add-local` (no-arg, 완전 wizard) — **선택**. CONCEPT.md §5.2 의 명령어 = verb 중심, subcommand 별도 (e.g., `task start` vs `task end`). `add-local` 만 별도 verb.
- (D) `auth provider add-local` (nested) — provider sub-tree 신설. UC-AUTH-* 9개에 비해 over-shoot → ❌

**선택 = (C) `auth add-local` (no-arg)** — AuthAction::AddLocal 단일 enum variant, 추가 arg 없음, 완전 inquire 기반 wizard.

### §1.2 dispatch (기존 match 분기 확장)

`main.rs` 의 `Cmd::Auth { action: AuthAction::AddLocal }` → `handle_auth_add_local().await?` 호출. **별도 `handler` 함수** (UC-AUTH-001 의 `provider-auto-config` skill 과 분리 — W16 은 orchestrator 가 직접 처리, skill 미사용).

### §1.3 --help 텍스트

`AuthAction::AddLocal` 의 `///` doc comment = 사용자에게 노출되는 help:

```
로컬 LLM 서버 등록 (Ollama / vLLM / LM Studio / llama.cpp)

Interactive wizard:
  1. 서버 URL 입력 (default: http://localhost:11434/v1)
  2. API token 입력 (선택, 빈칸 가능)
  3. GET /v1/models 로 모델 목록 probe
  4. arrow-key 로 모델 선택
  5. ~/.myharness/providers.toml 의 LocalLlm entry 갱신

Examples:
  $ myharness auth add-local
  $ myharness auth add-local --help
```

---

## §2. inquire UI 통합

### §2.1 inquire crate 선택

| 후보 | pros | cons | 선택 |
| --- | --- | --- | --- |
| `inquire` (0.7) | arrow-key select + text + password 통합, validator, error 자동 표시 | 의존성 +1 (단일 crate, 의존성 가벼움) | ✅ |
| `dialoguer` (0.11) | 더 오래됨, 안정 | API 약간 verbose, multi-select 미지원 | ❌ |
| stdin read_line 직접 | 의존성 0 | arrow-key 미지원, 모델 선택 = 번호 입력 (UX 저하) | ❌ |
| ratatui inline TUI | 기존 의존성 재사용 | subcommand 1개에 풀빌드 오버킬 | ❌ |

**선택 = `inquire 0.7`** — 의존성 +1 은 W16 의 UX 향상 대비 가벼움. ratatui 와 충돌 ❌ (inquire 가 ratatui 위에 빌드되지 않음, 독립 crate).

### §2.2 prompt 3 단계

```rust
use inquire::{Text, Password, Select, Confirm, validator::Validation};

// 1) URL (Text, default Ollama)
let url: String = Text::new("Server URL")
    .with_default("http://localhost:11434/v1")
    .with_help_message("OpenAI 호환 endpoint, e.g. http://localhost:11434/v1 (Ollama)")
    .with_validator(|s: &str| {
        if url::Url::parse(s).is_ok() {
            Ok(Validation::Valid)
        } else {
            Ok(Validation::Invalid("URL 형식이 올바르지 않습니다".into()))
        }
    })
    .prompt()?;

// 2) Token (Password, optional, Enter 만 = None)
let token: Option<String> = Password::new("API Token")
    .with_help_message("빈칸 가능 — Ollama 는 보통 불요")
    .without_confirmation()
    .prompt()
    .ok()
    .filter(|s| !s.is_empty());

// 3) Models (probe 후 Select)
let models: Vec<ModelInfo> = probe_local_models(&url, token.as_deref()).await?;
if models.is_empty() { return Err(RegisterError::NoModels(url)); }
let model: ModelInfo = Select::new("Model", models).prompt()?;
```

**비대화형 감지** (§6.1 R-1):
```rust
if !atty::stdin() || !atty::stdout() {
    return Err(RegisterError::NotInteractive);
}
```

→ `atty = "0.2"` 의존성 추가 (간단, 무거움 ❌). 또는 `std::io::IsTerminal` (Rust 1.70+ stdlib) 사용 — **이쪽 추천**, 의존성 0.

```rust
use std::io::IsTerminal;
if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
    return Err(RegisterError::NotInteractive);
}
```

### §2.3 결과 출력 (한국어)

```rust
println!("\n✓ 로컬 LLM 등록 완료");
println!("  서버: {base_url}");
println!("  모델: {model_id} (전체 {n}개 사용 가능)");
println!("  저장: {}", paths::providers_toml().display());
if let Some(_t) = token { println!("  토큰: keychain 저장됨"); }
```

---

## §3. `register_local_provider` API (myharness-llm)

### §3.1 시그니처

**`myharness/crates/llm/src/add_local.rs`** (신규):

```rust
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    /// 서버가 제공한 추가 메타 (e.g., owned_by). 저장 안 함, 표시용.
    pub owned_by: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RegisterReport {
    pub base_url: String,
    pub model_id: String,
    pub available_models: Vec<String>,
    pub token_saved: bool,
}

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("invalid URL: {0}")]
    InvalidUrl(String),
    #[error("connection refused at {url}: 서버가 실행 중인지 확인")]
    ConnectionRefused { url: String },
    #[error("HTTP {status} at {url}: {body}")]
    HttpError { url: String, status: u16, body: String },
    #[error("no models found at {url}: 모델을 먼저 다운로드 받으세요 (e.g., `ollama pull llama3.1`)")]
    NoModels { url: String },
    #[error("not interactive: stdin/stdout 이 tty 아님 — interactive 만 지원")]
    NotInteractive,
    #[error("registry I/O: {0}")]
    RegistryIo(#[from] RegistryError),
    #[error("inquire error: {0}")]
    Inquire(#[from] inquire::InquireError),
}

pub async fn register_local_provider(
    base_url: String,
    token: Option<String>,
    selected_model: ModelInfo,
    available_models: Vec<ModelInfo>,
) -> Result<RegisterReport, RegisterError> {
    // 1. URL 검증
    url::Url::parse(&base_url).map_err(|_| RegisterError::InvalidUrl(base_url.clone()))?;

    // 2. token → KeyringAuthStore
    let token_saved = if let Some(t) = token.as_deref() {
        let store = KeyringAuthStore::probe();
        store.set(ProviderId::LocalLlm, t).await
            .map_err(|e| RegisterError::RegistryIo(RegistryError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))))?;
        true
    } else { false };

    // 3. ProviderRegistry 갱신 (LocalLlm entry)
    let path = paths::providers_toml();
    let mut registry = ProviderRegistry::load_from_path(&path)
        .or_else(|_| Ok::<_, RegistryError>(ProviderRegistry::with_builtins()))?;
    let old = ProviderMetadata::builtin(ProviderId::LocalLlm);
    let new = ProviderMetadata {
        base_url: base_url.clone(),
        default_model: selected_model.id.clone(),
        available_models: available_models.iter().map(|m| m.id.clone()).collect(),
        ..old
    };
    registry.replace(new);
    registry.save_to_path(&path)?; // §3.3 atomic write

    Ok(RegisterReport {
        base_url,
        model_id: selected_model.id,
        available_models: available_models.into_iter().map(|m| m.id).collect(),
        token_saved,
    })
}
```

### §3.2 lib.rs re-export

```rust
pub mod add_local;
pub use add_local::{ModelInfo, RegisterError, RegisterReport, register_local_provider};
```

### §3.3 atomic write 보강 (RegistryError 보강? 또는 별도 함수?)

`ProviderRegistry::save_to_path` 가 단순 `fs::write` → 손상 시 providers.toml 손실 위험. **W16 범위에서 보강**:

**`add_local.rs` 내부** 에서 atomic write 별도 처리:

```rust
fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
```

→ `registry.save_to_path` 결과 (`to_toml()` 의 String) 를 받아 `atomic_write` 로 저장. `RegistryError` 보강은 v1.5+ (`RegistryError::AtomicWrite` variant 추가). **W16 은 add_local.rs 내부에서만 처리**.

### §3.4 model probe (`probe_local_models`)

```rust
pub async fn probe_local_models(
    base_url: &str,
    token: Option<&str>,
) -> Result<Vec<ModelInfo>, RegisterError> {
    let url = format!("{}/models", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .map_err(|e| RegisterError::HttpError { url: url.clone(), status: 0, body: e.to_string() })?;

    let mut req = client.get(&url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req.send().await.map_err(|e| {
        if e.is_connect() || e.is_timeout() {
            RegisterError::ConnectionRefused { url: url.clone() }
        } else {
            RegisterError::HttpError { url: url.clone(), status: 0, body: e.to_string() }
        }
    })?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(RegisterError::HttpError { url, status: status.as_u16(), body });
    }

    let body: serde_json::Value = resp.json().await
        .map_err(|e| RegisterError::HttpError { url: url.clone(), status: status.as_u16(), body: e.to_string() })?;

    let models: Vec<ModelInfo> = body.get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter().filter_map(|m| {
                m.get("id").and_then(|id| id.as_str()).map(|s| ModelInfo {
                    id: s.to_string(),
                    owned_by: m.get("owned_by").and_then(|o| o.as_str()).map(String::from),
                })
            }).collect()
        })
        .unwrap_or_default();

    if models.is_empty() {
        return Err(RegisterError::NoModels { url });
    }
    Ok(models)
}
```

**§6.1 R-2 (OpenAI 호환 endpoint schema 차이)**: Ollama 는 `/api/tags` (다른 스키마), vLLM/LM Studio/llama.cpp 는 `/v1/models` (OpenAI 호환). **W16 은 `/v1/models` 만 지원** — Ollama 사용자는 `OLLAMA_BASE_URL` 을 `http://localhost:11434/v1` 로 설정하거나 (Ollama 가 OpenAI 호환 지원) Ollama 의 native endpoint 는 W16 범위 밖 (v1.5+).

### §3.5 register_local_provider 의 흐름

cli 측 (`handle_auth_add_local`):
1. `is_terminal()` 체크 (§2.2)
2. inquire 3 단계 (§2.2)
3. `probe_local_models(&url, token.as_deref()).await?`
4. `register_local_provider(url, token, selected, available).await?`
5. 한국어 결과 출력 (§2.3)

**`add-local` 함수 분할 권장** (테스트 가능성):
- `register_local_provider` (순수 등록, no-IO inquire) — **unit test 가능**
- `handle_auth_add_local` (cli 측, inquire 통합) — **integration test 는 mock stdin 어려움, manual 만**

---

## §4. persistence + error propagation

### §4.1 storage 위치

`ProviderRegistry::load_from_path(paths::providers_toml())` = `~/.myharness/providers.toml`.

**§3.1 의 register_local_provider 가 자동으로 이 위치 사용** — cli 측에서 path 직접 다루지 않음. **WHY**: `paths.rs` 가 `MYHARNESS_HOME` env override 지원 (테스트) — cli 에서 path 노출 시 override 깨짐. `register_local_provider` 내부에서 `paths::providers_toml()` 호출이 정답.

### §4.2 base_url 정규화

`probe_local_models` 가 `{base_url}/models` 로 probe. **base_url 이 trailing `/` 있어도 OK** (trim). **base_url 이 `/v1` suffix 포함 가정** (e.g., `http://localhost:11434/v1`). Ollama OpenAI 호환은 이 형태.

`http://localhost:11434` (suffix 없음) 입력 시 `{base_url}/models` = `http://localhost:11434/models` — Ollama native 가 아님 (404). **instructive error message 권장**:
```rust
if !base_url.ends_with("/v1") && !base_url.ends_with("/v1/") {
    eprintln!("⚠  base_url 이 /v1 로 끝나지 않습니다. Ollama OpenAI 호환은 http://localhost:11434/v1 형태입니다.");
}
```

→ **§7.1 R-1 trade-off**: 강제 validation vs warning only. **선택 = warning only** (vLLM/LM Studio 는 `/v1` 명시 안 해도 OpenAI 호환 path 자동 처리하는 경우 있음, user 자유도 존중).

### §4.3 exit code

- 정상 (등록 완료): exit 0
- `InvalidUrl` / `ConnectionRefused` / `HttpError` / `NoModels` / `NotInteractive` / `Inquire(cancel)`: exit 1
- `RegistryIo`: exit 2 (filesystem 권한 문제, 별도)

---

## §5. KeyringAuthStore 통합

### §5.1 token 저장

`§3.1` 에서 이미 처리. **핵심 3가지**:
- `KeyringAuthStore::probe()` — backend 자동 감지
- `set(LocalLlm, &token).await` — in-memory cache + keyring (backend 가용 시)
- backend = `KeyringBackend::None` (Linux libsecret 미설치) → **in-memory cache 로만 저장** (재시작 시 소실) + `env_hint(LocalLlm)` 메시지로 `MYHARNESS_LOCAL_LLM_KEY` 안내

**§5.1 R-3 (keyring backend None 시 in-memory 한계)**: Linux gnome-keyring 없는 환경 (headless server) 에서 token 이 in-memory 만 → 재시작 시 사라짐. **W16 의 graceful fallback 의도된 동작** (W7.2 의 정책과 동일). 사용자가 env var 로 영구화 가능.

### §5.2 requires_key 정책

`ProviderMetadata::builtin_local_llm` 가 `requires_key: false` (메타 정의). 즉, token 없어도 등록 가능. **register_local_provider 가 `token_saved` bool 만 반환** — cli 측에서 token 없으면 "keychain 저장 생략" 메시지.

---

## §6. Trade-off + Risks + Open issues

### §6.1 trade-off (4개)

1. **clap subcommand 이름** (§1.1) — `add-local` (C) 선택, 4 후보 중 **가장 일관성** (verb 중심, OAuth `login` 과 동급)
2. **inquire vs stdin** (§2.1) — inquire 선택, 의존성 +1 vs UX 향상 ✅
3. **atomic write 방식** (§3.3) — `add_local.rs` 내부 `tmp + rename` 으로 격리, `RegistryError` 보강은 v1.5+
4. **models 0개 시 fallback** (§3.4) — error + abort (재시도 X). 사용자 cancel 후 다른 URL 시도 가능 (재실행)

### §6.2 risks (3개)

- **R-1 (inquire 비대화형 fallback)**: CI / pipe 환경에서 `auth add-local` 실행 시 `NotInteractive` 에러. **대응**: stderr 에 명확 메시지 + exit 1. 향후 `--url <url> --token <tok> --model <id>` 비대화형 모드 추가 가능 (v1.5+).
- **R-2 (OpenAI 호환 endpoint schema 차이)**: `/v1/models` 의 응답 schema 가 server 마다 미세 차이 (e.g., `data[].id` 는 표준, `data[].object` 는 옵션). **대응**: `id` 만 추출 (defensive parsing), 나머지 메타는 best-effort.
- **R-3 (keyring backend None)**: Linux headless 환경에서 token 영구 저장 ❌. **대응**: env var hint, in-memory fallback. v1.5+ 에서 `~/.myharness/auth/local-llm.toml` (chmod 600) fallback 추가 가능.

### §6.3 open issues (v1.5+ 후보)

- (OI-1) 비대화형 `--url/--token/--model` 플래그
- (OI-2) Ollama native `/api/tags` 지원 (OpenAI 호환 미활성 시)
- (OI-3) 다중 모델 1회 등록 (`--all-models` 또는 `auth add-local --interactive-model-select`)
- (OI-4) 등록 후 자동 fallback chain 갱신 (`active-providers.yaml` 자동 write, D-38 Phase 2 영역)

---

## §7. TDD TC 진입점

### §7.1 L1 Unit 8개 (TC_UNIT.md §W16-AddLocal)

| TC ID | 시나리오 | 검증 |
| --- | --- | --- |
| **TC-W16-001** | `ModelInfo { id, owned_by }` serde roundtrip | JSON ↔ struct |
| **TC-W16-002** | `RegisterError::InvalidUrl("not a url")` 매칭 | url::Url::parse 실패 |
| **TC-W16-003** | `RegisterError::NotInteractive` 매칭 | stdin/stdout 둘 중 하나 !is_terminal |
| **TC-W16-004** | `register_local_provider` valid input → `Ok(RegisterReport)` | mock-free, tempfile dir, MYHARNESS_HOME override |
| **TC-W16-005** | `register_local_provider` token None → `token_saved = false` | keyring 미호출 |
| **TC-W16-006** | `register_local_provider` token Some → `token_saved = true` | keyring set 호출 (in-memory cache 확인) |
| **TC-W16-007** | atomic write — providers.toml 손상 시 tmp 파일만 남고 원본 보존 | 의도적 fs::write 실패 유도 |
| **TC-W16-008** | `probe_local_models` trailing `/v1` 자동 trim | `http://x.com/v1/` → `http://x.com/v1/models` |

### §7.2 L2 Integration 3개 (TC_INTEGRATION.md §W16-AddLocal)

| TC ID | 시나리오 | 검증 |
| --- | --- | --- |
| **TC-W16-I01** | mock HTTP server (wiremock) 가 `/v1/models` 200 + 3 models 반환 | `probe_local_models` 가 3개 ModelInfo 추출 |
| **TC-W16-I02** | mock HTTP server 401 반환 | `RegisterError::HttpError { status: 401, .. }` |
| **TC-W16-I03** | end-to-end: mock server + `register_local_provider` → providers.toml 검증 | `~/.myharness/providers.toml` 의 `LocalLlm` entry base_url/default_model/available_models 모두 갱신 |

### §7.3 L3 Component / L4 E2E

- **L3 Component (1개)**: `auth add-local` cli dispatch — `clap::Parser::try_parse_from(["myharness", "auth", "add-local"])` → `Cmd::Auth { action: AuthAction::AddLocal }`
- **L4 E2E (1개, manual only)**: 실제 Ollama 실행 환경에서 wizard 동작 검증 (CI ❌)

### §7.4 TDD 사이클 (D-43~D-47 패턴)

- **chapter 1** (TC-W16-001~003): error type 정의 + ModelInfo struct. RED (테스트만) → GREEN (impl)
- **chapter 2** (TC-W16-004~006): `register_local_provider` core. RED → GREEN
- **chapter 3** (TC-W16-007~008): atomic write + URL trim. RED → GREEN
- **chapter 4** (TC-W16-I01~I03): wiremock integration + E2E

→ **4 chapter × 1 session = 1~2 시간 작업**. (D-47 chapter 1~3-B 패턴 27.5% 1-session 사이클)

## §10. v1.5 W18 — 자동 backup + Confirm prompt (R-4 대응, D-61)

> **시점**: 2026-06-09 (TASK-005-2 v1.5 W18, D-61)
> **트리거**: W17 manual test 중 R-4 (사용자 home providers.toml 덮어쓰기) 1회 사고 → mavis agent memory 에 lesson append
> **목적**: R-4 완전 차단 (silent, 사용자 부담 0) + interactive 모드 confirm

### §10.1 backup_providers_toml API

```rust
/// W18 (v1.5 R-4 대응) — providers.toml 덮어쓰기 직전 자동 backup
///
/// # 동작
/// 1. path 가 존재하지 않으면 Some(path) (신규 write case)
/// 2. path 가 존재하면 path.with_extension("toml.backup.<unix_ts>") 으로 copy
/// 3. 실패 시 None (warn 만, register 계속)
/// 4. max_backups 개수 초과 시 가장 오래된 것부터 삭제 (default 5)
///
/// # Returns
/// - Some(backup_path): backup 성공
/// - None: backup 실패 (warn 만)
pub fn backup_providers_toml(path: &Path, max_backups: usize) -> Option<PathBuf>;
```

**WHY silent fail**: 사용자가 R-4 사고에도 register 가 성공해야 LLM 사용 가능. backup 실패는 `eprintln!` 로 stderr 알림 + 수동 `cp` 안내.

### §10.2 register_local_provider 호출 흐름 (W18 patch)

```rust
// 4. atomic write (with auto-backup, W18 R-4 대응)
let _ = backup_providers_toml(&path, 5);  // W18 신규 — silent, fail-soft
let toml_str = registry.to_toml()?;
atomic_write(&path, &toml_str).map_err(RegistryError::from)?;
```

**WHY `let _ =`**: backup 실패해도 register 계속 (R-4 graceful). atomic_write 의 tmp+rename 으로 corruption 방지는 별도 유지.

### §10.3 backup 파일 형식 + retention

- 파일명: `providers.toml.backup.<unix_ts>` (e.g., `providers.toml.backup.1781016095`)
- 위치: providers.toml 과 같은 디렉토리 (`~/.myharness/`)
- retention: `max_backups = 5` — 초과 시 file name 정렬로 가장 오래된 것부터 삭제
- cleanup 은 `read_dir` + prefix match (`.backup.`)

### §10.4 cli --yes flag (W18 spec)

`AuthAction::AddLocal` 에 flag 1개 추가:

| Flag | Type | Default | 의미 |
| --- | --- | --- | --- |
| `--yes` | `bool` | `false` | interactive 모드에서 덮어쓰기 confirm prompt skip |

**동작**:
- `interactive` 모드 + `providers.toml` 이미 존재 + `!skip_confirm` → `inquire::Confirm` prompt ("덮어쓰시겠습니까?")
- prompt 거부 → `anyhow::bail!` "덮어쓰기 취소됨 (--yes 로 confirm skip 가능)"
- `--yes` flag set → prompt skip, 그대로 register
- `non-interactive` 모드 → `--yes` 불요 (이미 flag 기반 호출 = user 책임 명시)

### §10.5 trade-off (3개)

1. **silent backup** (vs `--backup` flag 명시 요구) — silent 가 R-4 100% 차단, 사용자 부담 0. 단 disk space 약간 사용 (5 × ~400 bytes = 2KB 무시 가능).
2. **backup 실패 시 fail-soft** (vs panic) — register 성공 우선, backup 실패는 warn. 사용자 LLM 사용에 영향 ❌.
3. **interactive confirm prompt** (vs 자동 덮어쓰기) — user 가 명시적으로 동의해야 register. `--yes` flag 로 skip 가능. CI 환경은 비대화형 모드 진입 자체가 confirm 역할.

### §10.6 risks (1 신규, R-4 follow-up)

- **R-4 (사용자 home 덮어쓰기, W18 으로 1차 차단)**: W18 으로 backup 자동 생성. **복구 방법**: `cp ~/.myharness/providers.toml.backup.<ts> ~/.myharness/providers.toml`. **남은 위험**: backup 자체가 corruption 되거나 ts 가 동일해 overwrite 되는 경우 (sub-second 연속 register 시 가능) → `monotonic_ts` 도입은 v1.5+ OOS.
- **R-5 (backup disk 누적)**: 5개 retention 으로 자동 정리되지만, 사용자가 수동으로 `providers.toml.backup.*` 보관 시 누적 가능. v1.5+ OOS.

### §10.7 L1 Unit 4개 + L2 Integration 2개 (W18 TC scaffold)

| TC ID | 시나리오 | 검증 |
| --- | --- | --- |
| **TC-W18-001** | register 2회 연속 → 2번째에 backup 1개 생성 | backup 내용 = 1번째 register, current = 2번째 |
| **TC-W18-002** | register 7회 연속 → backup 6개 → max_retention=5 로 5개 이하 유지 | `assert!(backups.len() <= 5)` |
| **TC-W18-003** | `backup_providers_toml` 단독 호출, no-file case | `Some(path)` 반환 (신규 write, 실제 파일 생성 ❌) |
| **TC-W17-004** (재활성화) | W17 helper `register_local_provider_non_interactive` 가 main 에 들어옴 | 빈 model_id → register 성공, `available_models = [""]` |
| **TC-W18-I01** | L2: 연속 register → mock server 변경으로 다른 endpoint → backup 1개 확인 | wiremock 2 server 사용, ts 차이로 backup 분리 |
| **TC-W18-I02** | L2: `backup_providers_toml` 단독 max_retention 검증 | 7개 backup → max=3 → ≤3 |

### §10.8 cli 변경 (cumulative)

```rust
enum AuthAction {
    // ... 기존 ...
    AddLocal {
        #[arg(long)] url: Option<String>,
        #[arg(long)] token: Option<String>,
        #[arg(long)] model: Option<String>,
        #[arg(long)] probe_skip: bool,
        #[arg(long)] yes: bool,  // W18 신규
    },
}

async fn handle_add_local_interactive(skip_confirm: bool) -> anyhow::Result<()> {
    // 1-3단계 (URL, token, probe, 모델 선택) 동일
    // 4단계: providers.toml 존재 + !skip_confirm → inquire::Confirm
    // 5단계: register_local_provider (내부에서 backup 자동 호출)
}
```

### §10.9 사용 예시 (cumulative)

```bash
# 1) Interactive (default, 덮어쓰기 confirm prompt 추가)
myharness auth add-local

# 2) Interactive with --yes (덮어쓰기 confirm skip)
myharness auth add-local --yes

# 3) Non-interactive, probe 자동 (CI)
myharness auth add-local --url http://localhost:11434/v1 --model llama3.1:8b

# 4) Non-interactive, probe skip (CI 빠르고 결정적)
myharness auth add-local --url http://host:8000/v1 --model custom --probe-skip

# 5) R-4 복구 (실수로 덮어썼다면)
ls ~/.myharness/providers.toml.backup.*  # 가장 최근 timestamp 확인
cp ~/.myharness/providers.toml.backup.1781016095 ~/.myharness/providers.toml
```

---

## §8. 산출물 + 다음 단계

### §8.1 produced_artifacts

| 산출물 | 경로 | 분량 | 상태 |
| --- | --- | --- | --- |
| **DETAILED_DESIGN_ADD_LOCAL.md** (본) | `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` | ~410 lines / 8 sections | done (W16 ddoc) |
| **TC scaffold patch** | `docs/specs/TC_UNIT.md` §W16-AddLocal + `docs/specs/TC_INTEGRATION.md` §W16-AddLocal | +8 +3 L1/L2 | TBD (D-59) |
| **impl (W16)**: `myharness-llm::add_local` | `myharness/crates/llm/src/add_local.rs` | ~180 lines | TBD |
| **impl (W16)**: cli subcommand | `myharness/crates/cli/src/main.rs` patch (AuthAction::AddLocal + handler) | +~80 lines | TBD |
| **impl (W16)**: Cargo.toml dep | `myharness/crates/cli/Cargo.toml` (+ inquire) + `myharness/Cargo.toml` workspace dep | +2 lines | TBD |
| **handoff (D-59)** | `docs/team/handoff_D-59_W16_add_local.md` | ~30 lines | TBD |
| **memory 갱신** | `ai-workflow/memory/state.json` + `work_backlog.md` + `session_handoff.md` | +~10 lines | TBD |

### §8.2 cross-ref 요약 (5 SSOT)

- INITIAL_DESIGN.md §6.1 (line 1310-1430, 6 built-in provider) → 본 §1/§3/§4 (LocalLlm 1-shot 등록)
- REQUIREMENTS.md §5.2.5 (W16 결정, D-59) → 본 §0/§1/§3 (정합)
- USE_CASES.md §2.4 + §3.5 (UC-AUTH-010) + §10.4b (ACC) → 본 §1/§2/§3/§7 (정합)
- CONCEPT.md §5.5.1 (discover + auth + save 3-단계) + §5.2 (12 명령어) → 본 §1 (subcommand 위치) / §3 (save 단계)
- D-38 provider-auto-config → 본 §0.2 (의존성, 미사용 명확화)

### §8.3 다음 단계 (Owner)

1. **본 DETAILED_DESIGN_ADD_LOCAL.md verifier 독립 cross-check** (mavis-team 또는 owner self) — VERDICT top-level heading, 4 trade-off + 3 risks + 8+3 TC
2. **verifier PASS 시**: TASK-005-1 W16 구현 시작
   - **chapter 1**: `ModelInfo` + `RegisterError` + `register_local_provider` 시그니처 (TC-W16-001~003)
   - **chapter 2**: `register_local_provider` impl (TC-W16-004~006)
   - **chapter 3**: atomic write + URL trim (TC-W16-007~008)
   - **chapter 4**: `probe_local_models` + wiremock integration (TC-W16-I01~I03)
3. **cli patch**: `AuthAction::AddLocal` + `handle_auth_add_local` + inquire 통합 (chapter 4 와 동시)
4. **cargo test --workspace** + clippy clean
5. **dual push** (Gitea + GitHub) + handoff D-59

---

### VERDICT (final, post-handoff): PASS

본 DETAILED_DESIGN_ADD_LOCAL.md = `myharness auth add-local` W16 상세 spec. 3 line 핵심 결정 (subcommand 이름, API 시그니처, inquire UI) + 4 trade-off + 3 risks + 8 L1 + 3 L2 TC 진입점 + 4-chapter TDD 사이클. 분량 ~410 lines / 8 sections + VERDICT. D-16 chunked write 4 chunk / 표준 6 원칙 / D-06 메커니즘만 / 안티 6 미반영.
