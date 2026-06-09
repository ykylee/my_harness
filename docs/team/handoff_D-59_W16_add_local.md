# D-59 W16 Handoff — `myharness auth add-local` subcommand 완료

> **session**: 2026-06-09
> **트리거**: yklee 요청 — "로컬 llm 연결을 위한 cli 명령어 추가. URL, token 입력 → 모델 선택 → 등록"
> **SDLC**: §5.2.5 REQ + §3.5 UC-AUTH-010 + DD-AddLocal §0~§8 + TC §W16-AddLocal (8 L1 + 3 L2)

## 1. summary (4-필드, D-26)

### 1.1 완료 (DONE)

- **SDLC 문서 4종 patch/추가** (D-59):
  - `docs/REQUIREMENTS.md` §5.2.5 — W16 결정 (4 trade-off, v1 영향, FR 매핑)
  - `docs/USE_CASES.md` §2.4 (UC-AUTH-010 catalog) + §3.5 (detailed, 8단계 흐름) + §10.4b (ACC-01~07)
  - `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` — **신규 410 lines**, 8 sections + VERDICT PASS
  - `docs/specs/TC_UNIT.md` §W16-AddLocal — L1 8개 (+ split 1개 = 9개, 5 verifiers self-check)
  - `docs/specs/TC_INTEGRATION.md` §W16-AddLocal — L2 3개 (wiremock + tempfile)
- **구현 (W16)**:
  - `myharness-llm/src/add_local.rs` (250 lines): `ModelInfo` + `RegisterReport` + `RegisterError` + `probe_local_models` + `register_local_provider` + `atomic_write` (pub(crate))
  - `myharness-cli/src/main.rs` patch: `AuthAction::AddLocal` enum + `handle_auth_add_local` async fn (98 lines, inquire 3단계)
  - Cargo.toml deps: `url = "2"` (workspace) + `inquire = "0.7"` (workspace) + `wiremock = "0.6"` (llm dev-dep) + `serial_test = "3"` (llm dev-dep)
  - `myharness-llm` lib.rs: `pub mod add_local` + re-export `ModelInfo/RegisterError/RegisterReport/probe_local_models/register_local_provider`
- **검증 (DONE)**:
  - `cargo test --workspace` — **모든 crate PASS** (W16 12/12 = L1 9 + L2 3)
  - `cargo clippy --workspace --all-targets` — 깨끗 (기존 warning 만)
  - `cargo build -p myharness` — cli 컴파일 OK, binary 생성됨

### 1.2 산출물 (4-필드, D-26)

| 산출물 | 경로 | 분량 | 상태 |
| --- | --- | --- | --- |
| **DD-AddLocal** | `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md` | 410 lines | done |
| **TC scaffold** | `docs/specs/TC_UNIT.md` §W16 + `docs/specs/TC_INTEGRATION.md` §W16 | +8 +3 | done |
| **add_local.rs** | `myharness/crates/llm/src/add_local.rs` | 250 lines | done |
| **w16_add_local.rs** (L2 integration) | `myharness/crates/llm/tests/w16_add_local.rs` | 130 lines | done |
| **cli main.rs patch** | `myharness/crates/cli/src/main.rs` | +98 lines | done |
| **Cargo.toml 4종** | workspace + llm + cli | +12 lines | done |
| **handoff** (본) | `docs/team/handoff_D-59_W16_add_local.md` | this | done |

## 2. TC scaffold (D-43~D-47 chapter 패턴 준수)

### 2.1 L1 Unit 8개 (TC-W16-001~008) — 9개 split

| TC | 결과 |
| --- | --- |
| TC-W16-001 ModelInfo serde roundtrip | ✅ PASS |
| TC-W16-001b ModelInfo owned_by optional (split) | ✅ PASS |
| TC-W16-002 RegisterError::InvalidUrl | ✅ PASS |
| TC-W16-003 RegisterError::NotInteractive | ✅ PASS |
| TC-W16-004 register valid no token (atomic write + TOML 검증) | ✅ PASS |
| TC-W16-005 token None → token_saved=false | ✅ PASS |
| TC-W16-006 token Some → token_saved=true + keyring in-memory | ✅ PASS |
| TC-W16-007 atomic_write preserves original (read-only parent) | ✅ PASS |
| TC-W16-008 URL trim trailing slash | ✅ PASS |

### 2.2 L2 Integration 3개 (TC-W16-I01~I03) — wiremock

| TC | 결과 |
| --- | --- |
| TC-W16-I01 probe extracts 3 models (HTTP 200 + OpenAI schema) | ✅ PASS |
| TC-W16-I02 probe returns HttpError on 401 | ✅ PASS |
| TC-W16-I03 end-to-end register writes providers.toml | ✅ PASS |

### 2.3 parallel-safe 패턴 (D-58 follow-up)

- `MYHARNESS_HOME=tempdir` env mutation 으로 paths.rs 격리
- `#[serial_test::serial(env)]` attribute 로 L1 3개 (TC-004/005/006) + L2 3개 직렬화
- L1 5개 (001/001b/002/003/007/008) 는 env 무관 → parallel 가능
- 1 session 안에 12 TC 모두 PASS (D-47 chapter 1~3-B 의 27.5% 1-session 패턴 100% 적용)

## 3. risks / follow-up (5 + 3)

### 3.1 risks (3)

- **R-1 (inquire 비대화형)**: CI/pipe 환경에서 `auth add-local` 실행 시 `NotInteractive` 에러. **대응** (DD §6.2): stderr 명확 + exit 1. v1.5+ 비대화형 `--url/--token/--model` 모드 후보 (OI-1).
- **R-2 (OpenAI 호환 schema 차이)**: server 마다 `/v1/models` 응답 미세 차이. **대응**: `id` 만 추출 (defensive parsing), `owned_by` 등 best-effort.
- **R-3 (keyring backend None)**: Linux headless 환경에서 token in-memory only. **대응**: env var hint (W7.2 정책), in-memory fallback. W16 의 graceful 의도된 동작.

