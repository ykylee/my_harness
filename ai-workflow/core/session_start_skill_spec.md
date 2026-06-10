# Session-Start Skill Spec

- 문서 목적: `session-start` skill 을 실제 구현 가능한 수준의 입력/출력 계약과 동작 순서로 구체화한다.
- 범위: 목표, 입력 계약, 출력 계약, 판단 절차, 실패 규칙, 쓰기 권한 제한, 수동 대체 절차
- 대상 독자: AI agent 설계자, skill 구현자, 운영자, 프로젝트 온보딩 담당자
- 상태: draft (D-71 wiki_vault 확장 진행 중, 2026-06-10)
- 최종 수정일: 2026-06-10 (D-71 wiki_vault 확장 — §3.4, §4, §6.7 추가)
- 관련 문서: `./workflow_skill_catalog.md`, `./global_workflow_standard.md`, `./workflow_agent_topology.md`, `../templates/session_handoff_template.md`, `../templates/project_workflow_profile_template.md`, [../skills/wiki-lint/SKILL.md](../skills/wiki-lint/SKILL.md)

## 1. 목적

`session-start` skill 의 목적은 새 세션이 시작될 때 현재 프로젝트의 작업 기준선을 빠르게 복원하는 것이다.

이 skill 은 단순히 문서를 읽는 기능이 아니라, handoff, 백로그, 프로젝트 프로파일을 읽고 아래 산출물을 안정적으로 만들어내는 역할을 가진다:

- 현재 상태 요약
- 우선 확인할 진행 중 또는 차단 작업
- 다음에 읽거나 확인할 문서 경로
- 현재 세션에서 바로 시작할 수 있는 첫 행동 제안

D-71 (2026-06-10) 확장: LLM Wiki vault 가 발견되면 vault 상태 + wiki-lint 결과를 함께 보고한다. vault 는 SSOT 가 아니라 **derived** view (CONCEPT.md §5.13) 이지만, second brain 의 현재 상태를 한눈에 보는 데 유용하다.

## 2. 선행 원칙

- 공통 세션 시작 순서는 `global_workflow_standard.md` 를 따른다.
- 프로젝트별 문서 구조와 명령 체계는 프로젝트 프로파일을 우선 기준으로 삼는다.
- 문서 상태가 불완전하더라도 사실을 지어내지 않고, 누락 또는 불확실성을 출력에 명시한다.
- 이 skill 은 기본적으로 읽기 전용이며, 상태 문서를 직접 수정하지 않는다.
- D-71: vault 발견은 **부드러운 실패** (없으면 `wiki_vault: null`, 경고 X). 명시 경로가 잘못된 경우만 `warnings` 기록.

## 3. 입력 계약

### 3.1 필수 입력

- `session_handoff_path` — 현재 프로젝트의 세션 인계 문서 경로
- `work_backlog_index_path` — 날짜별 백로그 인덱스 문서 경로
- `project_profile_path` — 프로젝트 특화 규칙과 문서 구조가 적힌 프로파일 문서 경로

### 3.2 선택 입력

- `latest_backlog_path` — 이미 외부에서 최신 날짜 백로그 경로를 계산했다면 직접 전달 가능
- `changed_files` — 세션 시작 전에 이미 알려진 변경 파일 목록
- `environment_hint` — 호스트명, 실행 환경, 접근 제약

### 3.3 입력 해석 규칙

- 필수 입력 문서는 모두 실제 파일이어야 한다.
- `latest_backlog_path` 가 없으면 백로그 인덱스에서 최신 백로그 후보를 찾는다.
- 프로젝트 프로파일에 문서 구조가 명시되어 있으면, 다른 휴리스틱보다 그 구조를 우선 적용한다.

### 3.4 D-71 LLM Wiki vault 입력 (선택)

