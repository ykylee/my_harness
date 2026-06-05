# Doc-Sync Skill Spec

- 문서 목적: `doc-sync` skill 을 실제 구현 가능한 수준의 입력/출력 계약과 동작 순서로 구체화한다.
- 범위: 목표, 입력 계약, 출력 계약, 영향 문서 추천 규칙, 허브 갱신 판단 규칙, 실패 규칙, 쓰기 권한 제한, 수동 대체 절차
- 대상 독자: AI agent 설계자, skill 구현자, 운영자, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: 2026-04-18
- 관련 문서: `./workflow_skill_catalog.md`, `./workflow_mcp_candidate_catalog.md`, `./global_workflow_standard.md`, `./workflow_agent_topology.md`, `../templates/project_workflow_profile_template.md`

## 1. 목적

`doc-sync` skill 의 목적은 코드나 문서 변경이 발생했을 때 함께 검토하거나 갱신해야 할 기준 문서, 허브 문서, 상태 문서를 보수적으로 추천하는 것이다.

이 skill 은 아래 역할에 집중한다.

- 변경 파일 목록을 기반으로 영향 문서 후보 추천
- 허브 문서 또는 인덱스 문서 재확인 필요 여부 판단
- 문서 stale 가능성 경고
- 사람이 후속으로 확인해야 할 문서 우선순위 제시

이 skill 은 문서 내용을 자동 확정하거나, 변경 사실을 과장해서 기록하는 역할을 맡지 않는다.

## 2. 선행 원칙

- 프로젝트별 문서 구조는 프로젝트 프로파일을 우선 기준으로 삼는다.
- 문서를 자동 확정하기보다 경고와 후보 추천을 우선 제공한다.
- 변경 파일과 직접적인 관련이 확인되지 않은 문서는 강한 확정 어조로 추천하지 않는다.
- 상태 문서 수정 여부는 별도 단계에서 판단하며, `doc-sync` 는 기본적으로 읽기 전용이다.
- `ai-workflow/` 경로는 workflow 메타 레이어로 보고, 일반 프로젝트 문서 탐색 후보에서는 기본적으로 제외한다.

## 3. 입력 계약

### 3.1 필수 입력

- `project_profile_path`
- 프로젝트 문서 구조와 예외 규칙이 적힌 프로파일 경로
- `changed_files`
- 이번 변경과 관련된 파일 경로 목록

### 3.2 선택 입력

- `baseline_documents`
- 반드시 기준으로 볼 문서 목록
- `hub_documents`
- 허브, 인덱스, quickstart, 운영 개요 문서 후보 목록
- `session_handoff_path`
- 현재 세션 상태 문서 경로
- `work_backlog_index_path`
- 날짜별 백로그 인덱스 경로
- `latest_backlog_path`
- 최신 날짜 backlog 경로
- `change_summary`
- 사람이 작성한 변경 요약
- `validation_result`
- 검증 결과 또는 미실행 사유 요약

### 3.3 입력 해석 규칙

- `changed_files` 는 상대 경로 목록이면 충분하다.
- `baseline_documents` 나 `hub_documents` 가 없으면 프로젝트 프로파일과 저장소 구조를 기준으로 후보를 추정할 수 있다.
- 문서 파일과 비문서 파일을 구분해 판단하되, 문서가 아닌 파일 변경도 문서 동기화 후보를 만들 수 있다.

## 4. 출력 계약

`doc-sync` 의 출력은 사람이 바로 검토하거나 후속 agent 가 이어서 처리할 수 있는 구조화 추천이어야 한다.

최소 출력 필드:

- `impacted_documents`
- 함께 확인하거나 갱신할 가능성이 높은 문서 후보 목록
- `hub_update_candidates`
- 허브 또는 인덱스 문서 중 재확인이 필요한 후보 목록
- `stale_warnings`
- 현재 변경과 문서 상태 사이의 불일치 가능성 또는 누락 가능성 목록
- `reasoning_notes`
- 각 추천의 근거를 짧게 설명한 메모
- `recommended_review_order`
- 어떤 문서부터 확인하면 좋은지 순서 제안
- `follow_up_actions`
- 후속으로 사람이 해야 할 짧은 행동 목록

권장 추가 출력 필드:

- `status_doc_candidates`
- handoff, backlog, runbook 같은 상태성 문서 중 함께 볼 후보
- `confidence_notes`
- 추천 강도 또는 불확실성 메모
- `validation_doc_candidates`
- 검증 결과를 반영해야 할 문서 후보

## 5. 권장 출력 예시

```text
impacted_documents:
- docs/operations/session_handoff.md
- docs/operations/backlog/2026-04-18.md
- docs/operations/runbooks/delivery-sync.md

hub_update_candidates:
- docs/README.md
- docs/operations/README.md

stale_warnings:
- delivery-sync 관련 코드가 바뀌었지만 runbook 반영 여부는 아직 확인되지 않았다.
- backlog에는 작업이 있으나 handoff 반영 여부가 불분명하다.

reasoning_notes:
- `app/jobs/delivery_sync.py` 변경은 운영 runbook 과 handoff 에 영향을 줄 가능성이 높다.
- `docs/operations/README.md` 는 runbook 허브 역할을 하므로 링크 stale 여부를 같이 봐야 한다.

recommended_review_order:
- docs/operations/runbooks/delivery-sync.md
- docs/operations/session_handoff.md
- docs/operations/backlog/2026-04-18.md

follow_up_actions:
- runbook 의 재시도 절차가 최신 코드와 일치하는지 확인한다.
- handoff 와 backlog 의 진행 상태를 맞춘다.
- 허브 문서 링크를 점검한다.
```

