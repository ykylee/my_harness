# Existing Project Onboarding Contract

- 문서 목적: `run_existing_project_onboarding.py` 가 기존 프로젝트 도입 직후 어떤 입력을 읽고 어떤 단계별 JSON 출력을 묶어 반환하는지 계약 형태로 정리한다.
- 범위: 입력 인자, 단계별 호출 순서, 각 단계의 핵심 출력, 생략/경고 조건, 후속 사용 지점
- 대상 독자: AI workflow 설계자, 구현자, 프로젝트 온보딩 담당자, 하네스 통합 담당자
- 상태: draft
- 최종 수정일: 2026-04-23
- 관련 문서: `./workflow_adoption_entrypoints.md`, `./output_schema_guide.md`, `./workflow_kit_roadmap.md`, `../scripts/run_existing_project_onboarding.py`, `../tests/check_existing_project_onboarding.py`
- 실제 파일럿 예시: `../examples/pilot_adoption_open_git_client_example.md`

## 1. 목적

기존 프로젝트 도입 모드는 bootstrap 이후에도 사람이 다음 절차를 다시 조합해야 하면 효용이 떨어진다. 이 문서는 `run_existing_project_onboarding.py` 가 어떤 입력을 받아 어떤 순서로 `latest_backlog`, `session-start`, `validation-plan`, `code-index-update` 를 연결하는지 고정하기 위한 계약 문서다.

핵심 목표는 아래 세 가지다.

1. bootstrap 직후 첫 세션에서 읽어야 할 입력을 명확히 한다.
2. 단계별 JSON 출력이 다음 단계에서 어떻게 소비되는지 드러낸다.
3. 신규 프로젝트 도입과 기존 프로젝트 도입의 분기 지점을 문서로 설명한다.

## 2. 입력 계약

### 필수 입력

| 인자 | 의미 | 비고 |
| --- | --- | --- |
| `--project-profile-path` | 기존 프로젝트용 `PROJECT_PROFILE.md` 경로 | bootstrap generated |
| `--session-handoff-path` | 초기 `session_handoff.md` 경로 | bootstrap generated |
| `--work-backlog-index-path` | `work_backlog.md` 경로 | bootstrap generated |
| `--backlog-dir-path` | backlog 디렉터리 경로 | bootstrap generated |

### 선택 입력

| 인자 | 의미 | 사용 시점 |
| --- | --- | --- |
| `--repository-assessment-path` | bootstrap 가 만든 저장소 분석 문서 경로 | 있으면 추정 명령/스택 요약을 결과에 포함 |
| `--latest-backlog-path` | 최신 backlog 를 이미 알고 있을 때 직접 지정 | latest backlog 탐색 단계를 우회 |
| `--changed-file` | 후속 validation/code-index 입력용 변경 파일 | 0개 이상 |
| `--change-summary` | 변경 요약 한 줄 | 변경 파일이 없거나 보조 설명이 필요할 때 |

## 3. 단계별 실행 순서

`run_existing_project_onboarding.py` 는 아래 순서를 고정한다.

1. `repository_assessment.md` 읽기
2. latest backlog 식별
3. `session-start` 실행
4. `validation-plan` 실행
5. `code-index-update` 실행
6. onboarding 요약 조립

이 순서는 “기준선 복원 -> 초기 검증 계획 -> 문서 허브 재확인” 흐름을 최소 비용으로 재현하는 데 맞춰져 있다.

## 4. 단계별 입출력 계약

### 4.1 repository assessment 읽기

입력:

- `repository_assessment.md`

추출 필드:

- `project_name`
- `primary_stack`
- `install_command`
- `run_command`
- `quick_test_command`
- `isolated_test_command`
- `smoke_check_command`
- `sample_paths`

출력 사용 지점:

- 최종 `repository_assessment.summary`
- 최종 `onboarding_summary.inferred_commands`

### 4.2 latest backlog 식별

입력:

- 직접 전달된 `--latest-backlog-path`
- 또는 `work_backlog.md` 와 backlog 디렉터리

출력:

