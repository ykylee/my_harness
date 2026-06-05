# Backlog-Update Skill

- 문서 목적: `backlog-update` skill 프로토타입의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 스펙, 예상 입력/출력, 권한 경계, 구현 메모
- 대상 독자: skill 구현자, AI agent 설계자, 운영자
- 상태: beta
- 최종 수정일: 2026-04-25
- 관련 문서: `../../core/backlog_update_skill_spec.md`, `../../core/workflow_skill_catalog.md`, `../../core/workflow_agent_topology.md`

## 1. 목적

날짜별 backlog 에 새 작업 항목을 생성하거나 기존 작업 항목의 상태 갱신 초안을 만든다.

## 2. 연결 스펙

- 상세 스펙: [../../core/backlog_update_skill_spec.md](../../core/backlog_update_skill_spec.md)
- 카탈로그: [../../core/workflow_skill_catalog.md](../../core/workflow_skill_catalog.md)

## 3. 예상 입력

- `project_profile_path`
- `task_brief`
- 조건부로 `daily_backlog_path`, `target_date`, `task_id`
- 선택적으로 `work_backlog_index_path`, `session_handoff_path`, `owner`, `affected_documents`, `validation_result`

## 4. 예상 출력

- `operation_type`
- `target_backlog_path`
- `task_id`
- `draft_entry`
- `status_recommendation`
- `fields_requiring_confirmation`
- `warnings`

## 5. 권한 경계

- 초안 생성과 갱신 제안 중심
- 검증 없는 `done` 확정 금지
- 존재하지 않는 task 를 사실처럼 갱신 금지
- `--apply` 를 주면 날짜별 backlog, backlog index, handoff 상태 목록을 좁은 범위에서 직접 갱신할 수 있다.

## 6. 구현 메모

- 신규 생성과 기존 갱신을 엄격히 분리
- backlog 경로는 프로젝트 프로파일 기준으로 해석
- handoff/index 후속 갱신은 메모로만 남기고 별도 단계로 분리
- backlog 또는 handoff 상태를 갱신한 뒤에는 source-of-truth 문서가 준비된 경우 `state.json` 을 자동 재생성하는 흐름을 기본 운영값으로 본다.

## 7. 프로토타입 실행

- 실행 스크립트: [scripts/run_backlog_update.py](./scripts/run_backlog_update.py)
- 기존 항목 갱신 예시:

```bash
python3 skills/backlog-update/scripts/run_backlog_update.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --daily-backlog-path examples/acme_delivery_platform/backlog/2026-04-18.md \
  --mode update \
  --task-id TASK-021 \
  --task-name "배송 상태 동기화 실패 대응 절차 문서 정리" \
  --task-brief "runbook 및 handoff 반영 상태를 점검했다." \
  --status in_progress
```

- 신규 항목 생성 예시:

```bash
python3 skills/backlog-update/scripts/run_backlog_update.py \
  --project-profile-path examples/acme_delivery_platform/PROJECT_PROFILE.md \
  --target-date 2026-04-19 \
  --mode create \
  --task-name "운영 허브 링크 무결성 재점검" \
  --task-brief "새 runbook 링크 반영 여부를 확인한다."
```

- 현재 프로토타입은 backlog 파일을 직접 수정하지 않고 JSON 초안만 출력한다.
- `--apply` 를 주면 초안을 대상 backlog 파일에 반영하고, 가능한 경우 backlog index 와 handoff 상태 목록도 같이 동기화한다.

## 8. 현재 상태

- 읽기 전용 초안 생성 프로토타입 있음
- 신규 항목 생성과 기존 항목 갱신을 구분해 draft entry 와 경고를 출력할 수 있음
- `done` 상태는 검증 결과 없이 자동 확정하지 않음

## 다음에 읽을 문서

- skills 허브: [../README.md](../README.md)
- 상세 스펙: [../../core/backlog_update_skill_spec.md](../../core/backlog_update_skill_spec.md)
