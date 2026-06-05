# Latest-Backlog MCP

- 문서 목적: `latest_backlog` MCP 프로토타입의 역할과 구현 진입점을 정리한다.
- 범위: 목적, 연결 카탈로그, 예상 입력/출력, 읽기/쓰기 성격, 구현 메모
- 대상 독자: MCP 구현자, AI agent 설계자, 운영자
- 상태: prototype
- 최종 수정일: 2026-04-18
- 관련 문서: `../../core/workflow_mcp_candidate_catalog.md`, `../../core/workflow_skill_catalog.md`

## 1. 목적

백로그 디렉터리 또는 backlog index 를 기준으로 가장 최신 날짜 backlog 문서를 찾는다.

## 2. 연결 카탈로그

- 후보 카탈로그: [../../core/workflow_mcp_candidate_catalog.md](../../core/workflow_mcp_candidate_catalog.md)

## 3. 예상 입력

- `backlog_dir_path`
- 선택적으로 `work_backlog_index_path`

## 4. 예상 출력

- `latest_backlog_path`
- `candidates`
- `warnings`

## 5. 읽기/쓰기 성격

- 읽기 전용

## 6. 구현 메모

- backlog index 링크가 있으면 우선 사용
- 없으면 날짜 패턴 기반 파일명을 정렬

## 7. 현재 상태

- 실행 프로토타입 스크립트 있음

## 다음에 읽을 문서

- mcp 허브: [../README.md](../README.md)
