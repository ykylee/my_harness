# Session-Start Skill

- 문서 목적: `session-start` skill 프로토타입의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 스펙, 예상 입력/출력, 권한 경계, 구현 메모
- 대상 독자: skill 구현자, AI agent 설계자, 운영자
- 상태: beta
- 최종 수정일: 2026-04-25
- 관련 문서: `../../core/session_start_skill_spec.md`, `../../core/workflow_skill_catalog.md`, `../../core/workflow_agent_topology.md`

## 1. 목적

새 세션 시작 시 handoff, backlog, 프로젝트 프로파일을 읽고 현재 기준선을 구조화된 요약으로 복원한다.

## 2. 연결 스펙

- 상세 스펙: [../../core/session_start_skill_spec.md](../../core/session_start_skill_spec.md)
- 카탈로그: [../../core/workflow_skill_catalog.md](../../core/workflow_skill_catalog.md)

## 3. 예상 입력

- `session_handoff_path`
- `work_backlog_index_path`
- `project_profile_path`
- 선택적으로 `latest_backlog_path`, `changed_files`, `environment_hint`

## 4. 예상 출력

- `summary`
- `in_progress_items`
- `blocked_items`
- `latest_backlog_path`
- `next_documents`
- `recommended_next_action`
- `warnings`

## 5. 권한 경계

- 기본적으로 읽기 전용
- 상태 문서 직접 수정 금지
- `done` 재판정 금지

## 6. 구현 메모

- 최신 backlog 탐색 로직은 backlog index 우선
- handoff 와 backlog 충돌은 경고로만 출력
- 프로젝트 프로파일의 문서 구조를 최우선 기준으로 사용

## 7. 프로토타입 실행

- 실행 스크립트: [scripts/run_session_start.py](./scripts/run_session_start.py)
- 예시 실행:

```bash
python3 skills/session-start/scripts/run_session_start.py \
  --session-handoff-path examples/acme_delivery_platform/session_handoff.md \
  --work-backlog-index-path examples/acme_delivery_platform/work_backlog.md \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md
```

- 현재 프로토타입은 JSON 요약을 stdout 으로 출력한다.
- 최신 backlog 경로를 직접 주지 않으면 backlog index 링크에서 마지막 항목을 사용한다.

## 8. 현재 상태

- 읽기 전용 실행 프로토타입 있음
- handoff, backlog index, project profile 을 읽어 구조화된 현재 상태 요약을 출력할 수 있음
- 경고 기반의 보수적 요약만 제공하며 문서 수정은 수행하지 않음

## 다음에 읽을 문서

- skills 허브: [../README.md](../README.md)
- 상세 스펙: [../../core/session_start_skill_spec.md](../../core/session_start_skill_spec.md)
