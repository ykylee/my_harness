# Validation-Plan Skill

- 문서 목적: `validation-plan` skill 의 역할, 입력/출력, 실행 예시를 정리한다.
- 범위: 검증 수준 판단, 권장 명령 구조화, 테스트 스캐폴딩 생성
- 대상 독자: AI agent 설계자, 개발자, 운영자
- 상태: beta
- 최종 수정일: 2026-04-26
- 관련 문서: `../../core/validation_plan_skill_spec.md`, `../../core/workflow_skill_catalog.md`

## 1. 목적

변경된 파일과 프로젝트 프로파일을 기준으로 이번 작업에서 필요한 검증 수준을 판단하고, 실제 검증에 즉시 사용할 수 있는 테스트 뼈대(Scaffold)를 생성한다.

## 2. 기대 입력

- `project_profile_path`
- `changed_files`
- `change_summary`

선택 입력:

- `session_handoff_path`
- `latest_backlog_path`
- `scaffold` (플래그): 테스트 뼈대 생성 여부
- `task_id`: 생성할 테스트 파일 이름 및 주석에 사용

## 3. 기대 출력

- 감지된 변경 유형 요약 (code, docs, ui, ops 등)
- 권장 검증 수준 및 실행 권장 명령 목록
- **테스트 스캐폴딩**: `tests/repro_{task_id}.py` 파일 생성
- `session_handoff.md` 자동 갱신 (생성된 파일 링크 및 운영 메모 추가)
- 증빙 기대값 및 문서화 체크 항목

## 4. 권한 경계

- 읽기 분석 및 제한적 쓰기(scaffold) 단계
- `--scaffold` 사용 시 `tests/` 디렉토리 내 신규 파일 생성 및 `session_handoff.md` 수정 허용
- 실제 테스트 명령의 실행은 호출한 agent 또는 개발자의 책임임

## 5. 구현 메모

- 프로젝트 프로파일의 `빠른 테스트`, `격리 테스트` 명령을 우선적으로 추출한다.
- `unittest` 기반의 Python 스크립트 뼈대를 생성하여 즉각적인 검증 로직 작성을 돕는다.
- `ai-workflow/` 경로는 workflow 메타 레이어로 보고, 일반 프로젝트 변경 파일 집합에서는 제외한다.

## 6. 스킬 실행

- **분석 전용**:
```bash
python3 skills/validation-plan/scripts/run_validation_plan.py \
  --project-profile-path ai-workflow/memory/PROJECT_PROFILE.md \
  --changed-file app/main.py \
  --change-summary "로그인 로직 수정"
```

- **테스트 뼈대 생성 및 반영**:
```bash
python3 skills/validation-plan/scripts/run_validation_plan.py \
  --project-profile-path ai-workflow/memory/PROJECT_PROFILE.md \
  --session-handoff-path ai-workflow/memory/session_handoff.md \
  --changed-file app/main.py \
  --change-summary "로그인 로직 수정" \
  --scaffold \
  --task-id TASK-123
```

## 7. 현재 상태

- Beta 단계: 검증 계획 수립 및 테스트 스캐폴딩 자동 생성 지원
- `--scaffold` 플래그로 `tests/repro_{task_id}.py` 생성 및 `session_handoff.md` 연동 완료

## 다음에 읽을 문서

- skills 허브: [../README.md](../README.md)
- 상세 스펙: [../../core/validation_plan_skill_spec.md](../../core/validation_plan_skill_spec.md)
