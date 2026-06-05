# Read-Only MCP Transport Promotion Spec

- 문서 목적: read-only JSON-RPC draft bridge 를 정식 MCP SDK transport 로 승격할 때 유지할 계약과 바뀔 수 있는 envelope 를 분리한다.
- 범위: descriptor 단일 출처, JSON-RPC fixture 기준선, SDK 승격 시 유지/변경 필드, 검증 경로
- 대상 독자: MCP server 구현자, 하네스 적용 담당자, AI workflow 설계자
- 상태: draft
- 최종 수정일: 2026-04-23
- 관련 문서: `./prototype_promotion_scope.md`, `../mcp_servers/read_only_bundle.md`, `../schemas/README.md`, `../workflow_kit/README.md`

## 1. 목적

현재 read-only bundle 은 정식 MCP SDK server 가 아니라 아래 세 계층으로 나뉜 draft 상태다.

- registry: `workflow_kit/server/read_only_registry.py`
- direct-call entrypoint: `workflow_kit/server/read_only_entrypoint.py`
- JSON-RPC draft bridge: `workflow_kit/server/read_only_jsonrpc.py`
- optional official SDK candidate: `workflow_kit/server/read_only_mcp_sdk.py`

이 문서는 정식 MCP SDK transport 를 붙일 때 draft bridge 전체를 그대로 “완성품”으로 오해하지 않도록, 유지해야 할 계약과 교체 가능한 envelope 를 분리한다.

## 2. 단일 출처

아래 값은 SDK transport 로 승격해도 registry 를 단일 출처로 유지한다.

- tool 이름
- tool 설명
- input schema
- output schema
- `annotations.readOnlyHint`
- `transport_ready`
- `descriptor_target`

현재 descriptor export 는 [../schemas/read_only_transport_descriptors.json](../schemas/read_only_transport_descriptors.json) 이고, 하네스 설정 검토용 draft 는 [../schemas/read_only_harness_mcp_examples.json](../schemas/read_only_harness_mcp_examples.json) 이다.

## 3. Fixture 기준선

SDK 승격 전 비교 기준선은 [../schemas/read_only_jsonrpc_fixtures.json](../schemas/read_only_jsonrpc_fixtures.json) 이다.

현재 fixture 는 아래 request/response 를 고정한다.

- `initialize`
- `initialize_with_supported_capabilities`
- `initialize_invalid_capabilities`
- `initialize_invalid_tools_list_changed`
- `initialize_invalid_roots_capability`
- `notification_initialized_no_response`
- `notification_unknown_no_response`
- `notification_with_id_invalid_request`
- `tools_list`
- `latest_backlog_call_success`
- `check_doc_metadata_call_schema_error`
- `unknown_method`
- `invalid_boolean_id`
- `malformed_json_parse_error`
- `non_object_invalid_request`

현재 fixture 는 아래 stdio session sequence 도 고정한다.

- `stdio_session_requires_initialize`
- `stdio_session_rejects_second_initialize`

이 fixture 는 실제 MCP client 호환성 보장이 아니라, draft bridge 의 envelope 변화가 의도된 것인지 확인하기 위한 diff 기준선이다.

## 4. 유지할 계약

SDK transport 로 승격할 때도 아래 계약은 유지해야 한다.

- `tools/list` 계열 응답은 registry descriptor 의 tool 목록과 같은 순서를 유지한다.
- tool descriptor 의 `inputSchema` 와 `outputSchema` 는 runtime contract 생성 결과를 사용한다.
- 읽기 전용 tool 은 `annotations.readOnlyHint: true` 를 유지한다.
- 성공한 tool call 은 원래 tool output payload 를 구조화 결과로 보존한다.
- 실패한 tool call 은 원래 entrypoint error payload 의 `error_code`, `warnings`, `source_context` 를 잃지 않는다.
- `initialize` 입력은 draft bridge 단계에서도 object `params` 와 object `capabilities` 경계를 유지한다.
- `initialize` 의 `capabilities.tools`, `capabilities.roots`, `capabilities.sampling`, `capabilities.elicitation`, `capabilities.experimental` 는 있으면 object 여야 한다.
- `capabilities.tools.listChanged`, `capabilities.roots.listChanged` 는 있으면 boolean 이어야 한다.
- `notifications/*` request 는 `id` 가 없을 때만 draft bridge 단계에서 응답 없이 무시한다.
- JSON-RPC `id` 는 있으면 string, number, null 중 하나여야 하고 boolean/object/array 는 허용하지 않는다.
- `--stdio-lines` session 에서는 `initialize` 성공 전 `tools/list`, `tools/call` 을 허용하지 않는다.
- `--stdio-lines` session 에서는 `initialize` 를 한 번만 허용한다.
- `transport_ready=false` 인 동안 하네스 예시는 `manual_review_only` 상태를 유지한다.

## 5. 바뀔 수 있는 envelope

정식 MCP SDK transport 로 바뀌면 아래 envelope 는 달라질 수 있다.

- JSON-RPC top-level `id` 처리
- parse error 와 invalid request 의 세부 `data` 필드
- SDK 초기화 응답의 capability 상세 필드
- initialize 입력에서 capability 내부 세부 필드 중 draft bridge 가 아직 검증하지 않는 나머지 필드
- tool call result 의 `content` wrapper 형식
- tool call error 의 JSON-RPC error code 와 message
- notification 처리 방식
- JSON-RPC request/notification lifecycle 세부 처리
- stdio session 상태 저장 방식과 handshake 세부 순서
- stdio framing 방식

이 항목이 바뀌더라도 4장의 유지 계약이 깨지지 않으면 정상적인 승격 변경으로 볼 수 있다.

## 6. 승격 전 검증

SDK transport 를 붙이기 전에는 아래 검증을 먼저 통과해야 한다.

```bash
python3 tests/check_read_only_mcp_server.py
python3 tests/check_read_only_jsonrpc_bridge.py
python3 tests/check_read_only_jsonrpc_fixtures.py
python3 tests/check_read_only_mcp_sdk_candidate.py
python3 tests/check_read_only_mcp_sdk_stdio.py
python3 tests/check_read_only_transport_descriptors.py
python3 tests/check_read_only_harness_mcp_examples.py
```

승격 구현 후에는 fixture diff 를 보고 envelope 변경이 5장의 허용 범위인지 확인한다.

## 7. 다음에 읽을 문서

- 승격 범위 문서: [./prototype_promotion_scope.md](./prototype_promotion_scope.md)
- read-only MCP bundle: [../mcp_servers/read_only_bundle.md](../mcp_servers/read_only_bundle.md)
- schema 허브: [../schemas/README.md](../schemas/README.md)
- package 루트: [../workflow_kit/README.md](../workflow_kit/README.md)
