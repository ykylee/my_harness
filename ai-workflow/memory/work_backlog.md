# 작업 백로그 인덱스

- 문서 목적: 프로젝트의 모든 작업 항목과 날짜별 백로그 링크를 관리한다.
- 범위: 전체 태스크 목록, 우선순위, 진행 상태, 날짜별 기록 연결
- 대상 독자: 개발자, AI 에이전트, 프로젝트 매니저
- 상태: stable
- 최종 수정일: 2026-06-11 (**D-73 T-v5-sync-1 plan_3c8c4a49 cancel + 4-plan split 결정** — v0.5.0→v0.5.11 sync 11 minor 24 commit 을 4 plan A/B/C/D 으로 분리. 3 memory entries 추가. 이전: D-68 v2.0 tract Commit 2 abort)
- 관련 문서: [세션 인계](./session_handoff.md), [프로젝트 프로필](../../docs/PROJECT_PROFILE.md), [CONCEPT.md](../../docs/CONCEPT.md)

## 1. 운영 원칙
1. 세션 시작 시 인덱스와 최신 백로그 확인
2. 세션 종료 전 인덱스 및 Handoff 갱신
3. 모든 작업 상태는 날짜별 백로그에 기록

## 2. 날짜별 백로그
- [2026-06-05](./backlog/2026-06-05.md) — 초기 부트스트랩 + 방향 전환
- [2026-06-06](./backlog/2026-06-06.md) — 5 reference clone + Gitea + 5-doc 분석
- [2026-06-07](./backlog/2026-06-07.md) — 7-doc 분석 + CONCEPT.md SSOT + 5 결정 (D-22~D-38)
- [2026-06-09](./backlog/2026-06-09.md) — TASK-005-1 W3~W6.5 (D-43) — 5 tool + 4 permission + 9 sanitizer + JSON schema + 5 provider wire format. 9 commit Gitea push.
- [2026-06-10](./backlog/2026-06-10.md) — D-62 W17 누락분 = 0건 정정 + D-62 W19-1 TC-W17-002 test fail 복구 (AuthStore 주입 패턴). 110 llm tests / 0 fail. cargo build/clippy OK
- [2026-06-11](./backlog/2026-06-11.md) — **D-73 T-v5-sync-1 plan_3c8c4a49 cancel + 4-plan split 결정**. v0.5.0→v0.5.11 sync 11 minor 24 commit. 4 plan: A (5.1 MCP, ~5 files, 3 commit) + B (5.2 bootstrap_lib refactor, 73 files, 7,941+ ins, 8 modules 3,710 lines, 9 chunk + 800줄 cap) + C (5.3-5.4 antigravity MCP + contract v1, ~5 files + Rust caller 갱신) + D (5.5-5.11 + R-4 SSOT drift, 12 commit, ~10 files + R-4). 3 memory entries (T-v5-sync-1 실패 분석 + Worker prompt 3원칙 + cross-session grep 1-liner)

## 3. 전체 작업 상태 요약

### Done (2026-06-05 ~ 2026-06-07)
- [x] **TASK-001**: my-harness 부트스트랩 (standard_ai_workflow 적용) — done (2026-06-05)
- [x] **TASK-003**: Gitea 미러 (5 repo) — done (D-05, D-07, D-08)
- [x] **TASK-004**: CLI/TUI 툴 레퍼런스 분석 — done
  - 1차 (8축 비교표): `docs/REFERENCES.md` (D-09)
  - 2차 (14섹션 심층분석, 7 reference): `docs/references/{codex,aider,goose,opencode,gemini-cli,claude-code,headroom}.md` (D-10, D-15, D-21)
  - 7-doc cross-review: `docs/references/README.md` (D-21)
- [x] **TASK-005**: 스택 결정 — **Rust 1안** (D-36, §11.3)
- [x] **TASK-006**: TUI 라이브러리 결정 — **ratatui + crossterm** (D-36, TASK-005 종속)
- [x] **TASK-007**: headroom v1 우선순위 결정 — **3 알고리즘** (D-37). CCR + Kompress-base v1.5+ 로 연기
- [x] **TASK-008**: Provider fallback 결정 — **`provider-auto-config` skill** (D-38). 하드코딩 fallback 폐기, 동적 discovered list

### Done (recent)
- [x] **TASK-005-1** (v1.0 MVP, Rust 빌드) — W3~W6.5 + W7 + W8 + W9 + W10 + W11 + W12 + W13 + W13.5 + W13.6 + W14 (Device Authorization Grant) + W14.4 (`--no-browser` 3 모드) + W14.5 (polling output + expired_in ms) + W14.6 (token ms→s 변환) + W15.a (OAuth 자동 resolve) + W15.b (OAuth 자동 refresh) 완료 (D-43, D-45~D-58) + dual-remote push. **8/8 waves + D-52 follow-up 6 작업 완료**, v1 MVP 완성. **388 tests pass, 0 fail, 2 ignored**. 38+ commit dual-push (Gitea + GitHub).

