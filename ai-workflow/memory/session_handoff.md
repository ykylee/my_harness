# Session Handoff

- Purpose: Compact restore context for the next AI agent session.
- Scope: current focus, task status, key changes, next actions, risks
- Audience: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- Status: active
- Updated: 2026-08-14 (**D-138 M3 done**. install.sh. 다음 = live MiniMax 또는 M4 승인)
- Updated: 2026-08-14 (이전: D-134 grok-build 14섹션. overlay/포크 당시 미결정 → D-135 로 해소)
- Related docs: [Project Profile](../../docs/PROJECT_PROFILE.md), [Work Backlog](./work_backlog.md), [State Cache](./state.json), [CONCEPT.md](../../docs/CONCEPT.md) (SSOT)

## Current Focus

- **v1 컨셉 Phase 종료 (2026-06-07)** — my_harness 의 SSOT (CONCEPT.md) 확립. 17 섹션 (12 + 5 신규: §5.10~§5.14). 4/5 결정 ✅ 완료, 1/5 (TASK-002) ⏸ 보류.
  - **TASK-005**: Rust 1안 (D-36) — ratatui + rig-core + rmcp + keyring + cargo-dist
  - **TASK-006**: ratatui + crossterm (D-36, TASK-005 종속)
  - **TASK-007**: headroom v1 = 3 알고리즘 (D-37). CCR + Kompress-base v1.5+ (D-46 W8.4 에서 CacheAligner/ContentRouter/SmartCrusher/CodeCompressor v1 구현)
  - **TASK-008**: provider-auto-config skill (D-38) — v1 simple 구현 (D-45)
  - **TASK-002**: ⏸ 보류 (yklee 인프라 정보 의존)
- **TASK-005-1 v1 MVP 8/8 waves + D-52 follow-up 6 작업 완료 (D-58)** — tools (W3~W6.5) + llm (W7) + context (W8) + compression (W9) + tui (W10) + core (W11) + MiniMax LLM API (W12, D-50) + OAuth 2.0 headless auth (W13, D-51) + W13.5 env override (D-52) + W13.6 mock e2e test (D-52) + **W14 Device Authorization Grant (D-53)** + **W14.4 `--no-browser` 3 모드 (D-54)** + **W14.5 polling output + expired_in ms 단위 (D-55)** + **W14.6 token 단위 변환 (D-56)** + **W15.a OAuth token 자동 resolve (D-57)** + **W15.b OAuth token 자동 refresh (D-58)**
- **TASK-005-1 v1 MVP 종료 선언 (D-59 follow-up, 2026-06-09 22:48)** — yklee 결정. **W16 add-local follow-up** 까지 main 머지 완료:
  - W16 = `myharness auth add-local` subcommand (Ollama/vLLM/LM Studio/llama.cpp URL+token+모델 선택 UI)
  - W16 follow-up = `resolve_llm_client()` 에 LocalLlm 분기 추가 (`MYHARNESS_USE_LOCAL_LLM=1` opt-in env)
  - 9 L1 + 3 L2 + 4 scenario TC = 16/16 PASS, 388+ tests workspace PASS, dual-push 완료
  - feature/w16-add-local 브랜치 local/origin/upstream 모두 삭제, main = 140acf9
  - 보존: `local/d-43-47-tc-scaffold` (이전 세션 TC scaffold 보존 브랜치) 그대로
- **다음 (선택)**: TASK-005-2 (v1.5) 진입 대기 — yklee 결정. v1.5 후보: Plugin 4-계층 + marketplace + auto memory + provider-auto-config skill 정식 + CCR/Kompress-base ONNX + 비대화형 add-local (`--url/--token/--model` flags, W16 OI-1)
- **W12 산출물 (D-50)**: librarian 조사 (api.minimax.io/v1, MiniMax-M3, OpenAI-호환 Bearer, tool_use 지원). `ProviderMetadata::builtin_minimax()` 갱신. `KeyringAuthStore` in-memory cache + env hint. cli default LLM = `MINIMAX_API_KEY` env 자동 detect → `OpenAiCompatProvider`
- **W13 산출물 (D-51)**: `myharness-auth` crate v1 (7 모듈: `pkce` RFC 7636 S256 + `flow` OAuth 2.0 Authorization Code with PKCE + `callback` loopback HTTP server 5min timeout + `browser` xdg-open/open/start + `store` `~/.myharness/oauth/{provider}.toml` chmod 600 + `provider` MiniMax/OpenAI/Google 3 provider + `manager` AuthManager login/refresh/status/logout). 38 tests
- **W13.5 산출물 (D-52)**: `OAUTH_PROVIDERS` static LazyLock 제거 → `oauth_providers()` 매번 새 instance. `MinimaxOAuth::from_env()` / `OpenAiOAuth::from_env()` / `GoogleOAuth::from_env()`. `MYHARNESS_OAUTH_CLIENT_ID_{MINIMAX,OPENAI,GOOGLE}` env override
- **W13.6 산출물 (D-52)**: local mock HTTP server + `MockProvider` + `auth_manager_end_to_end_with_mock_server` test. **real network 없이** build_authorize_url → reqwest get → 302 redirect → exchange_code → store save 전체 검증. 39 auth tests pass
- **W14 산출물 (D-53)**: MiniMax 의 **Device Authorization Grant** 패턴 발견 (Authorization Code + PKCE 가 404). 새 `MinimaxDeviceOAuth` provider + `DeviceCodeProvider` trait + `device_flow.rs` (request_code / poll_token / poll_until_success). OpenClaw/Hermes 공통 `client_id = 78257093-7e40-4613-99e0-527b14b39113`. scope = `group_id profile model.completion`. CN: `MYHARNESS_MINIMAX_CN=1` 또는 `MINIMAX_OAUTH_BASE_URL` env override. 기존 `MinimaxOAuth` (Authorization Code) deprecated 표시만 유지
- **W14.4 산출물 (D-54)**: `--no-browser` 의미 수정. `AuthManager::login_minimax_device` signature `(auto_open_browser: bool, non_interactive: bool)`. **3 모드**: (1) default = URL 출력 + browser 자동 open + polling + save, (2) `--no-browser` = URL 출력 + polling + save (user 직접 paste), (3) `--non-interactive` = URL 출력만 + 즉시 종료 (CI). `login_minimax_device_no_browser_polling_saves` mock e2e test 추가. 42 auth tests, 367 workspace tests
- **W14.5 산출물 (D-55)**: `login_minimax_device` polling 진입 전 URL + user_code + expires_in 항상 `tracing::info!` 출력 (browser 자동 open 무관). `poll_until_success` OpenClaw 와 동일 1.5x backoff + cap 10s + floor 1s. `expired_in` ms 단위 unix timestamp 처리 (D-55)
- **W14.6 산출물 (D-56)**: `device_token_to_oauth` 응답의 `expired_in` ms → s 변환 (`TokenStore::save` 초 단위 `expires_at` 와 일관성)
- **W15.a 산출물 (D-57)**: cli LLM client builder `resolve_llm_client()` helper 추출. **4 단계 credential chain**: (1) `~/.myharness/oauth/minimax.toml` OAuth access_token (env var 보다 우선) → (2) `MINIMAX_API_KEY` env → (3) `ANTHROPIC_API_KEY` env → (4) MockClient fallback. token 만료 시 WARN + env var fallback.
- **W15.b 산출물 (D-58)**: cli crate 에 `RefreshingLlmClient` wrapper 추가. `LlmError::ProviderCall(msg)` 의 `msg.to_lowercase()` 에 **"401"** / **"unauthorized"** / **"auth"** (단 "oauth" 제외) 키워드 감지 → `AuthManager::ensure_fresh(Arc<dyn OAuthProvider>)` 호출 → store 갱신 → 새 `OpenAiCompatProvider` 빌드 → **retry 1회**. retry 1회 한정 (무한루프 방지). `resolve_llm_client()` 의 OAuth 경로(1번)에만 wrap (env var / MockClient 경로는 불요). `TokenStore` / `AuthManager` 에 `#[derive(Clone)]` 추가. `cli/Cargo.toml` 에 `async-trait` + `tempfile` (dev) + `chrono` (dev) + `serde_json` (dev) 추가. 9 cli tests 추가 (`is_unauthorized_*` 6 + `with_no_stored_token` 1 + `without_refresh_token` 1 + `e2e_401_refresh_retry_200` 1). 388 workspace tests pass
- **누적 38+ commit dual-push** (D-44~D-58) + W17 PR main merge (336a766) + W18 cherry-pick main merge + cleanup (6e925a1) + handoff_D-61 복구 (91c1e34). **Workspace 411 tests pass, 0 fail, 2 ignored** (real API smoke)
- **D-62 정정 (2026-06-10)**: **W18 의 'W17 PR 누락분' 보강 자체가 잘못된 cross-check 였음**. W17 original 4 commit (766d1b6 register_local_provider_non_interactive fn / a46f480 TC-W17-I01+I02 / f8082bc cli --url/--model/--token/--probe-skip / d09a6a1 spec+TC scaffold) 모두 main 머지 확인 (git log main --grep='W17' + grep 'register_local_provider_non_interactive\\|tc_w17_' 으로 검증). W18 cherry-pick cleanup commit `6e925a1` 가 W18 중복 fn/test 만 제거하고 W17 본진은 보존. **→ W17 누락분 = 0건**, state.json / handoff / backlog / 2026-06-09.md 정정. lesson: cross-check 시 '누락분' 결론은 반드시 git log + file symbol grep 으로 직접 검증, 단순 머지 commit stat 만으로 판단 금지
- **다음**: TASK-005-2 W19+ 결정. MiniMax device grant real flow 검증 (W15.b 자동 refresh 도 real test 가능)

## Work Status

- TASK-001 my-harness 부트스트랩 (standard_ai_workflow 적용): done
- TASK-002 도메인별 명령 가이드 (코드/서버/환경): **⏸ deferred** (yklee 인프라 정보 필요, v1 Rust 구현 완료 시점에 재검토)
- TASK-003 Gitea 미러: done (D-05, D-07, D-08)
- TASK-004 CLI/TUI 레퍼런스 분석:
  - 1차 (8축 비교표): done (`docs/REFERENCES.md`, D-09)
  - 2차 (14섹션 심층분석, 7 reference): done (D-10, D-15, D-21) — `docs/references/{codex,aider,goose,opencode,gemini-cli,claude-code,headroom}.md`
  - 7-doc cross-review: done (D-21) — `docs/references/README.md`
- TASK-005 my_harness CLI/TUI 전환: **✅ done (스택 결정)** → **🔜 TASK-005-1 (v1 MVP 빌드)**
  - TASK-005-1 (v1.0 MVP): **in_progress** (W3~W15.b 완료, 8/8 waves + D-52 follow-up 6 작업, dual-remote push 완료 D-44~D-58)
  - TASK-005-2 (v1.5): planned
  - TASK-005-3 (v2.0): planned
  - TASK-005-4 (v2.5): planned
  - TASK-005-5 (v3.0): planned
- TASK-006 TUI 라이브러리: ✅ done (ratatui, D-36)
- TASK-007 headroom 우선순위: ✅ done (3 알고리즘 v1, D-37)
- TASK-008 Provider fallback: ✅ done (provider-auto-config skill, D-38)

## Key Changes (오늘 — D-49 ~ D-57)

