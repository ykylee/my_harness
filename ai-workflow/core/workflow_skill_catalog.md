# Workflow Skill Catalog

- 문서 목적: 표준 워크플로우를 skill 로 분리할 때 우선순위와 책임을 정리한다.
- 범위: skill 후보, 역할, 입력 문서, 기대 출력, 도입 우선순위
- 대상 독자: AI agent 설계자, 개발자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-21
- 관련 문서: `workflow_agent_topology.md`, `workflow_mcp_candidate_catalog.md`, `../templates/project_workflow_profile_template.md`

## 1. 핵심 도입 skill (Beta v2 완료)

| skill | 역할 | 주요 입력 | 기대 출력 | 구현 상태 | 수동 대체 |
| --- | --- | --- | --- | --- | --- |
| `session-start` | 세션 시작 기준선 복원 | handoff, 백로그, 프로젝트 프로파일 | 현재 상태 요약, 다음 문서 경로 | **Beta** (읽기/요약 안정화) | `global_workflow_standard.md` 의 세션 시작 순서를 수동 수행 |
| `backlog-update` | 작업 등록/갱신 | 오늘 날짜 백로그, 작업 브리핑 | 신규 작업 항목 초안, 상태 갱신 문안 | **Beta** (쓰기 지원) | 백로그 템플릿을 복사해 수동 갱신 |
| `doc-sync` | 문서 영향도와 허브 갱신 판단 | 변경 파일, 기준 문서, 허브 문서 | 영향 문서 후보, 허브 갱신 체크 | **Beta** (`--apply` 지원) | 변경 파일을 기준으로 관련 허브 문서를 수동 확인 |
| `merge-doc-reconcile` | 병합 후 문서 정합성 복구 | 병합 결과, handoff, 인덱스 문서 | 병합 후 재확정 포인트 | **Beta** (정합성 자동 복구) | 병합 후 handoff, 허브, 색인 문서를 수동 재정리 |
| `validation-plan` | 변경 유형별 검증 수준 판단 | 변경 요약, 프로젝트 프로파일 | 검증 계획, 미실행 사유 | **Beta** (`--scaffold` 지원) | 프로젝트 프로파일과 공통 표준을 읽고 수동 판단 |
| `code-index-update` | 색인 문서 갱신 판단 | 변경 파일, 기존 색인 문서 | 갱신 필요 색인 후보 | **Beta** (`--apply` 지원) | 변경 경로를 기준으로 색인 문서를 수동 검토 |

## 2. 운영 보조 및 지능화 skill (Beta 진입)

| skill | 역할 | 주요 입력 | 기대 출력 | 구현 상태 | 수동 대체 |
| --- | --- | --- | --- | --- | --- |
| `workflow-linter` | 워크플로우 문서 정합성 교정 | state.json, handoff, 백로그 | 불일치 리포트 및 교정안 | **Beta** (정합성 자동 검사) | 문서 간 TASK 상태와 링크를 수동 대조 |
| `project-status-assessment` | 프로젝트 도입 성숙도 진단 | 저장소 전체 구조, 테스트, 문서 | 성숙도 리포트, 보강 추천 | **Beta** (자동 진단 및 리포팅) | `repository_assessment.md` 를 수동 작성 |
| `automated-repro-scaffold` | 버그 재현 환경 자동 구축 | 버그 리포트, 기존 테스트 코드 | 재현 테스트 파일(`repro_*.py`) | 프로토타입 (`validation-plan` 연동) | 버그 리포트를 읽고 테스트 코드를 수동 작성 |

운영 보조 원칙:

- `backlog-update`, `merge-doc-reconcile` 는 source-of-truth 문서가 준비된 경우 `state.json` 을 자동 재생성해 빠른 세션 기준선을 맞춘다.
- `doc-sync`, `validation-plan`, `code-index-update`, `merge-doc-reconcile` 는 `ai-workflow/` 경로를 workflow 메타 레이어로 보고, 일반 프로젝트 문서/변경 탐색 범위에서는 기본적으로 제외한다.

## 3. 최소 입력 계약

- `session-start`: 현재 프로젝트의 handoff 문서, 백로그 인덱스, 프로젝트 프로파일 경로
- `backlog-update`: 오늘 날짜 백로그 경로 또는 생성 대상 날짜, 작업명, 작업 브리핑
- `doc-sync`: 변경 파일 목록, 기준 문서 후보, 허브 문서 후보
- `merge-doc-reconcile`: 병합 후 상태 문서와 허브 문서 경로

