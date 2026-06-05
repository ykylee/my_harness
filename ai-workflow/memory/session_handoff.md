# Session Handoff

- Purpose: Compact restore context for the next AI agent session.
- Scope: current focus, task status, key changes, next actions, risks
- Audience: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- Status: active
- Updated: 2026-06-05
- Related docs: [Project Profile](../../docs/PROJECT_PROFILE.md), [Work Backlog](./work_backlog.md), [State Cache](./state.json)

## Current Focus

- **방향 전환 (2026-06-05 22:00)** — yklee 가 `my_harness` 를 단순 워크플로우 컨슈머가 아니라 *직접 개발/배포할 CLI/TUI 코딩 에이전트 하네스* 의 소스 트리로 키우기로 결정.
  - 타겟 플랫폼: **Windows / Linux / macOS** 동시 지원
  - 표준 AI 워크플로우 (state.json / handoff / backlog / minimax-code 오버레이) 는 그대로 준수
  - 3-도메인 (코드 개발 / 서버 관리 / 환경 셋업) 진입점은 미래 CLI 의 기능 스코프
- **레퍼런스 수집 완료** — 5개 오픈소스 하네스를 `/Users/yklee/repos/harness-refs/` 에 클론 (총 1.1GB):
  - OpenCode (sst/opencode, Go+TS), Aider (Python), Codex CLI (openai/codex, Rust+TS), Goose (block/goose → aaif.io, Rust), Gemini CLI (TS)
  - 4/5 가 `AGENTS.md` / `GEMINI.md` / `CLAUDE.md` 진입점 보유 — 우리 표준 워크플로우가 산업 표준과 정합
- 다음 세션은 TASK-004 (레퍼런스 비교 분석) 부터 진행.

## Work Status

- TASK-001 my-harness 부트스트랩 (standard_ai_workflow 적용): done
- TASK-002 [차기] 하네스 워커 smoke check 컨슈머 보정: planned
- TASK-003 [차기] 도메인별 명령 가이드 작성 (코드/서버/환경): planned
- TASK-004 [즉시] CLI/TUI 툴 레퍼런스 5종 비교 분석: planned
- TASK-005 [방향 확정 후] my_harness 의 CLI/TUI 툴 전환: planned
- N/A: blocked

## Key Changes

- 2026-06-05 1차 (21:xx) — `bootstrap_workflow_kit.py` 로 minimax-code 오버레이 적용. 1bfae06 커밋.
- 2026-06-05 2차 (21:xx) — 스코프를 3-도메인으로 확장. `MiniMax.md` 와 `PROJECT_PROFILE.md` 보강. 0266610 커밋.
- 2026-06-05 3차 (22:xx) — **방향 전환**: 5개 오픈소스 하네스 레퍼런스 클론. TASK-004/005 추가.

## 다음에 할 일 (Next Actions)

- [ ] **TASK-004 즉시 착수** — 5개 레퍼런스를 8축(언어/TUI/cross-platform/토폴로지/컨텍스트/세션/확장/워크플로우 호환)으로 비교 분석. 결과는 `docs/REFERENCES.md` 에 1차 정리.
- [ ] TASK-002/003 은 우선순위 medium 으로 후순위 (방향 전환으로 인함). 단, TASK-005 진행 중 새 CLI 의 워크플로우 레이어 설계 시 TASK-002/003 의 결론 재활용 가능.
- [ ] TASK-004 종료 후 yklee 와 디자인 리뷰 — 언어 / TUI 라이브러리 / 패키징 / 이름 결정 → TASK-005 세부 분해.

## Risks & Blockers

- 5개 레퍼런스 중 **Goose** 가 Agentic AI Foundation 으로 이전 중. URL `block/goose` 는 동작하나 곧 aaif.io 가 canonical 이 될 가능성 — 분석엔 영향 없음, 추적.
- `my_harness` 의 현재 standard_ai_workflow 기반 구조와 미래 CLI/TUI 구조가 공존해야 함. 빌드 시스템 / 의존성 / 진입점 충돌 가능. TASK-005.1 (스택 결정) 시 우선 정리.
- 디스크 사용: harness-refs 1.1GB. 시간 지나도 분석용이라 유지가 기본. 불필요시 `trash` 로 회수.