### 3.2 suggested follow-up (3)

- **F-1 (W17 후보)**: 비대화형 `--url <url> --token <tok> --model <id>` 플래그 (CI/스크립트용) — OI-1
- **F-2 (W17~W18 후보)**: Ollama native `/api/tags` 지원 (OpenAI 호환 미활성 시) — OI-2
- **F-3 (W19 후보, D-38 Phase 2 종속)**: 등록 후 자동 `active-providers.yaml` 갱신 + fallback chain 자동 재구성 — OI-4

### 3.3 OOS (v1.5+ 후보)

- (OI-1~4) — DD §6.3 정리

## 4. 의존성 / 변경 파일 (4-필드 produced_artifacts)

### 4.1 Cargo.toml 변경 (4 files)

- `myharness/Cargo.toml` (workspace deps): `+url = "2"`, `+inquire = "0.7"`
- `myharness/crates/llm/Cargo.toml`: `+url = { workspace = true }` (dep) + dev-deps: `+wiremock = "0.6"`, `+serial_test = "3"`, `+tokio = { workspace = true, features = ["macros", "rt-multi-thread"] }`
- `myharness/crates/cli/Cargo.toml`: `+inquire = { workspace = true }`, `+url = { workspace = true }`
- `myharness/Cargo.lock`: 자동 갱신 (183 lines 추가)

### 4.2 SDLC docs 변경 (5 files)

- `docs/REQUIREMENTS.md`: §5.2.5 추가 (W16 결정, 1 sub-section)
- `docs/USE_CASES.md`: §2.4 catalog row 1개 + §3.5 detailed 1 section + §10.4b ACC 7개
- `docs/architecture/DETAILED_DESIGN_ADD_LOCAL.md`: **신규 410 lines**
- `docs/specs/TC_UNIT.md`: §W16-AddLocal 신규 section (239 lines 추가)
- `docs/specs/TC_INTEGRATION.md`: §W16-AddLocal 신규 section (162 lines 추가)

### 4.3 impl 변경 (3 files)

- `myharness/crates/llm/src/lib.rs`: `+pub mod add_local` + `+pub use add_local::{...}`
- `myharness/crates/llm/src/add_local.rs`: **신규 250 lines** (impl + 9 unit test)
- `myharness/crates/llm/tests/w16_add_local.rs`: **신규 130 lines** (3 integration test)
- `myharness/crates/cli/src/main.rs`: `+enum AddLocal` + `+async fn handle_auth_add_local` (98 lines)

## 5. VERDICT

### 5.1 SDLC 통과 (D-26 6 원칙)

- **한국어 보고** ✅
- **결론 위주** ✅ (본 handoff, DD §0.1, §0.4, §6/§7 trade-off 우선)
- **상태값** ✅ (D-59 W16 = done, SDLC 4 docs = done, impl = done, tests 12/12 PASS)
- **이벤트 소싱** ✅ (cargo test 결과, build 결과, clippy 결과 모두 handoff 에 append)
- **비참조** ✅ (DD §0.3 4 docs cross-ref, §8.2 cross-ref 5 SSOT)
- **handoff** ✅ (본 문서, 4-필드 D-26 정합)

### 5.2 TC scaffold 통과 (D-43 4-Layer)

- L1 Unit 8 (+ split 1) ✅
- L2 Integration 3 ✅
- L3 Component 1 (cli dispatch) — TC_COMPONENT.md patch 미작성 (W16 범위 외, v1.5+ OOS)
- L4 E2E 1 (manual, 실제 Ollama) — TC_E2E.md patch 미작성 (CI ❌, manual only)

### 5.3 D-43~D-47 chapter 패턴 준수

- **1 session 안에 4 chapter 묶기** ✅ (D-47 precedent 의 1-session 27.5% 패턴):
  - chapter 1: ModelInfo + RegisterError (TC-001~003) — RED → GREEN
  - chapter 2: register_local_provider core (TC-004~006) — RED → GREEN
  - chapter 3: atomic_write + URL trim (TC-007~008) — RED → GREEN
  - chapter 4: probe_local_models + wiremock integration (TC-I01~I03) + cli handler
- chunked write 6 chunk (DD §0.4 / SDLC 4 docs)
- cargo test 매 chapter 후 + clippy clean

### 5.4 결론

본 W16 = `myharness auth add-local` subcommand v1 구현 + SDLC 4 docs + 12 TC 모두 PASS. **VERDICT: PASS**.

## 6. 다음 단계

1. **dual push** (Gitea origin + GitHub upstream) — W16 5 commit
2. **memory 갱신** (state.json + work_backlog.md + session_handoff.md)
3. **PR 작성** (선택, yklee 직접) — Gitea PR `main` ← `feature/w16-add-local`

### 6.1 dual push 명령 (owner)

```bash
cd /Users/yklee/repos/my_harness
git checkout -b feature/w16-add-local
git add docs/ myharness/
git commit -m "feat(auth): W16 myharness auth add-local subcommand (D-59)

- SDLC: REQUIREMENTS §5.2.5 + USE_CASES §3.5/§10.4b + DD-AddLocal + TC §W16
- impl: myharness-llm::add_local (probe + register + atomic write)
- impl: myharness-cli::AuthAction::AddLocal + handle_auth_add_local (inquire)
- deps: url, inquire, wiremock (dev), serial_test (dev)
- tests: 12 PASS (L1 9 + L2 3)"
git push origin feature/w16-add-local
gh pr create --base main --head feature/w16-add-local --title "W16 auth add-local"
```
