# Validation Plan Skill Spec

- 문서 목적: `validation-plan` skill 이 변경 유형과 프로젝트 프로파일을 바탕으로 검증 수준을 판단하는 최소 계약을 정의한다.
- 범위: 입력, 출력, 판단 규칙, 보수적 예외 처리, 프로토타입 범위
- 대상 독자: AI agent 설계자, 개발자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `./workflow_skill_catalog.md`, `../skills/validation-plan/SKILL.md`, `../templates/project_workflow_profile_template.md`

## 1. 목적

`validation-plan` skill 은 변경된 파일과 변경 요약, 프로젝트 프로파일에 기록된 기본 명령/검증 포인트를 읽고 현재 세션에서 권장할 검증 계획을 구조화한다.

- `ai-workflow/` 경로는 workflow 메타 레이어로 보고, 일반 프로젝트 변경 파일 집합에서는 기본적으로 제외한다.

이 skill 의 목표는 모든 명령을 자동 실행하는 것이 아니라 다음 세 가지를 빠르게 정리하는 데 있다.

- 어떤 수준까지 검증해야 하는지
- 어떤 명령을 우선 추천해야 하는지
- 아직 실행하지 못한 항목과 그 이유를 어떻게 남겨야 하는지

## 2. 최소 입력 계약

- `project_profile_path`
- 프로젝트별 워크플로우 프로파일 경로
- `changed_files`
- 상대 경로 기준 변경 파일 목록
- `change_summary`
- 사람이 요약한 변경 설명. 파일 목록이 충분하지 않을 때 보강 입력으로 사용

선택 입력:

- `latest_backlog_path`
- 최신 backlog 문서 경로. 검증 미실행 사유를 backlog 에 남겨야 하는지 판단할 때 사용
- `session_handoff_path`
- handoff 문서 경로. 검증 제약이나 후속 인계 메모 필요 여부를 함께 판단할 때 사용

## 3. 기대 출력 계약

출력은 JSON 을 기본으로 하며 최소 아래 필드를 포함한다.

- `detected_change_types`
- `recommended_validation_levels`
- `recommended_commands`
- `commands_requiring_confirmation`
- `documentation_checks`
- `evidence_expectations`
- `deferred_validation_items`
- `warnings`
- `confidence_notes`
- `source_context`

## 4. 판단 규칙

### 4-1. 변경 유형 분류

- 코드 파일 변경:
- `code`
- 문서 파일 변경:
- `docs`
- UI 관련 경로 또는 프론트엔드 파일 변경:
- `ui`
- 설정 파일 변경:
- `config`
- 배포/운영 관련 경로 변경:
- `ops`
- 프롬프트/평가 자산 변경:
- `prompt_or_eval`

여러 유형이 동시에 감지되면 복수 유형 검증을 합산한다.

### 4-2. 기본 검증 수준

- `docs` 만 포함:
- 링크, 메타데이터, 허브 문서 동기화 점검 중심의 `documentation` 수준
- `code` 또는 `config` 포함:
- 빠른 테스트와 lint 를 포함하는 `standard` 수준
- `ui` 포함:
- `standard` 에 더해 smoke 또는 스크린샷 증빙을 요구하는 `ui_extended` 수준
- `ops` 포함:
- `standard` 에 더해 헬스체크, 롤백, 공지 여부를 확인하는 `release_sensitive` 수준
- `prompt_or_eval` 포함:
- 기준 실험 비교 또는 fixture 검증을 요구하는 `artifact_sensitive` 수준

## 5. 프로젝트 프로파일 활용 규칙

프로토타입은 프로젝트 프로파일의 아래 섹션을 우선 읽는다.

- `기본 명령`
- `프로젝트 특화 검증 포인트`
- `프로젝트 특화 예외 규칙`

이 세 섹션에서 다음 정보를 추출한다.

- 빠른 테스트 명령
- 격리 테스트 명령
- 실행 확인 명령
- 유형별 추가 검증 메모
- 환경 제약 또는 승인 규칙

## 6. 미실행 항목 처리 원칙

`validation-plan` 은 실제 명령 실행 여부를 모른다고 가정한다. 따라서 기본 출력은 "권장 계획" 이며, 실행 여부가 불명확하면 아래 방식으로 정리한다.

- `deferred_validation_items` 에 미실행 가능성이 있는 항목을 남긴다.
- 환경 제약, 권한 제약, 시간 제약이 프로파일에 있으면 그 내용을 경고나 메모에 반영한다.
- backlog 또는 handoff 경로가 입력되면 후속 기록 위치 후보를 함께 제안한다.

## 7. 프로토타입 범위

현재 단계의 프로토타입은 다음만 수행한다.

- 변경 파일을 보수적으로 분류
- 프로젝트 프로파일에서 검증 명령/메모를 추출
- 검증 수준과 추천 명령을 JSON 으로 정리
- 사람이 남겨야 할 증빙과 미실행 사유를 분리

현재 단계의 프로토타입은 다음은 수행하지 않는다.

- 실제 테스트 실행
- CI 상태 조회
- 브랜치/PR 맥락 분석
- 문서 직접 수정

## 8. 후속 확장 포인트

- 변경 diff 기반 영향도 정밀화
- 실행 결과 입력을 받아 미실행 항목 자동 축소
- harness 별 기본 검증 커맨드 템플릿 연결
- release, migration, data change 같은 세부 유형 분기 보강

## 다음에 읽을 문서

- skill 카탈로그: [./workflow_skill_catalog.md](./workflow_skill_catalog.md)
- skill 문서: [../skills/validation-plan/SKILL.md](../skills/validation-plan/SKILL.md)