- `wiki_vault_path` — 명시적 vault 경로. 우선순위 1.
- `MYHARNESS_WIKI_VAULT` 환경 변수 — 우선순위 2.
- 기본값: `~/wiki` (실제로 디렉터리가 존재할 때만) — 우선순위 3.
- `wiki_lint` (bool, default true) — vault 발견 시 wiki-lint 실행 여부
- `wiki_lint_timeout` (float seconds, default 15) — wiki-lint subprocess 타임아웃

## 4. 출력 계약

`session-start` 의 출력은 사람이 바로 읽고 다음 행동으로 이어갈 수 있는 구조화 요약이어야 한다.

### 4.1 최소 출력 필드 (기존)

- `summary` — 현재 세션 기준선을 3~6줄 정도로 요약한 텍스트
- `in_progress_items` — 현재 진행 중으로 판단한 작업 목록
- `blocked_items` — 현재 차단 상태로 판단한 작업 목록
- `latest_backlog_path` — 실제로 읽은 최신 백로그 문서 경로 또는 확인 실패 상태
- `next_documents` — 다음에 읽을 문서 경로 목록 (D-71: vault 발견 시 `~/wiki/index.md` 자동 추가)
- `recommended_next_action` — 세션 시작 직후 수행할 첫 행동 한 줄 제안
- `warnings` — 누락 문서, 충돌 정보, stale 가능성, 불확실성 목록

### 4.2 권장 추가 출력 필드 (기존)

- `validation_notes` — 이전 세션에서 검증이 미완료로 남은 항목 요약
- `environment_constraints` — 현재 세션에 영향을 주는 접근 제약 또는 환경 차이

### 4.3 D-71 신규 출력: `wiki_vault`

vault 발견 시 (없으면 `null`):

| 필드 | 타입 | 의미 |
|---|---|---|
| `path` | str | vault 절대 경로 |
| `exists` | bool | 디렉터리 존재 여부 |
| `log_md_exists` | bool | log.md 파일 존재 |
| `last_log_entry` | str \| null | log.md 의 가장 최근 `## [YYYY-MM-DD] ...` 한 줄 |
| `wiki_page_count` | int | wiki/{concepts,entities,topics,sources,comparisons,query,meta}/ 의 `*.md` 총 개수 |
| `raw_entry_count` | int | raw/_manifest.md 의 `## [YYYY-MM-DD]` 항목 수 |
| `lint` | object \| null | WikiLintSummary (errors/warns/infos/pages_scanned/last_report/rules_executed) |
| `warnings` | list[str] | vault 자체 문제 (wiki-lint 미설치, 타임아웃, 파싱실패 등) |
| `notes` | list[str] | vault 운영 메모 |

## 5. 권장 출력 예시

```text
summary:
- 현재 기준선은 TASK-005-2 v1.5 진입 준비 단계다.
- 진행 중: wiki-lint 스킬 + vault 초기화 (D-71).
- 다음 행동: vault 첫 ingest 시작.

in_progress_items:
- D-71 LLM Wiki vault + Obsidian second brain

blocked_items:
- 없음

latest_backlog_path:
- ai-workflow/memory/backlog/2026-06-10.md

next_documents:
- ai-workflow/memory/session_handoff.md
- ai-workflow/memory/backlog/2026-06-10.md
- docs/PROJECT_PROFILE.md
- /Users/yklee/wiki/index.md      # D-71: vault 발견

recommended_next_action:
- handoff와 최신 backlog의 불일치 여부를 먼저 확인한다.

warnings:
- 환경 기록 문서는 아직 정의만 있고 실제 샘플은 없다.

wiki_vault:                              # D-71
  path: /Users/yklee/wiki
  exists: true
  log_md_exists: true
  last_log_entry: "## [2026-06-10] init | LLM Wiki vault bootstrapped (D-71)"
  wiki_page_count: 1
  raw_entry_count: 0
  lint:
    errors: 0
    warns: 2
    infos: 0
    pages_scanned: 1
    last_report: /Users/yklee/wiki/_lint/report_2026-06-10.md
    rules_executed: [L01, L02, L03, L04, L05, L06, L07, L08, L09, L10]
  warnings: []
```

