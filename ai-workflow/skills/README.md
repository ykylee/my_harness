# Skills

- 문서 목적: 표준 워크플로우에서 공통으로 재사용할 skill 구현을 배치할 위치와 역할을 안내한다.
- 범위: skill 프로토타입 디렉터리 구조, 구현 진입점, 초기 도입 후보
- 대상 독자: AI agent 설계자, 개발자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-21
- 관련 문서: `../core/workflow_skill_catalog.md`, `./prototype_layout.md`

## 현재 상태

- 1차 핵심 skill 4종은 실행 가능한 읽기 전용 프로토타입까지 포함한다.
- 2차 skill `validation-plan`, `code-index-update` 도 실행 가능한 읽기 전용 프로토타입을 포함한다.
- 각 디렉터리의 `SKILL.md` 는 목적, 입력/출력 계약 연결, 구현 메모, 실행 예시를 포함한다.
- `workflow_kit.common` 기반 공통 파서와 helper 를 재사용하도록 실행 스크립트 정리가 진행 중이다.
- 실행형 skill 6종 중 `session-start`, `validation-plan`, `code-index-update`, `doc-sync`, `merge-doc-reconcile`, `backlog-update` 는 구조화된 실패 출력 `error_code` 패턴을 포함한다.
- 예시 프로젝트 기준 end-to-end 실행 흐름은 [../examples/end_to_end_skill_demo.md](../examples/end_to_end_skill_demo.md) 에 정리돼 있다.

## 현재 구조

- [prototype_layout.md](./prototype_layout.md)
- [session-start/SKILL.md](./session-start/SKILL.md)
- [backlog-update/SKILL.md](./backlog-update/SKILL.md)
- [doc-sync/SKILL.md](./doc-sync/SKILL.md)
- [merge-doc-reconcile/SKILL.md](./merge-doc-reconcile/SKILL.md)
- [validation-plan/SKILL.md](./validation-plan/SKILL.md)
- [code-index-update/SKILL.md](./code-index-update/SKILL.md)

## 구현 원칙

- 각 skill 디렉터리는 최소한 `SKILL.md` 를 가져야 한다.
- `SKILL.md` 에는 목적, 입력, 출력, 읽기/쓰기 권한 경계, 후속 구현 포인트가 포함되어야 한다.
- 정책 원문은 `core/` 에 두고, `skills/` 는 실행 절차와 구현 단위에 집중한다.
- 실제 구현이 추가되더라도 수동 대체 절차는 유지되어야 한다.

## 다음 구현 순서 권고

- `validation-plan`: 실행 결과 입력을 받아 미실행 항목을 자동 축소하는 단계 추가
- `code-index-update`: git diff 기반 rename 감지와 링크 그래프 분석 보강
- `session-start`: 프로젝트별 문서 템플릿 차이를 더 보수적으로 흡수하도록 파서와 실패 분류를 보강
- `doc-sync`: 허브 문서 후보 추론 규칙 정교화

## 다음에 읽을 문서

- skill 카탈로그: [../core/workflow_skill_catalog.md](../core/workflow_skill_catalog.md)
- 프로토타입 구조 안내: [./prototype_layout.md](./prototype_layout.md)
- end-to-end 데모: [../examples/end_to_end_skill_demo.md](../examples/end_to_end_skill_demo.md)
