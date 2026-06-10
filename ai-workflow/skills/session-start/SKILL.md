# Session-Start Skill

- 문서 목적: `session-start` skill 프로토타입의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 스펙, 예상 입력/출력, 권한 경계, 구현 메모
- 대상 독자: skill 구현자, AI agent 설계자, 운영자
- 상태: beta
- 최종 수정일: 2026-06-10 (D-71 wiki_vault 확장 — vault 발견 + wiki-lint)
- 관련 문서:
  - [../../core/session_start_skill_spec.md](../../core/session_start_skill_spec.md)
  - [../../core/workflow_skill_catalog.md](../../core/workflow_skill_catalog.md)
  - [../../core/workflow_agent_topology.md](../../core/workflow_agent_topology.md)
  - [../../skills/wiki-lint/SKILL.md](../../skills/wiki-lint/SKILL.md)

## 1. 목적

새 세션 시작 시 handoff, backlog, 프로젝트 프로파일을 읽고 현재 기준선을 구조화된 요약으로 복원한다.

D-71 확장 (2026-06-10): LLM Wiki vault (`~/wiki/`) 가 발견되면 vault 상태 + wiki-lint 결과를 함께 보고한다.

## 2. 연결 스펙

- 상세 스펙: [../../core/session_start_skill_spec.md](../../core/session_start_skill_spec.md)
- 카탈로그: [../../core/workflow_skill_catalog.md](../../core/workflow_skill_catalog.md)
- vault lint: [../../skills/wiki-lint/SKILL.md](../../skills/wiki-lint/SKILL.md)

## 3. 예상 입력

- `session_handoff_path` (필수)
- `work_backlog_index_path` (필수)
- `project_profile_path` (필수)
- 선택:
  - `latest_backlog_path`
  - `changed_files`
  - `environment_hint`
  - `wiki_vault_path` (D-71) — 기본: `$MYHARNESS_WIKI_VAULT` > `~/wiki` (존재 시)
  - `wiki_lint` (D-71, default true) — vault 발견 시 lint 실행
  - `wiki_lint_timeout` (D-71, default 15s)

## 4. 예상 출력

기본 필드 (스펙 §4):
- `summary`
- `in_progress_items`
- `blocked_items`
- `latest_backlog_path`
- `next_documents` (vault 발견 시 `~/wiki/index.md` 자동 추가)
- `recommended_next_action`
- `warnings`
- `validation_notes`
- `environment_constraints`
- `source_documents`

D-71 신규:
- `wiki_vault` (없으면 `None`): vault 발견 결과
  - `path`, `exists`, `log_md_exists`, `last_log_entry` (가장 최근 `## [YYYY-MM-DD]` 1줄)
  - `wiki_page_count` (wiki/{concepts,entities,topics,sources,comparisons,query,meta}/ 의 `*.md` 총 개수)
  - `raw_entry_count` (raw/_manifest.md 의 `## [YYYY-MM-DD]` 항목 수)
  - `lint` (None 또는 WikiLintSummary): errors/warns/infos/pages_scanned/last_report/rules_executed
  - `warnings` (vault 자체 문제: wiki-lint 미설치, 타임아웃, 파싱실패 등)
  - `notes`

## 5. 권한 경계

- 기본적으로 읽기 전용
- 상태 문서 직접 수정 금지
- `done` 재판정 금지
- D-71: wiki-lint 가 `_lint/report_YYYY-MM-DD.md` 를 **쓰지만**, 이는 wiki-lint 스킬의 권한이지 session-start 의 권한이 아님. session-start 는 그 출력만 읽음

## 6. 구현 메모

- 최신 backlog 탐색 로직은 backlog index 우선
- handoff 와 backlog 충돌은 경고로만 출력
- 프로젝트 프로파일의 문서 구조를 최우선 기준으로 사용
- D-71 vault 자동 발견은 default `~/wiki` 가 **실제로 존재할 때만** (의도치 않은 자동 발견 방지)
- `MYHARNESS_WIKI_VAULT` 환경 변수로 명시 override 가능
- `--wiki-lint=false` 로 lint skip 가능 (CI / 빠른 시작용)
- vault 발견 + lint 성공 시 `next_documents` 에 `~/wiki/index.md` 자동 추가 → 사용자가 Obsidian graph view 진입점 확보
- `wiki_vault.lint.errors > 0` 이면 `warnings` 에 lint error 발견 사실 + 리포트 경로 추가
- `wiki_vault.warnings` (lint 자체 실패 등) 도 `warnings` 에 prefix `wiki-vault:` 로 추가

## 7. 프로토타입 실행

- 실행 스크립트: [scripts/run_session_start.py](./scripts/run_session_start.py)
- vault 발견 모듈 (D-71): [scripts/wiki_vault_status.py](./scripts/wiki_vault_status.py)
- 테스트 (D-71, stdlib only): [scripts/test_wiki_vault_status.py](./scripts/test_wiki_vault_status.py)

```bash
# 기본 (vault 자동 발견 + lint)
python3 skills/session-start/scripts/run_session_start.py \
  --session-handoff-path ai-workflow/memory/session_handoff.md \
  --work-backlog-index-path ai-workflow/memory/work_backlog.md \
  --project-profile-path docs/PROJECT_PROFILE.md

# vault 경로 명시
python3 skills/session-start/scripts/run_session_start.py \
  --session-handoff-path ... \
  --work-backlog-index-path ... \
  --project-profile-path ... \
  --wiki-vault-path ~/wiki

# lint skip
python3 skills/session-start/scripts/run_session_start.py \
  --session-handoff-path ... \
  --work-backlog-index-path ... \
  --project-profile-path ... \
  --wiki-lint=false
```

- 현재 프로토타입은 JSON 요약을 stdout 으로 출력한다.
- 최신 backlog 경로를 직접 주지 않으면 backlog index 링크에서 마지막 항목을 사용한다.

## 8. 현재 상태

- 읽기 전용 실행 프로토타입 있음
- handoff, backlog index, project profile 을 읽어 구조화된 현재 상태 요약을 출력할 수 있음
- 경고 기반의 보수적 요약만 제공하며 문서 수정은 수행하지 않음
- D-71: vault 발견 + wiki-lint 통합 (13/13 unit test pass)
  - vault 미설치 시 `wiki_vault: null` (정상 — 경고 없음)
  - vault 미설치 + 명시 경로 → `exists: false`
  - wiki-lint 미설치 시 `warnings: ["wiki-vault: wiki-lint 스킬 미설치 ..."]`

## 다음에 읽을 문서
- skills 허브: [../README.md](../README.md)
- 상세 스펙: [../../core/session_start_skill_spec.md](../../core/session_start_skill_spec.md)
- vault lint: [../../skills/wiki-lint/SKILL.md](../../skills/wiki-lint/SKILL.md)
