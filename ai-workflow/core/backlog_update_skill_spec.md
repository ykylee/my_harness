# Backlog-Update Skill Spec

- 문서 목적: `backlog-update` skill 을 실제 구현 가능한 수준의 입력/출력 계약과 동작 순서로 구체화한다.
- 범위: 목표, 입력 계약, 출력 계약, 신규 작업 생성 규칙, 상태 갱신 규칙, 실패 규칙, 쓰기 권한 제한, 수동 대체 절차
- 대상 독자: AI agent 설계자, skill 구현자, 운영자, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: 2026-04-18
- 관련 문서: `./workflow_skill_catalog.md`, `./global_workflow_standard.md`, `./workflow_agent_topology.md`, `../templates/daily_backlog_template.md`, `../templates/work_backlog_template.md`

## 1. 목적

`backlog-update` skill 의 목적은 현재 세션의 작업 정보를 날짜별 백로그 형식으로 안정적으로 기록하거나 갱신할 수 있게 초안을 만드는 것이다.

- 날짜별 backlog 또는 handoff 상태를 갱신한 뒤에는 source-of-truth 문서가 준비된 경우 `state.json` 을 자동 재생성해 빠른 세션 기준선을 함께 맞춘다.

이 skill 이 다루는 핵심 역할은 아래와 같다.

- 새 작업을 날짜별 백로그 항목 형식으로 초안 생성
- 기존 작업 항목의 상태와 진행 현황 갱신 초안 생성
- 필요한 경우 해당 날짜의 backlog 파일 생성 초안 준비
- 검증되지 않은 작업을 성급히 `done` 으로 확정하지 않도록 안전장치 제공

## 2. 선행 원칙

- 작업 상태값은 `global_workflow_standard.md` 의 `planned`, `in_progress`, `blocked`, `done` 만 사용한다.
- 백로그 항목은 공통 최소 필드를 유지해야 한다.
- 검증 근거가 없으면 `done` 상태를 확정하지 않는다.
- 문서가 비어 있거나 정보가 모자라더라도 사실을 지어내지 않고 미기입 또는 확인 필요로 남긴다.
- 프로젝트 특화 경로와 예외 규칙은 프로젝트 프로파일을 우선 기준으로 삼는다.

## 3. 입력 계약

### 3.1 필수 입력

- `project_profile_path`
- 프로젝트 특화 문서 구조와 규칙이 적힌 프로파일 경로
- `task_brief`
- 이번 세션에서 등록 또는 갱신하려는 작업 브리핑

`task_brief` 최소 포함 요소:

- 작업명 또는 작업 목적
- 신규 등록인지 기존 항목 갱신인지
- 현재 알고 있는 상태 또는 목표 상태

### 3.2 조건부 필수 입력

- `daily_backlog_path`
- 해당 날짜 backlog 문서를 이미 알고 있으면 필수
- `target_date`
- `daily_backlog_path` 가 없을 때 backlog 생성 또는 탐색 기준 날짜로 사용
- `task_id`
- 기존 항목을 갱신할 때는 필수

### 3.3 선택 입력

- `work_backlog_index_path`
- 날짜별 backlog 인덱스 갱신 또는 경로 확인이 필요할 때 사용
- `session_handoff_path`
- 현재 기준선과 우선순위를 함께 참고할 때 사용
- `host_name`
- 백로그 필드 초안 채움에 사용 가능
- `host_ip`
- 백로그 필드 초안 채움에 사용 가능
- `owner`
- 담당 필드 초안 채움에 사용 가능
- `affected_documents`
- 영향 문서 필드 초안에 사용 가능
- `validation_result`
- 검증 결과 또는 미실행 사유가 있으면 결과 필드 초안에 반영 가능

### 3.4 입력 해석 규칙

- `daily_backlog_path` 가 없으면 `target_date` 를 기준으로 날짜별 backlog 경로 후보를 구성한다.
- 프로젝트 프로파일에 backlog 위치가 있으면 그 경로를 우선 기준으로 사용한다.
- 기존 작업 갱신인데 `task_id` 가 없으면 신규 생성으로 오판하지 말고 경고를 반환한다.

## 4. 출력 계약

`backlog-update` 의 출력은 실제 문서에 바로 반영하거나 사람이 검토 후 반영할 수 있는 구조화 초안이어야 한다.

최소 출력 필드:

- `operation_type`
- `create_entry`, `update_entry`, `create_daily_backlog`, `cannot_determine` 중 하나
- `target_backlog_path`
- 생성 또는 갱신 대상 날짜별 backlog 경로
- `task_id`
- 신규면 제안 ID, 기존이면 확인된 ID
- `draft_entry`
- 날짜별 backlog 형식에 맞춘 작업 항목 초안
- `status_recommendation`
- 제안 상태값과 그 이유
- `fields_requiring_confirmation`
- 사람이 확인해야 하는 필드 목록
- `warnings`
- 누락 입력, 상태 확정 위험, 중복 가능성, 경로 불확실성 목록

권장 추가 출력 필드:

- `index_update_note`
- backlog index 에도 반영이 필요한지 여부
- `handoff_update_note`
- handoff 에 반영할 필요가 있는지 여부
- `validation_note`
- 검증 결과와 미실행 사유 요약

## 5. 권장 출력 예시

```text
operation_type:
- update_entry

target_backlog_path:
- docs/operations/backlog/2026-04-18.md

task_id:
- TASK-031

draft_entry:
- 상태: in_progress
- 우선순위: high
- 요청일: 2026-04-18
- 완료일:
- 담당: platform-agent
- 호스트명: devbox-02
- 호스트 IP: 10.10.4.21
- 영향 문서:
  - docs/operations/session_handoff.md
- 작업 내용:
  - backlog-update skill 계약 초안을 작성한다.
- 진행 현황:
  - 2026-04-18 14:10 기준 입력 계약과 상태 갱신 규칙을 정리했다.
- 완료 기준:
  - 스펙 문서 작성과 문서 무결성 검사 통과
- 작업 결과:
  - 현재 스펙 초안 작성 완료, 검토 반영 대기
- 다음 세션 시작 포인트:
  - 카탈로그와 agent 토폴로지 연결 여부를 재확인한다.
- 남은 리스크:
  - 실제 구현 전까지는 상태 갱신 로직이 문서 규칙에만 머문다.
- 후속 작업:
  - MCP 후보와 연동 가능한 입력 계약을 분리한다.

status_recommendation:
- `in_progress`
- 이유: 문서 초안은 생성되었지만 구현 및 운영 검증이 끝나지 않았다.

fields_requiring_confirmation:
- 담당
- 영향 문서
- 완료 기준

warnings:
- 검증 결과가 없으므로 `done` 으로 올릴 수 없다.
```

## 6. 동작 절차

### 6.1 대상 문서 확인

1. `project_profile_path` 존재 여부를 확인한다.
2. `daily_backlog_path` 가 있으면 해당 문서를 확인한다.
3. `daily_backlog_path` 가 없으면 `target_date` 와 프로젝트 프로파일의 backlog 위치를 조합해 대상 경로 후보를 만든다.

### 6.2 작업 유형 판정

1. `task_brief` 와 `task_id` 를 기준으로 신규 생성인지 기존 갱신인지 판단한다.
2. 기존 갱신으로 보이지만 `task_id` 가 없으면 `cannot_determine` 로 두고 경고를 남긴다.
3. 날짜별 backlog 파일 자체가 없으면 `create_daily_backlog` 또는 `create_entry` 조합으로 처리한다.

### 6.3 기존 항목 탐색

1. 기존 갱신이면 대상 backlog 문서에서 `task_id` 항목을 찾는다.
2. 찾지 못하면 잘못된 ID일 가능성을 경고하고, 신규 생성으로 자동 전환하지 않는다.
3. 동일한 작업명 또는 유사 항목이 여러 개면 중복 가능성을 경고한다.

### 6.4 상태 추천

1. 작업을 막 시작했다면 `planned` 또는 `in_progress` 중 더 보수적인 값을 제안한다.
2. 외부 대기나 결정 필요가 명시되면 `blocked` 를 제안한다.
3. 검증 근거와 완료 기준 충족이 명확할 때만 `done` 후보를 제안한다.
4. `done` 제안 시에는 검증 결과 또는 완료 근거가 입력에 포함되어야 한다.

### 6.5 필드 초안 생성

1. 공통 최소 필드를 기준으로 문서 초안을 만든다.
2. 비어 있는 필드는 억지로 채우지 않고 확인 필요 항목으로 남긴다.
3. `진행 현황` 은 시각이 있으면 시각 포함 메모 형식으로 만든다.
4. `작업 결과` 와 `남은 리스크` 는 현재 사실만 반영한다.