- 2026-06-09 — **TASK-005-1 v1 MVP 6/8 waves 완료 (D-49)**: tools (W3~W6.5) + llm (W7) + context (W8) + compression (W9) + tui (W10) + core (W11) crates + cli. 32+ commit dual-push. 364 tests pass (workspace).
- 2026-06-09 — **TASK-005-1 W12 (D-50)**: MiniMax LLM API 연결. `api.minimax.io/v1`, `MiniMax-M3` default, 7 models (M3/M2.7/M2.7-highspeed/M2.5/M2.5-highspeed/M2.1/M2), tool_use 지원, OpenAI-호환 Bearer. `ProviderMetadata::builtin_minimax()` 갱신. `KeyringAuthStore` in-memory cache + env hint (libsecret 부재 fallback). cli default LLM = `MINIMAX_API_KEY` env 자동 detect → `OpenAiCompatProvider`
- 2026-06-09 — **TASK-005-1 W13 (D-51)**: `myharness-auth` crate v1. 7 모듈 (pkce/flow/callback/browser/store/provider/manager). 3 OAuth provider (MiniMax/OpenAI/Google, 모두 PKCE public client, client_secret 없음). loopback callback server (127.0.0.1, 5min timeout, port 0 random). `~/.myharness/oauth/{provider}.toml` chmod 600 store. `AuthManager` login/refresh/status/logout + `AuthError::is_not_found()` helper. cli auth subcommand (`auth list|login|logout|status`). 38 tests pass
- 2026-06-09 — **TASK-005-1 W13.5 (D-52)**: `OAUTH_PROVIDERS` static `LazyLock<HashMap<...>>` 제거 → 매번 `oauth_providers()` 새 instance (env 변경 즉시 반영). 3 provider `from_env()` 생성자 추가. `MYHARNESS_OAUTH_CLIENT_ID_{MINIMAX,OPENAI,GOOGLE}` env override + `MINIMAX_API_HOST` base_url override. env 검증: `cp-test-12345` 정상 반영
- 2026-06-09 — **TASK-005-1 W13.6 (D-52)**: local mock HTTP server (`TcpListener` + raw HTTP request line) + `MockProvider` (trait `&self` receiver, custom endpoints). `auth_manager_end_to_end_with_mock_server` test: build_authorize_url → reqwest get → mock 가 302 redirect → callback params → `exchange_code` → JSON token response → `TokenStore::save`. **real network 없이** 전체 OAuth 2.0 + PKCE flow 검증. CI 환경 가능. 39 auth tests pass
- 2026-06-09 — **TASK-005-1 W14 (D-53)**: MiniMax **Device Authorization Grant** (Authorization Code + PKCE 가 404). 새 `MinimaxDeviceOAuth` provider + `DeviceCodeProvider` trait + `device_flow.rs` (request_code / poll_token / poll_until_success). OpenClaw/Hermes 공통 `client_id = 78257093-7e40-4613-99e0-527b14b39113`. scope = `group_id profile model.completion`. CN: `MYHARNESS_MINIMAX_CN=1` 또는 `MINIMAX_OAUTH_BASE_URL` env override. 기존 `MinimaxOAuth` (Authorization Code) deprecated 표시만 유지 (OpenAI/Google redirect flow 와 일관성). dfb40b4
- 2026-06-09 — **TASK-005-1 W14.4 (D-54)**: `--no-browser` 의미 수정. `AuthManager::login_minimax_device` signature `(auto_open_browser: bool, non_interactive: bool)`. 3 모드: (1) default = URL 출력 + browser 자동 open + polling + save, (2) `--no-browser` = URL 출력 + polling + save (user 직접 paste), (3) `--non-interactive` = URL 출력만 + 즉시 종료. `login_minimax_device_no_browser_polling_saves` mock e2e test 추가. 42 auth tests, 367 workspace tests. 6c85655
- 2026-06-09 — **TASK-005-1 W14.5 (D-55)**: device flow polling 진입 전 URL + user_code + expires_in 항상 stdout 출력. 1.5x backoff + cap 10s + floor 1s. `expired_in` ms 단위 unix timestamp 처리. `login_minimax_device_with_mock_server` e2e test 통과. cf29576
- 2026-06-09 — **TASK-005-1 W14.6 (D-56)**: `device_token_to_oauth` 응답의 `expired_in` ms → s 변환 (`TokenStore::save` 초 단위 `expires_at` 와 일관성). 9f7f957
- 2026-06-09 — **TASK-005-1 W15.a (D-57)**: cli LLM client builder `resolve_llm_client()` helper 추출. **4 단계 credential chain**: (1) `~/.myharness/oauth/minimax.toml` OAuth access_token (env var 보다 우선) → (2) `MINIMAX_API_KEY` env → (3) `ANTHROPIC_API_KEY` env → (4) MockClient fallback. token 만료 시 WARN + env var fallback. **자동 refresh 는 W15.b**. f92e988
- 2026-06-09 — **TASK-005-1 W15.b (D-58)**: cli crate `RefreshingLlmClient` wrapper 추가. `LlmError::ProviderCall(msg)` 401/unauthorized/auth 키워드 감지 (oauth 제외) → `AuthManager::ensure_fresh` → store save → 새 `OpenAiCompatProvider` 빌드 → retry 1회. retry 1회 한정. `resolve_llm_client()` OAuth 경로에만 wrap. `TokenStore` / `AuthManager` `#[derive(Clone)]`. 9 cli tests (6 unit + 3 e2e) 추가. 388 workspace tests, 1 commit dual-push
- 2026-06-09 — **dual-remote 적용** (D-44): Gitea PAT `myharness-cli` 발급 (scopes: write:repository, write:user) in `~/.git-credentials` (chmod 600). `git remote add upstream https://github.com/ykylee/my_harness.git`. 누적 37+ commit 양쪽 push 완료
- 2026-06-07 10차 — **TASK-005 결정: Rust 1안** (D-36): §11.1 + §11.3 갱신. v1 스택 종합 (Rust 2024 + ratatui + rig-core + rmcp + keyring + cargo-dist)
- 2026-06-07 11차 — **TASK-007 결정: headroom v1 1안 유지** (D-37): v1 = 3 알고리즘. CCR + Kompress-base v1.5+ 로 연기
- 2026-06-07 12차 — **TASK-005-1 환경 검증 (D-41)**: W0-1 (Rust toolchain + crate) ✅ + W0-2 (cross-build + keychain + .myharness) ✅
- 2026-06-09 — **TASK-005-1 W3~W6.5 완료 (D-43)**: myharness-tools crate (5 tool + 4 permission + 9 sanitizer + JSON schema + 5 provider wire format). 63 tests passed
- 2026-06-09 — **TASK-005-1 W7 완료 (D-45)**: myharness-llm crate v1 (6 provider + LLMClient + rig-core wrap + auth + discover + chain + router). 95 tests pass
- 2026-06-09 — **TASK-005-1 W8 완료 (D-46)**: myharness-context crate v1. 56 tests pass
- 2026-06-09 — **TASK-005-1 W9 완료 (D-47)**: myharness-compression crate v1. 40 tests pass
- 2026-06-09 — **TASK-005-1 W10 완료 (D-48)**: myharness-tui crate v1. 51 tests pass
- 2026-06-09 — **TASK-005-1 W11 완료 (D-49)**: myharness-core crate v1. 32 tests pass
- 2026-06-11 — **D-74 LLM Wiki 정상화** (R-4 SSOT drift = D-73 Plan D 마지막 chunk 보다 먼저 해소). lint 0/98/0 → **0/0/0** (pages=72). 3 sub-task: (1) **L08 fix** — `~/wiki/index.md` entry 마다 backtick-fenced full path 추가 (L08 검사가 backtick 안의 .md 만 인식, 72 page 등록). (2) **L03 skip patch** — `run_wiki_lint.py` L03 호출에 `_is_skipped` 가드 추가 (L07 패턴 정합, 5 line patch). (3) **skip config 보강** — 3 project 의 `.wiki-lint.toml` (my-harness / devhub / cross) + `load_project_config` cross 분기. **R-4 drift 정정**: `docs/architecture/DETAILED_DESIGN_LLM_WIKI.md` 의 §2.1 tree + §2.3 (167, 174) + §5.1 flow (313) + §8.2 (412, 414) + §11.5 (452) 의 `raw/ai-workflow/` 경로를 `raw/projects/my-harness/ai-workflow/` 로 갱신 (D-72 cross-project 통합 반영). 검증: **26/26 wiki-lint unit + 3/3 D-72 cross-project tests pass**. wiki commit `bd9b108` (Gitea push). my_harness commit `5407f82` (dual push Gitea + GitHub). main = 5407f82. 다음: T-v5-sync-1 Plan A launch (yklee 결정 대기).
- 2026-06-11 — **D-75 Plan A-2 T-v5-sync-1 5.1 적응** (Plan A 완료, Plan B/C/D 결정 보류). **WHY 5.1 적응**: D-73 plan_3c8c4a49 abort 후 5.1 scope 측정 결과 22 file (4 conflict + 17 NEW + 1 large 2204 lines script) — my_harness ↔ upstream standard_ai_workflow divergence 가 D-73 추정보다 훨씬 큼 (2204 lines flat script vs 55 lines upstream, 9 mcp_servers/ vs upstream 의 1 server-multi-tools). yklee 결정: **Plan A-2 (개념만 차용)**. **결과**: 4 file 1 commit (5d9ad1b dual push Gitea + GitHub). (1) `ai-workflow/core/mcp_installation_by_harness.md` (NEW) — 5.1 의 transport 비교 + 6 troubleshooting 의 my_harness 적응. (2) `ai-workflow/examples/mcp_config_examples/codex-mcp.toml` (NEW) — 5 server entry. (3) `ai-workflow/examples/mcp_config_examples/opencode-mcp.json` (NEW) — 5 server entry. (4) `ai-workflow/mcp_servers/README.md` (UPDATE) — per-harness 가이드 reference. **검증**: opencode-mcp.json JSON valid, codex-mcp.toml TOML valid. **D-73 의 4 plan 중 Plan A 완료**, Plan B (5.2 bootstrap_lib refactor, 73 files / 7,941+ ins) 결정 보류, Plan C/D skip 가능성 (my_harness 자체 evolution). **다음**: yklee 의 Plan B/C/D 결정 대기. main = 5d9ad1b.
- 2026-06-11 — **D-76 Plan B-1 T-v5-sync-1 5.2 적응** (Plan A-2 패턴 적용, 9 file additive). **WHY 9 file additive**: D-73 plan_3c8c4a49 의 5.2 추정 73 files / 7,941+ ins 의 4x underestimate. 5.2 scope 측정 결과 18 file / +3,875 / -2,458. 5.2 본질 = `bootstrap_workflow_kit.py` (2204 lines in my_harness) → `bootstrap_lib/` 6 module package. **Mirror attempt → ImportError 발견**: 5.2 의 bootstrap_lib/ 8 module + workflow_kit/ 추가 3 file (upgrade_diff.py + contract_v1/) 의 직접 mirror 시 my_harness 의 2204 lines 가 5.2 보다 신 기능 (workflow_kit.upgrade_diff._build_version_marker 등 미존재), 자체 evolution 과 충돌. yklee 결정: **Plan B-1 mirror 전체 revert + 9 file additive 만** (D-75 Plan A-2 pattern 와 동일, 1 commit LOW risk). **결과**: 9 file 1 commit (2b88e1a dual push Gitea + GitHub, 861 lines). (1) `ai-workflow/pyproject.toml` (NEW) — standard-ai-workflow 0.5.11 meta-package. (2) `ai-workflow/workflow_kit/pyproject.toml` (NEW) — standard-ai-workflow-kit 0.5.2-beta. (3) `ai-workflow/examples/pilot_validation_devhub_example.md` (NEW) — Devhub pilot validation. (4) `ai-workflow/releases/Beta-v0.5.2.md` (NEW) — v0.5.2 release note. (5-8) `ai-workflow/memory/release/v0.5.2/{PROJECT_PROFILE.md,backlog/2026-06-06.md,session_handoff.md,state.json}` (4 file) — 5.2 release memory. (9) `ai-workflow/memory/work_backlog.md` (UPDATE) — 5.2 release note entry + D-76 row. **검증**: state.json JSON valid, pyproject.toml 2 file TOML valid, wiki-lint 0/0/0 (D-74 유지). **D-73 의 4 plan 중 Plan A (5.1) + Plan B-1 (5.2 additive) 완료**, Plan B-2 (2204 lines 분해) 결정 보류, Plan C/D skip 가능성. **다음**: yklee 의 Plan B-2 / Plan C/D 결정 대기. main = 2b88e1a.
- 2026-06-11 — **D-77 Plan B-2 T-v5-sync-1 5.2 2204 lines 분해** (Plan B-1 의 후속, 1 commit 14 file 7,287 lines). **WHY D-77**: D-76 에서 발견한 ImportError 의 진짜 원인이 **잘못된 symbol name `_build_version_marker`** 였음 (5.2 의 upgrade_diff.py 에 존재하지 않음). 5.2 의 실제 public symbol = `Action/Decision/decide_action/is_path_preserved/stamp_marker/suffix_marker_supported` 6 개. **재-mirror 결과**: 11 module (bootstrap_lib 8 + upgrade_diff 1 + contract_v1 3) 모두 my_harness 측 import OK. my_harness 의 2204 lines = broken legacy (bootstrap_harnesses 부재, workflow-source 부재) — 5.2 mirror 가 clean replacement. **결과**: 14 file 1 commit (340a025 dual push Gitea + GitHub, 7,287 lines). (1-8) `ai-workflow/scripts/bootstrap_lib/{__main__,discovery,mcp,paths,renderers,writes}.py` + `harnesses/{__init__,renderers}.py` (3,718 lines mirror). (9) `ai-workflow/workflow_kit/upgrade_diff.py` (490 lines mirror). (10-12) `ai-workflow/workflow_kit/contract_v1/{__init__,delegator,output_validator}.py` (820 lines mirror). (13) `ai-workflow/scripts/bootstrap_workflow_kit.py` (55 lines shim, replace) + `.legacy.bak` (2204 lines, backup). (14) `ai-workflow/tests/check_bootstrap.py` + `check_existing_project_onboarding.py` (2 file SOURCE_ROOT fix: `REPO_ROOT / "ai-workflow"`). **검증**: `python -m bootstrap_lib --help` 정상 (6 harness option: codex/opencode/gemini-cli/pi-dev/antigravity/minimax-code), `--enable-mcp --mcp-bridge {jsonrpc-bridge,stdio-sdk}` 지원, 5.1 의 per-harness MCP + 5.2 의 package 분해 모두 작동. **D-73 의 4 plan 중 Plan A (5.1) + Plan B-1 (5.2 additive) + Plan B-2 (5.2 2204 lines 분해) 완료**. **다음**: yklee 의 T-v5-sync-1 종료 vs Plan C/D 결정 대기. main = 340a025.
- 2026-06-11 — **D-78 T-v5-sync-1 종료** (Plan C/D skip). **WHY 종료**: 5.1+5.2 의 27 commit (Plan A + B-1 + B-2) 의 scope 측정 + 적용 결과 my_harness ↔ upstream standard_ai_workflow divergence 가 매우 큼. 5.3-5.11 의 5 commit (Plan C: antigravity MCP + contract v1, Plan D: 12 commit + R-4) 의 my_harness 측 가치 = **low** (my_harness 가 이미 antigravity/contract_v1 보유, R-4 는 D-74 에서 해소 완료). **my_harness 자체 evolution 으로 전환** (upstream 과 독립). **누적 결정 (2026-06-11)**: 9 결정 (D-73~D-78, T-v5-sync-1) + 6 결정 (D-74 LLM Wiki). my_harness commit chain: 5d9ad1b (D-75) → 2b88e1a (D-76) → 340a025 (D-77) → 3a8d443 (D-77 memory) → D-78 memory. main = 3a8d443. **T-v5-sync-1 phase 완전 종료**. **다음 1순위**: TASK-005-2 v2.0 진입 — yklee 결정 대기. v2.0 후보: (a) ONNX 3-commit (ort + CCR semantic + Kompress ONNX, D-66/D-68 abort lesson), (b) Plugin 4-계층 (auto memory + provider-auto-config + marketplace), (c) CCR/Kompress-back. 또는 v1.5 후속 안정화 (D-69/D-70 의 lint cycle). blocked_items: TASK-002 (도메인별 명령 가이드, yklee 인프라 정보 의존), MiniMax OAuth real flow (yklee device grant 활성화 후).
- 2026-06-11 — **D-80 TASK-005-2 v1.5.x 추가 안정화** (3 commit 103b6fa, dual push Gitea + GitHub). (1) `chore(deps)` MSRV 1.88→1.91 (tract 0.23 호환, D-78 ecosystem stability SSOT 정합, myharness/Cargo.toml +1 -1). (2) `refactor(cli+auth+llm)` 3× dead_code + service_prefix + TokenStore 분리 (4 file / +2 -20: cli/refreshing_client.rs -store field + signature -store + test import 분리, cli/main.rs -store call, llm/auth_keyring.rs -service_name fn + -service_prefix field, llm/tests/w16_add_local.rs -_suppress_unused_modelinfo fn). (3) `docs(concept)` §5.12 SSOT drift 정정: v1 구현 범위 주석 (11 top-level + root, sub-dir v1.5+) + OAuth token 실제 dir = ~/.myharness/oauth/ 명시. cargo build/clippy/test = clean. **447 tests pass / 0 fail / 2 ignored** (D-70 의 437 → 447, +10 cli). main = 103b6fa. **librarian/explore 의 부분적 빗나감**: librarian 의 webpki advisory 미영향 (이미 0.103.13) + explore 의 9 dirs 분석 (실제 paths.rs 12 dirs) — commit 메시지에 정직 명시.
- 2026-06-11 — **D-82 D-81 follow-up (T-d-79-2 + T-d-80-2 done)** — DevHub D-79/D-80 skill 의뢰의 my_harness 측 SSOT 6 file / **2,189 lines** / 2 commit (6e18e01 + 3c116a5, dual push Gitea + GitHub). **(1) D-79 wiki-query** (1,146 lines): spec 328L (§1-§11 verbatim, 10 옵션) + SKILL.md 102L (5 key YAML frontmatter) + impl 716L (Python 3.10+ stdlib only, 4 query primitive via ripgrep subprocess + pure Python fallback, AGENTS.md §2.2 Query 6 step, --file 모드 idempotent, smoke test PASS — 122 page vault). **(2) D-80 wiki-pr-update** (1,043 lines): spec 328L (§1-§11 verbatim, 9 옵션) + SKILL.md 149L (6 key YAML frontmatter) + impl 566L (gh CLI 2.46.0 + --pr-metadata file 양쪽 지원, mirror-list 7 patterns via --reingest dispatch, pr-<num>-<head.sha[:12]> idempotency, prs/+log.md+index.md idempotent). **3-way 검증 PASS**: AST parse / YAML safe_load / spec 12 H2 headers = D-72 verbatim (diff empty). **잔여 (DevHub 측 의존)**: T-d-79-3/4 + T-d-80-3/4/5/6 (PR #552 머지 후 dry-run) — my_harness 측 잔여 없음. main = 3c116a5. **누적 결정 (2026-06-11)**: 18 + 1 = **19 결정**.
- 2026-06-11 — **D-83 pedantic clippy batch** (D-75 batch 의 일부분). 1 commit (`634d269` dual push Gitea + GitHub). **결과**: 796 pedantic warnings → **51** (93.6% reduction, 745 fixed) — auto-fix 572 (cargo clippy --fix, 71.9%) + manual fix 173 (top 8 categories) + 8 broken test file git checkout 복구 (test code 만 망가짐, lib build OK 였음). 3-way verify PASS: cargo build = clean / clippy pedantic = 51 warnings residual / cargo test = **447 pass / 0 fail / 2 ignored** (D-70 의 437 → 447 회귀 0). **commit scope**: 61 file / +804 -393 (60 .rs + 1 Cargo.lock), **Cargo.toml 변경 없음** (MUST NOT DO 준수 — duplicate deps batch 와 충돌 방지). **residual 51 분포**: top 7 categories × 2 each (too_many_lines / struct_excessive_bools / needless_pass_by_value / missing_errors_doc / items_after_statements / format_* / float_cmp / doc_markdown) + 35 misc (clippy::pedantic prefix 만 catch). **lesson**: background agent 의 output 100K+ 줄 truncation = context overflow / doc 추가 script 의 \\n 빠짐 → syntax error / multi-line fn signature 처리 미흡 → fn body 내부에 doc 잘못 insert. main = 634d269. **누적 결정 (2026-06-11)**: 19 + 1 = **20 결정**.
- 2026-06-11 — **D-84 pedantic clippy residual final** (D-83 follow-up). 1 commit `5fc7f56` dual push Gitea + GitHub, branch `fix/d83-pedantic-residual-final` 보존. **결과**: D-83 residual 21 warnings → **0** (7 카테고리 fix). (1) **float_cmp × 3** — epsilon 비교 (compression/kompress.rs:272 f32, context/compression.rs:332-333 f64). (2) **match_same_arms × 1** — `#[allow(clippy::match_same_arms)]` + 주석 (llm/auth_keyring.rs:175, 의미상 다른 panic 분기 보존 — arm 병합 시도했으나 unreachable_patterns 발견). (3) **default_trait_access × 7** — `Default::default()` → `SanitizerMode::default()` (tools/{edit,grep,read,glob_,write}.rs, `SanitizerMode` use 를 `#[cfg(test)] mod` 안으로 이동). (4) **match_wildcard_for_single_variants × 1** — `_` → `PermissionDecision::Allow` (tools/permission.rs:94). (5) **too_many_lines × 3** — `#[allow(clippy::too_many_lines)]` + 주석 (cli/main.rs main 109L + resolve_llm_client 101L, cli/refreshing_client.rs e2e test 106L — 의미 단위 entrypoint / credential chain / mock e2e test). (6) **used_underscore_binding × 1** — `_tty` → `tty` + `drop(tty)` (cli/main.rs:260). (7) **items_after_statements × 4** — test fn 안 struct+impl → test mod-level helper struct 추출 (cli/refreshing_client.rs, `Always401Counter` + `Always401Simple` 모듈-level fixture, AtomicUsize/Ordering use 정리). **3-way verify PASS**: cargo build clean / cargo clippy --workspace --all-targets -- -W clippy::pedantic = **0 warning** / cargo test = **447 pass / 0 fail / 2 ignored** 회귀 0. **commit scope**: 11 file / +58 -46. **worktree**: `.worktrees/fix-d83-pedantic-final` (branch `fix/d83-pedantic-residual-final`, main 732d6eb 기반). main 직접 머지 X — yklee trigger 대기. **lesson (3)**: type-specific default 는 `#[cfg(test)] mod` 내부 `use` 정답 (lib top-level import 시 unused) / match arm 병합 시 unreachable_patterns risk → 단순 `#[allow]` + 명시적 주석이 safer (semantic 보존) / test fn 안 struct+impl 정의는 mod-level helper 가 더 깔끔 (재사용 가능). **누적 결정 (2026-06-11)**: 20 + 1 = **21 결정**.
- [x] **v1 컨셉 확립** (D-22~D-38) — 5/5 결정 검토 완료 (4 ✅, 1 ⏸)
- [x] **TASK-005-1 환경 검증 (D-41)** ✅
- [x] **W2~W6.5 (D-43)** — tools crate 5 tool + 4 permission + 9 sanitizer + JSON schema + 5 provider wire format ✅
- [x] **Gitea push** ✅
- [x] **dual-remote push (D-44)** ✅
- [x] **W7 (D-45)** — myharness-llm crate v1 ✅
- [x] **W7 dual-remote push (D-45)** ✅
- [x] **W8 (D-46)** — myharness-context crate v1 ✅
- [x] **W8 dual-remote push (D-46)** ✅
- [x] **W9 (D-47)** — myharness-compression crate v1 ✅
- [x] **W9 dual-remote push (D-47)** ✅
- [x] **W10 (D-48)** — myharness-tui crate v1 ✅
- [x] **W10 dual-remote push (D-48)** ✅
- [x] **W11 (D-49)** — myharness-core crate v1 ✅
- [x] **W12 (D-50)** — MiniMax LLM API 연결 (api.minimax.io/v1 + MiniMax-M3 + KeyringAuthStore in-memory + cli default LLM env 자동 detect) ✅
- [x] **W13 (D-51)** — myharness-auth crate v1 (OAuth 2.0 headless auth: pkce/flow/callback/browser/store/provider/manager) ✅
- [x] **W13 dual-remote push (D-51)** ✅
- [x] **W13.5 (D-52)** — OAuth env override (client_id/base_host, OAUTH_PROVIDERS static LazyLock 제거) ✅
- [x] **W13.6 (D-52)** — mock OAuth server + AuthManager end-to-end e2e test ✅
- [x] **W13.5 + W13.6 dual-remote push (D-52)** ✅
- [x] **W14 (D-53)** — MiniMax Device Authorization Grant (Authorization Code + PKCE 404 → device flow 교체) ✅
- [x] **W14.4 (D-54)** — `--no-browser` 의미 수정 (3 모드: default / --no-browser / --non-interactive) ✅
- [x] **W14.5 (D-55)** — polling output 항상 stdout + 1.5x backoff + expired_in ms 단위 ✅
- [x] **W14.6 (D-56)** — device_token_to_oauth expired_in ms → s 변환 ✅
- [x] **W15.a (D-57)** — OAuth token 자동 resolve (4 단계 credential chain) ✅
- [x] **W15.b (D-58)** — OAuth token 자동 refresh (LlmError::ProviderCall 401 감지 → AuthManager::ensure_fresh → 새 OpenAiCompatProvider → retry 1회) ✅
- [x] **D-52 follow-up 6 commit dual push (D-53~D-58)** ✅
- [ ] **Gitea PAT 회전** (이전 세션 노출 회전 권고) — yklee 가 회전 시 통지
- [x] **TASK-005-1 v1 MVP 종료 선언** (W3~W16 완료, 8/8 waves + D-52 follow-up 6 작업 + W16 add-local, 388+ tests pass, dual-push 완료, 2026-06-09 22:48)
- [x] **TASK-005-2 v1.5 W17 (D-60) + W18 (D-61) 완료** — auth add-local 비대화형 모드 (--url/--model/--token/--probe-skip) + R-4 backup + Confirm + --yes flag. main 머지 + cleanup (6e925a1) + handoff_D-61 복구 (91c1e34). 411 tests pass
- [x] **D-62 W17 누락분 = 0건 정정** (2026-06-10) — W18 cross-check 오류였음, W17 original 4 commit 모두 main 머지 확인
- [x] **D-62 W19-1 TC-W17-002 test fail 복구** (2026-06-10) — `register_local_provider_with_store` + `register_local_provider_non_interactive_with_store` 추가 (AuthStore 주입 패턴). 기존 fn 은 thin wrapper. 4 L1 test (TC-W17-002 수정 + TC-W19-001/002/003) PASS, 110 llm tests / 0 fail. cargo build/clippy OK
- [x] **D-63 W20 F-3 Ollama native cascade** (2026-06-10) — `probe_local_models` 2-stage (Ollama native /api/tags → OpenAI compat /v1/models fallback). `parse_ollama_tags` / `parse_openai_models` / `fetch_json_body` helper 분리. W16 7/7 회귀 없음, 3 L2 integration test (TC-W20-I01/I02/I03) PASS, 110 llm + 10 w16_add_local tests / 0 fail. cargo build/clippy OK
- [x] **D-64 W21 F-1+F-2 통합** (2026-06-10) — `hash8::content_hash_8(content)` SHA-256 8-char util (sha2=0.10, auth crate 재사용). backup filename = `<base>.backup.<ts>.<sha256_8>` 으로 변경. `cleanup_old_backups` **sort bug fix** (string sort → numeric parse on unix_ts). R-5-A (sub-second collision) + R-5-B (content 식별) + sort bug 동시 해결. 4 L1 test (TC-W21-001/002/003/004) PASS, 117 llm + 10 w16_add_local + 3 hash8 = 130 / 0 fail. cargo build/clippy OK
- [x] **D-65 TASK-005-2 v1.5 종료 선언** (2026-06-10) — v1.5 phase 완전 종료. 5 사이클 + D-62 + 누적 14 신규 test / 0 fail. ONNX 통합 v2.0 Planning 으로 연기 (Initial_design.tt-3 의 설계 의도 따름 — '+10-30MB v1.5+' 분류). 424 workspace tests / 0 fail. cargo build/clippy OK. binary 13MB 유지 (ort C++ dep 회피)
- [x] **D-66 v2.0 ONNX Commit 1 abort** (2026-06-10) — ort ecosystem 2026-06 unstable. ort 1.x (1.13.1, 1.16.3) **전부 yanked**. ort 2.0.0-rc.9/10/12 모두 빌드 깨짐 (ureq 3.1 API 변경 — `tls_config` method 없음, `unwrap_or_else` fn pointer 미구현). 코드/Cargo.toml 모두 revert. v2.0 ONNX 백로그 OOS. **lesson**: ecosystem stability SSOT (CONCEPT.md §11.3) — library 분석 시 crates.io API + lib.rs + **실제 cargo build 검증** 필수. library 보고만 의존 ❌
- [x] **D-67 v2.0 tract Commit 1** (2026-06-10) — tract 0.23.0 (Pure Rust, Sonos production, MSRV 1.91) 로 전환. 1차 cargo build 즉시 통과 (D-66 lesson). `ModelManager` skeleton (OnceLock lazy + `new()`/`get()` global + `ensure_downloaded` reqwest streaming + sha2 SHA256 verify + `load_runnable` `into_runnable()` verify, `embed()` Commit 2 stub). 5 L1 test / 0 fail. 429 workspace tests. binary 13MB (release lto=thin). **Commit 1 한계**: Runnable(Arc<dyn trait>) type-safe 보관 어려움 → Commit 2 에서 정착
- [x] **D-68 v2.0 tract Commit 2 abort** (2026-06-10) — actual `embed()` inference 시도 후 5+ error 누적. **API 한계**: tract 0.23 의 `Tensor(Arc<InternalTensor>)` wrapper field private + Deref 없음 + `to_array_view` 가 `plain_view::Tensor` 의 method (다른 type) + Runnable vs `SimplePlan<InferenceFact, Box<InferenceOp>>` direct cast 어려움. 변경 모두 revert. **lesson**: tract 0.23 low-level API 가 high-level inference 에 부적합. v2.0 ONNX 백로그 OOS 유지
- [x] **D-69 v1.5 안정화** (2026-06-10) — 3 작업 완료, 3 commit dual-push (6d2a3e8/4891bc6/767c71a, Gitea + GitHub), 437 tests pass / 0 fail / 2 ignored:
  - **(1) tool name uppercase 통일** (LLM contract 정합, v1.5+ dispatch 준비) — tools/*.rs `impl Tool::name()` 의 소문자 (read/grep/glob/bash/edit/write) ↔ schema/tui 의 대문자 (Read/Grep/Glob...) mismatch. TUI `ToolRegistry::get("Read")` 시 None 반환할 latent bug (v1.5+ dispatch 활성화 시 발현). 26 곳 변경: 6 impl + registry test 8 + permission 7 + lib 1 + adapter 4 (`to_lowercase()` 제거). cargo test -p myharness-tools: 63/0 fail
  - **(2) §5.12 init_home_dir()** (CONCEPT §5.12 SSOT, D-31) — v1 first run 시 11개 디렉토리 자동 생성 (7 top-level: config/state/memory/handoff/compression/sub-agents/auth + state subdir state/auth + 2 추가: runtime/cache + root). cli main() 진입 시 호출, best-effort (filesystem 권한 실패 시 tracing::warn 후 계속). paths.rs +125 / 4 unit test + 3 integration test (serial_test env race 방지). 3/3 PASS
  - **(3) clippy 핵심 5건 fix** — (a) `context/compression.rs:316` PI 상수 (`std::f64::consts::PI`, deny level test 컴파일 차단 해소), (b) `should_implement_trait` 4 file `#[allow]` (의도적 `Option<Self>` 반환), (c) `useless_format` 1 line (auth/manager.rs:508). 잔여 21 style lint v1.5.1+ OOS
- [x] **D-70 v1.5.1 lint cleanup (2026-06-10)** — 21 style lint → 0 warning. 자동 fix (cargo clippy --fix --allow-dirty) + manual fix (test code false positive 4 file: auth/manager.rs `AsyncWriteExt` + `TokenPoll`, llm/client_openai_compat.rs `Message`, tui/app.rs `Terminal` + closure type annotation `c: &ratatui::buffer::Cell`, tui/loop_mode.rs let-chain → nested if). 21 file / +94 -110. cargo clippy --workspace --all-targets = **0 warning**. 1 commit dual-push (79f38b8, Gitea + GitHub)
- [x] **D-70 v1.5 종료 선언 (2026-06-10)** — v1.5 phase 완전 종료. 누적 8 결정 (D-60 W17 + D-61 W18 + D-62 W19-1/정정 + D-63 W20 + D-64 W21 + D-65 v1.5 1차 종료 + D-69 안정화 + D-70 lint cleanup). v1.5 누적: 17 신규 test + 4 code commit + 5 memory commit + 21 file lint cleanup + 437 tests / 0 fail / 2 ignored
- [x] **D-73 T-v5-sync-1 plan_3c8c4a49 cancel + 4-plan split (2026-06-11)** — my_harness v0.5.0→v0.5.11 sync 11 minor 24 commit 을 4 plan (A/B/C/D) 으로 분리. **WHY 4 split**: 1 plan = 6 batch × 11 minor = scope 3x underestimate (실제 upstream diff v0.5.0→v0.5.2 = 73 files / 7,941 ins / 5,070 del). 31min cap 도달 시 25min = exploration (read 12+ files) + 0 write. **4 plan**: Plan A (5.1 MCP per-harness + round-trip, ~5 files, 3 commit, 워커 warm-up) + Plan B (5.2 bootstrap_lib refactor, **73 files / 7,941+ ins / 5,070 del**, 8 modules 3,710 lines, 9 chunk + 800줄 cap, 필요 시 2 plan sub-split) + Plan C (5.3-5.4 antigravity MCP + contract v1 도입, ~5 files + Rust caller 갱신, API breaking) + Plan D (5.5-5.11 12 commit + R-4 SSOT drift, ~10 files + R-4, smart update + Mavis engine hook + MCP fix). **3 memory entries 추가** (mavis memory): (1) T-v5-sync-1 실패 분석 (scope 3x underestimate + 25min exploration + module name 오류), (2) Worker prompt 3원칙 (first-chunk-largest / JSON split / Edit-last + Write fallback), (3) Cross-session memory grep 1-liner (worker prompt 머리에 grep 1줄). **async audit**: plan_3c8c4a49 cancelled, 워커 idle, T-SAW-1 별도 owner session 운영 중, pending ops 0건
- [ ] **T-v5-sync-1 Plan A prompt 작성 + launch** (5.1 only, ~5 files, 3 commit) — yklee 결정 후. tag `v0.5.1-beta` + commit hash (c3c9a90/73f8f2f/...) + per-batch file:line 박기. worker prompt 머리에 cross-session grep 1-liner
- [ ] **TASK-005-2 v2.0 다음 후보** — Plugin 4-계층 (큰 사이클, auto memory + provider-auto-config + marketplace) / Kompress-back (low priority) / 외부 blocker 해결 (TASK-002, Gitea PAT, OAuth, API key)
- [ ] **MiniMax Device OAuth real flow** 검증 — yklee 가 MiniMax console 에서 device grant 활성화 후 `myharness auth login minimax --no-browser` 실행 (OpenClaw/Hermes 공통 client_id 78257093-7e40-4613-99e0-527b14b39113, W15.b 자동 refresh 도 real test 가능)
- [ ] **OpenAI/Google 도 동일 패턴** (Authorization Code + PKCE, client_id 등록 후 검증)
- [ ] **ANTHROPIC_API_KEY 주입 시 LLM E2E 테스트** (real-anthropic ignored test 활성화)
- [ ] **§5.12 디렉토리 자동 생성** (v1 first run 시) — `~/.myharness/{config,state,memory,handoff,compression,sub-agents,auth}/` + `state.json`
- [ ] **TASK-002 (도메인별 명령)** — yklee 인프라 정보 수령 후 (SSH 별칭 / Brewfile / dotfiles / 런타임 버전) 진행
- [ ] **헤로쿠 / Synology NAS 인프라 검증** — yklee 가 인프라 정보 입력 시점에 작업

## Risks & Blockers

- **claude-code 2.1.169 changelog 미공개** (D-34 §11.2 pending): context var/cache, MCP, permission 변경이 우리 §5.6/§5.14/§5.4/§5.5 영향 가능. 공개 시 검증 후 §11.2 처리
- **minimax base_url** 검증 완료 (D-50): `https://api.minimax.io/v1` (librarian ⭐⭐⭐⭐⭐ 5/5). OpenAI-호환 Bearer, MiniMax-M3 default, 7 models, tool_use 지원. W12 에서 통합 완료
- **CCR + Kompress-base 연기** (D-37): v1.5+ TASK-005-2 시 재검토. ONNX 모델 weight ~수MB + CCR round-trip 1회 비용 trade-off
- **TASK-002 인프라 정보 의존** (D-39): yklee 가 SSH 호스트 목록 / Brewfile / dotfiles / asdf 버전 입력 전까지 보류. v1 Rust 구현 완료 시점에 자연 도출 가능
- **외부 4-워커 (Claude/Codex/Gemini/OpenCode) sibling 정책 유지** (D-24, D-25): my_harness 가 그 도구들을 통합/오케스트레이션 안 함, sibling 으로만 인식
- **Gitea + GitHub 듀얼 remote** (D-20, D-07): origin=Gitea (private) + upstream=GitHub (public). GitHub public 노출은 의도된 외부 미러링. 토큰 회전 시 yklee 가 Mavis 에 직접 전달
- **agent memory**: "Worker 세션 long Write call 죽음 패턴" (D-16) — `~/.mavis/agents/mavis/memory/MEMORY.md` 에 영구 저장
- **user memory** (yklee 프로필): Gitea 정보, 작업 스타일, PR 작업 패턴, 분석/리서치 작업 스타일 — `~/.mavis/memory/user.md`
- **yklee 비밀번호 / 토큰 값**: 메모리/문서/git 저장 금지 (D-06 정책). 회전 시 Mavis 가 매번 새로 전달
- **Gitea PAT 미회전** (D-44): 이전 세션 노출 가능성. yklee 회전 시 통지
- **ANTHROPIC_API_KEY absent (D-41)**: LLM E2E 테스트는 키 주입 후. v1 은 MiniMax API key 기반 (D-50 W12 결정). D-50 에서 `MINIMAX_API_KEY` env 자동 detect → `OpenAiCompatProvider` 흐름 검증
- **MiniMax OAuth client_id 미등록** (D-52 → D-53 해소): W14 에서 **OpenClaw/Hermes 공통 client_id 78257093-7e40-4613-99e0-527b14b39113** 으로 통합 (client_secret 불요, public device grant). mock e2e test (W13.6) 로 흐름 검증 완료. real device grant flow 는 yklee 가 MiniMax console 에서 device grant 활성화 후 `myharness auth login minimax --no-browser` 로 검증 가능
- **OpenAI/Google OAuth 미검증**: mock e2e test 로 흐름 검증 완료 (W13.6). real flow 는 client_id 등록 후 검증
- **CN endpoint 미구현** (D-52 → D-53 일부 해소): W14 에서 `MINIMAX_OAUTH_BASE_URL` env override 가능 (region 전환). CN 정식 endpoint (`api.minimaxi.com`) 는 v1.5+
- **W15.b 자동 refresh 완료** (D-58): OAuth token 만료 시 401 감지 → ensure_fresh → store save → 새 OpenAiCompatProvider → retry 1회. refresh_token 없으면 expired token 그대로 retry
- **TC-W17-002 test fail** (D-62 W19-1 해결, 2026-06-10): `add_local::tests::tc_w17_002_non_interactive_with_token` 가 libsecret 부재 환경 BackendUnavailable Err. **원인**: 기존 fn 의 `KeyringAuthStore::probe()` 가 caller 와 별개 in-memory cache → caller.store.get() 시 cache miss → Err. **해결**: `register_local_provider_with_store(base_url, token, selected, available, store: &dyn AuthStore)` + `register_local_provider_non_interactive_with_store(...)` 추가. caller 가 store 1개 만들어 with_store 에 명시 전달 → cache lifecycle 단일화. 기존 fn 은 thin wrapper (back-compat). cli caller 변경 없음
- **cross-check 정확성** (D-62 lesson): 머지 commit stat 만 보고 '누락분' 결론 내리지 말 것. 반드시 `git log main --grep='WAVE'` + `grep '<symbol>'` 로 file 직접 검증
- **D-73 T-v5-sync-1 lesson (2026-06-11)**: 1 plan = 6 batch × 11 minor = scope 3x underestimate 사망. 31min 중 25min = exploration 후 0 write. **mavis-team-patterns §1-B 패턴**: exploration 단계에서 죽음 (worker death 방지). **해결 5가지**: (1) prompt 작성 전 `git diff --shortstat` 1줄로 scope 측정, (2) 1 plan = 1 batch 의 1 sub-task, (3) prompt 의 module name / function name 은 upstream source 에서 직접 확인 후 박기, (4) exploration 5분 강제 cap (scratch check 30s + ls-tree 1번 3min + write 시작), (5) 30min cap 도달 시 1 fail internal / 2 fail steer (split) / 3 fail manual takeover. **cross-project 적용**: 모든 major version sync 작업 시

**Drift 해소 (D-96, 2026-06-12)**: D-85~D-95 vault 운영 11결정 → wiki 측 SSOT 격리 (raw 12 prompt path my-harness→wiki mv, AGENTS.md §12 신설). cross-project 결정만 my_harness 메모리에도 반영. my_harness 측 결정 = 21 유지, 누적 카운트 갭 해소.

**Lint 0/0/0 회복 (D-97, 2026-06-12)**: D-86~D-95 결정 + D-90 의 wiki-source-sync 111 page 갱신으로 lint 2/278/0 (420 pages) 회귀 → 0/0/0 회복. L04 6 cross-project mirror 면제 (.wiki-lint.toml 2 file) + L08 272 sources/*.md page-per-source 면제. wiki-lint script 에 fnmatch glob skip_paths 지원 추가 (rule_l04 + rule_l08).

**D-75 batch (2026-06-12)**: inquire 0.7.5 → 0.9.4 bump (crossterm 0.25 + bitflags v1.3.2 중복 제거) + D-84 lint fix cherry-pick 회귀 해소 (11 file). cargo build clean / test 447 pass / 0 fail / 2 ignored / clippy pedantic 0 warning.

**D-75 batch follow-up (2026-06-12)**: 4 dep bump (sha2 0.10→0.11, dirs 5→6, reqwest 0.12→0.13, toml 0.8→0.9) + 영향 file fix (sha2 LowerHex 미구현 → byte hex + reqwest feature 갱신). cargo tree -d 단일화 (4 dep transitive 중복 모두 해소). binary 13MB → 11MB.

**D-82 follow-up (cross-project, 2026-06-12)**: DevHub 측 PR #552 머지 + dry-run 검증 완료 처리. T-d-79-3/4 + T-d-80-3/4/5/6 잔여 0. my_harness 측 잔여 없음. D-72 §11.1 thin-wrapper 정공법 cycle (의뢰 수락 → SSOT 6 file 작성 → DevHub 검증) 완전 종료.

**세션 종료 (2026-06-12)**: 누적 결정 갱신 (D-84 21 → D-96 22 → D-97 23 → D-75 batch 24 → D-75 batch follow-up 25 → D-82 follow-up 26 = **26 결정**). main = `c8d2ca2` (D-75 batch follow-up). 5 commit (D-96 memory + D-97 wiki-lint + D-75 batch + D-75 batch follow-up + D-82 follow-up 완료 처리) + 3 wiki vault commit (D-96 + D-96 follow-up L06 + D-97 lint skip config). 다음 진입 결정 (yklee): TASK-005-2 v2.0 (Plugin 4-계층 / Kompress-back / 외부 blocker 해결) 또는 추가 안정화.

**06-13 docs cleanup (2026-06-13)**: 3 commit / 11 file / +47 -36. **새 결정 ID 추가 없음** (기존 결정의 docs 정합 + 보강만). **(1) d1c64e3 — D-36 cross-reference 정정**: D-101/D-102 잘못된 결정 ID 정리. 6 file (AGENTS.md + MiniMax.md + README.md + PROJECT_PROFILE.md + development_log.md + .gitignore 3 line 신규) / +14 -16. AGENTS.md/MiniMax.md 의 v1 결정 표에서 잘못된 ID cross-reference 수정. **(2) ab3713b — TASK-002 코드 개발 5 TODO 자동 채움 + MiniMax.md 동기화**: 2 file (AGENTS.md + MiniMax.md) / 12 line 변경. 코드 개발 명령 5 TODO (설치 / 로컬 실행 / 빠른 테스트 / 격리 테스트 / 실행 확인) 의 v1 Rust 명령 패턴 (`cargo build/test --manifest-path myharness/Cargo.toml --workspace`) 자동 반영. README.md 의 "다음에 정해야 할 것 (TASK-NNN)" 표의 TASK-002 상태 = "코드 개발 5 TODO 자동 채움 완료, 서버 관리 / 환경 셋업 TODO 별도 세션 trigger 후 진행" 으로 갱신. **(3) d1f4a39 — D-101/102 TASK-005/006 결정 log 보강**: 3 file (README.md + PROJECT_PROFILE.md + development_log.md) / +19 -10. README.md "다음에 정해야 할 것 (TASK-NNN)" 표 + PROJECT_PROFILE.md §1/§3.1 + development_log.md 의 TASK-005/006 결정 log 보강. main = `d1f4a39`. 누적 결정 카운트 불요 (state.json decisions.decided.length = **45**, TASK-005/006/007/008 + D-42 + D-50~D-84 + D-96/D-97/D-75-batch/D-75-batch-followup/D-82-followup = 45 entry).

**task_list drift 정정 (2026-06-13)**: opencode task_list 의 T-a5ced899/ae2b2c5e/97a66732 (D-83 follow-up "Step 3/4/5") = **stale**. D-83 batch + D-84 follow-up 모두 06-11 closeout 으로 완료 처리됨 (commit `634d269` + `732d6eb` + `5fc7f56`). 06-12 closeout 결정 5 commit (D-96/D-97/D-75 batch/D-75 batch follow-up/D-82 follow-up) 으로 모든 Step 이 stale. 다음 세션 시작 시 opencode task_list 정리 또는 그대로 두기 (자동 만료 대기).

**Sub-task 1 완료 (D-98 + D-99, 2026-06-14)**: TASK-005-2 v2.0 Plugin 4-계층 의 첫 sub-task (Layer 1: Auto Memory) 완료. **2 commit dual push (Gitea + GitHub)**. **(1) D-98 (57db117) Commit A — Auto Memory pure refactor**: 단일 `myharness/crates/context/src/auto_memory.rs` (293 lines, D-46 NDJSON append-only) → 4 file 모듈 (mod.rs + types.rs + store.rs + query.rs) 분할 + `MemoryStore` async trait 추출 + `NdjsonMemoryStore` adapter (back-compat 100%, sync wrappers via `Arc::clone + async move` + `block_on` bridge). 7 new tests (`ndjson_*`) ported from 8 existing (drop: `append_and_recent_roundtrip` merged). lib.rs re-export 확장. Cargo.toml 변경 없음. **(2) D-99 (c64d0ff) Commit B — SqliteMemoryStore (rusqlite FTS5 + BM25)**: `auto_memory/sqlite_store.rs` (303 lines) 신규 + `tests/auto_memory.rs` (220 lines, 6 integration tests) 신규 + Cargo.toml (workspace + context, `rusqlite = { version = '0.31', features = ['bundled'] }`) 추가. FTS5 virtual table schema (memory + memory_fts + 3 triggers) + bm25(memory_fts) ranking + porter unicode61 tokenizer. default backend = NDJSON (back-compat), opt-in via `MYHARNESS_MEMORY_BACKEND=sqlite`. block_on bridge 단순화: std::thread::spawn escape for `#[tokio::test]` context (current_thread runtime executor deadlock 회피) + Runtime::block_on fallback. clippy 1 `#[allow(clippy::collapsible_if)]` on query method (let-chain 회피). **3-way verify**: cargo build clean / cargo clippy --workspace --all-targets -- -D warnings clean / cargo test --workspace **453 pass + 0 fail + 2 ignored** (baseline 447 + 6 Commit B integration, Commit A 의 7 ndjson test 는 context crate 의 55 안에 포함, 회귀 0). **누적 결정 카운트 갱신 (2026-06-14)**: 28 → **30** (D-98 + D-99 추가). state.json decisions.decided.length = **47**. main = `c64d0ff`. **다음 1순위 (yklee 결정)**: Sub-task 2 (provider-auto-config Skill 정식, llm + cli 영향, MEDIUM effort, 의존성 없음, parallel 가능) / Sub-task 3 (Plugin Installer, HIGH effort) / Sub-task 4 (Marketplace) / 세션 종료.

**방향 전환 + A-min tool dispatch (D-100, 2026-06-30)**:
- **방향 전환 (yklee)**: oh-my-pi (`can1357/oh-my-pi`, v15.1.8, 11.1k~14.7k stars, MIT, Mario Zechner의 `badlogic/pi-mono` fork) 를 reference + 부분 차용 (Hybrid 안). 점진 차용: Hashline 편집 / Skill·Extension 시스템 / hindsight memory / LSP+DAP. 비차용 (유지): OAuth PKCE+Device Grant / Local LLM cascade / R-4 backup / W15.a/b / standard_ai_workflow / 한국어 workflow / 백업 안전장치.
- **선결 과제 = 사용 가능한 형태**: 진단 (22:08~22:50) → binary OK / init_home_dir OK / auth OK / **LLM credential 0건 P0**. 해결: `MINIMAX_API_KEY` 주입 → real LLM OK (chat-bot tier). `ask`/`code review`/`env diagnose`/`git commit` 모두 LLM 응답 OK, 단 tool 자동 실행 안 함.
- **D-100 A-min text-based tool dispatch 1차 cycle**: `agent.rs` +52 (tool_spec_section), `orchestrator.rs` +194 (extract_tool_call + dispatch loop max 3 round), 5 test 추가 (18/18 tui PASS). clippy clean. Real LLM `code review myharness/crates/cli/src/main.rs` → `[tool_call] Read → ok` × 3 round 자동 dispatch. `env diagnose` → `[tool_call] Bash → ok` × 3 round.
- 한계: max_round 3 → 5/10, Bash 결과 stdout visible, A-proper native tool calling (v1.5+ CompletionRequest::tools + CompletionResponse::tool_calls + provider wire format).
- 누적 결정 47 → **48** (D-100). main = `5fe1e90` (D-100 final, post-amend).
- **D-101 (2026-06-30) A-min follow-up polish**: max_tool_rounds 3 → **10 default** + `with_max_tool_rounds(n)` builder (configurable). tool result stdout **visible in response** (2000자 truncation, 이전엔 `[tool_call] X → ok` 마커만). `dispatch_tool_call` 에 `with_confirm_override(true)` 추가 — AcceptEdits + confirm_override → Bash prompt skip (비대화형 환경 hang 방지). 4 test 추가. **22/22 tui PASS (D-100 18 + D-101 4 신규), clippy clean**. Real LLM `env diagnose` → 10 round 자동 dispatch + `[tool_result]` 에 uname/PATH/whoami/pwd stdout visible + prompt 안 뜸. 한계: max_round 10 도 큰 file 부족 + LLM 같은 Bash 반복 (prompt 개선 필요) + A-proper native tool calling (v1.5+).
- 누적 결정 48 → **49** (D-101). main = D-101 commit (hash 다음 세션 확정).
- **D-102 (2026-06-30) prompt 개선 + dedup 안전망** — LLM 무한 루프 방지. (1) `tool_spec_section` 에 Stop conditions 4가지 (enough info / same tool+args 반복 / last 2-3 similar / previous turn covered) + safety net 명시. (2) `canonical_tool_call(name, args)` helper (BTreeMap key 정렬, 순서 무관) + `call_counts: HashMap` 추적 + 2회 중복 시 synthetic final prompt + break. 5 test 추가. **27/27 tui PASS (D-100 18 + D-101 4 + D-102 5), clippy clean**. Real LLM `ask "1+1은?"` → LLM tool 안 쓰고 즉시 plain 응답 (`2입니다.`) — prompt stop condition 작동 확인. **효과**: 2회 중복 시점에 즉시 break → 효율 + 비용 절감. 한계: synthetic final prompt 1회만 / A-proper native tool calling (v1.5+) 미적용 / 큰 file chunked Read 권장 prompt 필요.
- 누적 결정 49 → **50** (D-102). main = D-102 commit (hash 다음 세션 확정).
- **D-103 (2026-06-30) prompt 보강 (large file chunked Read + overlap dedup)** — `tool_spec_section` 의 Read description 강화 (>500 lines, ~200 chunk, offset+limit 권장) + Large files 섹션 4가지 (Glob first, no overlap, progress forward, always offset+limit for >500). 3 test 추가. **68/68 tui PASS (D-100 18 + D-101 4 + D-102 5 + D-103 3 + 다른 38), clippy clean**. Real LLM `code review myharness/crates/cli/src/main.rs` (770 lines) → 2 round + **D-102 dedup 자동 발동** + final prompt. **이전 10 round → 현재 2 round, ~5x 빠르고 비용 절감**. D-102 + D-103 합동으로 무한 루프 + 큰 파일 낭비 동시 방지. 한계: LLM prompt 무시 가능 → 강제하려면 tool wrapper (v1.5+) / A-proper native tool calling (v1.5+) 미적용 / 1000+ lines 은 chunked Read + content fingerprint (Hashline) 가 진짜 해결책.
- 누적 결정 50 → **51** (D-103). main = D-103 commit (hash 다음 세션 확정).
- **D-104 (2026-07-01) oh-my-pi Hashline 분석 + Read v2 (LINE:TEXT + 4-hex content_hash)** — `@oh-my-pi/hashline` v15.11.0 (`can1357/oh-my-pi`, MIT, Mario Zechner의 `badlogic/pi-mono` fork) 의 LINE:TEXT prefix + content hash tag 패러다임을 v1 점진 차용 1차 cycle 로 채택. D-103 의 "1000+ lines 진짜 큰 파일은 content fingerprint 가 진짜 해결책" 한계의 직접 응답. **구현**: (1) `myharness/crates/tools/src/content_hash.rs` 신규 — `compute_content_hash` = SHA-256 truncate low 16-bit → 4-hex uppercase (oh-my-pi `HL_FILE_HASH_LENGTH=4` 정합, sha2 0.11 이미 workspace, xxhash-rust 새 dep 회피 — session-scope 16-bit fingerprint 으로 충분) + `normalize_for_hash` (line 별 `[ \t\r]+` trim + final newline 보존, oh-my-pi `normalizeFileHashText` 정합) + `format_line_anchored` (str::lines() 기반, phantom-trailing row 회피, 1-indexed 절대 line 번호). (2) `read.rs` v2 — output 항상 LINE:TEXT format default + metadata `{path, size, line_count, format:line_text, content_hash, hash_length:4, start_line, end_line}`. chunked Read 도 절대 line 번호 보존 (D-103 anchor 1차 보강). (3) `Cargo.toml` tools crate sha2 dep 추가. (4) `lib.rs` content_hash mod + re-export. (5) **spec memo `ai-workflow/memory/hashline_v2_spec.md`** (200+ lines, 9-section — 왜 Hashline / v1.5+ 차용 범위 결정 table / content hash 결정 sha2 vs xxhash / Read v2 spec / Edit v2 line_anchored spec §5 / v1.5+ tree-sitter + v2 snapshot store roadmap / 위험 + 회피). **12 test 추가** (content_hash 8: hash determinism/content-change/trailing-whitespace normalize/uppercase-hex + format_line_anchored basic/offset/no-trailing-newline / read 4 신규: chunked absolute line preserved + full vs chunked hash 동일성 invariant). **검증**: cargo clippy --workspace --all-targets -- -D warnings clean (sha2 0.11 → 3 clippy catch 한 번에 해소: `while_let_on_iterator` + `manual_pattern_char_comparison` ×2, 모두 text-based rewrite 로 fix) / cargo test --workspace --lib **436 pass + 0 fail + 2 ignored** (D-103 baseline 424 → D-104 tools 50 → 62, 나머지 crate 회귀 0) / Invariant: full vs chunked Read → 동일 content_hash / line TEXT format = `LINE:TEXT` 1-indexed absolute. **효과**: (a) D-105 Edit v2 가 hash check 로 stale anchor 자동 reject 가능. (b) LLM 이 LINE:TEXT 를 그대로 보고 line 번호 cite (`5:fn main()`). (c) oh-my-pi 의 `prompt.md` tight-range / 1-hunk-per-range 룰셋이 D-105 의 line_anchored prompt 에 그대로 적용 가능. **차용 범위 결정**: D-104 = Read v2 (LINE:TEXT + content_hash) 2 area 점유. D-105+ = Edit v2 line_anchored (replace N..M + hash check) 3 area. v1.5+ = tree-sitter `replace block N` 2 area. v2 = SnapshotStore + 3-way merge recovery 2 area. (총 9-area 점진 채움, 한 cycle 1-2 area.) **한계**: (a) Edit v2 line_anchored 미구현 (D-105), Read anchor 정보만 제공. (b) tree-sitter 미도입 — D-106+ long-function rewrite 시 line 손 count. (c) sha2 vs xxhash 16-bit fingerprint 의미론 동일 (충돌 회피 충분) — D-105 hash 검증 시 xxhash-rust 도입 결정 가능. **누적 결정 51 → 52** (D-104 추가). main = D-104 commit (hash 다음 세션 확정). 다음 1순위: D-105 Edit v2 line_anchored mode / A-proper native tool calling / TASK-002 / TUI shell.
- **D-105 (2026-07-01) Edit v2 line_anchored mode** — oh-my-pi Hashline 점진 차용 2차 cycle. D-104 의 Read v2 (LINE:TEXT + 4-hex content_hash) 가 line N anchor + content_hash 를 emit 하면, LLM 이 이를 `expected_hash` 로 즉시 사용 가능 → stale anchor 자동 reject. **구현**: (1) `LineAnchoredEdit` struct (private, Deserialize: start_line/end_line/expected_hash/replacement, 1-indexed inclusive). (2) `apply_line_replacement(content, start_line_1, end_line_1, replacement) -> Result<String, String>` helper — validation (start>=1, end>=start, end<=total_lines) + empty replacement = 0 lines (delete) + non-empty = split('\n') lines + pre+repl+post Vec<&str> concat + join('\n') + trailing '\n' 보존 (content.ends_with('\n') 시). (3) `EditTool::execute_line_anchored` private async method — read → compute_content_hash → stale anchor reject (Err InvalidInput) → range check → apply → write → new_hash 재계산 → metadata `{path, mode:'line_anchored', start_line, end_line, replaced_lines: end-start+1, old_hash, new_hash}`. (4) `EditTool::execute` dispatch: `if input.get('line_anchored').is_some() { return execute_line_anchored(...) }` 위치 = file_path parse 직후, old_string branch 진입 전 → **legacy path byte-identical 보장**. (5) `lib.rs` / `content_hash.rs` / 다른 file **변경 없음** (LineAnchoredEdit internal, sha2 기존 dep 재사용). **10 test 추가** (8 `#[tokio::test]` + 1 `#[test]` unit + 1 old mode regression): happy_path (6-line file, replace 2..=4, content + metadata 검증) / stale_anchor (hash 캡처 → 외부 write → InvalidInput + 파일 unchanged) / out_of_range / invalid_range / single_line / preserve_trailing_newline (assert read_back.ends_with('\n')) / entire_file / multiline_replacement (replacement='X\nY\nZ' → 3 lines split) + `apply_line_replacement` direct unit + `test_edit_old_mode_still_works` (regression: legacy 'replacements: 2', NO 'mode:line_anchored'). **Spec deviation (1건)**: stale anchor 메시지에 `(current hash XXX, expected YYY)` suffix 추가 — spec 의 `'stale anchor: file modified; re-read with Read tool'` prefix 는 그대로. LLM drift 규모 즉시 판별. **3-way verify PASS**: cargo build clean / cargo clippy --workspace --all-targets -- -D warnings 0 warning (no `#[allow]` 추가, no unsafe) / cargo test --workspace **446 pass + 0 fail + 2 ignored** (D-104 baseline 436 → D-105 tools 62 → 72, 회귀 0). **commit scope**: 1 file / +537 -11 (myharness/crates/tools/src/edit.rs 132 → 658 lines). main = `5e39f5e`. **누적 결정 (2026-07-01)**: 52 + 1 = **53 결정**. **Anti-pattern 준수**: tight range (LLM 책임), keeper line 재입력 ❌, tree-sitter 미도입 (D-106+ 별도).

## 세션 종료 (2026-07-01)

- **상태**: D-105 Edit v2 line_anchored mode 완료 + commit + dual-push 완료. **oh-my-pi Hashline 점진 차용 2차 cycle 종료**.
- **main**: `d52ef78` (코드 `5e39f5e` + 메모리 `d52ef78` 2 commit, 단일 push).
- **누적 결정**: **53** (D-22~D-38 컨셉 + D-42~D-84 + D-96~D-105).
- **build/test 상태**: `cargo test --workspace --lib` = **446 pass + 0 fail + 2 ignored** / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo build --release -p myharness-tools` = clean.
- **Hashline 점진 차용 누적 (Hybrid 안, D-100 9-area roadmap)**: **5 area 점유** (LINE:TEXT Read + content hash + replace N..M). 잔여 4 area = tree-sitter `replace block` 2 + Lark multi-section parser 1 + SnapshotStore 1 (v2).
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (a) D-106+ tree-sitter 도입 (v1.5+, `replace block N`, dep weight ↑ trade-off)
  - (b) D-106+ pure insert/delete (`insert before/after N`, multi-section atomic)
  - (c) A-proper native tool calling (OpenAI/Anthropic wire format, v1.5+, 2-3 commit)
  - (d) D-100 한계 follow-up (큰 file 1000+ lines chunked Read + prompt 보강)
  - (e) TASK-002 도메인 명령 (서버/환경 3-도메인, yklee 인프라 정보 의존)
  - (f) MiniMax OAuth real flow 검증 (console device grant 활성화 대기)
  - (g) TUI shell + interactive mode 검증 (binary `myharness` 만 실행, LoopRunner 통합)
  - (h) 추가 안정화 (cargo tree -d hygiene / 41GB target dir cleanup)
- **외부 blocker 유지**: TASK-002 (yklee 인프라 정보) / MiniMax Device OAuth real flow (console 활성화 대기) / OpenAI/Google OAuth (client_id 등록 대기) / Gitea PAT 회전 (이전 세션 노출 가능성).
- **다음 세션 시작 진입점**: 본 파일 + state.json + work_backlog.md + backlog/2026-07-01.md (4 file 모두 2026-07-01 동기화 완료). D-106+ 진입 결정 대기.

(End of file - total 102 lines)

---

## D-106 진입 시도 + sandbox blocker (2026-07-01 추가)

- **시도**: D-106+ tree-sitter 도입 (oh-my-pi `replace block N`). Rust only v1.5, `block_anchored` 모드 (start_line + expected_hash + replacement), tree-sitter-rust 0.23 으로 line N 의 가장 큰 node resolve → end_line 까지 swap. spec 설계 완료.
- **blocker**: sandbox `workspace-write` 모드에서 `index.crates.io` DNS 차단 → `cargo fetch` 실패. tree-sitter 0.26 / tree-sitter-rust 0.23 둘 다 Cargo.lock 미존재.
- **다음 결정 (yklee, 다음 세션)**: (A) 에스컬레이션 — cargo build sandbox 밖 실행 (1-2 commit, D-106 완) / (B) 외부 cargo fetch 1회로 Cargo.lock 만 확정 / (C) D-106 skip → (b) pure insert/delete 또는 (c) A-proper native tool calling pivot (dep 0) / (D) nexus mirror 설정.
- **현재 main**: `a9b5a63` (변동 없음, working tree clean).
- **누적 결정**: 53 (D-106 미완, 변동 0).
- **상세**: `backlog/2026-07-01.md` §10.

---

## D-107 (2026-07-01 추가) — pure_edit mode

- **oh-my-pi Hashline 점진 차용 4차 cycle (D-107, v1.5+)** — pure insert/delete 의미론. `insert_before` / `insert_after` / `insert_head` / `insert_tail` / `insert_after_block` (D-106 tree-sitter reuse) / `delete N..M` (single = start==end). multi-section atomic: 모든 op 를 line-DESCENDING 으로 정렬 + priority (Delete > After > Before > Head|Tail) → high anchor 가 low anchor shift 안 함.
- **stale-anchor gate**: v1.5 safe-by-default — `expected_hash` 필수 (없으면 reject). line_anchored 와 동급.
- **구현**: `myharness/crates/tools/src/edit.rs` 658 → 1220 lines, +562. PureEdit + PureInsertion enum (serde rename 으로 JSON op tag 보존) + PureDeletion + PendingOp + OpKind + apply_insert_before/after helper + EditTool::execute_pure private async method + dispatch 분기.
- **10 test 추가** (2 unit + 8 tokio): apply_insert_before/after_basic / insert_before / insert_after_head_tail / delete_single_line / delete_range / multi_section_atomic (insert_before line 2 + delete line 4, atomic) / stale_anchor / empty_rejected / missing_hash_rejected.
- **3-way verify**: cargo build clean / cargo clippy --workspace --all-targets -- -D warnings 0 warning (no `#[allow]` 추가, no unsafe) / cargo test --workspace **467 pass + 0 fail + 2 ignored** (D-106 baseline 457 → +10, 회귀 0).
- **main**: `cda6330` (코드 1 commit) + 메모리 commit 단일 push.
- **누적 결정**: 53 (D-106) → **54** (D-107 추가).
- **Hashline 점진 차용 누적**: 6 area → **7 area 점유** (LINE:TEXT Read + content hash + replace N..M + replace block N + insert/delete N..M + multi-section atomic). 잔여 2 area = Lark multi-section parser + SnapshotStore.

## 세션 종료 (2026-07-01 2nd)

- **상태**: D-107 pure_edit mode 완료 + commit + dual-push 완료. **oh-my-pi Hashline 점진 차용 4차 cycle 종료**.
- **main**: `cda6330` (코드) + 메모리 commit (단일 push 2 commit).
- **누적 결정**: **54** (D-22~D-38 + D-42~D-84 + D-96~D-107).
- **build/test 상태**: `cargo test --workspace --lib` = **467 pass + 0 fail + 2 ignored** / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo build -p myharness-tools` = clean.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (c) A-proper native tool calling (OpenAI/Anthropic wire format, v1.5+, 2-3 commit)
  - (d) D-100 한계 follow-up (큰 file 1000+ lines chunked Read)
  - (e) TASK-002 도메인 명령 (yklee 인프라 정보 의존)
  - (f) MiniMax OAuth real flow (console 활성화 대기)
  - (g) TUI shell 검증
  - (h) 추가 안정화 (cargo hygiene)
  - (i) D-108+ Lark multi-section parser (cross-file patch)
  - (j) D-108+ block-aware insert/replace 통합

---

## D-108 (2026-07-01 추가) — A-proper native tool calling

- **v1.5+ 진입 (D-108)** — `CompletionRequest::tools: Vec<ToolSpec>` + `CompletionResponse::tool_calls: Vec<ToolCall>` 추가. LLM 이 structured output 으로 tool call emit 가능. dispatch 우선순위: native (r.tool_calls.is_empty() == false) → text-based (D-100 A-min) → final text.
- **구현** (8 file / +305 / -1):
  - `myharness/crates/llm/src/client.rs` — +ToolSpec (name+description+input_schema) + +ToolCall (id+name+arguments)
  - `myharness/crates/llm/src/client_mock.rs` — +MockResponse::ToolCalls variant (native emit simulation)
  - `myharness/crates/llm/src/client_{anthropic,gemini,openai_compat}.rs` — tool_calls: Vec::new() literal field (wire format 은 D-108 follow-up)
  - `myharness/crates/tui/src/orchestrator.rs` — +tool_specs_for helper + native dispatch path + D-102 dedup native 적용
  - `myharness/crates/{compression,cli}/...` — tools: Vec::new() literal
- **6 test 추가**: native_dispatch_and_continues / request_includes_tools (6 tool name) / native_takes_precedence_over_text / native_repeated_breaks_loop / tool_specs_for_empty / tool_specs_for_default.
- **3-way verify**: build clean / clippy 0 / **473/0/2** (D-107 baseline 467 → +6, 회귀 0).
- **main**: `0771228` (코드 1 commit) + 메모리 commit 단일 push.
- **누적 결정**: 54 → **55** (D-108 추가).
- **Follow-up (D-108 follow-up)**: OpenAI-compat wire format (reqwest 직접 호출, rig-core 0.38.1 의 CompletionRequest 에 tools builder 없음) / Anthropic wire format (content tool_use block parse) / Tool trait description + input_schema method (현재 v1.5 = name only minimal spec).

## 세션 종료 (2026-07-01 3rd)

- **상태**: D-108 A-proper native tool calling 완료 + commit + dual-push 완료. v1.5+ 첫 진입.
- **main**: `0771228` (코드) + 메모리 commit (단일 push 2 commit).
- **누적 결정**: **55** (D-22~D-38 + D-42~D-84 + D-96~D-108).
- **build/test 상태**: `cargo test --workspace --lib` = **473 pass + 0 fail + 2 ignored** / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo build -p myharness-tui` = clean.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (A) D-108 follow-up: OpenAI-compat wire format (reqwest 직접 호출, MiniMax/DeepSeek/Ollama native tool calling)
  - (B) D-108 follow-up: Anthropic wire format (content tool_use block parse)
  - (C) Tool trait description + input_schema (JSON Schema)
  - (d) D-100 한계 follow-up (큰 file chunked Read)
  - (e) TASK-002 도메인 명령 (yklee 인프라 정보 의존)
  - (f) MiniMax OAuth real flow
  - (g) TUI shell 검증
  - (h) 추가 안정화 (cargo hygiene)
  - (i) D-109+ Lark multi-section parser (cross-file patch)
  - (j) D-109+ block-aware insert/replace

---

## D-108 follow-up (2026-07-01 추가) — OpenAI-compat wire format

- **OpenAI-compat wire format 직접 구현 (D-108 follow-up)** — rig-core 0.38.1 의 CompletionsClient builder 가 tools field 미노출. reqwest 로 `{base_url}/chat/completions` 직접 호출. MiniMax / DeepSeek / Ollama / llama.cpp 등 OpenAI-compat provider 전부 native tool calling 가능.
- **구현** (1 file / +396 / -1):
  - `myharness/crates/llm/src/client_openai_compat.rs` — +api_key field + dispatch (`!req.tools.is_empty()` 이면 `complete_wire_format()` 분기) + `complete_wire_format` inherent method (impl OpenAiCompatProvider) + `build_chat_payload` (OpenAI request shape) + `parse_chat_response` (OpenAI response shape) + 6 private struct (ChatResponse / ChatChoice / ChatMessage / ChatToolCall / ChatToolCallFunction / ChatUsage, forward-compat field 에 `#[allow(dead_code)]`)
- **dispatch 우선순위**:
  1. `!req.tools.is_empty()` → wire format path (authoritative)
  2. rig-core path (plain text, byte-identical 기존 동작)
- **8 test 추가**: build_chat_payload (3: includes_tools_and_messages / omits_when_empty / tool_message_call_id) + parse_chat_response (5: plain_text / with_tool_calls / invalid_tool_args / no_choices / mixed_text_and_tool_calls)
- **3-way verify**: build clean / clippy 0 (단 `#[allow(dead_code)]` 3건 forward-compat) / **481/0/2** (D-108 baseline 473 → +8, 회귀 0).
- **main**: `23a205a` (코드 1 commit) + 메모리 commit 단일 push.
- **누적 결정**: 55 → **56** (D-108 follow-up 추가).
- **Follow-up**: (B) Anthropic wire format (content tool_use block) / (C) Tool trait description + input_schema / (D) real MiniMax native E2E (MINIMAX_API_KEY).

## 세션 종료 (2026-07-01 4th)

- **상태**: D-108 follow-up OpenAI-compat wire format 완료 + commit + dual-push 완료.
- **main**: `23a205a` (코드) + 메모리 commit (단일 push 2 commit).
- **누적 결정**: **56** (D-22~D-38 + D-42~D-84 + D-96~D-108 follow-up).
- **build/test 상태**: `cargo test --workspace --lib` = **481 pass + 0 fail + 2 ignored** / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo build -p myharness-llm` = clean.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (B) D-108 follow-up Anthropic wire format (content tool_use block)
  - (C) Tool trait description + input_schema method
  - (D) Real MiniMax native E2E (MINIMAX_API_KEY 주입)
  - (d) D-100 한계 follow-up
  - (e) TASK-002 도메인 명령
  - (f) MiniMax OAuth real flow
  - (g) TUI shell 검증
  - (h) 추가 안정화
  - (i) D-109+ Lark multi-section parser
  - (j) D-109+ block-aware insert/replace

## 세션 종료 (2026-07-01 5th)

- **상태**: D-109 Tool trait description + input_schema 완료 + commit + dual-push 완료.
- **main**: D-109 commit (코드 + 메모리 단일 push 2 commit, hash push 후 확정).
- **누적 결정**: **57** (D-22~D-38 + D-42~D-84 + D-96~D-109).
- **build/test 상태**: `cargo test --workspace --lib` = **489 pass + 0 fail + 2 ignored** / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo build --workspace` = clean.
- **구현 요약** (옵션 C, D-109):
  - `myharness/crates/tools/src/tool.rs` — `Tool` trait 에 `description()` + `input_schema()` default method 추가 (description == "" / input_schema == empty object — 기존 6 tool / 외부 impl 깨지지 않음).
  - 6 tool impl override: `read.rs` / `write.rs` / `edit.rs` / `bash.rs` / `glob_.rs` / `grep.rs` — 각 tool 의 human-readable description + JSON Schema (required + properties) 추가. Edit tool 의 schema 는 3 modes (line_anchored D-105 / block_anchored D-106 / pure_edit D-107) + classic old_string/new_string 모두 surface.
  - `myharness/crates/tools/src/lib.rs` — 7 신규 test (`d109_all_default_tools_declare_description_and_schema` / `d109_read_tool_schema_is_well_formed` / `d109_write_tool_schema_requires_content` / `d109_edit_tool_schema_includes_modes` / `d109_bash_tool_schema_requires_command` / `d109_glob_tool_schema_requires_pattern` / `d109_grep_tool_schema_requires_pattern_and_supports_include`).
  - `myharness/crates/tui/src/orchestrator.rs` — `tool_specs_for` 개선: `reg.get(name)` 으로 `Arc<dyn Tool>` 받아 `description()` + `input_schema()` 호출. 기존 name-only fallback 도 안전망으로 유지. +1 test (`d109_tool_specs_carry_description_and_schema`).
- **wire format effect** (D-108 follow-up 과의 시너지): OpenAI-compat wire format 의 `function.description` + `function.parameters` 가 이제 빈 string/empty object 가 아니라 진짜 tool 메타데이터로 emit. LLM 이 tool 선택 시 더 정확하게 분기 가능.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (B) D-108 follow-up Anthropic wire format (content tool_use block)
  - (D) Real MiniMax native E2E (MINIMAX_API_KEY 주입)
  - (d) D-100 한계 follow-up
  - (e) TASK-002 도메인 명령
  - (f) MiniMax OAuth real flow
  - (g) TUI shell 검증
  - (h) 추가 안정화
  - (i) D-109+ Lark multi-section parser
  - (j) D-109+ block-aware insert/replace

## 세션 종료 (2026-07-01 6th)

- **상태**: D-110 Real MiniMax native E2E test 추가 (옵션 D) — test-only 변경. **이 세션에서 API key 부재로 실제 실행은 다음 세션에 yklee 가 `MINIMAX_API_KEY` 주입 후**.
- **main**: D-110 commit (코드 + 메모리 단일 push 2 commit, hash push 후 확정).
- **누적 결정**: **58** (D-22~D-38 + D-42~D-84 + D-96~D-110).
- **build/test 상태**: `cargo test --workspace --lib` = **490 pass + 0 fail + 3 ignored** (D-109 baseline 489 → +1 ignored, 회귀 0) / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo build --workspace` = clean.
- **구현 요약** (옵션 D, D-110):
  - `myharness/crates/llm/src/client_openai_compat.rs::tests::minimax_real_native_tool_call` — D-108 follow-up + D-109 의 wire format 이 실제 MiniMax API 까지 도달하는지 검증하는 `#[ignore]` test.
  - 검증 단계 5가지: (1) `complete_wire_format` 분기 발동, (2) payload 가 D-109 description + input_schema 정확히 실어 나르는지 dump, (3) 응답 `tool_calls` 비어있지 않음, (4) `tool_calls[0].name == "Read"`, (5) arguments 에 `file_path` 존재.
  - 실행 명령: `MINIMAX_API_KEY=... cargo test -p myharness-llm minimax_real_native_tool_call -- --ignored --nocapture`.
  - API key 부재 시 `eprintln!` + `return` 으로 silently skip → CI 안전.
- **다음 세션 시작 시 yklee 결정**:
  - **(D 실행)**: `export MINIMAX_API_KEY=...` → 위 명령 실행 → 결과 dump → 옵션 (B) / 다음 backlog 로 진행
  - 또는 (B) Anthropic wire format (test-only 가능, API key 불요 — mock server)
  - 또는 (d)~(j) 잔여 backlog

## 세션 종료 (2026-07-01 7th)

- **상태**: **D-110 ignored test real network PASS** — yklee 가 `MINIMAX_API_KEY` 주입 후 1-shot 실행 → 5단계 assertion 모두 통과. D-108 follow-up + D-109 의 wire format 이 end-to-end 로 verified.
- **main**: D-111 commit (메모리 only, hash push 후 확정).
- **누적 결정**: **59** (D-22~D-38 + D-42~D-84 + D-96~D-111).
- **실행 결과**:
  ```
  $ MINIMAX_API_KEY=... cargo test -p myharness-llm minimax_real_native_tool_call -- --ignored --nocapture
  MiniMax native response: model=MiniMax-M3 content="<think>...</think>" tool_calls=1
    call[0]: id=call_019f1b96d53f7f21b04dca15 name=Read args={"file_path":"/tmp/ping.txt"}
  test client_openai_compat::tests::minimax_real_native_tool_call ... ok
  test result: ok. 1 passed; 0 failed; 0 ignored
  ```
- **검증 단계 5가지 모두 PASS**:
  1. `complete_wire_format` 분기 발동 (real HTTP POST `https://api.minimax.io/v1/chat/completions` 도달)
  2. payload 가 D-109 description + input_schema 정확히 전달 (LLM 이 spec 이해)
  3. `tool_calls` 1개 emit (non-empty)
  4. `tool_calls[0].name == "Read"` (LLM 이 올바른 tool 선택)
  5. `arguments.file_path == "/tmp/ping.txt"` (LLM 이 prompt 의 path 정확히 추출)
- **의미**: D-108 follow-up (OpenAI-compat wire format native tool calling) + D-109 (Tool description + input_schema) 의 full chain 이 실증됨. MiniMax / DeepSeek / Ollama / llama.cpp 등 모든 OpenAI-compat provider 에 대해 native tool calling 이 동작.
- **다음 세션 시작 시 yklee 결정**:
  - (B) Anthropic wire format (test-only 가능, mock server)
  - (d) D-100 한계 follow-up
  - (e) TASK-002 도메인 명령
  - (f) MiniMax OAuth real flow
  - (g) TUI shell 검증
  - (h) 추가 안정화
  - (i) D-109+ Lark multi-section parser
  - (j) D-109+ block-aware insert/replace

## 세션 종료 (2026-07-01 8th)

- **상태**: **D-112 Read tool auto-truncation + has_more/next_offset hint 완료** — yklee 옵션 (d) D-100 한계 follow-up 진행. 1MB cap 유지 + chunked Read 를 tool layer 로 enforce.
- **main**: D-112 commit (코드 + 메모리 단일 push 2 commit, hash push 후 확정).
- **누적 결정**: **60** (D-22~D-38 + D-42~D-84 + D-96~D-112).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** (no `#[allow]` 추가) / `cargo test -p myharness-tools --lib` = **13/13 read test PASS (기존 9 + 신규 4)**, tools crate 전체 **105/105 pass** / 회귀 0. (workspace 전체 test 는 auth crate 의 7개 test 가 sandbox xdg-open env issue 로 pre-existing fail — D-112 와 무관, 다음 sandbox 외부 실행에서 PASS 확인 가능.)
- **구현 요약** (옵션 d, D-112):
  - `myharness/crates/tools/src/read.rs` (291 → ~395 lines, +47 -20 net) — `DEFAULT_READ_LINE_LIMIT=500` / `MAX_READ_LINE_LIMIT=5_000` 2 const 추가, `limit` clamp 로직 (`Some(0) | None → 500` / `Some(n) → min(n, 5000)`), `emitted = limit.min(available_after_offset)`, `has_more = (offset + emitted) < total_lines`, metadata 에 `limit` + `has_more` + `has_more=true` 일 때만 `next_offset` 노출. description + input_schema 의 `limit` description 도 sync.
  - **4 신규 test**: `test_d112_auto_truncates_large_file` (1000-line → 500 truncate, line 501 미포함, has_more=true, next_offset=500) / `test_d112_clamps_excessive_limit` (7000-line + limit 10000 → 5000 clamp) / `test_d112_no_truncation_signals_correctly` (10-line file → has_more=false, next_offset 부재) / `test_d112_limit_zero_uses_default` (defensive) + `test_d112_paginated_walk_through_large_file` (1500-line 3-chunk walk 검증).
- **영향**:
  - D-103 prompt 의 `>500 lines 는 offset+limit` 권고가 tool layer 로 enforce → prompt 변경 시 description 만 sync.
  - LLM 이 1000+ line 파일을 한 번에 요청해도 자동 truncate → large file code review 의 무한 루프 위험 (D-102 dedup 과 결합) 자동 차단.
  - `has_more` / `next_offset` 가 LLM 에게 next chunk 의 정확한 위치 (`offset: 500, limit: 500`) 알려줌 → chunked walk 의 LLM prompt 부담 0.
  - **1MB cap + 5000 line hard cap 유지** — binary-ish 큰 file, maliciously large `limit` 요청 모두 방어.
- **한계**:
  - 1MB cap 은 그대로 (binary 같은 큰 file 거부) — D-100 follow-up 의 (a) cap 완화는 별도 결정.
  - 5000 line cap 도 매우 큰 file (10000+ lines) 은 chunked walk 필요 — D-112 가 walk 자체는 가능케 함.
  - Lark parser / block-aware insert/replace / Anthropic wire format 등 v1.5+ 영역 미착수.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (B) Anthropic wire format (test-only 가능, mock server)
  - (e) TASK-002 도메인 명령 (yklee 인프라 정보 의존)
  - (f) MiniMax OAuth real flow (console 활성화 대기)
  - (g) TUI shell 검증
  - (h) 추가 안정화 (cargo hygiene)
  - (i) D-109+ Lark multi-section parser
  - (j) D-109+ block-aware insert/replace

## 세션 종료 (2026-07-01 9th)

- **상태**: **D-113 Real MiniMax OAuth Device flow client-side E2E PASS** — 옵션 f 진행. `minimax_real_device_request_code` ignored test 가 real MiniMax API 에 도달 → 6단계 assertion 모두 통과.
- **main**: D-113 commit (코드 + 메모리 단일 push 2 commit, hash push 후 확정).
- **누적 결정**: **61** (D-22~D-38 + D-42~D-84 + D-96~D-113).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **494 pass + 0 fail + 4 ignored** (D-110 + D-113 = 2 신규 ignored, D-113 real network PASS) / auth 7 test 회복 (sandbox 외부 환경에서).
- **발견 (D-113 핵심)**:
  - `https://api.minimax.io/oauth/code` → **307 redirect** → `https://account.minimax.io/oauth2/device/code` (RFC 8628 Device Authorization Grant 표준 endpoint)
  - PKCE `code_challenge` (S256) 필수 — 미제출 시 400 `invalid_request: code_challenge is required`
  - 응답: `user_code` (XXXX-XXXX 9자), `verification_uri=https://platform.minimax.io/oauth-authorize?user_code=...&client=OpenClaw`, `interval=3000ms`, `expired_in=<future epoch ms>`
  - Token endpoint: `https://account.minimax.io/oauth2/token` 도달 확인 (fake device_code → `invalid_grant` 정상)
- **구현 요약** (옵션 f, D-113):
  - `myharness/crates/auth/src/device_flow.rs::tests::minimax_real_device_request_code` — `#[ignore = "requires real network access to api.minimax.io (D-113)"]` 1 test 추가. `request_code(&MinimaxDeviceOAuth::from_env())` 호출 → user_code/verification_uri/interval/expired_in 검증. **API key 불요** (Device flow un-authed). 네트워크/방화벽 fail 시 `eprintln!` + early return → CI 안전.
  - 1 file / +115 lines (test only) / production code 0 변경.
- **Endpoint 갱신 결정 보류** (v1.5+):
  - production `MinimaxDeviceOAuth::code_endpoint()` / `token_endpoint()` 가 여전히 `https://api.minimax.io/oauth/{code,token}` 인데, 307 redirect 자동 follow 로 production 동작은 OK.
  - 명시적 endpoint 변경 (`account.minimax.io/oauth2/{device/code,token}` 직접 hit) 은 v1.5+ 에서 결정. **이유**: (a) `client=OpenClaw` 식별자가 응답에 박혀있어 client_id 가 OpenClaw 와 공유되는지 정책 검증 필요, (b) PKCE `code_challenge_method=S256` 사용은 spec RFC 8628 standard — `device_flow.rs` 가 이미 PKCE 적용 중 (확인 필요), (c) base_url override 가 동작하는지 e2e 검증 필요.
  - **즉시 후속 결정 (D-114 후보)**: production `MinimaxDeviceOAuth` 의 `code_endpoint` / `token_endpoint` 를 `account.minimax.io/oauth2/device/code` / `account.minimax.io/oauth2/token` 으로 명시 변경 + `device_flow.rs` 의 `request_code` 가 `code_challenge` + `code_challenge_method=S256` 정확히 emit 하는지 검증 + mock server test 갱신.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (f-1) **D-114 production endpoint 갱신** (위 결정 보류 사항 즉시 해소, 1 commit)
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (g) TUI shell 검증
  - (h) 추가 안정화
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace

## 세션 종료 (2026-07-01 10th)

- **상태**: **D-114 MinimaxDeviceOAuth endpoint URL 갱신 완료** — 옵션 f-1. production `MinimaxDeviceOAuth` 가 `https://account.minimax.io/oauth2/{device/code,token}` 직접 hit.
- **main**: D-114 commit (코드 + 메모리 단일 push 2 commit, hash push 후 확정).
- **누적 결정**: **62** (D-22~D-38 + D-42~D-84 + D-96~D-114).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **494 pass + 0 fail + 4 ignored** (회귀 0) / D-113 real network 재실행 **PASS** (1-hop, redirect 없음).
- **구현 요약** (옵션 f-1, D-114):
  - `myharness/crates/auth/src/provider.rs` — `MinimaxDeviceOAuth::from_env()` 의 default base_url 변경 (`https://api.minimax.io` → `https://account.minimax.io`, CN 도 `https://api.minimaxi.com` → `https://account.minimaxi.com`). `code_endpoint()` / `token_endpoint()` 가 새 path (`/oauth2/device/code` / `/oauth2/token`) emit.
  - `myharness/crates/auth/src/device_flow.rs` — module doc + D-113 test doc 갱신 (endpoint URL + D-114 follow-up 메모).
  - 0 신규 test, 0 mock test 갱신 (mock server 는 endpoint 를 직접 주입받으니 영향 없음).
- **Deferred (D-115/116 후보, 다음 세션 결정)**:
  - **D-115**: `base_resp.status_code==0` 처리. real `MiniMax` API 의 성공 응답은 `status: "success"` 필드 없이 `base_resp.status_code=0` 만 포함. 현재 우리 코드는 `status: "success"` 만 accept → `poll_token` 의 real 사용 시 `TokenPoll::Error("unknown status:")` 로 떨어질 위험. real flow 완전 동작 위해 필요.
  - **D-116**: `expired_in` / `interval` 의 ms/초 통일. real API 는 epoch ms (`expired_in=1782880012212`), 우리 `DeviceAuthorization::expired_in: u64` / `interval: u64` 는 초 가정. real flow 의 `OAuthToken::is_expired` 가 1000배 어긋남. manager.rs 도 같이 갱신 필요.
  - **D-117 (optional)**: `response_type=code` 파라미터 제거. real spec 에 없음 (무시되긴 함). mock test 호환성 위해 유지.
- **D-113 doc 갱신**: endpoint 히스토리 추가 (D-113 → D-114 path). `verification_uri` 의 `client=OpenClaw` 식별자 메모 보존.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (f-2) **D-115 base_resp 처리** (real poll_token 동작 위해)
  - (f-3) **D-116 ms/초 통일** (real token expire check 위해)
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (g) TUI shell 검증
  - (h) 추가 안정화
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace

## 세션 종료 (2026-07-01 11th)

- **상태**: **D-115 base_resp envelope 처리 완료** — 옵션 f-2, D-114 의 deferred 1순위. `request_code` + `poll_token` 의 응답 envelope 분기 + mock spec 일치 + 6 신규 test.
- **main**: D-115 commit (코드 + 메모리 단일 push 2 commit, hash push 후 확정).
- **누적 결정**: **63** (D-22~D-38 + D-42~D-84 + D-96~D-115).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** (let-chain 으로 collapsible_if 회피) / `cargo test --workspace --lib` = **500 pass + 0 fail + 4 ignored** (D-114 baseline 494 → +6, 회귀 0) / D-113 real network 재실행 **PASS** (real 응답의 `base_resp.status_code=0` 분기 그대로 통과).
- **구현 요약** (옵션 f-2, D-115):
  - `myharness/crates/auth/src/device_flow.rs`:
    - helper `base_resp_status_code(value) -> Option<i64>` 추가 (`{base_resp: {status_code: N}}` → `Some(N)`, 부재 시 `None`).
    - `request_code`: HTTP 200 + `base_resp.status_code != 0` → `DeviceError::Provider(format!("oauth/code base_resp status_code={code} status_msg={msg}"))`. legacy `status: "error"` 분기는 유지.
    - `poll_token`: HTTP 200 + `base_resp.status_code != 0` → `TokenPoll::Error(format!("oauth/token base_resp status_code={code} status_msg={msg}"))`. legacy `status: "error"` 분기는 유지. **양쪽 envelope 모두 accept** (mock test 의 `{base_resp:0, status:success}` 동시 emit 도 정상).
    - **6 신규 test**: 3 unit (`d115_base_resp_status_code_zero` / `_nonzero` / `_absent`) + 3 e2e (`d115_request_code_base_resp_zero_succeeds` / `_nonzero_errors` / `d115_poll_token_base_resp_nonzero_errors`).
  - `myharness/crates/auth/src/manager.rs`:
    - mock server test 응답 4개 (2개 test × 2 endpoint: code + token) 에 `base_resp.status_code=0` + `status_msg="success"` envelope 추가. legacy `status:"success"` 동시 emit (양쪽 envelope accept 회귀 0).
- **효과**:
  - real `MiniMax` API 의 `base_resp.status_code != 0` 실패 시나리오 즉시 분기 (legacy `status:"error"` 가 없는 real 응답도 처리).
  - token 단계 (`poll_token`) 의 real polling 도 정상 — `40001 invalid_client` / `40004 invalid_grant` 등 `base_resp.status_code` 로 emit 된 실패를 즉시 `TokenPoll::Error` 로 surface.
  - D-115 spec 과 mock server 일치 (production spec = mock spec = 1:1).
  - D-113 real test 그대로 PASS (real `MiniMax` 응답의 `base_resp.status_code=0` 분기 통과).
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (f-3) **D-116 ms/초 통일** — real API 의 `expired_in` / `interval` 이 epoch ms 인데 우리 `DeviceAuthorization` / `manager.rs` 는 초 가정. real token expire check 1000배 영향 fix.
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (g) TUI shell 검증
  - (h) 추가 안정화
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace

## 세션 종료 (2026-07-01 12th)

- **상태**: **D-116 DeviceAuthorization 단위 contract 명시 완료** — 옵션 f-3, D-114 의 deferred 2순위. `expired_in` / `interval` 단위를 **ms** 로 일원화 + doc-comment 명시 + mock spec 갱신.
- **main**: D-116 commit (코드 + 메모리 단일 push 2 commit, hash push 후 확정).
- **누적 결정**: **64** (D-22~D-38 + D-42~D-84 + D-96~D-116).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **502 pass + 0 fail + 4 ignored** (D-115 baseline 500 → +2, 회귀 0) / D-113 real network 재실행 **PASS** (`interval=3000` ms invariant 그대로 통과).
- **구현 요약** (옵션 f-3, D-116):
  - `myharness/crates/auth/src/device_flow.rs`:
    - doc-comment 4 곳 단위 contract 명시: `DeviceAuthorization::expired_in` (ms unix timestamp) / `::interval` (ms) / `TokenPoll::Success::expired_in` (ms) / `DeviceToken::expired_in` (ms).
    - `default_interval()` 변경: `2` → `2_000` (2초 = 2000ms).
    - `poll_until_success(interval_ms, expired_in_unix)` 시그니처: `interval: u64` → `interval_ms: u64` + 내부에서 `(interval_ms / 1000).clamp(1, 10)` 으로 seconds 변환. mock test 의 `interval=1_000` (1초) / real `MiniMax` 의 `interval=3_000` (3초) 모두 정상 sleep.
    - **2 신규 invariant test**: `d116_interval_is_milliseconds_unit` (3000 ms invariant + 1-60초 범위) / `d116_expired_in_is_milliseconds_unix_timestamp` (1e12+ ms + now+5y cap).
  - `myharness/crates/auth/src/manager.rs`:
    - mock server test 응답 spec 갱신 (2 test × code + token): `interval=1` → `1000` / `expired_in=now+60` → `now+60_000` / `expired_in=now+3600` → `now+3_600_000`.
    - mock test assertion 2곳: `req.authorization.interval == 1` → `1_000`.
- **안전망**: `expired_in_to_chrono` (W14.7) 의 ms/μs/s auto-detect 로직 그대로 유지 — production 안전망. token 단계의 mixed-unit 응답도 흡수.
- **효과**:
  - mock spec = real spec (ms) — `poll_until_success` 의 `interval_ms / 1000` 변환으로 통일.
  - D-116 invariant test 가 추후 spec 변경 (e.g. seconds 전환) 시 즉시 fail.
  - doc-comment 4 곳 단위 명시 → 새 contributor 의 혼란 방지.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (f-4) **D-117 response_type 제거** (optional, mock test 호환성 위해 유지 — v1.5+)
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (g) TUI shell 검증
  - (h) 추가 안정화
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace

## 세션 종료 (2026-07-01 13th)

- **상태**: **D-117 request_code response_type=code 제거 완료** — 옵션 f-4, D-114 의 deferred 3순위 (optional). Real `MiniMax` Device Authorization Grant spec 에 `response_type=code` 미포함.
- **main**: D-117 commit (코드 + 메모리 단일 push 2 commit, hash push 후 확정).
- **누적 결정**: **65** (D-22~D-38 + D-42~D-84 + D-96~D-117).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **503 pass + 0 fail + 4 ignored** (D-116 baseline 502 → +1, 회귀 0) / D-113 real network 재실행 **PASS**.
- **구현 요약** (옵션 f-4, D-117):
  - `myharness/crates/auth/src/device_flow.rs::request_code` form body 에서 `("response_type", "code".to_string())` 제거. doc-comment 추가 (D-114 deferred note 의 "Authorization Code + redirect flow 표준 param" + "real `MiniMax` API 가 무시" 명시).
  - **1 신규 test**: `d117_request_code_form_body_omits_response_type` — mock server 가 form body 를 capture 후 (1) `response_type` 미포함 + (2) PKCE/state/client_id/scope sanity 5가지 assert. `tokio::sync::Mutex` 로 body capture, `CRLF_CRLF` separator 로 HTTP wire body portion 정확히 분리.
  - mock test (W14 7개) 회귀 0 — JSON body decode 가 `response_type` 무관.
- **scope 명확화**:
  - D-117 = `device_flow.rs:160` 의 `response_type` 만. `flow.rs:121` (Authorization Code + redirect 표준) 은 손대지 않음. `manager.rs:377` 의 `login_non_interactive_returns_url_only` assertion 은 `MinimaxOAuth` (Authorization Code path) 의 URL 검증 → 무관.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - **D-114 의 deferred list 전부 해소** (D-115/116/117 ✓ 모두 완료)
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (g) TUI shell 검증
  - (h) 추가 안정화
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace

## 세션 종료 (2026-07-01 14th)

- **상태**: **D-118 TUI shell render snapshot-style test 4종 추가 완료** — 옵션 g, D-114~117 cycle 종료 후 다음 사이클. TUI `app.rs::tests` 에 회귀 가드 강화.
- **main**: D-118 commit (코드 + 메모리 단일 push, hash push 후 확정).
- **누적 결정**: **65 + D-118 = 66** (decision_count handoff SSOT 갱신).
- **build/test 상태**: `cargo build -p myharness-tui` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **507 pass + 0 fail + 4 ignored** (D-117 503 → +4, 회귀 0) / tui crate 75 → 79.
- **구현 요약** (옵션 g, D-118):
  - `myharness/crates/tui/src/app.rs::tests` 에 4개 신규 test:
    1. `d118_render_renders_all_role_prefixes` — 5 role (`[sys]`/`[you]`/`[bot]`/`[tool]`/`[err]`) 의 prefix + 본문이 모두 80x24 buffer 에 렌더되는지 검증
    2. `d118_render_header_includes_title_and_mode` — 상단 헤더 line 의 `title` + `[mode]` 태그 text 존재 확인
    3. `d118_render_status_reflects_message_count` — status line `N msg` 카운트 정확성 (welcome 1 + push 4 = 5)
    4. `d118_draw_and_render_to_buffer_agree_on_text` — `Terminal::draw` 경로와 `render_to_buffer` 직접 경로의 text 출력 동등성 (한쪽만 깨지는 회귀 감지)
  - helper: `fn buffer_text(buf: &Buffer) -> String` — cell symbol 평문화.
  - 회귀 0 — 기존 9개 test (입력/렌더/keymap/엔터/CtrlC 등) 모두 영향 없음.
- **scope 명확화**:
  - D-118 = TUI render test 만. `app.rs` 의 production code (draw, render_to_buffer, AppKey, App state) 는 손대지 않음.
  - Option B (SubAgent dispatch 통합 test), Option C (AppKey 엣지케이스) 는 미수행.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (h) 추가 안정화
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace
  - (g-추가) TUI 검증 보강 — SubAgent dispatch 통합 test 또는 AppKey 엣지케이스

## 세션 종료 (2026-07-01 15th)

- **상태**: **D-119 workspace-level lints baseline 추가 완료** — 옵션 h-1, cargo hygiene.
- **main**: D-119 commit (코드 + 메모리 단일 push, hash push 후 확정).
- **누적 결정**: **66 + D-119 = 67** (decision_count handoff SSOT 갱신).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** (새 lint 6개로 인한 새 warning 0개) / `cargo test --workspace --lib` = **507 pass + 0 fail + 4 ignored** (D-118 507 → 동, 회귀 0).
- **구현 요약** (옵션 h-1, D-119):
  - `myharness/Cargo.toml` 에 `[workspace.lints.rust]` + `[workspace.lints.clippy]` baseline 추가 (Cargo 1.74+ 정식).
  - 6개 lint 모두 `warn` (강한 deny/forbid 는 의도적 opt-in):
    - rust: `unsafe_code`, `missing_debug_implementations`, `rust_2018_idioms`
    - clippy: `module_name_repetitions`, `needless_pass_by_value`, `redundant_closure_for_method_calls`
  - 8 crate 가 자동 상속 (각 crate Cargo.toml 에 [lints] 없음).
  - 모든 lint 가 안전한 baseline — 기존 코드에 새 warning 0개. clippy -D warnings 통과.
- **scope 명확화**:
  - D-119 = workspace-level lints 추가만. `unsafe_code = "deny"` 같은 강한 정책은 의도적 opt-in (현재 add_local.rs 에 26+ unsafe {env::set_var} 존재, 전부 test code).
  - h-2 (cargo fmt 일괄 적용) 은 **별도 사이클로 분리** — 현재 511 line drift 존재, large commit (수십 KB), review 부담 큼. D-120+ 로 분리 결정.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (h-2) cargo fmt 일괄 적용 (511 line diff) — 별도 사이클
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace
  - (g-추가) TUI SubAgent dispatch 또는 AppKey 엣지케이스

## 세션 종료 (2026-07-01 16th)

- **상태**: **D-120 cargo fmt 일괄 적용 완료** — 옵션 h-2, h scope 종료.
- **main**: D-120 commit (코드 + 메모리 단일 push, hash push 후 확정).
- **누적 결정**: **67 + D-120 = 68** (decision_count handoff SSOT 갱신).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **507 pass + 0 fail + 4 ignored** (D-119 507 → 동, 회귀 0) / `cargo fmt --check` = **0 diff** (drift 해소).
- **구현 요약** (옵션 h-2, D-120):
  - `cargo fmt` 일괄 적용 → **67 files / +1980 / -948** (rustfmt whitespace reformat only).
  - production code 의미 0 변경. 모든 변경은 rustfmt 의 stable formatting (struct literal multi-line, function arg multi-line, closure 위치, 등) 표준화.
  - `cargo fmt --check` 0 diff — drift 완전 해소.
- **scope 명확화**:
  - D-120 = rustfmt 일괄 적용만. 8 crate 모두 영향. 가장 큰 사이즈 (1980 lines 추가, 948 제거) 의 single commit.
  - 의미 변경 0 — review 부담은 줄 수는 없지만 git blame 보존 (large commit 으로 흡수).
- **h scope 종료**:
  - h-1 (D-119, workspace lints baseline) ✓
  - h-2 (D-120, cargo fmt 일괄) ✓
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace
  - (g-추가) TUI SubAgent dispatch 또는 AppKey 엣지케이스

## 세션 종료 (2026-07-01 17th)

- **상태**: **D-121 TUI SubAgent dispatch 통합 test 4종 추가 완료** — 옵션 g-추가, TUI 검증 보강.
- **main**: D-121 commit (코드 + 메모리 단일 push, hash push 후 확정).
- **누적 결정**: **68 + D-121 = 69** (decision_count handoff SSOT 갱신).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **511 pass + 0 fail + 4 ignored** (D-120 507 → +4, 회귀 0) / tui crate 79 → 83.
- **구현 요약** (옵션 g-추가, D-121):
  - `myharness/crates/tui/src/orchestrator.rs::tests` 모듈에 4개 test 추가:
    1. `d121_dispatch_prefix_each_subagent` — 6개 prefix (`code review`/`code implement`/`code refactor`/`env diagnose`/`git `/`git-operator`) 각각이 정확한 `SubAgentKind` + `DispatchKind::Direct` + `extracted_input` 으로 라우팅
    2. `d121_dispatch_domain_keyword_fallback` — prefix 매칭 실패 후 `code_kw` → `CodeReviewer`, `env_kw` → `EnvDiagnose`, `git_kw` → `GitOperator` (각각 `DispatchKind::DomainKeyword`)
    3. `d121_dispatch_default_fallback` — `Default` 분기 (`hello world`, `!!! ? ?`)
    4. `d121_subagent_registry_4_unique` — `SubAgentRegistry::all()` 4개 + `for_kind` 4종 모두 `Some` + `by_domain` 분류 2/1/1 (Code/Environment/Utility) + `by_name` 동작
  - 회귀 0. production code 무변경.
- **scope 명확화**:
  - D-121 = orchestrator dispatch + registry 통합 test 만. `SubAgent::run` (LLM 호출) 은 mock LLM 필요 → 미수행.
  - 옵션 (2) `AppKey::from_crossterm` 엣지케이스 보강은 미수행 (기존 5개 test + Release 이벤트 자연 reject 으로 충분).
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (B) Anthropic wire format
  - (e) TASK-002 도메인 명령
  - (i) Lark multi-section parser
  - (j) block-aware insert/replace

## 세션 종료 (2026-07-01 18th)

- **상태**: **D-122 Anthropic wire format tool_use block parse + mock server test 완료** — 옵션 B. 옵션 i (Lark) 보류 후 진행.
- **main**: D-122 commit (코드 + 메모리 단일 push, hash push 후 확정).
- **누적 결정**: **69 + D-122 = 70** (decision_count handoff SSOT 갱신).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **512 pass + 0 fail + 4 ignored** (D-121 511 → +1, 회귀 0) / llm crate 128 → 131.
- **구현 요약** (옵션 B, D-122):
  - `myharness/crates/llm/src/client_anthropic.rs`:
    - `AnthropicProvider` 에 `api_key: String`, `base_url: String` field + `with_base_url()` builder 추가.
    - `complete()` 진입 시 `req.tools.is_empty()` 아니면 `complete_wire_format()` 으로 분기.
    - `complete_wire_format`: reqwest 직접 POST `{base_url}/v1/messages` (x-api-key, anthropic-version: 2023-06-01). tool spec 은 Anthropic native shape (`{name, description, input_schema}`) 그대로 wire.
    - `parse_anthropic_response`: `serde_json::Value` content[] 를 `text` / `tool_use` / `other` variant 로 deserialize. `tool_use` → `ToolCall { id, name, arguments }`.
    - rig-core native path 도 `extract_tool_calls()` helper 로 `AssistantContent::ToolCall` → `ToolCall` 변환 (양 path 동일 shape).
  - **1 신규 mock server test** `d122_wire_format_parses_tool_use_block`: TcpListener mock 이 text + tool_use 동시 emit → 우리 parser 가 content 분리 + tool_call (id, name, arguments) 정확히 파싱 검증.
- **scope 명확화**:
  - D-122 = hand-rolled wire format + tool_use parse + 1 mock test. **anthropic-version `2023-06-01` 고정** (현재 stable). system/top-level 처리, tool_result user content[] 매핑, usage 파싱 모두 포함.
  - 옵션 i (Lark) 은 source code 에 Lark 가 부재 (handoff 메모리에만 언급) — 의미 정의 필요. 보류 결정.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (e) TASK-002 도메인 명령
  - (j) block-aware insert/replace
  - (B-추가) Anthropic streaming / system prompt injection / tool_result tool_use_id 매핑 정확성 추가 test
  - (i) Lark multi-section parser — 의미 정의 후 재개

## 세션 종료 (2026-07-01 19th)

- **상태**: **D-123 pure_edit replace_block op 추가 완료** — 옵션 j.
- **main**: D-123 commit (코드 + 메모리 단일 push, hash push 후 확정).
- **누적 결정**: **70 + D-123 = 71** (decision_count handoff SSOT 갱신).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **514 pass + 0 fail + 4 ignored** (D-122 512 → +2, 회귀 0) / tools 105 → 107.
- **구현 요약** (옵션 j, D-123):
  - `myharness/crates/tools/src/edit.rs`:
    - `PureInsertion::ReplaceBlock { line, content }` variant 추가 (serde rename: `replace_block`).
    - `OpKind::Replace { start, end, content: &'a str }` 추가. priority = 0 (Delete 와 동순위).
    - `pure_edit` dispatch: arm pattern `content` 가 outer `content` (file string) 가리는 scope shadow 발견 → arm field 를 `replacement` 로 rename + `resolve_block_span(&content, *line)` 로 원본 file 에서 block END resolve.
    - apply: `apply_line_replacement(&new_content, start, end, content)` 호출 + `applied` 에 `replace_block` op 기록.
    - validation: AfterBlock + ReplaceBlock 둘 다 Rust 한정 (block_ops gather + error message 갱신).
  - **2 신규 test**:
    1. `d123_replace_block_replaces_entire_fn` — `fn foo` 전체 5줄을 3줄로 교체 (`let x = 1; let y = 2; x + y }` 제거, `100` 로 body 단일화).
    2. `d123_replace_block_in_pure_edit_multi_op` — `use std::fmt;` insert_before + `fn greet` 전체 replace_block 동시. multi-section atomic, line-descending sort.
- **scope 명확화**:
  - D-123 = `replace_block` 만. `insert_before_block` 는 미수행 (handoff 옵션 A: 1 op 1 commit 으로 결정).
  - 발견한 `AfterBlock` 의 동일 scope-shadow bug 도 본 D-123 에서 같이 fix (같은 arm pattern 구조, 회귀 0). 원래는 `resolve_block_span(content, *line)` 호출 — file content 가 아닌 replacement 에서 resolve 하려 시도 → 우리 fix 에서 `&content` 로 outer file 전달.
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (e) TASK-002 도메인 명령
  - (j-추가) `insert_before_block` op (mirror of `AfterBlock`)
  - (B-추가) Anthropic streaming / tool_result 매핑 추가 test
  - (i) Lark multi-section parser — 의미 정의 후 재개

## 세션 종료 (2026-07-01 20th)

- **상태**: **D-124 pure_edit insert_before_block op 추가 완료** — 옵션 j-추가, AfterBlock 의 짝 + AfterBlock scope-shadow bug fix.
- **main**: D-124 commit (코드 + 메모리 단일 push, hash push 후 확정).
- **누적 결정**: **71 + D-124 = 72** (decision_count handoff SSOT 갱신).
- **build/test 상태**: `cargo build --workspace` = clean / `cargo clippy --workspace --all-targets -- -D warnings` = **0 warning** / `cargo test --workspace --lib` = **516 pass + 0 fail + 4 ignored** (D-123 514 → +2, 회귀 0) / tools 107 → 109.
- **구현 요약** (옵션 j-추가, D-124):
  - `myharness/crates/tools/src/edit.rs`:
    - `PureInsertion::BeforeBlock { line, content }` variant 추가 (serde rename: `insert_before_block`).
    - dispatch: `resolve_block_span(&content, *line)` 로 block resolve → anchor = `*line` (caller 가 start_line 명시) + `OpKind::Before(replacement.as_str())`.
    - **bug fix**: AfterBlock 의 scope-shadow bug (D-123 코멘트에서 "구분" 이라 적었지만 실제로 fix 안 됨) → arm pattern `content` 를 `replacement` 로 rename + `resolve_block_span(&content, *line)` 로 원본 file resolve.
    - `block_ops` filter 에 BeforeBlock 추가 + validation message 갱신 ("insert_after_block / insert_before_block / replace_block").
  - **2 신규 test**:
    1. `d124_insert_before_block_inserts_at_block_start` — `fn bar` 직전에 `use std::fmt;` insert. anchor line 5 (fn 시작점) 사용 — line 6 (indent) 은 fail.
    2. `d124_insert_before_block_with_replace_block` — `insert_before_block` + `replace_block` 동시. 같은 block 의 start/end anchor 가 양쪽에서 쓰여도 line-descending sort 가 안전 적용.
- **scope 명확화**:
  - D-124 = `insert_before_block` op + AfterBlock bug fix. 6종 insert op (Before/After/Head/Tail/AfterBlock/BeforeBlock/ReplaceBlock 7종이 맞음).
  - `pure_edit.insertions` 가 6종 → **7종** 으로 확장 (BeforeBlock 추가).
- **다음 세션 시작 시 yklee 결정 옵션**:
  - (e) TASK-002 도메인 명령
  - (B-추가) Anthropic streaming / tool_result 매핑 추가 test
  - (i) Lark multi-section parser — 의미 정의 후 재개

## 2026-07-01 D-125 myharness `--mode=orchestrator` `unknown mode` 회귀 복구

- **증상**: `./target/release/myharness --mode=orchestrator` → `unknown mode: orchestrator` 즉시 종료.
- **원인**: D-83 follow-up (732d6eb) 에서 `"orchestrator" | "single" =>` arm 을 `"cli" =>` 로 교체하면서 모드 분기 2종 누락. 모듈 doc 주석도 dead reference.
- **수정**: `myharness/crates/cli/src/main.rs` L279 `"cli" =>` → `"orchestrator" | "single" =>`, L189 stale 주석 정정.
- **검증 (3-way)**: cargo build --workspace ✅ / cargo clippy -D warnings ✅ 0 warning / cargo test --workspace --lib ✅ 516 pass + 0 fail + 4 ignored (회귀 0) / release binary --version ✅ / `ask "ping"` ✅ MiniMax 응답 / `--mode=orchestrator|single` (비-TTY) ✅ TtyGuard ENXIO 정상 실패 (`unknown mode` 메시지 사라짐).
- **결정 ID**: **D-125** (누적 73). main = D-125 commit (메모리+코드 단일 push).
- **다음 (yklee 결정 시)**: 동일 1순위 후보 유지 — (a) D-106+ tree-sitter / (b) D-106+ pure insert/delete / (c) A-proper native tool calling OpenAI·Anthropic wire / (d) D-100 chunked Read follow-up / (e) TASK-002 도메인 명령 / (f) MiniMax OAuth real flow / (g) TUI shell + interactive mode 검증 / (h) cargo hygiene / (i) Lark multi-section parser / (j) D-109+ block-aware insert/replace.

---

## TASK-004 재방문 (D-127~D-133, 2026-08-14) — 7 reference v2 영향 분석

### 1. 환경

- **worktree**: main (clean, after 14 commit push — 7 reference commit + 7 merge commit)
- **시작**: 사용자 "세월이 많이 지나서 레퍼런스들이 발전을 많이 했어. 다시 조사하자"
- **선택된 옵션 (yklee)**: (1) 전체 7-doc 재조사 + 결론 갱신

### 2. 작업 D-127~D-133

#### 2.1 06-09 이후 발전량 (실측)

| reference | 새 commit 수 (06-09~08-14, 66일) | default branch | 최신 release/tag | 비고 |
|---|---|---|---|---|
| opencode | 1,457 | `dev` | v1.18.18 | TUI 안정화 + 모델 cache fix |
| aider | 0 | `main` | v0.86.3.dev | 안정 (정직) |
| codex | 1,996 | `main` | - | app-server + Guardian V2 + Skill validation + Luna sampler + interrupted turn recovery |
| **headroom** | **106 (실측, prompt 의 1085 = hallucination)** | `main` | v0.23.0 (06-04) + Unreleased | RTK=v0.22.4 (v0.23.0 link hallucination), D-66/D-67/D-68 tract 재평가 REJECT |
| goose | 661 | `main` | - | ACP provider 다수 + 보안 강화 + Recipe/Slash + 8 provider 추가 |
| gemini-cli | 130 | `main` | v0.55.1 | TOML extensions + MCP OAuth refresh + Capacity Exhaustion terminal + caretaker agent |
| claude-code | 594 (실측, prompt 의 66 = 자동화 commit 만) | `main` | - | 자동화 commit 다수, **D-34/D-40 §11.2 잠금 정합 검증** |

#### 2.2 워커 운영 (D-16/D-73 lesson 적용)

- **D-16 chunked write**: 1 plan = 1 reference. 7 worktree (`analysis/<name>-v2` branch) 격리.
- **D-73 prompt lesson**: Worker prompt 머리에 cross-session grep 1-liner.
- **D-73 §3 hallucination 회귀**: headroom D-130 prompt 의 "RTK=v0.23.0" 가 hallucination — 실측 = v0.22.4 변경. 워커가 §16.14 에 정직 명시. **lesson**: commit ref / module name 은 upstream source 에서 직접 확인 필수.
- **워커 출력**: 모두 1 commit + 7 reference 영향 분석 (+1,987 lines total). 메모리 sync 별도 task.
- **순차 실행 (rate limit 회피)**: 첫 병렬 spawn = 4 fail (rate limit 2062 + serialization error). 순차 재시도 = 7/7 완료. aider / claude-code / codex / goose / headroom / gemini-cli / opencode 순서로 진행.

#### 2.3 D-130 (headroom v2) 의 핵심 결과

- **D-66/D-67/D-68 tract 재평가**: REJECT. Rust 측 안정성 ↑ = RTK (Rust observability, **v0.22.4 변경**) + Rust proxy metrics (headroom 자체 안정화, ONNX 무관). **v2.0 ONNX 백로그 = OOS 유지** (D-66/D-68 abort 결정 유지).
- **CCR workspace scope** (D-130 §16.2) → §5.6 Layer2 CCR. **즉시 follow-up 2 commit**: (a) SqliteMemoryStore FTS5 schema 에 `workspace_path` 컬럼 추가 (D-99 의 6 integration test + 1 신규 test, ~150 lines). (b) Memory fail-closed — `MemoryStore::open()` 의 `~/.myharness/memory/` 부재 시 명확한 에러 + 안내 (D-98 의 7 ndjson_* test 회귀 0, ~80 lines).
- **cli wrap-subcommand** (D-130 §16.3) → §5.10 sub-agent dispatch. TASK-005-2 v1.5+ Sub-task 2 (provider-auto-config Skill) 와 동시 차용 권장.

#### 2.4 D-133 (claude-code v2) 의 핵심 결과

- **66 commit 명세 오류 정직 검증**: prompt 의 "66 commit" ≠ 실제 594 commit (자동화 commit 525 + 실질 ~68). 워커가 §18.2 에 정직 명시.
- **D-34/D-40 §11.2 잠금 정합**: 06-09 이후 claude-code 변경 = 우리 §5.6/§5.14/§5.4/§5.5 영향 0. D-40 의 §11.2 완전 제거 결정이 정합이었음 사후 확인. 결정 변경 불요.
- **PR #79898** (Royarsan/anthropics): AWS gateway example deployment assets — 우리 §5.5 OAuth 패턴 참고 가치 (low priority, v2+).

#### 2.5 코드 영향 (다음 1순위 후보, yklee 결정 시)

| 결정 ID | 출처 | 내용 | 우선순위 |
|---|---|---|---|
| **D-130 즉시** | headroom §16.2 | SqliteMemoryStore FTS5 schema + `workspace_path` 컬럼 | 즉시 |
| **D-130 즉시** | headroom §16.4 | Memory fail-closed (`~/.myharness/memory/` 부재 시 에러) | 즉시 |
| **D-130 백로그** | headroom §16.6 | Learned Plugin 4-layer fix (TASK-005-2 v1.5+ Sub-task 5) | v1.5+ |
| **D-130 §16.3** | headroom §16.3 | cli wrap-subcommand (TASK-005-2 Sub-task 2 와 동시 차용) | v1.5+ |
| D-131 (goose) §16.1 | goose | ACP SDK 0.12 추가 (feature `acp`) | v2+ |
| D-131 §16.4 | goose | provider 50 → 58 (8종 추가) | v1.5+ |
| D-132 §16.7 (gemini-cli) | gemini-cli | TOML extensions 표준 | v1.5+ |
| D-129 §16.2 (codex) | codex | effective permission 우선 → §5.4 v0 회귀 차단 | v1 즉시 |
| D-129 §16.3 | codex | Skill validation → §5.14 skill system 1차 cycle | v1.5+ |
| D-127 §16.a | opencode | reasoning effort 표준화 → §5.5 LLM wire format | v1.5+ |
| D-127 §16.b | opencode | compaction → §5.6 Layer2 | v2+ |
| D-127 §16.c | opencode | session retry jitter cap → §5.5 router | v1 즉시 |
| D-127 §16.e | opencode | Copilot PDF detect → §5.5 multimodal | v2+ |
| D-127 §16.f | opencode | R2 data catalog → §5.13 observability | v2+ |

### 3. 다음 1순위 (yklee 결정 시)

- (a) **D-130 즉시 follow-up 2 commit** — CCR + Memory fail-closed (즉시)
- (b) **D-130 §16.3 cli wrap** → TASK-005-2 v1.5+ Sub-task 2 와 동시 차용
- (c) **D-127 §16.c session retry jitter cap** — §5.5 router 에 즉시 적용 (opencode 의 5-tuple 기반, ~50 lines)
- (d) **D-129 §16.2 effective permission** — §5.4 permission mode 의 v0 회귀 차단 (~100 lines)
- (e) TASK-002 도메인 명령 / OAuth real flow / TUI shell / cargo hygiene / Lark parser / block-aware insert/replace (이전 9 옵션)

### 4. 누적 결정

- 69 → **76** (D-127~D-133, 7 신규). main = `8782abf`. 결정 log 후속 (D-104 메모리 sync 직후).

---

## 세션 종료 (2026-08-14) — v3 reset 방향 보류

### 1. 사용자 메시지 이력

- "여기 작업 내역 확인하자 뭐 하던 곳인가" → 세션 시작 + D-104 메모리 sync (commit `ac69733`)
- "세월이 많이 지나서 레퍼런스들이 발전을 많이 했어. 다시 조사하자" → TASK-004 재방문 (D-127~D-133, 7 reference v2, commit `4d031ff`)
- "우리 하네스가 디자인도 영 별로고 동작도 이상한데 레퍼런스 하나 잡고 뼈대로 삼아서 커스터마이징 하는 방향으로 가는건 어떨까?" → v3 reset 방향 제안 (사용자 결정 보류)
- "일단 정리하고 종료" → 세션 종료

### 2. v0 디자인/동작 진단 (7건 발견)

| # | 문제 | 심각도 |
|---|---|---|
| 1 | sub-agent 15개 vs cli 7개 갭 (CONCEPT §5.11 vs 실제) | HIGH |
| 2 | Orchestrator prefix 매칭 (확장 어려움) | HIGH |
| 3 | SubAgentKind 4-5개 vs §5.11 15개 갭 | HIGH |
| 4 | resolve_llm_client() inline (테스트 어려움, MiniMax 우선순위 모순) | MEDIUM |
| 5 | Text-based dispatch 만 노출 (D-108 native tool calling 미사용) | MEDIUM |
| 6 | mode 3개 미사용 (orchestrator default, ENXIO 가드만) | LOW |
| 7 | credential chain 우선순위 + 5 단계 inline 의 복잡성 | MEDIUM |

### 3. v0 LOC 매트릭스 (23,697 lines)

- llm (6155) + tools (4887) + auth (3229) + context (2661) + tui (2579) = **19,511 lines / 82%** (핵심 5 crate)
- core (1335) + cli (1364) + compression (1487) = **4,186 lines / 18%** (thin shell)

### 4. 뼈대 후보 비교 (실측 데이터)

| reference | stars | stack | 우리 정합 | multi-provider | Recipe/Skill | 활발성 (06-09~) |
|---|---|---|---|---|---|---|
| **goose** (추천) | 52,758 | Rust + TS | **100%** | ✅ 50→58 | ✅ Recipe | 661 commit |
| codex | 105,715 | Rust | 100% | ❌ (OpenAI only) | Skill validation 만 | 1,996 |
| oh-my-pi | ~26k | TypeScript/Bun | ❌ | ✅ | Skill/Extension | 활발 |
| opencode | 196,998 | TypeScript/Bun | ❌ | ✅ | extensions | 1,457 |

### 5. 3-way reset 옵션 (사용자 결정 보류)

| 옵션 | trade-off | 작업량 |
|---|---|---|
| **A: goose fork + 커스터마이징** | v0 의 60% 폐기 (cli/llm/auth 보존, core/tui/tools 재구축) | 1~2개월 |
| B: goose module 1~3개 adopt + 점진 재구축 | v0 보존, Recipe + ACP 만 차용 | 2~4주 |
| C: goose skeleton (v0 일부만 재구축) | tui + tools crate 만 goose 스타일로 재구축 | 2~3주 |
| D: v0 그대로 + 디자인/동작 7건 만 수정 | 차용 없음, v0 유지 | 1~2주 |

### 6. 추천: 옵션 A (goose fork)

- goose 의 Recipe/Slash + ACP + multi-provider 8종 (xAI SuperGrok OAuth / Kimi Code DF / Perplexity / Qwen DashScope / Databricks GW / NEAR AI / Scaleway / HF OAuth) 차용
- 우리 v0 의 cli/llm/auth crate 보존 (Multi-provider 우선, Hybrid 안, MiniMax 우선)
- Recipe 시스템 = 우리 §5.14 Skill/MCP first-class 의 직접 차용
- ACP = pluggable protocol (v3 의 app-server 같은 plug-in)
- Loop 안정화 6 commit (turn-count timeout / blocking Stop hook / session mutex / LRU token cache) = §5.10 LoopRunner 차용

### 7. 다음 세션 진입점 (yklee 결정 시)

- (a) **v3 reset 옵션 선택** — A/B/C/D
- (b) **v0 디자인/동작 7건 수정** (옵션 D 선택 시)
- (c) **D-130 즉시 follow-up 2 commit** — CCR + Memory fail-closed (옵션 무관하게 유효)
- (d) TASK-002 도메인 명령 / OAuth real flow / TUI shell / Lark parser (이전 9 옵션)

### 8. 누적 결정

- 76 → **76** (v3 reset 결정 보류 = 결정 추가 불요, 사용자 결정 대기)
- main = `4d031ff`. 7 reference 재방문 (D-127~D-133) 완료. 다음 세션 시작점 = v3 reset 옵션 결정 (yklee).

---

## 세션 종료 (2026-08-14, 2차) — D-134 grok-build reference

### 1. 사용자 메시지

- "저장소 작업 내역 확인해봐" → 상태 복원 (main=`591609f`, v3 reset 보류)
- "reference에 grok build도 추가하자 그리고 grok build를 기반으로 커스텀 하네스를 만들어볼거야 검토해봐" → 8번째 reference + 뼈대 적합성 검토
- "3" → 레퍼런스만 두고 14섹션을 goose.md 급으로 깊게
- "일단 세션 정리하고 다음 세션에 이어하자" → 본 종료

### 2. 한 일 (D-134)

- 로컬 클론 `/Users/yklee/repos/grok-build` + `grok 1.0.3` + user-guide 실측
- 신규 [docs/references/grok-build.md](../../docs/references/grok-build.md) 14섹션 + §15 영향 (약 700줄, 코드 인용)
- [docs/references/README.md](../../docs/references/README.md) 7-doc → **8-doc** (8축 매트릭스 행 추가)
- CONCEPT §12 링크만 추가. positioning 변경 없음

### 3. 코드로 닫힌 사실 (결정은 아님)

- Grok Build = Apache 2.0, Rust 1.92, crate 79, `*.rs` 136만 줄, 외부 PR 거부, 모노레포 dump
- CONCEPT 5 components 이미 1:1 (Tools/Context/Session/Plugins/Sub-agents)
- TUI 는 in-process `MvpAgent` + ACP. `grok agent stdio` / `--plugin-dir` 가 래퍼 정공법
- 소스 포크는 비권장 (generated Cargo.toml + 기여 거부 + 암호화 프롬프트)
- 독립 런타임이 필요하면 이전 추천(goose fork) 유지

### 4. 보류 (다음 세션 진입점)

1. **커스텀 하네스 경로** — A overlay (`grok` 엔진 + myharness plugin/래퍼) / B grok 소스 포크(비권장) / C goose 포크(독립 런타임)
2. A 선택 시 CONCEPT §0 positioning 수정 필요
3. D-130 follow-up (CCR + Memory fail-closed) — 경로와 무관하게 유효
4. TASK-002 / OAuth real flow (이전부터 blocked)

### 5. 누적 결정

- 76 → **77** (D-134). overlay/포크 자체는 미결정.
- 문서+메모리 단일 commit. working tree 는 본 commit 후 clean 이어야 함.

---

## 세션 종료 (2026-08-14, D-135) — overlay 문서 재구성

### 1. 사용자 메시지

- "이전 작업 확인하자. 레퍼런스 검토 작업 하다가 뭔가 꼬였어" → 진입점 A 4중 정의 진단
- "A 안을 브리핑해줘" → D-134 overlay 브리핑
- "좋아 A 안에 따라 재구성을 시작하자. 문서부터 컨셉, 설계 등 전체적으로 갱신해줘" → D-135

### 2. 한 일

- **D-135 확정**: 제품 경로 = grok overlay. 자체 Plugin A1~A4 = OOS
- 신규 [`docs/architecture/DETAILED_DESIGN_OVERLAY.md`](../../docs/architecture/DETAILED_DESIGN_OVERLAY.md)
- CONCEPT §0 / §5.1 / §5.3 / §5.7 / §5.8 / §5.9.4 / §6 / §8 / §11.4
- README, PROJECT_PROFILE, MiniMax, AGENTS, REFERENCES, grok-build §15, INITIAL_DESIGN·REQUIREMENTS 배너, development_log
- 코드 0. v0 crates 신규 기능 금지

### 3. 다음 세션 진입점

1. **PR-1** `plugins/myharness/` 스캐폴드 (`plugin.json` + stub)
2. **PR-2** thin CLI 래퍼 + grok 가드 + 12 동사
3. PR-3 MiniMax `[model.*]` smoke
4. D-130 follow-up (CCR/Memory) — 경로 무관, 별도

### 4. 누적 결정

- 77 → **78** (D-135)
- in_progress: D-135 PR-1
- blocked: TASK-002, MiniMax OAuth real flow
- recently-done 머리: D-135

- 