- `status`
- `tool_version`
- `latest_backlog_path`
- `candidates`
- `warnings`

분기 규칙:

- `--latest-backlog-path` 가 있으면 탐색을 생략하고 해당 경로를 그대로 사용한다.
- 직접 경로를 주지 않으면 `latest-backlog` MCP 프로토타입을 호출한다.

후속 사용 지점:

- `session-start --latest-backlog-path`
- `validation-plan --latest-backlog-path`
- 최종 `latest_backlog`

### 4.3 session-start

입력:

- `PROJECT_PROFILE.md`
- `session_handoff.md`
- `work_backlog.md`
- latest backlog 경로

핵심 출력:

- `summary`
- `in_progress_items`
- `blocked_items`
- `recommended_next_action`
- `warnings`

후속 사용 지점:

- 최종 `session_start`
- 사람 검토 기준선

### 4.4 validation-plan

입력:

- `PROJECT_PROFILE.md`
- `session_handoff.md`
- latest backlog 경로
- `changed_files`
- `change_summary`

핵심 출력:

- `recommended_validation_levels`
- `recommended_commands`
- `commands_requiring_confirmation`
- `deferred_validation_items`
- `warnings`

후속 사용 지점:

- 최종 `validation_plan`
- `onboarding_summary.recommended_next_steps` 중 검증 단계 설명의 근거

### 4.5 code-index-update

입력:

- `PROJECT_PROFILE.md`
- `work_backlog.md`
- `session_handoff.md`
- `changed_files`
- `change_summary`

핵심 출력:

- `priority_index_candidates`
- `index_update_candidates`
- `suggested_index_actions`
- `warnings`

후속 사용 지점:

- 최종 `code_index_update`
- `onboarding_summary.recommended_next_steps` 중 허브/README 재확인 단계의 근거

## 5. 최종 출력 계약

온보딩 runner 의 최종 JSON 은 아래 상위 키를 가진다.

| 필드 | 의미 |
| --- | --- |
| `status` | 현재 runner 실행 상태 |
| `tool_version` | 현재 runner 버전 식별자 |
| `onboarding_mode` | 항상 `existing_project_followup` |
| `warnings` | 하위 단계에서 집계한 상위 경고 목록 |
| `orchestration_plan` | 메인 오케스트레이터와 worker 분배 메타데이터 |
| `repository_assessment` | assessment 문서 경로와 요약 |
| `latest_backlog` | latest backlog 결과 |
| `session_start` | 기준선 복원 결과 |
| `validation_plan` | 초기 검증 계획 |
| `code_index_update` | 문서 허브/색인 재확인 후보 |
| `onboarding_summary` | 사람이 바로 읽을 요약 |
| `source_context` | 입력 경로와 변경 요약 |

추가 규칙:

- 정상 경로에서는 `status: "ok"` 를 기본으로 사용한다.
- 하위 step 이 구조화된 `error` JSON 을 반환하거나 비정상 종료하면 runner 도 top-level `error` JSON 으로 감싸서 반환한다.
- 이때 상위 `error_code` 는 runner 분류용 값을 사용하고, 실제 실패 step 이름과 upstream `error_code` 는 `source_context.failed_step`, `source_context.upstream_error_code` 로 남긴다.

## 6. 생략 및 경고 규칙

- `repository_assessment.md` 가 없으면 `repository_assessment.summary` 는 `null` 이고, `review_assessment_first` 는 `false` 가 된다.
- `changed_files` 가 비어 있어도 `change_summary` 가 있으면 validation/code-index 단계는 계속 실행한다.
- 최신 backlog 를 찾지 못해도 runner 자체는 가능한 범위의 결과를 계속 반환한다.
  이 경우 `latest_backlog.latest_backlog_path` 와 top-level `source_context.latest_backlog_path` 는 `null` 이고, 관련 경고는 `latest_backlog.warnings` 및 top-level `warnings` 에 남는다.
- 세부 프로토타입의 도메인 경고는 각 단계의 `warnings` 로 남기고, runner 는 이를 축약하지 않는다.

