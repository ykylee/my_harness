# Skill Prototype Layout

- 문서 목적: `skills/` 아래 프로토타입 디렉터리를 어떤 규칙으로 구성하는지 설명한다.
- 범위: 디렉터리 규칙, 파일 최소 구성, 실제 구현으로 확장하는 방법
- 대상 독자: skill 구현자, AI agent 설계자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-18
- 관련 문서: `./README.md`, `../core/workflow_skill_catalog.md`, `../core/session_start_skill_spec.md`, `../core/backlog_update_skill_spec.md`, `../core/doc_sync_skill_spec.md`, `../core/merge_doc_reconcile_skill_spec.md`

## 1. 목적

이 문서는 `skills/` 아래에 배치하는 프로토타입 디렉터리를 어떤 공통 규칙으로 유지할지 설명한다.

현재 단계에서는 실제 런타임 코드보다 아래 목표가 더 중요하다.

- 각 skill 의 실행 단위를 디렉터리 단위로 분리
- 코어 스펙 문서와 실제 구현 진입점을 느슨하게 연결
- 구현 전에도 목적, 입력, 출력, 권한 경계를 빠르게 파악 가능하게 만들기

## 2. 최소 디렉터리 규칙

각 skill 디렉터리는 최소한 아래 파일을 가져야 한다.

- `SKILL.md`

향후 구현이 시작되면 필요에 따라 아래를 추가할 수 있다.

- `examples/`
- `tests/`
- `schemas/`
- `scripts/`

## 3. 현재 프로토타입 구조

- [session-start/SKILL.md](./session-start/SKILL.md)
- [backlog-update/SKILL.md](./backlog-update/SKILL.md)
- [doc-sync/SKILL.md](./doc-sync/SKILL.md)
- [merge-doc-reconcile/SKILL.md](./merge-doc-reconcile/SKILL.md)

## 4. `SKILL.md` 필수 포함 요소

- skill 목적
- 연결된 코어 스펙 문서
- 예상 입력
- 예상 출력
- 읽기/쓰기 권한 경계
- 후속 구현 포인트
- 현재 구현 상태

## 5. 실제 구현으로 확장할 때 권장 규칙

- `SKILL.md` 의 입력/출력 용어는 코어 스펙과 동일하게 유지한다.
- 구현 코드는 수동 대체 절차를 제거하지 않고 보조하는 방향으로 만든다.
- 테스트는 성공 경로뿐 아니라 정보 부족, 경로 누락, 상태 충돌 경고 같은 보수적 동작도 검증한다.
- 상태 확정이 위험한 skill 은 자동 수정 기능보다 초안 생성 기능을 먼저 구현한다.

## 다음에 읽을 문서

- skills 허브: [./README.md](./README.md)
- skill 카탈로그: [../core/workflow_skill_catalog.md](../core/workflow_skill_catalog.md)
