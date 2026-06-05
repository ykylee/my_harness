# Suggest-Impacted-Docs MCP

- 문서 목적: `suggest_impacted_docs` MCP 프로토타입의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 카탈로그, 예상 입력/출력, 읽기/쓰기 성격, 구현 메모
- 대상 독자: MCP 구현자, AI agent 설계자, 운영자
- 상태: prototype
- 최종 수정일: 2026-04-18
- 관련 문서: `../../core/workflow_mcp_candidate_catalog.md`, `../../skills/doc-sync/SKILL.md`

## 1. 목적

변경 파일 목록을 받아 함께 볼 문서 후보를 간단히 추천한다.

## 2. 연결 카탈로그

- 후보 카탈로그: [../../core/workflow_mcp_candidate_catalog.md](../../core/workflow_mcp_candidate_catalog.md)

## 3. 예상 입력

- `changed_files`
- 선택적으로 `session_handoff_path`, `latest_backlog_path`, `work_backlog_index_path`

## 4. 예상 출력

- `impacted_documents`
- `reasoning_notes`
- `warnings`

## 5. 읽기/쓰기 성격

- 읽기 전용 추천

## 6. 구현 메모

- `doc-sync` 프로토타입보다 가벼운 문서 후보 추천에 집중

## 7. 현재 상태

- 실행 프로토타입 스크립트 있음

## 다음에 읽을 문서

- mcp 허브: [../README.md](../README.md)
