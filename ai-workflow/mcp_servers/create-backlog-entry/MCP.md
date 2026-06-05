# Create-Backlog-Entry MCP

- 문서 목적: `create_backlog_entry` MCP 프로토타입의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 카탈로그, 예상 입력/출력, 읽기/쓰기 성격, 구현 메모
- 대상 독자: MCP 구현자, AI agent 설계자, 운영자
- 상태: prototype
- 최종 수정일: 2026-04-18
- 관련 문서: `../../core/workflow_mcp_candidate_catalog.md`, `../../skills/backlog-update/SKILL.md`

## 1. 목적

작업 ID, 작업명, 날짜를 받아 날짜별 backlog 항목 초안을 생성한다.

## 2. 연결 카탈로그

- 후보 카탈로그: [../../core/workflow_mcp_candidate_catalog.md](../../core/workflow_mcp_candidate_catalog.md)

## 3. 예상 입력

- `task_id`
- `task_name`
- `request_date`
- 선택적으로 `status`, `priority`

## 4. 예상 출력

- `draft_entry`
- `warnings`

## 5. 읽기/쓰기 성격

- 읽기 전용 초안 생성

## 6. 구현 메모

- 실제 파일 수정 없이 구조화된 backlog entry JSON 반환

## 7. 현재 상태

- 실행 프로토타입 스크립트 있음

## 다음에 읽을 문서

- mcp 허브: [../README.md](../README.md)
