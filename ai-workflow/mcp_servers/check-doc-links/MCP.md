# Check-Doc-Links MCP

- 문서 목적: `check_doc_links` MCP 프로토타입의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 카탈로그, 예상 입력/출력, 읽기/쓰기 성격, 구현 메모
- 대상 독자: MCP 구현자, AI agent 설계자, 운영자
- 상태: prototype
- 최종 수정일: 2026-04-18
- 관련 문서: `../../core/workflow_mcp_candidate_catalog.md`, `../../tests/check_docs.py`

## 1. 목적

문서 디렉터리의 markdown 상대 링크가 실제 파일을 가리키는지 점검한다.

## 2. 연결 카탈로그

- 후보 카탈로그: [../../core/workflow_mcp_candidate_catalog.md](../../core/workflow_mcp_candidate_catalog.md)

## 3. 예상 입력

- `doc_dir_path`

## 4. 예상 출력

- `checked_files`
- `broken_links`
- `warnings`

## 5. 읽기/쓰기 성격

- 읽기 전용

## 6. 구현 메모

- 상대 링크만 검사하고 외부 URL 은 제외

## 7. 현재 상태

- 실행 프로토타입 스크립트 있음

## 다음에 읽을 문서

- mcp 허브: [../README.md](../README.md)
