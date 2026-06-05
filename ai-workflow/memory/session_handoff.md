# Session Handoff

- Purpose: Compact restore context for the next AI agent session.
- Scope: current focus, task status, key changes, next actions, risks
- Audience: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- Status: active
- Updated: 2026-06-05
- Related docs: [Project Profile](../../docs/PROJECT_PROFILE.md), [Work Backlog](./work_backlog.md), [State Cache](./state.json)

## Current Focus

- **my_harness 스코프 확정** — yklee의 개인 코딩 에이전트 하네스로 다음 3개 도메인을 커버:
  1. **코드 개발 전반** (구현/리팩토링/리뷰/PR)
  2. **기본 서버 관리** (프로세스/로그/설정/배포 헬퍼)
  3. **환경 셋업** (로컬/원격 부트스트랩, 의존성, dotfiles)
- 모든 작업은 `standard_ai_workflow` (ykylee/standard_ai_workflow) 의 코어 표준을 따른다.
- 다음 세션은 `MiniMax.md` → `state.json` → 본 handoff → `work_backlog.md` → `docs/PROJECT_PROFILE.md` 순으로 세션 복원.

## Work Status

- TASK-001 my-harness 부트스트랩 (standard_ai_workflow 적용): done
- TASK-002 [차기] 하네스 워커별 smoke check 컨슈머 보정: planned
- TASK-003 [차기] 도메인별 명령 가이드 작성 (코드/서버/환경): planned
- N/A: blocked

## Key Changes

- 2026-06-05 1차 부트스트랩 — `bootstrap_workflow_kit.py` 로 minimax-code 오버레이 적용.
- 2026-06-05 2차 컨텍스트 보강:
  - `docs/PROJECT_PROFILE.md` §1 목적에 3개 도메인 명시, §3.1 도메인별 명령 섹션 신설, §4 검증 포인트 도메인별 추가.
  - `MiniMax.md` 의 "프로젝트 실행 기본값" 섹션을 **코드 개발 / 서버 관리 / 환경 셋업** 3개 블록으로 재구조화.
  - `README.md` 의 첫 줄을 신규 목적에 맞춰 갱신.
  - `state.json` 재생성.

## 다음에 할 일 (Next Actions)

- [ ] TASK-003: 도메인별 표준 명령을 `MiniMax.md` §"프로젝트 실행 기본값" 과 `docs/PROJECT_PROFILE.md` §3.1 에 채우기. yklee 의 실제 셋업 / 서버 호스트 / dotfiles 경로 정보 필요.
- [ ] TASK-002: `ai-workflow/tests/check_*.py` 컨슈머 레이아웃 보정 (소스 프레임워크 `workflow-source/` 경로 의존성 제거)
- [ ] `.MiniMax/config.json` 을 `MiniMax_config.example.json` 으로 초기화 (서버 토큰 등 시크릿은 환경변수 주입)
- [ ] 도메인별 첫 실제 작업 발생 시 일별 백로그(`ai-workflow/memory/backlog/<date>.md`)에 TASK 등록 → `state.json` 재생성

## Risks & Blockers

- `ai-workflow/tests/check_*.py` 일부가 소스 프레임워크 레이아웃 가정 — 컨슈머에서는 `ModuleNotFoundError: workflow_kit` 또는 경로 오류로 실패 (TASK-002 로 대응).
- `MiniMax.md` 가 `AGENTS.md` 를 진입점으로 함께 언급하나, 현재 컨슈머에는 `AGENTS.md` 미생성. AGENTS.md 가 실제로 필요한지(외부 4-워커 워크플로우와 직교 가능성) yklee 결정 대기.
- 도메인별 표준 명령이 미정(TODO). 첫 코드/서버/환경 작업 시 어느 도메인 표준을 따를지 Mavis 가 자체 추론해야 하므로 결정론적이지 않을 수 있음.