### 6.6 파생 업데이트 메모 생성

1. 진행 중 또는 차단 항목이 바뀌면 handoff 갱신 필요성을 메모한다.
2. 날짜별 backlog 파일이 새로 생기면 backlog index 갱신 필요성을 메모한다.

## 7. 상태 추천 규칙

- `planned`
- 작업이 등록되었지만 아직 본격 수행 전일 때
- `in_progress`
- 현재 세션 또는 다음 세션에서 계속 처리 중일 때
- `blocked`
- 외부 승인, 접근 권한, 의사결정 등으로 진행이 멈췄을 때
- `done`
- 완료 기준 충족과 검증 근거가 함께 있을 때만 제안 가능

추가 규칙:

- 상태를 모르면 `planned` 또는 `in_progress` 중 더 보수적인 값을 제안한다.
- 검증 없는 `done` 자동 승격은 금지한다.
- `blocked` 에서 `in_progress` 로 바꿀 때는 차단 해소 근거가 있어야 한다.

## 8. 실패 및 경고 규칙

### 8.1 실패로 처리할 조건

- `project_profile_path` 를 읽을 수 없어 경로와 규칙 해석이 모두 불가능한 경우
- `task_brief` 가 너무 부족해 신규/갱신 여부와 작업 목적을 전혀 판단할 수 없는 경우

### 8.2 경고로 처리할 조건

- 갱신 대상인데 `task_id` 가 없는 경우
- `task_id` 는 있으나 backlog 문서에서 찾지 못한 경우
- 검증 근거 없이 `done` 요청이 들어온 경우
- 영향 문서, 완료 기준, 작업 결과가 비어 있어 추후 오해 가능성이 큰 경우
- 날짜별 backlog 경로는 추정됐지만 실제 인덱스 반영 여부를 확인하지 못한 경우

### 8.3 실패 시 최소 출력

실패하더라도 아래 정보는 남기는 것을 권장한다.

- 확인한 입력과 누락 입력 목록
- 대상 backlog 경로 후보
- 사람이 수동으로 먼저 정해야 할 결정 사항

## 9. 권한과 수정 제한

- 기본적으로는 초안 생성과 갱신 제안 역할을 우선한다.
- 제한된 write mode 에서는 날짜별 backlog 반영, backlog index 링크 보장, handoff 상태 목록 동기화까지는 자동 처리할 수 있다.
- 자동 수정 기능이 있더라도 `done` 확정은 검증 근거 확인 후에만 허용한다.
- 존재하지 않는 task 항목을 사실처럼 갱신하지 않는다.
- 기존 작업 내용을 덮어쓸 때는 원문 보존 또는 차이 확인 가능성이 보장되어야 한다.
- handoff 와 backlog index 는 필요 메모만 남기고 직접 수정은 별도 단계로 분리하는 것을 권장한다.

## 10. 수동 대체 절차

tool 이 없거나 skill 구현이 아직 없으면 아래 순서로 수동 수행한다.

1. 프로젝트 프로파일에서 backlog 위치와 예외 규칙을 확인한다.
2. 오늘 날짜 backlog 문서를 열거나 새로 만든다.
3. 템플릿 최소 필드를 기준으로 작업 항목 초안을 작성한다.
4. 검증 결과가 없으면 `done` 으로 쓰지 않는다.
5. 진행 중 또는 차단 상태 변화가 있으면 handoff 갱신 필요 여부를 함께 메모한다.

## 11. 구현 체크리스트

- 신규 생성과 기존 갱신을 구분하는가
- 날짜별 backlog 경로를 안정적으로 찾거나 제안하는가
- 템플릿 최소 필드를 빠뜨리지 않는가
- 검증 없는 `done` 확정을 막는가
- 누락 필드를 `fields_requiring_confirmation` 으로 분리하는가
- handoff 와 backlog index 후속 갱신 필요성을 메모하는가

## 다음에 읽을 문서

- skill 카탈로그: [./workflow_skill_catalog.md](./workflow_skill_catalog.md)
- 공통 표준: [./global_workflow_standard.md](./global_workflow_standard.md)
- agent 토폴로지: [./workflow_agent_topology.md](./workflow_agent_topology.md)
