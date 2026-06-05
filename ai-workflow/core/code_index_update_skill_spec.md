# Code Index Update Skill Spec

- 문서 목적: `code-index-update` skill 이 변경 경로를 기준으로 어떤 색인 문서와 허브 문서를 재검토해야 하는지 판단하는 최소 계약을 정의한다.
- 범위: 입력, 출력, 추천 규칙, 색인 문서 범위 정의, 보수적 경고 규칙, 프로토타입 범위
- 대상 독자: AI agent 설계자, skill 구현자, 개발자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `./workflow_skill_catalog.md`, `./doc_sync_skill_spec.md`, `../skills/code-index-update/SKILL.md`

## 1. 목적

`code-index-update` skill 의 목적은 변경 파일 목록을 보고 어떤 색인 문서, 허브 문서, README 계열 문서가 stale 되었을 가능성이 높은지 보수적으로 추천하는 것이다.

- `ai-workflow/` 경로는 workflow 메타 레이어로 보고, 일반 프로젝트 색인/허브 탐색 범위에서는 기본적으로 제외한다.

이 skill 은 아래 질문에 답하는 데 집중한다.

- 이번 변경으로 어떤 인덱스 문서를 다시 확인해야 하는가
- 허브 문서 링크나 문서 목록이 stale 되었을 가능성이 있는가
- 문서 구조 변경이 있었는지, 있었다면 어떤 허브부터 봐야 하는가

## 2. 색인 문서 범위 정의

현재 저장소에서 `code-index-update` 가 다루는 색인 문서는 아래 범주로 본다.

- 저장소 루트 `README.md`
- 프로젝트 문서 홈 또는 위키 홈 문서
- 운영/도메인 허브 `README.md`
- 날짜별 backlog 를 가리키는 backlog index 문서
- runbook, report, dataset, prompt 디렉터리를 묶어 설명하는 허브 문서

프로젝트별로 별도 색인 문서가 있으면 선택 입력으로 추가할 수 있다.

## 3. 최소 입력 계약

- `project_profile_path`
- `changed_files`

선택 입력:

- `index_documents`
- `session_handoff_path`
- `work_backlog_index_path`
- `change_summary`

## 4. 기대 출력 계약

출력은 JSON 을 기본으로 하며 최소 아래 필드를 포함한다.

- `index_update_candidates`
- `priority_index_candidates`
- `stale_index_warnings`
- `reasoning_notes`
- `suggested_index_actions`
- `confidence_notes`
- `source_context`

권장 추가 필드:

- `document_structure_signals`
- `missing_index_candidates`

## 5. 추천 규칙

### 5-1. 문서 구조 변화

아래 상황은 색인 갱신 우선순위를 높인다.

- 새 Markdown 문서 추가
- 문서 경로 이동 또는 이름 변경이 추정되는 경우
- `README.md` 또는 허브 문서 자체가 변경된 경우
- runbook, report, prompt, dataset 문서가 추가되거나 변경된 경우

### 5-2. 코드 변화와 색인 연결

- 코드 변경이 운영 절차와 직접 연결될 가능성이 크면 운영 허브와 backlog index 를 재확인 후보로 올린다.
- UI, 배포, 평가 자산 변경은 관련 문서 허브나 release/readme 계열 문서를 후보로 올린다.
- 프로젝트 프로파일의 문서 홈이 존재하면 기본 후보에 포함한다.

### 5-3. 강한 추천 조건

아래 상황은 `priority_index_candidates` 로 분리한다.

- 새 문서 추가 또는 문서 이동이 추정될 때
- 허브 문서와 하위 문서가 함께 바뀌었을 때
- backlog index, 문서 홈, 루트 README 가 직접 변경됐을 때

## 6. 경고 규칙

- 문서 변경은 있는데 허브 문서 후보를 찾지 못하면 stale 가능성을 경고한다.
- 새 문서 추가로 보이는데 인덱스 문서 후보가 비어 있으면 수동 검토 경고를 남긴다.
- 코드 변경만 있고 문서 구조 연결 근거가 약하면 확정 어조 대신 확인 후보로만 제안한다.

## 7. 프로토타입 범위

현재 단계의 프로토타입은 아래만 수행한다.

- 변경 파일을 문서/코드/허브/상태 문서로 느슨하게 분류
- 프로젝트 프로파일에서 문서 홈, 운영 문서 위치, backlog index 후보를 읽음
- 색인 문서 후보와 stale 경고를 JSON 으로 출력

현재 단계의 프로토타입은 아래는 수행하지 않는다.

- git diff 기반 rename 감지
- 문서 링크 그래프 분석
- 색인 문서 직접 수정

## 다음에 읽을 문서

- skill 카탈로그: [./workflow_skill_catalog.md](./workflow_skill_catalog.md)
- doc-sync 스펙: [./doc_sync_skill_spec.md](./doc_sync_skill_spec.md)
- skill 문서: [../skills/code-index-update/SKILL.md](../skills/code-index-update/SKILL.md)
