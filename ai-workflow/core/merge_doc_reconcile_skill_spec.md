# Merge-Doc-Reconcile Skill Spec

- 문서 목적: `merge-doc-reconcile` skill 을 실제 구현 가능한 수준의 입력/출력 계약과 동작 순서로 구체화한다.
- 범위: 목표, 입력 계약, 출력 계약, 병합 후 충돌/불일치 판단 규칙, 재확정 포인트 생성 규칙, 실패 규칙, 쓰기 권한 제한, 수동 대체 절차
- 대상 독자: AI agent 설계자, skill 구현자, 운영자, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: 2026-04-18
- 관련 문서: `./workflow_skill_catalog.md`, `./global_workflow_standard.md`, `./workflow_agent_topology.md`, `./session_start_skill_spec.md`, `./backlog_update_skill_spec.md`, `./doc_sync_skill_spec.md`

## 1. 목적

`merge-doc-reconcile` skill 의 목적은 브랜치 병합 이후 handoff, backlog, 허브 문서, 인덱스 문서 사이의 어긋남을 보수적으로 재정리하기 위한 재확정 포인트를 만드는 것이다.

- `ai-workflow/` 경로는 workflow 메타 레이어로 보고, 일반 프로젝트 변경 파일 집합에서는 기본적으로 제외한다.
- handoff/backlog 재확정 뒤에는 source-of-truth 문서가 준비된 경우 `state.json` 을 자동 재생성해 빠른 세션 기준선을 함께 맞춘다.

이 skill 은 아래 역할에 집중한다.

- 병합 후 상태 문서 간 불일치 탐지
- 어느 문서를 다시 읽고 재확정해야 하는지 추천
- handoff, 최신 backlog, 허브 문서의 우선 재정리 포인트 제시
- 사람이 후속으로 수정할 수 있는 최소 재정리 초안 제공

이 skill 은 병합 충돌을 기계적으로 해결하거나, 병합 이전 두 상태를 임의로 합쳐 사실처럼 확정하지 않는다.

## 2. 선행 원칙

- 병합 후 상태는 저장소 실제 문서 기준으로 다시 확인해야 한다.
- handoff, backlog, 허브 문서가 서로 다르면 자동으로 한쪽을 정답으로 확정하지 않는다.
- 검증 여부, 완료 상태, 차단 해제 상태는 병합 후에도 다시 확인한다.
- 프로젝트별 문서 구조와 우선 문서는 프로젝트 프로파일을 기준으로 해석한다.
- 이 skill 은 기본적으로 재정리 초안과 경고를 제공하는 읽기 중심 단계다.

## 3. 입력 계약

### 3.1 필수 입력

- `project_profile_path`
- 프로젝트 문서 구조와 예외 규칙이 적힌 프로파일 경로
- `merge_result_summary`
- 병합된 변경 요약 또는 병합 후 관찰된 차이 요약

`merge_result_summary` 최소 포함 요소:

- 병합 대상 또는 관련 브랜치 정보
- 병합 후 충돌이 있었는지 여부
- 병합 후 재확인이 필요하다고 보이는 문서 또는 작업 축

### 3.2 조건부 필수 입력

- `session_handoff_path`
- 현재 handoff 문서를 재확인 대상에 포함할 때 사용
- `work_backlog_index_path`
- backlog 인덱스 정합성도 확인할 때 사용
- `latest_backlog_path`
- 최신 backlog 를 이미 알고 있으면 사용

### 3.3 선택 입력

- `hub_documents`
- 운영 허브, README, quickstart, 색인 문서 후보 목록
- `changed_files`
- 병합에 포함된 변경 파일 목록
- `pre_merge_notes`
- 병합 전 상태 차이를 요약한 메모
- `validation_result`
- 병합 후 수행된 검증 결과 또는 미실행 사유

### 3.4 입력 해석 규칙

