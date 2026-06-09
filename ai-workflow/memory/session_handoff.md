# Session Handoff

- Purpose: Compact restore context for the next AI agent session.
- Scope: current focus, task status, key changes, next actions, risks
- Audience: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- Status: active
- Updated: 2026-06-09 (D-59 + **TASK-005-1 v1 MVP 종료 선언**, yklee 결정. W16 add-local follow-up 으로 main 머지 완료 → feature/w16 브랜치 정리 + origin/upstream 양쪽 삭제)
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
- **누적 38+ commit dual-push** (D-44~D-58). **Workspace 388 tests pass, 0 fail, 2 ignored** (real API smoke)
- **다음**: TASK-005-1 v1 MVP 종료 또는 TASK-005-2 (v1.5) 진입. MiniMax device grant real flow 검증 (W15.b 자동 refresh 도 real test 가능)

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

## 다음에 할 일 (Next Actions)

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
- [ ] **TASK-005-2 (v1.5)** 진입 — Plugin 4-계층 + marketplace + auto memory + provider-auto-config skill 정식 + CCR/Kompress-base ONNX
- [ ] **MiniMax Device OAuth real flow** 검증 — yklee 가 MiniMax console 에서 device grant 활성화 후 `myharness auth login minimax --no-browser` 실행 (OpenClaw/Hermes 공통 client_id 78257093-7e40-4613-99e0-527b14b39113, W15.b 자동 refresh 도 real test 가능)
- [ ] **OpenAI/Google 도 동일 패턴** (Authorization Code + PKCE, client_id 등록 후 검증)
- [ ] **ANTHROPIC_API_KEY 주입 시 LLM E2E 테스트** (real-anthropic ignored test 활성화)
- [ ] **§5.12 디렉토리 자동 생성** (v1 first run 시) — `~/.myharness/{config,state,memory,handoff,compression,sub-agents,auth}/` + `state.json`
- [ ] **TASK-002 (도메인별 명령)** — yklee 인프라 정보 수령 후 (SSH 별칭 / Brewfile / dotfiles / 런타임 버전) 진행
- [ ] **헤로쿠 / Synology NAS 인프라 검증** — yklee 가 인프라 정보 입력 시점에 작업
- [ ] **tool name mismatch**: sub-agent 가 기대하는 도구 이름 (Read/Grep/Glob, 대문자) vs tools crate 의 도구 이름 (read/grep/glob_, 소문자). v1.5 에서 통일

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

(End of file - total 102 lines)