## 6. 동작 절차

### 6.1 문서 존재 확인

1. `session_handoff_path`, `work_backlog_index_path`, `project_profile_path` 존재 여부를 확인한다.
2. 누락 문서가 있으면 즉시 실패하지 말고, 읽을 수 있는 범위까지 진행하되 `warnings` 에 누락 사실을 기록한다.

### 6.2 handoff 읽기

1. 현재 기준선, 현재 주 작업 축, 진행 중 작업, 차단 작업, 최근 완료 작업, 주요 제약을 읽는다.
2. handoff 에서 다음에 읽을 문서가 명시되어 있으면 우선 수집한다.

### 6.3 최신 backlog 결정

1. `latest_backlog_path` 가 입력으로 있으면 그 경로를 우선 사용한다.
2. 없으면 backlog index에서 최신 날짜 문서를 찾는다.
3. 인덱스에 최신 링크가 없거나 애매하면 프로젝트 프로파일에 정의된 백로그 위치를 참고해 경고를 남긴다.

### 6.4 backlog 읽기

1. 최신 backlog 에서 `in_progress`, `blocked`, 최근 완료 또는 미검증 항목을 추린다.
2. handoff 와 backlog 의 상태가 다르면 둘 중 무엇이 최신인지 단정하지 말고 불일치 경고를 남긴다.

### 6.5 프로젝트 프로파일 읽기

1. 문서 구조, 기본 명령, 특화 검증 포인트, 환경 제약을 읽는다.
2. 세션 시작 후 곧바로 필요한 명령이나 접근 제약이 있으면 `recommended_next_action` 과 `environment_constraints` 에 반영한다.

### 6.6 최종 요약 생성

1. handoff 와 backlog 에 공통으로 나타나는 현재 우선 작업을 우선 요약한다.
2. 차단 항목이 있으면 이유와 영향 범위를 함께 짧게 요약한다.
3. 다음에 읽을 문서와 첫 행동 제약을 만든다.

### 6.7 D-71 LLM Wiki vault 발견 (선택)

1. `wiki_vault_path` > `MYHARNESS_WIKI_VAULT` > `~/wiki` (존재 시) 순으로 경로 결정
2. `Path.is_dir()` 로 존재 확인 → `wiki_vault.exists` 갱신
3. 부재 시: `wiki_vault = WikiVaultStatus(path=...)` (`exists=False`), 조기 반환
4. 존재 시:
   - `log.md` 의 가장 최근 `## [YYYY-MM-DD]` 한 줄 → `last_log_entry`
   - `wiki/{sub}/*.md` 카운트 → `wiki_page_count`
   - `raw/_manifest.md` 의 `## [YYYY-MM-DD]` 항목 수 → `raw_entry_count`
   - `--wiki-lint=true` (기본) 이면 wiki-lint subprocess 호출
     - `--output both` 로 호출해 `_lint/report_YYYY-MM-DD.md` materialize
     - exit 2 / 타임아웃 / JSON 파싱 실패 → `lint=None`, `warnings` 에 사유 기록
     - 성공 → `lint` 에 WikiLintSummary 채움
5. vault 의 `index.md` 가 있으면 `next_documents` 에 추가
6. `wiki_vault.lint.errors > 0` 이면 메인 `warnings` 에 `wiki-lint: N error 발견 — 보고: ...` 추가
7. `wiki_vault.warnings` 항목은 `wiki-vault:` prefix 로 메인 `warnings` 에 추가

## 7. 판단 규칙

