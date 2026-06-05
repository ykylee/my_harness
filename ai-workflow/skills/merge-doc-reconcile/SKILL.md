# Merge-Doc-Reconcile Skill

- 문서 목적: `merge-doc-reconcile` skill 의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 스펙, 예상 입력/출력, 권한 경계, 구현 메모
- 대상 독자: skill 구현자, AI agent 설계자, 운영자
- 상태: beta
- 최종 수정일: 2026-04-26
- 관련 문서: `../../core/merge_doc_reconcile_skill_spec.md`, `../../core/workflow_skill_catalog.md`, `../../core/workflow_agent_topology.md`

## 1. 목적

병합 이후 handoff, backlog, 허브 문서 사이의 상태 불일치와 재확정 포인트를 구조화해 후속 정리를 돕고 `state.json` 을 최신화한다.

## 2. 연결 스펙

- 상세 스펙: [../../core/merge_doc_reconcile_skill_spec.md](../../core/merge_doc_reconcile_skill_spec.md)
- 카탈로그: [../../core/workflow_skill_catalog.md](../../core/workflow_skill_catalog.md)

## 3. 예상 입력

- `project_profile_path`
- `merge_result_summary`
- 조건부로 `session_handoff_path`, `work_backlog_index_path`, `latest_backlog_path`
- 선택적으로 `hub_documents`, `changed_files`, `pre_merge_notes`, `validation_result`

## 4. 예상 출력

- `reconcile_targets`
- `state_conflicts`
- `reconfirmation_points`
- `draft_reconcile_notes`
- `recommended_review_order`
- `warnings`

## 5. 권한 경계

- 읽기 중심 재정리 및 제한적 쓰기(apply) 단계
- `--apply` 사용 시 `session_handoff.md` 에 재정리 노트를 추가하고 `state.json` 을 재생성함
- `done` 확정과 차단 해제 자동 처리 금지 (수동 확인 필요)

## 6. 구현 메모

- handoff 와 최신 backlog 를 먼저 대조
- 허브/인덱스 문서는 링크와 구조 설명 최신성 중심으로 점검
- 병합 후 검증 미완료 상태는 재확정 포인트로 유지
- `ai-workflow/` 경로는 workflow 메타 레이어로 보고, 일반 프로젝트 변경 파일 집합에서는 기본적으로 제외한다.
- handoff/backlog 재확정 뒤에는 source-of-truth 문서가 준비된 경우 `state.json` 을 자동 재생성하는 흐름을 기본 운영값으로 본다.

## 7. 스킬 실행

- 실행 스크립트: [scripts/run_merge_doc_reconcile.py](./scripts/run_merge_doc_reconcile.py)
- 실행 예시 (재정리 필요 항목 확인):
```bash
python3 skills/merge-doc-reconcile/scripts/run_merge_doc_reconcile.py \
  --project-profile-path ai-workflow/memory/PROJECT_PROFILE.md \
  --session-handoff-path ai-workflow/memory/session_handoff.md \
  --merge-result-summary "기능 브랜치 병합"
```
- 실행 예시 (자동 반영):
```bash
python3 skills/merge-doc-reconcile/scripts/run_merge_doc_reconcile.py \
  --project-profile-path ai-workflow/memory/PROJECT_PROFILE.md \
  --session-handoff-path ai-workflow/memory/session_handoff.md \
  --merge-result-summary "기능 브랜치 병합" \
  --apply
```

## 8. 현재 상태

- Beta 단계: 병합 후 정합성 분석 및 제한적 쓰기 기능 지원
- `--apply` 플래그 사용 시 `session_handoff.md` 에 재정리 노트 추가
- 최신 상태를 반영하여 `state.json` 캐시 자동 갱신
- 병합 후 검증 미완료 상태는 경고 및 재확정 포인트로 유지함

## 다음에 읽을 문서

- skills 허브: [../README.md](../README.md)
- 상세 스펙: [../../core/merge_doc_reconcile_skill_spec.md](../../core/merge_doc_reconcile_skill_spec.md)
