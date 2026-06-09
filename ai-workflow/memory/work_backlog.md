# 작업 백로그 인덱스

- 문서 목적: 프로젝트의 모든 작업 항목과 날짜별 백로그 링크를 관리한다.
- 범위: 전체 태스크 목록, 우선순위, 진행 상태, 날짜별 기록 연결
- 대상 독자: 개발자, AI 에이전트, 프로젝트 매니저
- 상태: stable
- 최종 수정일: 2026-06-09 (D-44, dual-remote 적용 — Gitea PAT + GitHub upstream + 9 commit 양쪽 push)
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

### In Progress
- [ ] **TASK-005-1** (v1.0 MVP, Rust 빌드) — W3~W6.5 + W7 + W8 + W9 완료 (D-43, D-45, D-46, D-47) + dual-remote push. W10 tui shell 또는 W11 standard_ai_workflow 대기.

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
- [ ] W10 tui shell + sub-agent 1-2개
- [ ] W11 standard_ai_workflow output + 4 permission 완성
- [ ] **TASK-005-2** (v1.5) — Plugin 4-계층 + marketplace + auto memory + provider-auto-config skill 정식 구현 + CCR/Kompress-base
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

## 5. 관련 문서 (SSOT)

- ★ [docs/CONCEPT.md](../../docs/CONCEPT.md) — my_harness v1 컨셉 SSOT
- [docs/development_log.md](../../docs/development_log.md) — 결정 이력 D-01~D-39
- [docs/references/README.md](../../docs/references/README.md) — 7-doc 통합 인덱스
- [docs/references/claude-code.md](../../docs/references/claude-code.md) — closed source 분석 (D-21)
- [docs/references/headroom.md](../../docs/references/headroom.md) — context compression (D-15)
- [docs/references/PROVIDERS.md](../../docs/references/PROVIDERS.md) — LLM provider 비교
- [docs/skills/provider-auto-config/SKILL.md](../../docs/skills/provider-auto-config/SKILL.md) — TASK-008 reference design
