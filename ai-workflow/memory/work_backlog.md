# 작업 백로그 인덱스

- 문서 목적: 프로젝트의 모든 작업 항목과 날짜별 백로그 링크를 관리한다.
- 범위: 전체 태스크 목록, 우선순위, 진행 상태, 날짜별 기록 연결
- 대상 독자: 개발자, AI 에이전트, 프로젝트 매니저
- 상태: stable
- 최종 수정일: 2026-06-07 (D-39, v1 컨셉 Phase 종료)
- 관련 문서: [세션 인계](./session_handoff.md), [프로젝트 프로필](../../docs/PROJECT_PROFILE.md), [CONCEPT.md](../../docs/CONCEPT.md)

## 1. 운영 원칙
1. 세션 시작 시 인덱스와 최신 백로그 확인
2. 세션 종료 전 인덱스 및 Handoff 갱신
3. 모든 작업 상태는 날짜별 백로그에 기록

## 2. 날짜별 백로그
- [2026-06-05](./backlog/2026-06-05.md) — 초기 부트스트랩 + 방향 전환
- [2026-06-06](./backlog/2026-06-06.md) — 5 reference clone + Gitea + 5-doc 분석
- [2026-06-07](./backlog/2026-06-07.md) — 7-doc 분석 + CONCEPT.md SSOT + 5 결정 (D-22~D-38)

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
- (없음) — v1 컨셉 Phase 종료, TASK-005-1 (v1 MVP Rust 빌드) 대기

### Planned (다음 세션)
- [ ] **TASK-005-1** (v1.0 MVP, Rust 빌드) — cargo workspace init → ratatui shell → rig-core Anthropic → basic tools → /compact → standard_ai_workflow output
- [ ] **TASK-005-2** (v1.5) — Plugin 4-계층 + marketplace + auto memory + provider-auto-config skill 정식 구현 + CCR/Kompress-base
- [ ] **TASK-005-3** (v2.0) — TUI/IDE/Web hand-off (5 surfaces) + Routines + OAuth + MCP-based discover
- [ ] **TASK-005-4** (v2.5) — Multi-agent parallel + confidence scoring
- [ ] **TASK-005-5** (v3.0) — Computer Use + Multi-user + RBAC
- [ ] **claude-code 2.1.169 changelog 검증** (D-34 §11.2) — Anthropic fallback / context var / MCP 변경 시뮬레이션
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

## 5. 관련 문서 (SSOT)

- ★ [docs/CONCEPT.md](../../docs/CONCEPT.md) — my_harness v1 컨셉 SSOT
- [docs/development_log.md](../../docs/development_log.md) — 결정 이력 D-01~D-39
- [docs/references/README.md](../../docs/references/README.md) — 7-doc 통합 인덱스
- [docs/references/claude-code.md](../../docs/references/claude-code.md) — closed source 분석 (D-21)
- [docs/references/headroom.md](../../docs/references/headroom.md) — context compression (D-15)
- [docs/references/PROVIDERS.md](../../docs/references/PROVIDERS.md) — LLM provider 비교
- [docs/skills/provider-auto-config/SKILL.md](../../docs/skills/provider-auto-config/SKILL.md) — TASK-008 reference design
