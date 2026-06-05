# Check-Quickstart-Stale-Links MCP

- 문서 목적: `check_quickstart_stale_links` MCP 프로토타입의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 카탈로그, 예상 입력/출력, 읽기/쓰기 성격, 구현 메모
- 대상 독자: MCP 구현자, AI agent 설계자, 운영자
- 상태: prototype
- 최종 수정일: 2026-04-20
- 관련 문서: `../../core/workflow_mcp_candidate_catalog.md`, `../../core/workflow_adoption_entrypoints.md`, `../../scripts/run_existing_project_onboarding.py`

## 1. 목적

quickstart, README, 도입 안내 문서가 현재 워크플로우 진입 문서를 제대로 가리키는지 점검한다.

이 MCP 는 단순 broken link 검사보다 한 단계 위에서 아래를 함께 본다.

- 상대 링크가 실제로 존재하는지
- quickstart 문서가 핵심 진입 문서 링크를 빠뜨리지 않았는지
- 문서가 현재 워크플로우 진입 구조와 어긋났을 가능성이 있는지

## 2. 연결 카탈로그

- 후보 카탈로그: [../../core/workflow_mcp_candidate_catalog.md](../../core/workflow_mcp_candidate_catalog.md)

## 3. 예상 입력

- `quickstart_paths`
- 선택적으로 `project_profile_path`, `session_handoff_path`, `work_backlog_index_path`, `agents_path`

## 4. 예상 출력

- `checked_files`
- `broken_links`
- `missing_expected_links`
- `stale_link_warnings`
- `reasoning_notes`
- `warnings`

## 5. 읽기/쓰기 성격

- 읽기 전용 점검

## 6. 구현 메모

- broken link 여부와 핵심 진입 문서 링크 누락을 분리한다.
- bootstrap 생성물, project quickstart, README 같은 문서를 우선 대상으로 가정한다.
- 핵심 진입 문서 링크는 profile, handoff, backlog, 선택적으로 `AGENTS.md` 를 기본 세트로 본다.

## 7. 현재 상태

- 실행 프로토타입 스크립트 있음

## 다음에 읽을 문서

- mcp 허브: [../README.md](../README.md)
