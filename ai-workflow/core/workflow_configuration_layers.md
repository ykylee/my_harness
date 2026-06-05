# Workflow Configuration Layers

- 문서 목적: 표준 AI 워크플로우를 하네스 전역 설정과 프로젝트별 커스터마이징에 함께 주입할 때 필요한 계층 구조와 우선순위 규칙을 정의한다.
- 범위: global config, shared workflow package, project-local override, 충돌 해결 원칙, 하네스별 적용 관점
- 대상 독자: 저장소 관리자, 하네스 통합 담당자, AI workflow 설계자, 운영 담당자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `./workflow_harness_distribution.md`, `./workflow_adoption_entrypoints.md`, `./workflow_global_injection_policy.md`, `../harnesses/codex/apply_guide.md`, `../harnesses/opencode/apply_guide.md`
- 상태 문서/프로젝트 문서 경계: `./workflow_state_vs_project_docs.md`

## 1. 왜 계층이 필요한가

표준 워크플로우를 실제 운영에 넣을 때는 아래 두 요구가 동시에 존재한다.

1. 하네스 전역 설정에 기본 workflow 를 주입하고 싶다.
2. 각 프로젝트는 자기 규칙에 맞게 workflow 를 수정하거나 보강할 수 있어야 한다.

이 둘을 동시에 만족하려면, 설정을 한 파일에 몰아넣는 대신 계층으로 나누고 우선순위를 명확히 해야 한다.

## 2. 권장 계층 구조

권장 계층은 아래 3단계다.

1. Harness Global Layer
2. Shared Workflow Layer
3. Project Local Layer

### 2.1 Harness Global Layer

역할:

- 하네스가 항상 읽을 전역 기본 규칙
- 공통 작업 태도, 기본 안전 규칙, 공통 시작 문서 위치

예시:

- Codex 전역 `config.toml`
- OpenCode 사용자 전역 설정 또는 기본 에이전트 정책

이 레이어에는 프로젝트 특화 명령이나 경로를 넣지 않는다.

### 2.2 Shared Workflow Layer

역할:

- 여러 프로젝트에 공통으로 복제하거나 배포할 표준 workflow 패키지
- 코어 문서, 템플릿, 하네스 오버레이 기본형

예시:

- `ai-workflow/core/*.md`
- `ai-workflow/memory/*.md` 초안
- 루트 `AGENTS.md`
- `opencode.json`, `.opencode/...`, `.codex/config.toml.example`

이 레이어는 조직 또는 팀 공통 기준선 역할을 한다.

### 2.3 Project Local Layer

역할:

- 특정 저장소에서만 유효한 명령, 실제 프로젝트 문서 경로, 승인 규칙, 환경 제약
- 현재 workflow state 와 세션 기준선

예시:

- `ai-workflow/memory/PROJECT_PROFILE.md`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- 최신 날짜 backlog 문서

주의:

- `ai-workflow/memory/*` 는 workflow state docs 다.
- `PROJECT_PROFILE.md` 안의 `docs/...` 경로는 실제 프로젝트 문서 위치를 가리킨다.
- 즉, `ai-workflow/memory/backlog/*.md` 에 작업 상태를 기록하는 것과 `docs/operations/backlog/*.md` 같은 실제 운영 문서를 참조하거나 동기화하는 것은 서로 다른 레이어다.

이 레이어는 항상 최우선으로 수정 가능해야 한다.

## 3. 우선순위 규칙

충돌 시 우선순위는 아래와 같이 둔다.

1. Project Local Layer
2. Shared Workflow Layer
3. Harness Global Layer

즉, 전역 설정은 기본값만 제공하고, 실제 프로젝트 규칙은 로컬 문서가 덮어쓰는 구조가 되어야 한다.

## 4. 무엇을 어디에 두면 안 되는가