### Planned
- [ ] **TASK-005-2** (v1.5) — Plugin 4-계층 + marketplace + auto memory + provider-auto-config skill 정식 + CCR/Kompress-base ONNX
- [ ] **TASK-005-3** (v2.0) — TUI/IDE/Web hand-off + Routines + OAuth + MCP-based discover
- [ ] **TASK-005-4** (v2.5) — Multi-agent parallel + confidence scoring
- [ ] **TASK-005-5** (v3.0) — Computer Use + Multi-user + RBAC

### Planned (다음 세션)
- [x] prerequisite 5건 설치
- [x] cargo workspace init
- [x] ratatui shell (defer — W6 에서 dispatch, TUI shell 은 W10+)
- [x] rig-core Anthropic (defer — W7)
- [x] basic Tools (W3) ✅
- [x] Context (defer — W8+)
- [x] standard_ai_workflow output (defer — W11)
- [x] 4 permission mode (W4) ✅
- [x] 1-2 built-in sub-agent (defer — W10)
- [x] JSON schema + 5 provider wire format (W6 + W6.5) ✅
- [x] Bash sanitization (W5) ✅
- [x] Gitea PAT 설정 (myharness-cli, D-44) ✅
- [x] GitHub upstream 추가 (D-20, D-44) ✅
- [x] 9 commit dual push (Gitea + GitHub, D-44) ✅
- [x] W7 llm crate (rig-core 0.38 + 6 provider + auth + discover + chain + router) (D-45) ✅
- [x] W7 5 commit dual push (D-45) ✅
- [x] W8 context crate (CLAUDE.md + auto memory + ContextManager + Layer 2 4 알고리즘 + ContextConfig) (D-46) ✅
- [x] W8 5 commit dual push (D-46) ✅
- [x] W9 compression crate (Summarizer + CCR + Kompress-base v1 + BuiltinRegistry 6 알고리즘) (D-47) ✅
- [x] W9.2 context ContextManager Summarize/Hybrid 정식 (LLM-driven) (D-47) ✅
- [x] W9 5 commit dual push (D-47) ✅
- [x] W10 tui crate (ratatui App + 4 SubAgent + Orchestrator + LoopRunner) (D-48) ✅
- [x] W10.5 cli → tui 통합 (code/env/git/ask subcommand) (D-48) ✅
- [x] W10 5 commit dual push (D-48) ✅
- [x] W11 core crate v1 (standard_ai_workflow 6 원칙 + 4 permission + tool alias) (D-49) ✅
- [x] W11.3 MockClient FIFO + Orchestrator fatal_llm_error (D-49) ✅
- [x] W11.4 cli subcommand task start|end + handoff (D-49) ✅
- [x] W11 5 commit dual push (D-49) ✅
- [x] W12.1 KeyringAuthStore in-memory cache (D-50) ✅
- [x] W12.2 ProviderMetadata::builtin_minimax 갱신 (api.minimax.io/v1 + MiniMax-M3) (D-50) ✅
- [x] W12.3 MiniMax OpenAI-compat tests + real-api smoke ignored (D-50) ✅
- [x] W12.4 discover smoke test (D-50) ✅
- [x] W12.5 cli default LLM = MINIMAX (D-50) ✅
- [x] W12 5 commit dual push (D-50) ✅
- [x] W13.1 OAuth 2.0 + PKCE + local callback + browser + token store (D-51) ✅
- [x] W13.2 3 provider (MiniMax / OpenAI / Google) (D-51) ✅
- [x] W13.3 AuthManager (login + refresh + status + logout) (D-51) ✅
- [x] W13.4 cli auth subcommand (D-51) ✅
- [x] W13 4 commit dual push (D-51) ✅
- [x] W13.5 OAuth env override (MYHARNESS_OAUTH_CLIENT_ID_{MINIMAX,OPENAI,GOOGLE} + MINIMAX_API_HOST) + OAUTH_PROVIDERS static LazyLock 제거 (D-52) ✅
- [x] W13.6 mock OAuth server + AuthManager end-to-end e2e test (real network 없이, CI 가능) (D-52) ✅
- [x] W13.5 + W13.6 2 commit dual push (D-52) ✅
- [x] W14 MiniMax Device Authorization Grant (Authorization Code + PKCE 404 → device flow 교체, OpenClaw/Hermes 공통 client_id 78257093-7e40-4613-99e0-527b14b39113) (D-53) ✅
- [x] W14.4 --no-browser 의미 수정 (3 모드: default / --no-browser / --non-interactive, browser open 과 polling 독립) (D-54) ✅
- [x] W14.5 device flow polling output 항상 stdout + 1.5x backoff + expired_in ms 단위 unix timestamp (D-55) ✅
- [x] W14.6 device_token_to_oauth expired_in ms → s 변환 (TokenStore::save 초 단위 일관성) (D-56) ✅
- [x] W15.a OAuth token 자동 resolve (4 단계 credential chain: oauth store > env var > MockClient) (D-57) ✅
- [x] W15.b OAuth token 자동 refresh (LlmError::ProviderCall msg 401/unauthorized/auth 키워드 감지 → AuthManager::ensure_fresh → store save → 새 OpenAiCompatProvider 빌드 → retry 1회) (D-58) ✅
- [x] D-52 follow-up 6 commit dual push (D-53~D-58) ✅
- [x] **TASK-005-1 v1 MVP 종료 선언** (8/8 waves + D-52 follow-up 6 작업 완료, yklee 결정, 2026-06-09 22:48)
- [x] **TASK-005-2 v1.5 W17 (D-60) + W18 (D-61) 완료** — auth add-local 비대화형 모드 + R-4 backup + Confirm + --yes flag. main 머지 + cleanup 6e925a1 + handoff_D-61 복구 91c1e34. **D-62 정정**: W17 PR 누락분 = 0건 (W18 cross-check 오류)
- [x] **D-62 W19-1 TC-W17-002 test fail 복구** (2026-06-10) — `register_local_provider_with_store` + `register_local_provider_non_interactive_with_store` 추가 (AuthStore 주입 패턴). 4 L1 test PASS, 110 llm tests / 0 fail
- [x] **D-63 W20 F-3 Ollama native cascade** (2026-06-10) — `probe_local_models` 2-stage (Ollama native /api/tags → OpenAI compat /v1/models fallback). 3 L2 integration test (TC-W20-I01/I02/I03) PASS, 110 llm + 10 w16_add_local tests / 0 fail
- [x] **D-64 W21 F-1+F-2 통합** (2026-06-10) — hash8 (sha256 8-char) util + backup filename `<ts>.<sha256_8>` + cleanup_old_backups sort bug fix. 4 L1 test (TC-W21-001/002/003/004) PASS, 117 llm + 10 w16_add_local + 3 hash8 = 130 / 0 fail
- [x] **D-65 TASK-005-2 v1.5 종료 선언** (2026-06-10) — v1.5 phase 완전 종료. 5 사이클 + D-62 + 14 신규 test / 0 fail. ONNX v2.0 Planning 으로 연기. binary 13MB 유지
- [x] **D-66 v2.0 ONNX Commit 1 abort** (2026-06-10) — ort ecosystem unstable (1.x yanked, 2.0.0-rc.9/10/12 빌드 깨짐). 코드/Cargo.toml 모두 revert. v2.0 ONNX 백로그 OOS. **lesson**: ecosystem stability SSOT — library 분석 시 실제 cargo build 검증 필수
- [x] **D-67 v2.0 tract Commit 1** (2026-06-10) — tract 0.23 (Pure Rust, Sonos production) 전환. 1차 build 통과. ModelManager skeleton 5 L1 / 0 fail. binary 13MB 유지
- [x] **D-68 v2.0 tract Commit 2 abort** (2026-06-10) — tract 0.23 API 한계 (Tensor wrapper, Runnable mismatch). 5+ error 누적. 변경 모두 revert. v2.0 ONNX 백로그 OOS 유지
- [x] **D-69 v1.5 안정화** (2026-06-10) — 3 작업: (1) tool name uppercase 통일 (LLM contract 정합, 26 곳), (2) §5.12 init_home_dir() (11개 디렉토리 자동 생성, paths.rs +125, integration test 3), (3) clippy 핵심 5건 fix (PI + 4 should_implement_trait allow + 1 useless_format). 3 commit dual-push (6d2a3e8/4891bc6/767c71a), 437 tests pass / 0 fail / 2 ignored
- [x] **D-70 v1.5.1 lint cleanup** (2026-06-10) — 21 style lint → 0 warning. 21 file / +94 -110. 1 commit dual-push (79f38b8). cargo clippy --workspace --all-targets = 0 warning
- [x] **D-70 v1.5 종료 선언** (2026-06-10) — v1.5 phase 완전 종료 (D-60~D-65 + D-69 + D-70, 8 결정)
- [x] **D-73 T-v5-sync-1 plan_3c8c4a49 cancel + 4-plan split 결정 (2026-06-11)** — my_harness v0.5.0→v0.5.11 sync 11 minor 24 commit. 1 plan = 6 batch × 11 minor = scope 3x underestimate (실제 73 files / 7,941 ins / 5,070 del). 31min cap 도달 시 25min = exploration + 0 write. 4 plan 분리: Plan A (5.1 MCP per-harness + round-trip, ~5 files, 3 commit, 워커 warm-up) + Plan B (5.2 bootstrap_lib refactor, **73 files / 7,941+ ins / 5,070 del**, 8 modules 3,710 lines, 9 chunk + 800줄 cap, 필요 시 2 plan sub-split) + Plan C (5.3-5.4 antigravity MCP + contract v1 도입, ~5 files + Rust caller 갱신, API breaking) + Plan D (5.5-5.11 12 commit + R-4 SSOT drift, ~10 files + R-4, smart update + Mavis engine hook + MCP fix). **3 memory entries** (mavis memory): (1) T-v5-sync-1 실패 분석 (scope 3x underestimate + 25min exploration + module name 오류), (2) Worker prompt 3원칙 (first-chunk-largest / JSON split / Edit-last + Write fallback), (3) Cross-session memory grep 1-liner (worker prompt 머리에 grep 1줄). **async audit**: plan_3c8c4a49 cancelled, 워커 idle, T-SAW-1 별도 owner session 운영 중, pending ops 0건
- [ ] **T-v5-sync-1 Plan A prompt 작성 + launch (5.1 only, ~5 files, 3 commit)** — yklee 결정 (4 plan A/B/C/D vs 5 plan Plan B sub-split) 후. tag `v0.5.1-beta` + commit hash (c3c9a90/73f8f2f/...) + per-batch file:line 박기
- [ ] **TASK-005-2 v2.0 다음 후보** — Plugin 4-계층 / Kompress-back / 외부 blocker 해결
- [ ] yklee MiniMax Device OAuth real flow 검증 — yklee 가 MiniMax console 에서 device grant 활성화 후 `myharness auth login minimax --no-browser` 실행 (OpenClaw/Hermes 공통 client_id 사용, W15.b 자동 refresh 도 real test 가능)
- [ ] yklee OpenAI/Google OAuth client_id 등록 후 동일 패턴 검증 (OpenAI: `platform.openai.com` OAuth Apps, Google: Google Cloud Console Credentials OAuth 2.0 Client IDs)
- [ ] **TASK-005-3** (v2.0) — TUI/IDE/Web hand-off (5 surfaces) + Routines + OAuth + MCP-based discover
- [ ] **TASK-005-4** (v2.5) — Multi-agent parallel + confidence scoring
- [ ] **TASK-005-5** (v3.0) — Computer Use + Multi-user + RBAC
- [ ] **minimax base_url 검증** (D-28 TBD) — yklee 가 base_url + API 형식 확인 후 v1 또는 v1.5 통합

