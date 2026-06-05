# MCP

- 문서 목적: 표준 워크플로우에서 공통으로 재사용할 MCP 구현을 배치할 위치와 역할을 안내한다.
- 범위: MCP 프로토타입 디렉터리 구조, 구현 진입점, 초기 도입 후보
- 대상 독자: AI agent 설계자, 개발자, 운영자
- 상태: draft
- 최종 수정일: 2026-04-18
- 관련 문서: `../core/workflow_mcp_candidate_catalog.md`, `./prototype_layout.md`

## 현재 상태

- 우선순위 1 MCP 후보 5종에 대한 프로토타입 디렉터리 구조를 추가했다.
- 현재 단계는 실행 가능한 프로토타입과 문서 기반 인터페이스 설명이 공존하는 상태다.
- 실제 MCP 서버 패키징이나 transport 계층은 아직 없다.
- 대신 읽기 전용 MCP 1차 묶음을 위한 draft registry 와 entrypoint 가 `workflow_kit/server/` 아래에서 시작됐다.

## 현재 구조

- [prototype_layout.md](./prototype_layout.md)
- [read_only_bundle.md](./read_only_bundle.md)
- [latest-backlog/MCP.md](./latest-backlog/MCP.md)
- [check-doc-metadata/MCP.md](./check-doc-metadata/MCP.md)
- [check-doc-links/MCP.md](./check-doc-links/MCP.md)
- [create-backlog-entry/MCP.md](./create-backlog-entry/MCP.md)
- [suggest-impacted-docs/MCP.md](./suggest-impacted-docs/MCP.md)
- [check-quickstart-stale-links/MCP.md](./check-quickstart-stale-links/MCP.md)

## 구현 원칙

- 각 MCP 디렉터리는 최소한 `MCP.md` 를 가져야 한다.
- `MCP.md` 에는 목적, 입력, 출력, 읽기/쓰기 성격, 구현 메모가 포함되어야 한다.
- 프로젝트 프로파일 기준 해석이 필요한 도구는 입력 또는 실행 옵션에 그 경로를 포함하는 것을 권장한다.
- 자동 확정보다 구조화된 결과와 경고를 우선한다.

## 현재 프로토타입 범위

- `latest-backlog`: backlog 디렉터리에서 최신 날짜 markdown 찾기
- `check-doc-metadata`: markdown 메타데이터 누락 검사
- `check-doc-links`: 상대 링크 무결성 검사
- `create-backlog-entry`: backlog entry 초안 JSON 생성
- `suggest-impacted-docs`: 변경 파일 기준 영향 문서 후보 추천
- `check-quickstart-stale-links`: quickstart/README 계열 문서의 stale 링크와 핵심 진입 문서 누락 점검

## 다음에 읽을 문서

- MCP 카탈로그: [../core/workflow_mcp_candidate_catalog.md](../core/workflow_mcp_candidate_catalog.md)
- 프로토타입 구조 안내: [./prototype_layout.md](./prototype_layout.md)
- 읽기 전용 bundle 초안: [./read_only_bundle.md](./read_only_bundle.md)
