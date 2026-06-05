# Workflow Agent Topology

- 문서 목적: 배포형 표준 워크플로우에서 사용할 agent 역할과 권한 경계를 요약한다.
- 범위: 추천 agent 목록, 책임, 입력/출력, 상태 문서 수정 원칙
- 대상 독자: AI agent 설계자, 개발자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-21
- 관련 문서: `workflow_skill_catalog.md`, `workflow_mcp_candidate_catalog.md`, `../templates/project_workflow_profile_template.md`

## 1. 1차 필수 agent

| agent | 역할 | 입력 | 출력 |
| --- | --- | --- | --- |
| `session-orchestrator` | 세션 시작 기준선 복원 | handoff, 백로그, 프로젝트 프로파일 | 현재 상태 요약, 다음 행동 |
| `backlog-steward` | 작업 등록/갱신 보조 | 오늘 날짜 백로그, 작업 브리핑 | 작업 항목 초안, 상태 갱신 초안 |
| `doc-sync-guardian` | 문서 동기화 점검 | 변경 파일, 기준 문서, 허브 문서 | 영향 문서 후보, stale 경고 |

## 2. 2차 확장 agent

| agent | 역할 | 입력 | 출력 |
| --- | --- | --- | --- |
| `validation-coordinator` | 검증 수준 결정 | 변경 요약, 프로젝트 프로파일 | 검증 계획, 결과 요약 |
| `merge-reconciler` | 병합 후 정합성 복구 | 병합 결과, handoff, 허브 문서 | 재확정 포인트 |
| `workflow-governor` | 공통/특화 규칙 경계 관리 | 표준 문서, 프로젝트 프로파일 | 체계 변경 제안 |
| `doc-worker` | 대량 문서 읽기, 비교, 초안 작성 | 지정 문서 집합, 변경 요약 | 비교 요약, 문안 초안 |
| `code-worker` | 범위가 명확한 구현, 코드/설정 수정, 빌드 작업 | 지정 파일, 변경 요구, 빌드/검증 기준 | bounded patch, 빌드 결과, 핵심 리스크 |
| `validation-worker` | 검증 실행과 증빙 수집 | 지정 명령, 로그 범위, 기록 위치 | pass/fail 요약, 증빙 메모 |

## 3. 권한 매트릭스

| agent | 읽기 가능 | 수정 가능 | 수정 금지 또는 주의 |
| --- | --- | --- | --- |
| `session-orchestrator` | handoff, 백로그, 프로젝트 프로파일 | 기본적으로 없음 | 상태 문서를 직접 완료 처리하지 않음 |
| `backlog-steward` | 오늘 날짜 백로그, handoff, 프로젝트 프로파일 | 날짜별 백로그 초안, 상태 갱신 초안 | 검증 없이 `done` 확정 금지 |
| `doc-sync-guardian` | 기준 문서, 허브 문서, 변경 파일 목록 | 허브 갱신 제안 문안 | 상태 문서를 직접 사실 확정 용도로 수정하지 않음 |
| `validation-coordinator` | 프로젝트 프로파일, 검증 문서, 결과 요약 | 검증 계획/요약 초안 | 테스트 결과를 확인하지 않고 성공 표기 금지 |
| `merge-reconciler` | 병합 후 handoff, 백로그, 허브 문서 | 병합 후 재확정 초안 | 병합 전 상태를 기계적으로 합치지 않음 |
| `workflow-governor` | 표준 문서, 프로젝트 프로파일, skill/MCP 카탈로그 | 체계 변경 제안 문안 | 특정 프로젝트 상태 문서를 직접 운영하지 않음 |

## 4. 상태 문서 수정 원칙

- 존재하지 않는 백로그 파일을 사실처럼 쓰지 않는다.
- 검증하지 않은 작업을 `done` 으로 바꾸지 않는다.
- 외부 리뷰 코멘트보다 저장소 실제 상태를 먼저 확인한다.
- 상태 문서는 가능한 한 보수적으로 수정한다.

## 5. 메인 오케스트레이터 운영 원칙

- 메인 오케스트레이터는 읽기/쓰기 작업을 직접 모두 떠안기보다 서브 에이전트를 적극 활용해 작업을 분담한다.
- 메인 오케스트레이터가 직접 도구 호출을 수행하는 패턴은 기본값이 아니며, 가능하면 task delegation 과 결과 통합만 맡는다.
- 단, 세션 복원에 필요한 최소 read 와 아주 작은 triage read 는 메인 오케스트레이터가 직접 수행해도 된다.
- 대량 파일 탐색, 문서 비교, 로그 확인, 초안 생성처럼 컨텍스트를 빠르게 부풀리는 작업은 가능한 한 서브 에이전트로 분리한다.
- 메인 오케스트레이터는 최종 판단, 우선순위 조정, 산출물 통합, 사용자 보고에 집중한다.
- 서브 에이전트가 수집한 결과는 원문 전체보다 요약과 결정에 필요한 핵심 사실 중심으로 다시 가져온다.
- 읽기 전용 탐색과 쓰기 작업을 분리해, 한 에이전트의 컨텍스트가 다른 작업의 판단을 오염시키지 않도록 한다.
- 문서 수정 초안, 코드 수정 초안, 검증 로그 수집을 서로 다른 서브 에이전트로 나누면 세션 컨텍스트를 더 작고 선명하게 유지할 수 있다.
- ask 는 기본 상호작용 수단이 아니라 genuinely blocking decision 이나 위험한 외부 액션이 있을 때만 쓰고, 나머지는 worker 가 최소 가정으로 계속 진행하는 편을 기본 운영값으로 둔다.
- 단, 최종 상태 확정과 사용자-facing 결정은 메인 오케스트레이터가 다시 확인한 뒤 수행한다.