- `latest_backlog_path` 가 없으면 backlog index 기준으로 최신 backlog 후보를 찾는다.
- `hub_documents` 가 없으면 프로젝트 프로파일과 changed files 를 기준으로 허브 문서 후보를 추정한다.
- 병합 결과 요약이 충분하지 않으면 과도한 재정리를 제안하지 않고 경고를 남긴다.

## 4. 출력 계약

`merge-doc-reconcile` 의 출력은 사람이 병합 후 문서를 다시 확정할 수 있도록 돕는 구조화된 재정리 초안이어야 한다.

최소 출력 필드:

- `reconcile_targets`
- 우선 재확인해야 할 문서 목록
- `state_conflicts`
- handoff, backlog, 허브 문서 간 상태 또는 설명 충돌 목록
- `reconfirmation_points`
- 사람이 다시 확정해야 할 핵심 포인트 목록
- `draft_reconcile_notes`
- 후속 갱신 시 참고할 짧은 문안 초안
- `recommended_review_order`
- 어떤 문서부터 다시 읽고 정리할지 순서 제안
- `warnings`
- 정보 부족, 최신 경로 불확실성, 검증 미완료 등 경고 목록

권장 추가 출력 필드:

- `handoff_update_note`
- handoff 재작성 또는 수정 필요 여부
- `backlog_update_note`
- 최신 backlog 또는 index 갱신 필요 여부
- `hub_update_note`
- 허브 문서 링크/설명 갱신 필요 여부
- `validation_follow_up`
- 병합 후 다시 실행해야 할 검증 또는 미실행 사유

## 5. 권장 출력 예시

```text
reconcile_targets:
- docs/operations/session_handoff.md
- docs/operations/backlog/2026-04-18.md
- docs/operations/README.md

state_conflicts:
- handoff 는 TASK-031 을 in_progress 로 보지만 backlog 에는 done 으로 적혀 있다.
- runbook 링크는 추가됐지만 operations README 에 반영 여부가 불분명하다.

reconfirmation_points:
- TASK-031 의 최종 상태
- 병합 후 최신 기준 runbook 경로
- handoff 의 현재 주 작업 축

draft_reconcile_notes:
- 병합 후 문서 기준선 재정리가 필요하다.
- handoff 와 최신 backlog 의 상태값을 실제 저장소 기준으로 다시 맞춘다.
- 허브 문서의 링크와 설명이 최신 runbook 구조를 반영하는지 확인한다.

recommended_review_order:
- docs/operations/backlog/2026-04-18.md
- docs/operations/session_handoff.md
- docs/operations/README.md

warnings:
- 병합 후 검증 결과가 없어 done 상태를 재확정할 수 없다.
```

## 6. 동작 절차

### 6.1 프로젝트 구조 복원

1. `project_profile_path` 를 읽어 handoff, backlog, 허브 문서 위치를 확인한다.
2. 프로젝트 프로파일에 병합 관련 예외 규칙이 있으면 우선 적용한다.

### 6.2 병합 후 핵심 문서 수집

1. handoff, backlog index, 최신 backlog, 허브 문서 후보를 수집한다.
2. 입력으로 전달된 changed files 가 있으면 그와 직접 연결되는 문서 후보를 추가한다.
3. 문서가 없거나 읽을 수 없으면 누락 사실을 경고에 남긴다.

### 6.3 상태 불일치 탐지

1. handoff 와 최신 backlog 의 진행 중, 차단, 완료 상태를 비교한다.
2. 허브 문서가 새 문서 경로나 변경된 문서 설명을 반영하는지 확인 포인트를 만든다.
3. 병합 결과 요약과 실제 문서 상태가 다르게 보이면 불일치로 기록한다.

### 6.4 재확정 포인트 생성

1. 상태값이 다른 작업 항목은 재확정 포인트로 올린다.
2. 병합으로 새 문서가 추가되었거나 문서 이동이 있었다면 허브 갱신 포인트를 만든다.
3. 검증 미완료 상태라면 완료 여부보다 검증 필요성을 우선 메모한다.

### 6.5 검토 순서 제안

