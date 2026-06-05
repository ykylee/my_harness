# Session Handoff

- Purpose: Compact restore context for the next AI agent session.
- Scope: current focus, task status, key changes, next actions, risks
- Audience: yklee, Mavis orchestrator, .MiniMax 워커 에이전트
- Status: active
- Updated: 2026-06-05
- Related docs: [Project Profile](../../docs/PROJECT_PROFILE.md), [Work Backlog](./work_backlog.md), [State Cache](./state.json)

## Current Focus

- **my_harness 부트스트랩 완료** — `standard_ai_workflow` (ykylee/standard_ai_workflow) 의 `minimax-code` 하네스 오버레이를 이 저장소에 적용.
- 다음 세션은 본 하네스(`MiniMax.md` + `.MiniMax/agents/`)를 진입점으로 워크플로우 세션을 시작.

## Work Status

- TASK-001 my-harness 부트스트랩 (standard_ai_workflow 적용): done
- TASK-002 [차기] 하네스 워커별 smoke check 추가: planned
- N/A: blocked

## Key Changes

- `bootstrap_workflow_kit.py` 실행으로 minimax-code 하네스 overlay 적용.
- `MiniMax.md` (Mavis 진입점), `MiniMax_config.example.json`, `.MiniMax/agents/workflow-*.md` (orchestrator/worker/doc/code/validation) 생성.
- `docs/PROJECT_PROFILE.md` yklee 하네스 컨텍스트로 커스텀 (TODO 플레이스홀더 제거).
- `ai-workflow/memory/state.json` 재생성 — `commands` / `project` / `session` 필드 보정.
- `ai-workflow/core/global_workflow_standard.md` 등 코어 문서 7종 복사.
- `ai-workflow/skills/`, `mcp_servers/`, `workflow_kit/`, `tests/` 등 표준 키트 자산 포함.

## 다음에 할 일 (Next Actions)

- [ ] `MiniMax.md` 의 TODO 5개 항목(설치/실행/테스트/실행확인 명령) — 실제 하네스 운영 명령으로 채우기
- [ ] `.MiniMax/config.json` 을 `MiniMax_config.example.json` 으로 초기화 (서버 토큰 등 시크릿은 환경변수 주입)
- [ ] 첫 실제 작업(예: 개인 PR 리뷰 봇, 코드 인덱서 등) 진행 시 `ai-workflow/memory/backlog/<date>.md` 에 태스크 등록 → `state.json` 동기화
- [ ] 워커 스모크(`ai-workflow/tests/check_*.py`)를 컨슈머 레이아웃에 맞게 보정 (소스 프레임워크 `workflow-source/` 경로 의존성 제거)

## Risks & Blockers

- `ai-workflow/tests/check_*.py` 일부가 소스 프레임워크 레이아웃(`workflow-source/` at root) 가정 — 컨슈머에서는 `ModuleNotFoundError: workflow_kit` 또는 경로 오류로 실패. 첫 정식 세션에서 컨슈머용 smoke 으로 재작성 검토.
- `MiniMax.md` 가 `AGENTS.md` 를 진입점으로 함께 언급하나, 현재 컨슈머에는 `AGENTS.md` 미생성 — 필요 시 별도 생성 또는 `MiniMax.md` 문구 보정.
