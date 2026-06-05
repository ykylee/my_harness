# Code-Index-Update Skill

- 문서 목적: `code-index-update` skill 의 역할, 입력/출력, 실행 예시를 정리한다.
- 범위: 색인 문서 갱신 후보 추천, stale 경고, 자동 색인 동기화
- 대상 독자: AI agent 설계자, 개발자, 운영자
- 상태: beta
- 최종 수정일: 2026-04-26
- 관련 문서: `../../core/code_index_update_skill_spec.md`, `../../core/workflow_skill_catalog.md`

## 1. 목적

변경 파일과 프로젝트 프로파일을 기준으로 다시 확인하거나 업데이트해야 할 색인(Index) 및 허브(Hub) 문서를 추출하고 `session_handoff.md` 에 자동 반영한다.

## 2. 기대 입력

- `project_profile_path`
- `changed_files`

선택 입력:

- `work_backlog_index_path`
- `session_handoff_path`
- `change_summary`
- `apply` (플래그): 색인 문서 자동 동기화 여부

## 3. 기대 출력

- 색인 문서 갱신 후보 및 우선순위 제안
- stale 가능성 경고 및 추천 근거 메모
- `session_handoff.md` 자동 갱신 (추천 색인 링크 및 운영 메모 추가)
- `state.json` 캐시 자동 갱신

## 4. 권한 경계

- 읽기 분석 및 제한적 쓰기(apply) 단계
- `--apply` 사용 시 `session_handoff.md` 및 `state.json` 수정 허용
- 색인 문서(README 등)의 본문을 직접 수정하지는 않으며, 사용자가 확인하도록 유도함

## 5. 구현 메모

- 문서 홈, 운영 허브, backlog index, 루트 README 를 기본 후보군으로 다룬다.
- 하위 문서 변경 시 상위 허브 문서의 stale 가능성을 감지하여 우선순위를 높인다.
- `ai-workflow/` 경로는 workflow 메타 레이어로 보고, 일반 프로젝트 색인 탐색 범위에서 제외한다.

## 6. 스킬 실행

- **분석 전용**:
```bash
python3 skills/code-index-update/scripts/run_code_index_update.py \
  --project-profile-path ai-workflow/memory/PROJECT_PROFILE.md \
  --changed-file docs/operations/runbooks/new-guide.md
```

- **자동 색인 동기화**:
```bash
python3 skills/code-index-update/scripts/run_code_index_update.py \
  --project-profile-path ai-workflow/memory/PROJECT_PROFILE.md \
  --session-handoff-path ai-workflow/memory/session_handoff.md \
  --changed-file docs/operations/runbooks/new-guide.md \
  --apply
```

## 7. 현재 상태

- Beta 단계: 색인 문서 갱신 추천 및 자동 동기화 지원
- `--apply` 플래그로 `session_handoff.md` 연동 및 `state.json` 캐시 갱신 완료

## 다음에 읽을 문서

- skills 허브: [../README.md](../README.md)
- 상세 스펙: [../../core/code_index_update_skill_spec.md](../../core/code_index_update_skill_spec.md)