- 프로젝트별 실행 명령을 하네스 전역 설정에 직접 넣지 않는다.
- 특정 저장소 backlog 상태를 공유 workflow 패키지에 넣지 않는다.
- 하네스 전용 권한 정책에 프로젝트 특화 승인 규칙을 강하게 고정하지 않는다.

이런 정보는 시간이 지나면 바뀌므로 항상 project-local 문서에서 조정 가능해야 한다.

## 5. Codex 적용 관점

Codex 에서는 아래처럼 해석하는 것이 적절하다.

- 전역: 사용자 `~/.codex/config.toml` 에 공통 MCP 또는 기본 정책
- 공유: 프로젝트 루트 `AGENTS.md` 와 `ai-workflow/` 패키지
- 로컬: `ai-workflow/memory/` 문서와 backlog/handoff

권장 원칙:

- 전역 Codex 설정은 "어떤 문서를 먼저 보라" 수준까지만 둔다.
- 실제 프로젝트 명령과 상태는 `AGENTS.md` 와 `ai-workflow/memory/` 문서에서 읽게 한다.

## 6. OpenCode 적용 관점

OpenCode 에서는 아래처럼 해석하는 것이 적절하다.

- 전역: 사용자 기본 agent/permission 정책
- 공유: 프로젝트 루트 `AGENTS.md`, `opencode.json`, `.opencode/...`
- 로컬: `ai-workflow/memory/` 문서와 backlog/handoff

권장 원칙:

- `opencode.json` 은 `AGENTS.md` 와 `ai-workflow/memory/` 문서를 참조하는 연결 계층으로 둔다.
- `.opencode/agents/` 권한 정책은 보수적 기본값으로 두고, 프로젝트 규칙은 문서에서 읽게 한다.
- 메인 오케스트레이터는 task-only coordinator 로 두고, 직접 `bash`/`edit`/`webfetch` 를 수행하지 않은 채 실제 수정/광범위 탐색은 서브 에이전트 쪽 권한으로 분리하는 구성이 기본값에 가깝다.
- 서브 에이전트는 bounded scope 안에서 실제 실행을 담당하고, low-risk 작업에서는 `ask` 를 최소화하는 쪽이 운영성과 컨텍스트 효율에 더 잘 맞는다.
- `ai-workflow/` 문서는 workflow 메타 레이어이므로, 세션 기준선 복원과 상태 기록 외의 프로젝트 문서 탐색에서는 기본적으로 제외하는 편이 좋다.

## 7. 추천 운영 방식

- 1단계:
- 하네스 전역 설정에는 공통 workflow 사용 여부와 기본 문서 진입점만 둔다.
- 2단계:
- 프로젝트에 `ai-workflow/` 패키지와 하네스 overlay 를 주입한다.
- 3단계:
- 프로젝트별 명령, 문서 경로, 승인 규칙, backlog 상태를 local 문서에서 확정한다.

이 방식이면 전역 설정을 바꾸지 않고도 프로젝트별 커스터마이징이 가능하고, 반대로 프로젝트를 새로 만들 때도 전역 기본값을 재사용할 수 있다.

## 8. 앞으로 추가로 고려할 항목

- 조직 공통 workflow 버전 태그를 어디에 기록할지
- 프로젝트가 shared workflow 에서 어떤 항목을 override 했는지 추적할지
- 전역 설정과 프로젝트 설정이 어긋날 때 어떤 경고를 낼지
- 하네스별 global config snippet 을 별도 디렉터리로 관리할지

## 다음에 읽을 문서

- 하네스 배포 가이드: [./workflow_harness_distribution.md](./workflow_harness_distribution.md)
- 비침투적 주입 정책: [./workflow_global_injection_policy.md](./workflow_global_injection_policy.md)
- Codex 적용 가이드: [../harnesses/codex/apply_guide.md](../harnesses/codex/apply_guide.md)
- OpenCode 적용 가이드: [../harnesses/opencode/apply_guide.md](../harnesses/opencode/apply_guide.md)
