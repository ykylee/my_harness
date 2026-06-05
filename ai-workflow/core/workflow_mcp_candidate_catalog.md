# Workflow MCP Candidate Catalog

- 문서 목적: 표준 워크플로우를 보조할 MCP 후보를 입력/출력과 함께 정리한다.
- 범위: 탐색, 검사, 초안 생성, 영향도 추천 등 반복 작업에 적합한 MCP 후보
- 대상 독자: AI agent 설계자, 개발자, 운영자
- 상태: Beta v2 통합 완료
- 최종 수정일: 2026-04-26
- 관련 문서: `workflow_skill_catalog.md`, `workflow_agent_topology.md`

## 1. 우선순위 1 (Beta v2 통합 완료)

| MCP 후보 | 역할 | 구현 상태 | 비고 |
| --- | --- | --- | --- |
| `latest_backlog` | 최신 백로그 파일 경로 탐색 | **Implemented** | 공식 SDK stdio 통합 |
| `check_doc_metadata` | 문서 메타데이터 무결성 검사 | **Implemented** | 공식 SDK stdio 통합 |
| `check_doc_links` | 문서 간 상대 링크 유효성 검사 | **Implemented** | 공식 SDK stdio 통합 |
| `create_backlog_entry` | 백로그 항목 자동 생성 (쓰기) | **Implemented** | 공식 SDK stdio 통합 |
| `suggest_impacted_docs` | 변경 파일 기준 영향 문서 추천 | **Implemented** | 공식 SDK stdio 통합 |

## 2. 우선순위 2 (Beta v2 통합 완료)

| MCP 후보 | 역할 | 구현 상태 | 비고 |
| --- | --- | --- | --- |
| `create_session_handoff_draft` | 백로그 기반 handoff 초안 생성 | **Implemented** | 공식 SDK stdio 통합 |
| `create_environment_record_stub` | 호스트 환경 정보 자동 추출 | **Implemented** | 공식 SDK stdio 통합 |
| `check_quickstart_stale_links` | 퀵스타트 문서 링크 정합성 검사 | **Implemented** | 공식 SDK stdio 통합 |

## 3. 우선순위 3 (Phase 5 차기 목표)

| MCP 후보 | 역할 | 입력 | 출력 | 구현 상태 |
| --- | --- | --- | --- | --- |
| `git_history_summarizer` | 변경 이력 자동 요약 | 커밋 범위 또는 날짜 | 백로그 스타일 요약 텍스트 | **Prototype** |
| `workflow_log_rotator` | 마일스톤 요약 및 문서 로테이션 | 문서 경로, 임계치 | 마일스톤 요약 초안 | 프로토타입 (내부 로직) |
| `dependency_vulnerability_checker` | 의존성 취약점 점검 | 의존성 파일 경로 | 취약점 리포트 및 권장 버전 | 미구현 |

## 4. 최소 입력 계약

- 문서 경로 입력은 프로젝트 프로파일에 정의된 문서 구조를 기준으로 해석한다.
- 변경 파일 목록 입력은 상대 경로 목록이면 충분하다.
- 출력은 사람이 바로 검토 가능한 텍스트 또는 구조화 목록이어야 한다.

## 5. 원칙

- 문서를 자동 확정하기보다 초안과 경고를 우선 제공한다.
- 구조화된 출력이 가능해야 한다.
- 프로젝트 특화 규칙은 프로젝트 프로파일을 입력으로 받아야 한다.

## 6. 공식 SDK 통합 상태

현재 모든 우선순위 1, 2 도구들은 `workflow_kit/server/read_only_mcp_sdk.py`를 통해 하나의 서버 엔트리포인트로 통합되어 있다. 하네스는 `stdio` 트랜스포트를 통해 이 도구들을 즉시 소비할 수 있다.

## 다음에 읽을 문서

- skill 카탈로그: [./workflow_skill_catalog.md](./workflow_skill_catalog.md)
- agent 토폴로지: [./workflow_agent_topology.md](./workflow_agent_topology.md)
- mcp 허브: [../mcp_servers/README.md](../mcp_servers/README.md)
