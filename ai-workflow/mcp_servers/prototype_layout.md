# MCP Prototype Layout

- 문서 목적: `mcp_servers/` 아래 프로토타입 디렉터리를 어떤 규칙으로 구성하는지 설명한다.
- 범위: 디렉터리 규칙, 파일 최소 구성, 실제 MCP 서버로 확장하는 방법
- 대상 독자: MCP 구현자, AI agent 설계자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-18
- 관련 문서: `./README.md`, `../core/workflow_mcp_candidate_catalog.md`

## 1. 목적

이 문서는 `mcp_servers/` 아래에 배치하는 프로토타입 디렉터리를 어떤 공통 규칙으로 유지할지 설명한다.

현재 단계에서는 실제 서버 프로세스보다 아래 목표가 더 중요하다.

- 각 MCP 후보를 디렉터리 단위로 분리
- 입력/출력 계약과 실행 예시를 빠르게 파악 가능하게 만들기
- 향후 MCP 서버로 감쌀 수 있는 실행 스크립트를 먼저 제공

## 2. 최소 디렉터리 규칙

각 MCP 디렉터리는 최소한 아래 파일을 가져야 한다.

- `MCP.md`

향후 구현이 시작되면 필요에 따라 아래를 추가할 수 있다.

- `scripts/`
- `schemas/`
- `examples/`
- `tests/`

## 3. 현재 프로토타입 구조

- [latest-backlog/MCP.md](./latest-backlog/MCP.md)
- [check-doc-metadata/MCP.md](./check-doc-metadata/MCP.md)
- [check-doc-links/MCP.md](./check-doc-links/MCP.md)
- [create-backlog-entry/MCP.md](./create-backlog-entry/MCP.md)
- [suggest-impacted-docs/MCP.md](./suggest-impacted-docs/MCP.md)

## 4. `MCP.md` 필수 포함 요소

- MCP 목적
- 연결된 카탈로그 문서
- 예상 입력
- 예상 출력
- 읽기/쓰기 성격
- 후속 구현 포인트
- 현재 구현 상태

## 5. 실제 MCP 서버로 확장할 때 권장 규칙

- 프로토타입 스크립트의 입력/출력 용어를 그대로 도구 인터페이스로 승격한다.
- 성공 경로뿐 아니라 정보 부족, 경로 누락, 경고 출력도 테스트 대상에 포함한다.
- 문서 자동 확정이 위험한 도구는 초안 생성이나 검사 위주로 먼저 구현한다.

## 다음에 읽을 문서

- mcp 허브: [./README.md](./README.md)
- MCP 카탈로그: [../core/workflow_mcp_candidate_catalog.md](../core/workflow_mcp_candidate_catalog.md)
