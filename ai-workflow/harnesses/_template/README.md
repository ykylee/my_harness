# Harness Template

- 문서 목적: 새로운 AI 하네스를 이 저장소에 추가할 때 필요한 최소 구성과 체크리스트를 제공한다.
- 범위: 문서 추가 위치, bootstrap 확장 포인트, 테스트 포인트
- 대상 독자: 저장소 관리자, AI workflow 설계자, 하네스 통합 담당자
- 상태: draft
- 최종 수정일: 2026-04-19
- 관련 문서: `../../core/workflow_harness_distribution.md`, `../../scripts/bootstrap_workflow_kit.py`, `../README.md`

## 1. 새 하네스 추가 최소 단계

1. `python3 scripts/scaffold_harness.py --harness-name <target>` 로 starter 문서를 생성한다.
2. `scripts/bootstrap_workflow_kit.py` 에 하네스 파일 생성 함수를 추가한다.
3. `HARNESS_DEFINITIONS`, `HARNESS_FILE_BUILDERS` 에 새 하네스를 등록한다.
4. bootstrap smoke test에 새 하네스 생성 검증을 추가한다.
5. 루트 `README.md`, `scripts/README.md`, `core/workflow_harness_distribution.md` 에 새 타겟을 반영한다.
6. export bundle 에 포함되는 `bundle/source-docs/schemas/read_only_transport_descriptors.json` 를 새 하네스 MCP 설정에서 참고 산출물로 둘지, 실제 연결 입력으로 변환할지 결정한다.

## 2. 새 하네스 설계 원칙

- 긴 정책 본문을 하네스 전용 파일에 복제하지 않는다.
- 가능한 한 `ai-workflow/memory/` 문서를 읽도록 연결한다.
- 하네스가 요구하는 루트 파일과 최소 설정 파일만 생성한다.
- 권한 정책이나 에이전트 정의는 보수적인 기본값으로 시작한다.
- read-only MCP descriptor 는 draft 이므로 `transport_ready=false` 상태에서는 자동 연결보다 검토용 산출물로 다룬다.

## 3. bootstrap 확장 포인트

- `SUPPORTED_HARNESSES`
- `HARNESS_DEFINITIONS`
- `HARNESS_FILE_BUILDERS`

새 하네스는 위 세 군데에 등록되면 CLI 옵션과 실제 파일 생성 경로가 함께 연결된다.

## 4. 테스트 포인트

- `tests/check_bootstrap.py` 에 생성 파일 존재 여부를 추가한다.
- 필요하면 새 하네스 설정 파일의 최소 구조를 문자열 검사한다.
- `tests/check_docs.py` 로 문서 링크와 메타데이터가 유지되는지 확인한다.
- 하네스 export 를 지원한다면 `tests/check_export_harness_package.py` 에 descriptor 포함 여부를 확인한다.

## 다음에 읽을 문서

- 하네스 허브: [../README.md](../README.md)
- 배포 전략: [../../core/workflow_harness_distribution.md](../../core/workflow_harness_distribution.md)