## 4. 상세 스펙 진행 상태

- `session-start`: 상세 입력/출력 계약 + 실행형 프로토타입 + 구조화된 실패 출력 시범 패턴 있음
- 참고 문서: [./session_start_skill_spec.md](./session_start_skill_spec.md)
- `backlog-update`: 상세 입력/출력 계약 + 실행형 초안 생성 프로토타입 + 구조화된 실패 출력 패턴 있음
- 참고 문서: [./backlog_update_skill_spec.md](./backlog_update_skill_spec.md)
- `doc-sync`: 상세 입력/출력 계약 + 실행형 읽기 전용 프로토타입 + 구조화된 실패 출력 패턴 있음
- 참고 문서: [./doc_sync_skill_spec.md](./doc_sync_skill_spec.md)
- `merge-doc-reconcile`: 상세 입력/출력 계약 + 실행형 읽기 전용 프로토타입 + 구조화된 실패 출력 패턴 있음
- 참고 문서: [./merge_doc_reconcile_skill_spec.md](./merge_doc_reconcile_skill_spec.md)
- `validation-plan`: 상세 입력/출력 계약 + 실행형 읽기 전용 프로토타입 + 구조화된 실패 출력 패턴 있음
- 참고 문서: [./validation_plan_skill_spec.md](./validation_plan_skill_spec.md)
- `code-index-update`: 상세 입력/출력 계약 + 실행형 읽기 전용 프로토타입 + 구조화된 실패 출력 패턴 있음
- 참고 문서: [./code_index_update_skill_spec.md](./code_index_update_skill_spec.md)

## 5. 설계 원칙

- skill 은 정책 원문이 아니라 절차와 판단 순서를 담당한다.
- skill 은 가능하면 프로젝트 프로파일을 읽고 분기해야 한다.
- tool 이 없어도 최소 수동 절차는 남아 있어야 한다.

## 6. 권장 skill 묶음

이번 릴리즈에서는 개별 skill 자체보다 “어떤 순서로 묶어 쓰는가” 를 더 중요하게 본다.

### 6.1 기존 프로젝트 첫 세션 묶음

1. `session-start`
2. `validation-plan`
3. `code-index-update`

목적:

- 현재 handoff/backlog 기준선 복원
- 첫 검증 계획 수립
- README/허브/index 재확인 후보 정리

### 6.2 일상 작업 진행 묶음

1. `backlog-update`
2. `doc-sync`

목적:

- 오늘 작업 등록/갱신
- 영향 문서 후보와 허브 갱신 포인트 정리

### 6.3 병합 후 정리 묶음

1. `merge-doc-reconcile`
2. 필요 시 `doc-sync`

목적:

- 병합 후 handoff/backlog/index 정합성 회복
- 허브 문서 누락 여부 재확인

### 6.4 이번 릴리즈 기준 제외 범위

- MCP 를 기본 진입 경로로 두는 skill 구성
- 하네스 자동 연결을 전제로 한 tool-first 소비 경로

즉, 이번 릴리즈에서는 skill 을 사람이 읽는 workflow 순서와 runner 결과 소비 순서에 맞춰 배치하는 것이 우선이다.

## 다음에 읽을 문서

- `session-start` 상세 스펙: [./session_start_skill_spec.md](./session_start_skill_spec.md)
- `backlog-update` 상세 스펙: [./backlog_update_skill_spec.md](./backlog_update_skill_spec.md)
- `doc-sync` 상세 스펙: [./doc_sync_skill_spec.md](./doc_sync_skill_spec.md)
- `merge-doc-reconcile` 상세 스펙: [./merge_doc_reconcile_skill_spec.md](./merge_doc_reconcile_skill_spec.md)
- `validation-plan` 상세 스펙: [./validation_plan_skill_spec.md](./validation_plan_skill_spec.md)
- `code-index-update` 상세 스펙: [./code_index_update_skill_spec.md](./code_index_update_skill_spec.md)
- MCP 후보 카탈로그: [./workflow_mcp_candidate_catalog.md](./workflow_mcp_candidate_catalog.md)
- agent 토폴로지: [./workflow_agent_topology.md](./workflow_agent_topology.md)
