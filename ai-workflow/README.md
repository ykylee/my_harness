# Standard AI Workflow Kit

- 문서 목적: `My Harness` 저장소에 표준 AI 워크플로우 기본 문서 세트를 도입할 수 있도록 bootstrap 결과를 안내한다.
- 범위: 공통 코어 문서 위치, 프로젝트 상태 문서 세트, 도입 모드별 후속 작업
- 대상 독자: 개발자, 운영자, AI agent, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: 2026-06-05
- 관련 문서: `docs/PROJECT_PROFILE.md`, `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`

## 1. 도입 모드

- 선택한 도입 모드: `new`
- 요약:
- 신규 프로젝트용 기본 문서 세트를 생성했다.

## 2. 생성된 파일

- [docs/PROJECT_PROFILE.md](../docs/PROJECT_PROFILE.md)
- [ai-workflow/memory/state.json](./memory/state.json)
- [ai-workflow/memory/session_handoff.md](./memory/session_handoff.md)
- [ai-workflow/memory/work_backlog.md](./memory/work_backlog.md)
- [ai-workflow/memory/backlog/2026-06-05.md](./memory/backlog/2026-06-05.md)


## 3. 코어 문서

- [core/global_workflow_standard.md](./core/global_workflow_standard.md)
- [core/workflow_skill_catalog.md](./core/workflow_skill_catalog.md)
- [core/workflow_mcp_candidate_catalog.md](./core/workflow_mcp_candidate_catalog.md)
- [core/workflow_agent_topology.md](./core/workflow_agent_topology.md)
- [core/output_schema_guide.md](./core/output_schema_guide.md)
- [core/workflow_adoption_entrypoints.md](./core/workflow_adoption_entrypoints.md)
- [core/workflow_harness_distribution.md](./core/workflow_harness_distribution.md)

## 4. 하네스 오버레이

- `minimax-code` 하네스용 오버레이 파일 생성

## 5. 도입 직후 해야 할 일

1. `PROJECT_PROFILE.md` 에 프로젝트 목적, 명령, 검증 규칙을 실제 값으로 채운다.
2. `state.json`, `session_handoff.md`, 오늘 날짜 backlog 를 현재 진행 작업 기준으로 갱신한다.
3. 기존 프로젝트 모드였다면 `repository_assessment.md` 의 추정값을 실제 저장소 규칙과 대조해 수정한다.
4. 선택한 하네스가 있으면 생성된 overlay 파일을 각 하네스 실행 경로에 맞게 검토한다.
5. 이후 표준 skill/MCP 도입 범위는 `core/` 문서를 기준으로 결정한다.

## 6. 언어와 컨텍스트 운영 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, handoff/backlog 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 중간 분류는 모델이 가장 효율적인 형태로 처리하고, 사용자에게는 필요한 결론만 짧게 전달한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 7. 프로젝트 실제 문서 경로 설정값

- 문서 위키 홈: `README.md`
- 운영 문서 위치: `ai-workflow/memory/`
- 백로그 위치: `ai-workflow/memory/backlog/`
- 세션 인계 문서 위치: `ai-workflow/memory/session_handoff.md`
- 환경 기록 위치: `ai-workflow/memory/environments/`

## 다음에 읽을 문서

- 프로젝트 프로파일: [../docs/PROJECT_PROFILE.md](../docs/PROJECT_PROFILE.md)
- 빠른 상태 요약: [./memory/state.json](./memory/state.json)
- 세션 인계 문서: [./memory/session_handoff.md](./memory/session_handoff.md)
- 작업 백로그 인덱스: [./memory/work_backlog.md](./memory/work_backlog.md)