## 6. 권장 툴 권한 프로필

### 6.1 메인 오케스트레이터

- 목적: 조정, 우선순위 결정, 결과 통합, 사용자 보고
- 권장 툴 성격: task-only delegation
- 기본 허용 권장:
- sub-agent `task` 위임
- 세션 복원용 직접 read:
- `state.json`, `session_handoff.md`, `work_backlog.md`, `PROJECT_PROFILE.md`
- 작은 triage read:
- 단일 파일 또는 단일 경로의 존재 확인, 매우 짧은 단일 파일 확인
- 승인 기반 또는 제한 권장:
- 직접 `edit`
- 직접 `bash`
- 직접 로그 수집
- 직접 네트워크 fetch
- 운영 원칙:
- 세션 복원과 아주 작은 triage read 는 직접 수행할 수 있지만, 그 범위를 넘는 read 는 worker 로 넘기는 편을 기본값으로 둔다.
- 대량 파일 읽기, 비교, 초안 생성, 실제 수정, 검증은 서브 에이전트로 넘기는 편을 기본값으로 둔다.

### 6.2 서브 에이전트

- 목적: 탐색, 초안 생성, 실제 수정, 로그 수집, 검증 실행
- 권장 툴 성격: task-scoped execution
- 기본 허용 권장:
- 맡은 범위 안의 `edit`
- 범위가 명확한 `bash`
- 필요한 로그/테스트 명령
- 운영 원칙:
- write scope 와 책임 파일을 명확히 나눈다.
- 메인 오케스트레이터에는 원문 dump 보다 요약과 핵심 사실을 반환한다.
- 다른 서브 에이전트나 메인 오케스트레이터의 작업을 불필요하게 되돌리지 않는다.
- low-risk 범위에서는 `ask` 를 최소화하고, genuinely blocking case 에서만 상향 질의한다.

## 7. Worker 분화 권장안

- `doc-worker`: 대량 문서 읽기, 문서 비교, 허브/상태 문서 초안 작성에 우선 사용한다.
- `code-worker`: 구현 작업, 코드 수정, 설정 수정, 범위가 좁은 리팩터링, 빌드/컴파일 확인, 테스트 보강에 우선 사용한다.
- `validation-worker`: 테스트 실행, 로그 확인, 검증 증빙 수집, 실패 원인 요약에 우선 사용한다.
- generic `workflow-worker`: 위 세 가지로 나누기 애매한 bounded task 나 임시 작업에 사용한다.

## 8. 모델 분배 권장안 (`main` + `small`)

`main` 과 `small` 두 계층 모델을 함께 운영한다는 가정에서는 아래 분배를 권장한다.

- 메인 오케스트레이터:
- 기본 권장 모델: `main`
- 이유: 우선순위 판단, 결과 통합, 위험 조정, 사용자 보고는 깊은 문맥 유지가 중요하다.

- `doc-worker`:
- 기본 권장 모델: `small`
- 예외: 문서 체계 재설계나 정책 충돌 조정처럼 구조 판단이 크면 `main`

- `code-worker`:
- 기본 권장 모델: `small`
- 예외: 아키텍처 변경, 범위가 넓은 리팩터링, 높은 회귀 위험 수정, 긴 빌드 실패 분석은 `main`

- `validation-worker`:
- 기본 권장 모델: `small`
- 예외: 실패 원인이 복합적이고 로그 해석이 어렵거나, 테스트 전략 재설계가 필요하면 `main`

- generic `workflow-worker`:
- 기본 권장 모델: `small`
- 예외: 여러 도메인 판단을 함께 묶어야 하면 `main`

운영 요약:

- 기본값은 `main orchestrator + small workers`
- `main` 은 조정과 어려운 판단에 집중
- `small` 은 대량 탐색, bounded 수정, 검증 실행에 넓게 활용
- 위험도나 설계 복잡도가 올라가면 해당 worker 만 일시적으로 `main` 으로 승격

## 다음에 읽을 문서

- 프로젝트 프로파일 템플릿: [../templates/project_workflow_profile_template.md](../templates/project_workflow_profile_template.md)
- skill 카탈로그: [./workflow_skill_catalog.md](./workflow_skill_catalog.md)