### 6.1 실패 시 기대 동작

- `project_profile_path`, `session_handoff_path`, `work_backlog_index_path`, `backlog_dir_path` 같은 필수 입력 경로가 없으면 runner 는 즉시 `status: "error"` 를 반환한다.
- `repository_assessment_path` 는 선택 입력이므로 누락 자체가 즉시 실패 조건은 아니다.
- 하위 step 이 실패해도 가능한 한 그 실패를 구조화된 JSON 으로 감싸 반환하고, 호출자는 Python 예외 대신 JSON 상태값을 우선 읽는 편이 좋다.

### 6.2 하네스 소비 우선순위

기존 프로젝트 첫 세션에서 하네스가 이 runner 결과를 읽을 때는 아래 우선순위를 권장한다.

1. `status`
2. `onboarding_summary.recommended_next_steps`
3. `warnings`
4. `orchestration_plan`
5. `validation_plan`
6. `code_index_update`
7. `session_start`
8. `repository_assessment.summary`

실제 적용 참고:

- `open_git_client` 파일럿에서는 bootstrap 직후 경로 표현 보정, handoff/backlog 상태 정렬, quick test 실행 증빙까지 반영한 기록을 [../examples/pilot_adoption_open_git_client_example.md](../examples/pilot_adoption_open_git_client_example.md) 에 남겼다.

이 순서는 “사람에게 바로 보일 요약 -> 리스크 -> worker 분배 -> 세부 근거 -> 추정 명령/스택 확인” 흐름을 유지하기 위한 것이다.

## 7. 권장 후속 사용 방식

- Codex/OpenCode 하네스는 기존 프로젝트 첫 세션에서 이 runner 결과를 읽고, `onboarding_summary.recommended_next_steps` 를 우선 사용자 노출 요약으로 사용할 수 있다.
- `status == "ok"` 인 경우:
  하네스는 `warnings` 와 `orchestration_plan` 을 함께 읽어 첫 작업 분배 방향을 정하고, 필요하면 `validation_plan` 과 `code_index_update` 를 세부 근거로 참조한다.
  `repository_assessment.summary` 가 있으면 추정 명령과 스택 정보는 onboarding 요약 다음 우선순위로 확인하는 편이 좋다.
  assessment 가 포함된 예시는 [../examples/output_samples/existing_project_onboarding.with_assessment.sample.json](../examples/output_samples/existing_project_onboarding.with_assessment.sample.json) 을 참고한다.
- `status == "error"` 인 경우:
  하네스는 `error`, `error_code`, `source_context.failed_step` 를 우선 사용자에게 요약하고, 바로 다음 복구 액션은 `source_context` 의 누락/실패 입력을 기준으로 제안하는 편이 좋다.
- OpenCode 쪽에서는 `orchestration_plan.worker_assignments` 를 바탕으로 doc/code/validation worker 에 bounded 작업을 분배하기 쉽다.
  이때 실제 구현, 설정 수정, 빌드 확인은 `workflow-code-worker` 에 우선 배정하는 해석이 자연스럽다.
- Codex 쪽에서는 개별 worker 권한 분리가 강제되지는 않으므로, 같은 `orchestration_plan` 을 운영 원칙 문서와 서브 에이전트 배정 힌트로 사용하는 편이 현실적이다.
- 이후 세션부터는 `session-start`, `backlog-update`, `doc-sync`, `validation-plan`, `code-index-update` 를 상황별로 직접 호출하는 쪽이 더 적합하다.
- 즉, 이 runner 는 “도입 직후 기준선 정렬용 오케스트레이션 레이어”로 유지하는 것이 권장된다.

## 다음에 읽을 문서

- 도입 분기 가이드: [./workflow_adoption_entrypoints.md](./workflow_adoption_entrypoints.md)
- 출력 스키마 가이드: [./output_schema_guide.md](./output_schema_guide.md)
- 상위 로드맵: [./workflow_kit_roadmap.md](./workflow_kit_roadmap.md)
- 스크립트 안내: [../scripts/README.md](../scripts/README.md)