- handoff 는 세션 맥락 복원용 기준 문서로 우선 신뢰하되, backlog 와 충돌 시 단정하지 않는다.
- backlog 는 작업 단위 상태 확인용 기준 문서로 사용한다.
- 프로젝트 프로파일은 명령과 경로 해석의 최우선 기준이다.
- `done` 상태는 재판정하지 않으며, 검증 여부를 별도 메모로만 드러낸다.
- 정보가 부족하면 "없음" 보다 "확인되지 않음" 을 우선 사용한다.
- D-71: vault 부재는 실패가 아니다. `wiki_vault: null` 로 끝.

## 8. 실패 및 경고 규칙

### 8.1 실패로 처리할 조건

- 필수 입력 3개가 모두 없거나 읽을 수 없는 경우
- 프로젝트 프로파일이 없어 경로 해석과 기본 규칙 복원이 모두 불가능한 경우

### 8.2 경고로 처리할 조건

- handoff 는 있으나 최신 backlog 를 찾을 수 없는 경우
- backlog 는 있으나 handoff 와 진행 상태가 다르게 보이는 경우
- 프로젝트 프로파일에 정의된 문서 구조와 실제 입력 경로가 다른 경우
- 문서 메타데이터는 있으나 실제 내용이 비어 있는 경우
- D-71: wiki-lint 자체 실패 (subprocess, 파싱, 타임아웃)
- D-71: vault 의 `lint.errors > 0`

### 8.3 실패 시 최소 출력

실패하더라도 아래 정보는 남기는 것을 권장한다:

- 읽기에 성공한 문서 목록
- 읽기에 실패한 문서 목록과 원인
- 사람이 수동으로 먼저 확인해야 할 경로

## 9. 권한과 수정 제한

- 기본 권한은 읽기 전용이다.
- 상태 문서, backlog, handoff 를 직접 수정하지 않는다.
- `done` 상태 확정이나 차단 해제 판단을 수행하지 않는다.
- 후속 agent 나 사용자에게 넘길 요약과 경고만 만든다.
- D-71: wiki-lint subprocess 가 `_lint/report_YYYY-MM-DD.md` 를 쓰는 것은 wiki-lint 스킬의 권한이지 session-start 의 권한이 아니다. session-start 는 그 **출력만** 읽는다.

## 10. 수동 대체 절차

tool 이 없거나 skill 구현이 아직 없으면 아래 순서로 수동 수행한다.

1. handoff 문서를 읽고 현재 기준선과 진행 중 작업을 확인한다.
2. backlog index 와 최신 날짜 backlog 를 읽고 실제 작업 상태를 확인한다.
3. 프로젝트 프로파일을 읽고 문서 구조, 명령, 환경 제약을 확인한다.
4. 현재 세션의 첫 행동과 확인할 문서를 짧게 요약한다.
5. (D-71) `ls ~/wiki/`, `cat ~/wiki/index.md`, `cat ~/wiki/log.md` 로 vault 상태 빠르게 확인.

## 11. 구현 체크리스트

- 입력 경로 존재 여부를 검증하는가
- backlog 최신 문서를 안정적으로 찾는가
- handoff 와 backlog 불일치를 경고로 드러내는가
- 프로젝트 프로파일을 기준으로 경로와 명령을 해석하는가
- 출력이 구조화되어 다음 agent 또는 사람이 재사용 가능한가
- 읽기 전용 원칙을 지키는가
- D-71: vault 부재 시 graceful (null 반환) 인가
- D-71: vault 발견 시 lint 결과가 `wiki_vault.lint` 에 잘 담기는가
- D-71: lint 실패가 메인 `warnings` 에 prefix 와 함께 노출되는가
- D-71: wiki-lint subprocess 타임아웃이 동작하는가

## 다음에 읽을 문서
- skill 카탈로그: [./workflow_skill_catalog.md](./workflow_skill_catalog.md)
- 공통 표준: [./global_workflow_standard.md](./global_workflow_standard.md)
- agent 토폴로지: [./workflow_agent_topology.md](./workflow_agent_topology.md)
- vault lint: [../skills/wiki-lint/SKILL.md](../skills/wiki-lint/SKILL.md)