1. 최신 backlog 와 handoff 를 먼저 재확인 대상으로 둔다.
2. 그다음 허브, 인덱스, quickstart 계열 문서를 둔다.
3. 마지막으로 상태 설명을 뒷받침하는 보조 문서를 확인하도록 제안한다.

## 7. 판단 규칙

- handoff 와 backlog 가 충돌하면 둘 중 하나를 자동 정답으로 선택하지 않는다.
- backlog 는 작업 단위 상태, handoff 는 세션 요약 상태로 구분해 본다.
- 허브 문서는 링크와 구조 안내의 최신성 여부를 중심으로 판단한다.
- `done` 상태는 병합 전 어느 브랜치에서 보였더라도 병합 후 검증 근거가 불명확하면 재확정 포인트로 남긴다.
- 근거가 약한 경우는 "수정 필요"보다 "재확인 필요"로 표현한다.

## 8. 실패 및 경고 규칙

### 8.1 실패로 처리할 조건

- `project_profile_path` 를 읽을 수 없어 문서 구조 해석이 불가능한 경우
- `merge_result_summary` 가 너무 부족해 병합 후 어떤 문서 축을 볼지 전혀 판단할 수 없는 경우

### 8.2 경고로 처리할 조건

- handoff 또는 최신 backlog 경로를 찾지 못한 경우
- 허브 문서 후보는 있으나 병합 영향 범위를 명확히 연결하지 못한 경우
- 병합 후 검증 결과가 없어 상태 재확정이 위험한 경우
- 병합 충돌은 해결됐지만 실제 문서 설명이 최신인지 확인하지 못한 경우

### 8.3 실패 시 최소 출력

실패하더라도 아래 정보는 남기는 것을 권장한다.

- 확인된 문서 목록과 누락 문서 목록
- 사람이 우선 수동으로 열어야 할 문서 경로
- 추가로 필요한 병합 정보

## 9. 권한과 수정 제한

- 기본 권한은 읽기 전용이며 재정리 초안 제공에 집중한다.
- 제한된 write mode 에서는 `session_handoff.md` 운영 메모에 재정리 노트를 남기는 수준까지만 자동 처리한다.
- 병합 전 상태를 기계적으로 합쳐 새 사실처럼 쓰지 않는다.
- `done` 상태 확정, 차단 해제, 완료일 확정은 별도 확인 후에만 가능하다.
- handoff, backlog, 허브 문서 직접 수정은 별도 실행 단계로 분리하는 것을 권장한다.

## 10. 수동 대체 절차

tool 이 없거나 skill 구현이 아직 없으면 아래 순서로 수동 수행한다.

1. 프로젝트 프로파일에서 handoff, backlog, 허브 문서 위치를 확인한다.
2. 병합 후 handoff 와 최신 backlog 를 나란히 읽고 상태 충돌을 표시한다.
3. 새 문서 추가, 문서 이동, 링크 변경이 있었으면 허브 문서를 함께 읽는다.
4. 검증 근거가 없는 완료 상태는 재확인 대상으로 남긴다.
5. 다음 세션에서 바로 정리할 수 있도록 재확정 포인트를 짧게 메모한다.

## 11. 구현 체크리스트

- 프로젝트 프로파일 기준으로 병합 후 핵심 문서를 찾는가
- handoff, backlog, 허브 문서 충돌을 분리해 보여주는가
- 상태 재확정과 링크 재확정을 구분하는가
- 검증 미완료 상태를 경고로 드러내는가
- 자동 확정 없이 재정리 초안 중심으로 결과를 만드는가

## 다음에 읽을 문서

- skill 카탈로그: [./workflow_skill_catalog.md](./workflow_skill_catalog.md)
- `session-start` 상세 스펙: [./session_start_skill_spec.md](./session_start_skill_spec.md)
- `backlog-update` 상세 스펙: [./backlog_update_skill_spec.md](./backlog_update_skill_spec.md)
- `doc-sync` 상세 스펙: [./doc_sync_skill_spec.md](./doc_sync_skill_spec.md)