## 6. 동작 절차

### 6.1 프로젝트 문서 구조 확인

1. `project_profile_path` 를 읽어 문서 위치, 운영 문서 디렉터리, backlog 위치, handoff 위치를 확인한다.
2. 프로젝트 프로파일에 허브 문서 규칙이나 특화 검증 포인트가 있으면 우선 적용한다.

### 6.2 변경 파일 분류

1. `changed_files` 를 문서 파일과 비문서 파일로 나눈다.
2. 운영 문서, 템플릿, runbook, 허브, 코드, 설정 파일로 느슨하게 분류한다.
3. 분류 결과는 추천 근거 메모에 사용한다.

### 6.3 직접 영향 문서 추천

1. 변경 파일 자체가 문서라면 해당 문서를 직접 영향 문서에 포함한다.
2. 코드 또는 설정 파일 변경이면 관련 runbook, handoff, backlog, 운영 허브 문서를 후보로 올린다.
3. 변경 요약에 특정 기능명 또는 운영 절차명이 있으면 그 이름과 연결되는 문서를 우선 후보로 잡는다.

### 6.4 허브 및 인덱스 문서 판단

1. 변경된 문서가 허브에서 링크되는 성격이면 해당 허브 문서를 `hub_update_candidates` 로 올린다.
2. 새 문서 추가 또는 문서 이동이 있으면 인덱스, README, quickstart 계열 문서도 재확인 후보로 올린다.
3. backlog 또는 handoff 상태가 바뀔 수 있는 변경이면 상태성 문서 후보를 함께 제안한다.

### 6.5 stale 경고 생성

1. 코드 변경은 있었는데 관련 운영 문서가 후보에 없으면 경고를 남긴다.
2. runbook 변경은 있었는데 허브 문서 후보가 없으면 링크 stale 가능성을 경고한다.
3. 검증 결과가 있는데 결과 기록 문서 후보가 없으면 누락 가능성을 경고한다.

### 6.6 검토 순서 제안

1. 사용자 영향이 큰 runbook 또는 운영 절차 문서를 먼저 추천한다.
2. 그다음 상태 문서와 backlog 를 추천한다.
3. 마지막으로 허브, 색인, quickstart 계열 문서를 추천한다.

## 7. 추천 규칙

- 코드 변경과 직접 연결될 가능성이 큰 runbook 은 우선 추천한다.
- 세션 단위 상태가 흔들릴 수 있는 변경이면 handoff 와 최신 backlog 를 후보에 포함한다.
- 새 문서 생성, 이동, 삭제가 있으면 허브 또는 인덱스 문서를 강하게 추천한다.
- 링크나 메타데이터 문제 가능성이 보이면 허브 문서 점검을 추가한다.
- 근거가 약한 문서는 `확인 후보` 로만 제안하고 확정 어조를 피한다.

## 8. 실패 및 경고 규칙

### 8.1 실패로 처리할 조건

- `project_profile_path` 를 읽을 수 없어 문서 구조 복원이 불가능한 경우
- `changed_files` 가 비어 있고 change summary 도 없어 영향도 판단을 전혀 할 수 없는 경우

### 8.2 경고로 처리할 조건

- 변경 파일은 있으나 프로젝트 문서 구조와 연결 규칙이 불분명한 경우
- 허브 문서 후보는 추정되지만 링크 관계를 확인하지 못한 경우
- 상태 문서 반영이 필요해 보이지만 실제 최신 backlog 경로를 확인하지 못한 경우
- 검증 결과는 있으나 어떤 결과 기록 문서에 반영해야 하는지 확실하지 않은 경우

### 8.3 실패 시 최소 출력

실패하더라도 아래 정보는 남기는 것을 권장한다.

- 받은 변경 파일 목록
- 확인 가능한 문서 구조 정보
- 사람이 수동으로 먼저 확인해야 할 문서 후보

## 9. 권한과 수정 제한

- 기본 권한은 읽기 전용이다.
- 문서 내용을 직접 수정하거나 사실을 확정하지 않는다.
- handoff, backlog, 허브 문서 갱신은 별도 단계에서 수행한다.
- 추천 결과는 항상 후보와 경고 중심으로 표현한다.

## 10. 수동 대체 절차

tool 이 없거나 skill 구현이 아직 없으면 아래 순서로 수동 수행한다.

1. 프로젝트 프로파일에서 문서 구조와 운영 문서 위치를 확인한다.
2. 변경 파일 목록을 보고 직접 연관된 runbook, handoff, backlog, 허브 문서를 추린다.
3. 새 문서, 문서 이동, 링크 변경이 있으면 README 나 인덱스 문서를 함께 확인한다.
4. 코드 변경이면 관련 운영 절차 문서가 최신인지 수동으로 대조한다.
5. 확인이 필요한 문서를 우선순위 순으로 메모한다.

## 11. 구현 체크리스트

- 프로젝트 프로파일 기준으로 문서 구조를 해석하는가
- 변경 파일을 문서와 비문서로 구분해 추천하는가
- 허브, 인덱스, 상태 문서 후보를 분리해 제안하는가
- 추천 근거와 불확실성을 함께 출력하는가
- 문서 자동 확정 없이 경고와 후보 중심으로 결과를 만드는가

## 다음에 읽을 문서

- skill 카탈로그: [./workflow_skill_catalog.md](./workflow_skill_catalog.md)
- MCP 후보 카탈로그: [./workflow_mcp_candidate_catalog.md](./workflow_mcp_candidate_catalog.md)
- agent 토폴로지: [./workflow_agent_topology.md](./workflow_agent_topology.md)