### Deferred (yklee 인프라 정보 의존)
- [ ] **TASK-002**: 도메인별 명령 가이드 (코드/서버/환경) — yklee 인프라 정보 (SSH 별칭 / Brewfile / dotfiles / asdf/rtx 버전) 수령 시 진행. v1 Rust 구현 시점에 자연 도출 가능.

## 4. 핵심 결정 요약 (D-22~D-38)

| D-NN | 내용 | § |
| --- | --- | --- |
| D-22 | my_harness v1 컨셉 확립 (CONCEPT.md SSOT) | (master) |
| D-23 | 4 docs align (MiniMax/PROJECT_PROFILE/REFERENCES/PROVIDERS) | — |
| D-24 | 4-워커 통합 framing 제거 (sibling standalone) | §0.5 |
| D-25 | Mavis zero coupling | §5.8 |
| D-26 | standard_ai_workflow 6 원칙 native + 옵션 통합 | §5.9 |
| D-27 | headroom = built-in 압축 (외부 proxy X) | §5.6 |
| D-28 | Provider 6개 + OpenAI 호환 lingua franca | §5.5 |
| D-29 | Agent 3 모드 + 15개 built-in sub-agents | §5.10, §5.11 |
| D-30 | 2-계층 Context 압축 (Layer 1 필수 + Layer 2 선택) | §5.6 |
| D-31 | `~/.myharness/` 디렉토리 구조 | §5.12 |
| D-32 | LLM Wiki memory (Karpathy pattern, v2+) | §5.13 |
| D-33 | Skill/MCP first-class | §5.14 |
| D-34 | TASK-NNN 형식 통일 + 2.1.169 pending 표 | §6, §11 |
| D-35 | 관련 문서 align (5 docs) | — |
| D-36 | **TASK-005 결정: Rust 1안** (스택) | §11.3 |
| D-37 | **TASK-007 결정: headroom v1 1안 유지** (3 알고리즘) | §11.1 |
| D-38 | **TASK-008 결정: provider-auto-config skill** (동적 발견 + per-provider auth) | §5.5, §11.3 |
| **D-39** | **세션 마무리: handoff + backlog + state 갱신** | — |
| **D-41** | **TASK-005-1 환경 검증 완료** (W0-1 + W0-2) | — |
| **D-42** | **config 포맷 = TOML 통일** (config.yaml → config.toml) | §5.12, §5.5 |
| **D-43** | **TASK-005-1 W3~W6.5 완료** (tools crate 63 tests) | — |
| **D-44** | **dual-remote 적용** (Gitea + GitHub) | D-20 |
| **D-45** | **TASK-005-1 W7 완료** (myharness-llm crate v1, 87 tests) | §5.5 |
| **D-46** | **TASK-005-1 W8 완료** (myharness-context crate v1, 54 tests) | §5.6 |
| **D-47** | **TASK-005-1 W9 완료** (myharness-compression crate v1, 40 tests + context summarize 정식) | §5.6 |
| **D-48** | **TASK-005-1 W10 완료** (myharness-tui crate v1, 50 tests + cli 통합) | §5.10, §5.11 |
| **D-49** | **TASK-005-1 W11 완료 (v1 MVP 6/8 waves)** (myharness-core crate v1, 32 tests + standard_ai_workflow 6 원칙 native + 4 permission + tool alias + MockClient FIFO + Orchestrator fatal + cli task/handoff) | §5.4, §5.9 |
| **D-50** | **TASK-005-1 W12 완료 (MiniMax LLM API 연결, minimax TBD D-28 해소)** (librarian ⭐⭐⭐⭐⭐ 5/5, api.minimax.io/v1 + MiniMax-M3 + 7 models + OpenAI-호환 Bearer + tool_use, KeyringAuthStore in-memory, MINIMAX_API_HOST env, cli default LLM env 자동 detect) | §5.5 |
| **D-51** | **TASK-005-1 W13 완료 (OAuth 2.0 headless auth)** (myharness-auth crate v1, 7 모듈: pkce/flow/callback/browser/store/provider/manager, 3 provider MiniMax/OpenAI/Google 모두 PKCE public client, cli auth subcommand) | §5.5 |
| **D-52** | **TASK-005-1 W13.5 + W13.6 완료 (v1 MVP 8/8 waves 종료)** (env override + mock OAuth e2e test, real network 없이 CI 가능, 39 auth tests pass) | §5.5 |
| **D-53** | **TASK-005-1 W14 완료 (MiniMax Device Authorization Grant)** — Authorization Code + PKCE 가 404, device flow (RFC 8628 MiniMax 구현, OpenClaw/Hermes 공통 client_id 78257093-7e40-4613-99e0-527b14b39113) 로 교체. CN: `MYHARNESS_MINIMAX_CN=1` 또는 `MINIMAX_OAUTH_BASE_URL` env | §5.5 |
| **D-54** | **TASK-005-1 W14.4 완료 (`--no-browser` 3 모드)** — signature `(auto_open_browser, non_interactive)` 분리. 3 모드: default / --no-browser / --non-interactive (CI/스크립트). 42 auth tests, 379 workspace tests | §5.5 |
| **D-55** | **TASK-005-1 W14.5 완료 (polling output + expired_in 단위)** — URL + user_code + expires_in 항상 stdout, OpenClaw 1.5x backoff cap 10s, expired_in ms 단위 unix timestamp 처리 | §5.5 |
| **D-56** | **TASK-005-1 W14.6 완료 (token 단위 변환)** — `device_token_to_oauth` expired_in ms → s (`TokenStore::save` 초 단위 일관성) | §5.5 |
| **D-57** | **TASK-005-1 W15.a 완료 (OAuth token 자동 resolve)** — cli `resolve_llm_client()` helper, 4 단계 credential chain: `~/.myharness/oauth/<provider>.toml` > env var > MockClient. token 만료 시 WARN + env var fallback (자동 refresh 는 W15.b) | §5.5 |
| **D-58** | **TASK-005-1 W15.b 완료 (OAuth token 자동 refresh)** — cli `RefreshingLlmClient` wrapper, `LlmError::ProviderCall(msg)` 401/unauthorized/auth 키워드 감지 (oauth 제외) → `AuthManager::ensure_fresh` → store save → 새 `OpenAiCompatProvider` 빌드 → retry 1회. retry 1회 한정 (무한루프 방지). refresh_token 없으면 expired token 그대로 retry. 9 cli tests | §5.5 |
| **D-59** | **TASK-005-1 W16 완료 (`myharness auth add-local` subcommand)** — Ollama/vLLM/LM Studio/llama.cpp URL+token(선택)+모델 선택 UI (inquire 3단계). OpenAI 호환 `/v1/models` probe 4 서버 통합. atomic write (tmp+rename) 로 providers.toml 손상 방지. **W16 follow-up**: `resolve_llm_client()` 에 LocalLlm 분기 추가 (`MYHARNESS_USE_LOCAL_LLM=1` opt-in env, providers.toml 의 LocalLlm entry 자동 사용). live 검증: 192.168.0.101:1234 LM Studio gemma-4-12b-qat `"2+2 is 4."` 응답. 9 L1 + 3 L2 + 4 scenario TC = 16/16 PASS, 388+ workspace tests, dual-push 완료 | §5.2, §5.5 |
| **D-59 follow-up** | **TASK-005-1 v1 MVP 종료 선언** (2026-06-09 22:48, yklee 결정) — W3~W16 40+ commit dual-push, 브랜치 정리 (feature/w16-add-local local/origin/upstream 모두 삭제), main = 140acf9. 보존: `local/d-43-47-tc-scaffold`. handoff + state + decisions + next_actions 갱신 | — |
| **D-60** | **TASK-005-2 v1.5 W17 완료 (`myharness auth add-local` 비대화형 모드)** — DD-AddLocal §6.3 OI-1 해소 (v1.5 첫 작업). clap flag 4개 (`--url/--model/--token/--probe-skip`). `myharness-llm::register_local_provider_non_interactive(url, token, model_id)` probe 스킵 helper. `handle_auth_add_local` 3-mode 분기. 4 L1 + 2 L2 TC = 6/6 PASS, 410 workspace tests, 4 commit dual-push (feature/v15-add-local-non-interactive). **R-4 (사용자 home providers.toml 덮어쓰기)**: manual test 중 yklee 의 LM Studio 설정 1회 덮어쓰기 → mavis-trash recovery, F-1 backup / F-2 --yes flag v1.5+ OOS. **PR main merge 안 됨 (W18 정합성 cross-check 에서 W17 일부 main 누락 발견)** | §5.2 + §5.5 + §6.3 (OI-1 ✅) |
| **D-61** | **TASK-005-2 v1.5 W18 완료 (`myharness auth add-local` 자동 backup + Confirm prompt, R-4 직접 차단)** — DD-AddLocal §10 신규 spec. (1) `myharness-llm::backup_providers_toml(path, max_backups=5)` — register_local_provider 안에 silent fail-soft 호출 연결, providers.toml 덮어쓰기 직전 `.backup.<unix_ts>` 자동 생성 + retention, (2) cli `--yes` flag + `inquire::Confirm` prompt (interactive 모드), (3) W17 PR 누락분 main 재추가 (register_local_provider_non_interactive fn + TC-W17-004). 3 L1 + 2 L2 + W17-004 = 5/5 PASS, 406 workspace tests, 4 commit dual-push (feature/v15-add-local-backup). **복구**: `cp ~/.myharness/providers.toml.backup.<ts> ~/.myharness/providers.toml`. sub-second ts 충돌 / backup corruption R-5, F-1 monotonic_ts, F-2 git-style versioning v1.5+ OOS | §5.2 + §5.5 + §6.3 (R-4 ✅) |
| **D-62** | **TASK-005-2 W17 PR 누락분 = 0건 정정 (state.json cross-check 오류, 2026-06-10)** — W18 정합성 cross-check 가 'W17 original 4 commit 중 fn + TC-W17-001~003 + TC-W17-I01~I02 main 누락' 으로 결론 내렸으나, `git log main --grep='W17'` + `grep 'register_local_provider_non_interactive\\|tc_w17_'` 검증 시 W17 4 commit (766d1b6 register fn / a46f480 TC-W17-I01+I02 / f8082bc cli 4 flag / d09a6a1 spec+TC scaffold) 모두 main 머지 확인. cleanup commit 6e925a1 가 W18 cherry-pick 중복 fn/test 만 제거하고 W17 본진 보존. **lesson**: cross-check 시 '누락분' 결론은 반드시 git log + file symbol grep 으로 직접 검증, 단순 머지 commit stat 만으로 판단 금지. side effect: TC-W17-002 test fail (libsecret 부재 환경 BackendUnavailable) 별도 식별 | §6 + §11.2 (메모리 정합성 규칙) |
| **D-62 W19-1** | **TASK-005-2 TC-W17-002 test fail 복구 (AuthStore 주입 패턴, 2026-06-10)** — `register_local_provider_with_store(base_url, token, selected, available, store: &dyn AuthStore)` + `register_local_provider_non_interactive_with_store(...)` 추가. 기존 fn 은 thin wrapper (KeyringAuthStore::probe() 1회 → with_store 위임). **WHY**: 기존 fn 의 `KeyringAuthStore::probe()` 가 caller 와 별개 in-memory cache 를 만들어 caller.store.get() 시 cache miss → BackendUnavailable 으로 fail. 해결: caller 가 store 1개 만들어 with_store 에 명시 전달 → cache lifecycle 단일화. **L1 test 4개**: TC-W17-002 수정 (probe 1번 + with_store 호출) + TC-W19-001 (with_store cache hit) + TC-W19-002 (None token 시 cache 무변경) + TC-W19-003 (thin wrapper 별개 store 회귀 방지). 110 llm tests / 0 fail, cargo build/clippy OK. cli caller 변경 없음 (back-compat) | §5.5 + §6.3 (DD-AddLocal) |
| **D-63** | **TASK-005-2 W20 F-3 Ollama native /api/tags cascade (2026-06-10)** — `probe_local_models` 2-stage cascade. **WHY**: `scan_local.rs` 는 이미 Ollama native `/api/tags` 사용 (line 27), `add_local.rs::probe_local_models` 는 OpenAI 호환 `/v1/models` 만 — split-brain. Ollama default (native only) 사용자가 `auth add-local` 시 404 fail. **해결**: (1) `GET {base}/api/tags` (Ollama native, 2s timeout) → `parse_ollama_tags` (name=id, details.family=owned_by), (2) 실패 시 `GET {base}/v1/models` (OpenAI compat, 3s timeout, back-compat W16 ends_with `/v1` 분기). `parse_ollama_tags` / `parse_openai_models` / `fetch_json_body` helper 분리. **L2 test 3개**: TC-W20-I01 (native only) + I02 (cascade fallback) + I03 (native 우선, OpenAI 미호출). W16 7/7 회귀 없음. scan_local.rs 와 priority 정합. 110 llm + 10 w16_add_local tests / 0 fail, cargo build/clippy OK | §5.5 + §6.3 (DD-AddLocal) |
| **D-64** | **TASK-005-2 W21 F-1+F-2 통합 (2026-06-10)** — `hash8::content_hash_8(content)` SHA-256 8-char util (sha2=0.10, auth crate 재사용, 새 dep 불요). backup filename = `<base>.backup.<ts>.<sha256_8>`. **WHY (3 risk)**: (1) R-5-A 동일 second 내 rapid register 시 backup filename 동일 → 앞 backup 덮어쓰기. (2) R-5-B backup file 식별 불가 (content fingerprint 없음). (3) `cleanup_old_backups` 의 string sort 가 `backup.999` < `backup.10000` 거꾸로 retention. **해결**: hash8 으로 collision 방지 + content 식별. `cleanup_old_backups` 의 sort: `parse unix_ts` numeric sort (string sort → numeric parse on unix_ts). **L1 test 4개**: TC-W21-001 (filename 형식) + 002 (numeric sort) + 003 (sub-second 모두 보존, sleep 없이) + 004 (동일 content 동일 hash). W18 sleep(1100ms) 불요해지지만 back-compat. 117 llm + 10 w16_add_local + 3 hash8 = 130 / 0 fail, cargo build/clippy OK | §5.5 + §6.3 (DD-AddLocal) |
| **D-65** | **TASK-005-2 v1.5 종료 선언 (2026-06-10)** — v1.5 phase 누적: W17 (D-60, 비대화형 add-local) + W18 (D-61, R-4 backup + Confirm + --yes) + D-62 (W17 누락분 정정) + W19-1 (AuthStore 주입) + W20 (Ollama cascade) + W21 (sha256_8 + sort fix). 누적 test 14 신규 / 0 fail. **v1.5 build phase 완전 종료**. **ONNX 통합 v2.0 Planning 으로 연기** — Initial_design.tt-3 의 '+10-30MB v1.5+' 의도 따름, ort C++ build dep 회피, binary 13MB 유지, Layer 2 opt-in rule-based fallback 으로 기능적 gap 없음. **v2.0 후보 (yklee 결정)**: (1) ONNX 3-commit, (2) Plugin 4-계층, (3) Kompress-back | §6 (TASK-005-2 v1.5 phase) + §11.3 (TT-3 binary size) |
| **D-66** | **TASK-005-2 v2.0 ONNX Commit 1 abort (2026-06-10)** — ort ecosystem 2026-06 unstable. ort 1.x (1.13.1, 1.16.3) **전부 yanked** (cargo download 불가). ort 2.0.0-rc.9 (Nov 2024), 2.0.0-rc.10 (Jun 2025), 2.0.0-rc.12 (Mar 2026) 모두 빌드 깨짐 — rc.12 는 ureq 3.1 API 변경 (`tls_config` method 없음, `download-binaries` build script 실패), rc.10/9 는 fn pointer 에 `unwrap_or_else` 미구현으로 type annotation error 다수. **abort 결정**: 코드/Cargo.toml 모두 revert, v2.0 ONNX 백로그 OOS. **lesson**: ecosystem stability 는 SSOT (CONCEPT.md §11.3). library 분석 시 crates.io + lib.rs + **실제 cargo build 검증** 필수. library 보고만 의존 ❌. **차후 옵션**: tract (Pure Rust, ort보다 안정적) 검토 또는 ort 안정화 (1-2 RC 후) 까지 보류. v2.0 다음 후보: Plugin 4-계층 / Kompress-back | §11.3 (ecosystem stability) + §6 (v2.0 backlog) |
| **D-67** | **TASK-005-2 v2.0 tract Commit 1 (W23, 2026-06-10)** — **D-66 abort 후 tract 로 전환** — Pure Rust (no C++ toolchain), Sonos production 검증 (wake-word/ASR/LLM/TTS), 1차 cargo build 즉시 통과 (D-66 lesson). **WHY tract**: ort ecosystem 2026-06 unstable 의 대안, Apache 2.0/MIT dual ↔ myharness 호환. **Commit 1 scope**: `ModelManager` skeleton (OnceLock lazy + Send+Sync + `new()`/`get()` global + `ensure_downloaded` reqwest streaming + sha2 SHA256 verify + `load_runnable` `into_runnable()` verify). **Commit 1 한계**: Runnable(Arc<dyn trait>) type-safe 보관 어려움 → embed() Commit 2 stub. 5 L1 test PASS (cache_path/sha256_of_known_data/model_info_defaults/model_manager_new/embed_stub). 429 workspace tests / 0 fail. **binary size**: 13MB (release lto=thin 효과, dev profile 22-33MB). **다음 Commit 2**: actual `embed()` inference (tokenization + tract run + Runnable 보관 API 정착) | §6 (v2.0 backlog) + §11.3 (ecosystem stability) |
| **D-68** | **TASK-005-2 v2.0 tract Commit 2 abort (W24, 2026-06-10)** — actual `embed()` inference 시도 후 abort. **API 한계**: tract 0.23 의 `Tensor(Arc<InternalTensor>)` wrapper field private + Deref 없음 + `to_array_view` 가 `plain_view::Tensor` (다른 type) 의 method + `Runnable` vs `SimplePlan<InferenceFact, Box<InferenceOp>>` direct cast 어려움. **5+ error 누적** (Tensor type mismatch, Runnable trait bound, to_array_view method missing). 변경 모두 revert. **lesson**: tract 0.23 의 low-level API 가 high-level inference 에 적합하지 않음 — `tract_onnx` 의 prelude 가 type-safe 한 wrap 을 노출 안 함. **v2.0 ONNX 백로그 OOS 유지**. 대안: Plugin 4-계층 또는 외부 blocker 해결 | §6 (v2.0 backlog) + §11.3 (ecosystem stability) |
| **D-73** | **T-v5-sync-1 plan_3c8c4a49 cancel + 4-plan split (2026-06-11)** — my_harness v0.5.0→v0.5.11 sync 11 minor 24 commit. **WHY 4 split**: 1 plan = 6 batch × 11 minor = scope 3x underestimate (실제 73 files / 7,941 ins / 5,070 del). 31min cap 도달 시 25min = exploration + 0 write. **4 plan**: A (5.1 MCP per-harness + round-trip, ~5 files, 3 commit, 워커 warm-up) + B (5.2 bootstrap_lib refactor, **73 files / 7,941+ ins / 5,070 del**, 8 modules 3,710 lines, 9 chunk + 800줄 cap, 필요 시 2 plan sub-split) + C (5.3-5.4 antigravity MCP + contract v1 도입, ~5 files + Rust caller 갱신, API breaking) + D (5.5-5.11 12 commit + R-4 SSOT drift, ~10 files + R-4, smart update + Mavis engine hook + MCP fix). **3 memory entries** (mavis memory): (1) T-v5-sync-1 실패 분석 (scope 3x underestimate + 25min exploration + module name 오류), (2) Worker prompt 3원칙 (first-chunk-largest / JSON split / Edit-last + Write fallback), (3) Cross-session memory grep 1-liner (worker prompt 머리에 grep 1줄). **async audit**: plan_3c8c4a49 cancelled, 워커 idle, T-SAW-1 별도 owner session 운영 중, pending ops 0건 | §6 (TASK-005-2 v1.5 phase 잔여 — major version sync) + §11.3 (ecosystem stability + scope 측정) |

## 5. 관련 문서 (SSOT)

- ★ [docs/CONCEPT.md](../../docs/CONCEPT.md) — my_harness v1 컨셉 SSOT
- [docs/development_log.md](../../docs/development_log.md) — 결정 이력 D-01~D-39
- [docs/references/README.md](../../docs/references/README.md) — 7-doc 통합 인덱스
- [docs/references/claude-code.md](../../docs/references/claude-code.md) — closed source 분석 (D-21)
- [docs/references/headroom.md](../../docs/references/headroom.md) — context compression (D-15)
- [docs/references/PROVIDERS.md](../../docs/references/PROVIDERS.md) — LLM provider 비교
- [docs/skills/provider-auto-config/SKILL.md](../../docs/skills/provider-auto-config/SKILL.md) — TASK-008 reference design
