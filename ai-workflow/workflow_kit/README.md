# Workflow Kit Package

- 문서 목적: `workflow_kit/` Python 패키지 루트의 현재 역할과 포함 범위를 설명한다.
- 범위: 현재 포함된 공통 모듈, 초기 사용처, 향후 확장 방향
- 대상 독자: AI workflow 설계자, 구현자, MCP server 정리 담당자
- 상태: draft
- 최종 수정일: 2026-04-23
- 관련 문서: `../core/prototype_promotion_scope.md`, `../core/read_only_mcp_transport_promotion.md`, `../core/workflow_kit_roadmap.md`, `../mcp_servers/README.md`

## 1. 목적

`workflow_kit/` 은 문서/스크립트 수준에 흩어져 있던 공통 로직을 reusable package 로 옮기기 위한 초기 루트다. 현재는 모든 프로토타입을 한 번에 옮기지 않고, 중복이 큰 읽기 전용 MCP 유틸부터 단계적으로 정리한다.

## 2. 현재 포함된 모듈

- `common.paths`
- `common.markdown`
- `common.docs`
- `common.text`
- `common.project_docs`
- `common.change_types`
- `common.normalize`
- `common.reconcile`
- `common.planning`
- `common.doc_sync`
- `common.session_outputs`
- `common.runner`
- `common.errors`
- `common.output_contracts`
- `common.read_only_bundle`
- `server.read_only_registry`
- `server.read_only_tools`
- `server.read_only_entrypoint`
- `server.read_only_jsonrpc`
- `server.read_only_mcp_sdk`

이 모듈들은 아래 책임을 가진다.

- 존재하는 경로 해석
- Markdown 상대 링크 파싱과 깨진 링크 탐지
- 문서 메타데이터 누락 필드 확인
- project profile / handoff / backlog 파싱
- changed-file 분류와 validation change type 추정
- 문자열 정규화와 공통 dedupe
- handoff/backlog 상태 비교 설명 생성
- validation level 계산과 보수적 task status 결정
- doc-sync 계열 문서 후보 조립
- session 요약/권장 액션/merge reconcile note 조립
- JSON subprocess 실행, step 실패 구조화, 반복 flag 조립, runner 상위 warnings/orchestration 조립
- latest backlog 단계 조립과 optional path flag 생성
- runner top-level step 실패 error wrapping 공통화
- runner top-level success payload 조립 공통화
- 공통 error JSON payload 생성
- 샘플/스모크 테스트에서 재사용하는 출력 계약 맵 제공
- `schemas/output_sample_contracts.json` 과 나란히 유지할 런타임 계약 표현 제공
- read-only MCP 5종의 공통 callable layer 제공
- 읽기 전용 MCP 1차 묶음의 draft tool registry, direct-call adapter, schema-validated entrypoint, JSON-RPC draft bridge 제공
- 공식 MCP Python SDK 가 설치된 환경에서 사용할 optional stdio server candidate 제공

## 3. 현재 사용처

현재 아래 MCP 프로토타입이 `workflow_kit.common` 을 사용한다.

- `latest_backlog`
- `check_doc_links`
- `check_doc_metadata`
- `suggest_impacted_docs`
- `check_quickstart_stale_links`
- `create_backlog_entry`

현재 아래 skill/MCP 프로토타입도 `workflow_kit.common` 을 사용한다.

- `session-start`
- `validation-plan`
- `doc-sync`
- `suggest_impacted_docs`
- `backlog-update`
- `merge-doc-reconcile`
- `code-index-update`

현재 아래 orchestration script 도 `workflow_kit.common` 을 사용한다.

- `scripts/run_existing_project_onboarding.py`
- `scripts/run_demo_workflow.py`

즉, 읽기 전용 MCP 1차 묶음의 공통 기반이 이제 스크립트 내부 복사 로직이 아니라 package 모듈로 이동하기 시작했다.

## 4. 다음 확장 후보

- result payload builder 추가 정리
- runner 단계 조립 함수와 step 실패 컨텍스트 공통화
- `tool_version` 및 output contract 검증 기준의 단일 출처 유지
- generated JSON Schema 와 manifest 외부 소비 지점 연결
- 읽기 전용 MCP server transport descriptor 와 SDK 서버 루프 연결

현재 상태 메모:

- 코드 경로의 `tool_version` 은 이미 `workflow_kit.__version__` 에서 가져오도록 정리돼 있다.
- output contract 런타임 맵은 generated JSON Schema 와 sample validation smoke 의 단일 출처로도 쓰이고 있다.
- read-only MCP bundle manifest 는 tool 별 generated JSON Schema 를 runtime output contract 에서 직접 가져와 노출한다.
- read-only MCP bundle 은 draft transport tool descriptor 도 registry 에서 생성한다.
- read-only MCP bundle 은 SDK 서버 루프 전 단계의 JSON-RPC draft bridge 로 `initialize`, initialize capability validation, notification lifecycle, stdio session gating, `tools/list`, `tools/call` 경계를 smoke test 한다.
- read-only MCP bundle 은 `workflow_kit.server.read_only_mcp_sdk` 에 공식 MCP Python SDK low-level server 후보도 두고, SDK 미설치 환경에서는 availability metadata 와 오류 경계만 유지한다.
- read-only MCP bundle 은 개발 환경에서 official MCP SDK client 기준 stdio round-trip smoke 도 수행한다.
- 남은 정렬 작업은 draft bridge 를 실제 SDK server transport 로 승격할 때 필요한 protocol/runtime 차이를 분리하는 쪽이다.
- SDK transport 승격 시 유지할 field 와 바뀔 수 있는 envelope 는 [../core/read_only_mcp_transport_promotion.md](../core/read_only_mcp_transport_promotion.md) 에서 관리한다.

## 5. 원칙

- 지금 단계에서는 작은 유틸 모듈부터 옮긴다.
- 출력 계약은 기존 프로토타입과 동일하게 유지한다.
- orchestration runner 는 나중에 package 와 MCP server 조합의 상위 레이어로 정리한다.

## 다음에 읽을 문서

- 승격 범위 문서: [../core/prototype_promotion_scope.md](../core/prototype_promotion_scope.md)
- read-only transport 승격 기준: [../core/read_only_mcp_transport_promotion.md](../core/read_only_mcp_transport_promotion.md)
- 상위 로드맵: [../core/workflow_kit_roadmap.md](../core/workflow_kit_roadmap.md)
- mcp 허브: [../mcp_servers/README.md](../mcp_servers/README.md)
