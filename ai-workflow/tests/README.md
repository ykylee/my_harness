# Tests

- 문서 목적: 표준 워크플로우 패키지의 링크, 메타데이터, 템플릿 smoke test 또는 향후 구현 테스트를 배치할 위치를 안내한다.
- 범위: 문서 무결성 검사와 향후 skill/MCP/agent 구현 검증
- 대상 독자: 개발자, 운영자, AI agent 설계자
- 상태: stable
- 최종 수정일: 2026-05-03

## 🚀 빠른 전체 테스트 (One-shot)

저장소 루트에서 다음 명령을 실행하여 모든 smoke test를 한 번에 검증할 수 있습니다:

```bash
for t in workflow-source/tests/check_*.py; do python3 "$t" || exit 1; done
```

## 현재 상태

- 기본 문서 스모크 체크 스크립트 `check_docs.py` 를 제공한다.
- bootstrap 스캐폴딩 결과를 확인하는 `check_bootstrap.py` 를 제공한다.
- 하네스 스텁 생성기를 확인하는 `check_scaffold_harness.py` 를 제공한다.
- 하네스 패키지 export 를 확인하는 `check_export_harness_package.py` 를 제공한다.
- `session-start`, `doc-sync`, `merge-doc-reconcile` 개별 skill smoke 를 제공한다.
- 기존 프로젝트 bootstrap 후속 온보딩 흐름을 확인하는 `check_existing_project_onboarding.py` 를 제공한다.
- demo runner 성공/실패 경로를 확인하는 `check_demo_workflow.py` 를 제공한다.
- quickstart/README stale 링크 점검 MCP 를 확인하는 `check_quickstart_stale_links.py` 를 제공한다.
- draft read-only MCP bundle manifest 와 payload schema 검증을 확인하는 `check_read_only_mcp_server.py` 를 제공한다.
- draft read-only JSON-RPC bridge 를 확인하는 `check_read_only_jsonrpc_bridge.py` 를 제공한다.
- read-only JSON-RPC fixture 산출물이 runtime bridge 와 같은지 확인하는 `check_read_only_jsonrpc_fixtures.py` 를 제공한다.
- read-only MCP transport 승격 기준 문서가 fixture 이름과 유지 계약을 반영하는지 확인하는 `check_read_only_transport_promotion_spec.py` 를 제공한다.
- runtime 계약에서 생성한 JSON Schema draft 와 체크인된 generated schema 정합성을 확인하는 `check_output_json_schema.py` 를 제공한다.
- generated JSON Schema draft 로 실제 output sample JSON 을 검증하는 `check_generated_schema_validation.py` 를 제공한다.
- read-only transport descriptor export 산출물이 registry 와 같은지 확인하는 `check_read_only_transport_descriptors.py` 를 제공한다.
- read-only harness MCP 예시 산출물이 descriptor 기준과 같은지 확인하는 `check_read_only_harness_mcp_examples.py` 를 제공한다.
- backlog-update 프로토타입의 성공/실패 경로와 output contract 를 확인하는 `check_backlog_update.py` 를 제공한다.
- create-backlog-entry MCP 프로토타입의 draft entry shape 를 확인하는 `check_create_backlog_entry.py` 를 제공한다.
- 현재 스크립트는 문서 무결성과 기본 생성 흐름이 깨지지 않았는지 빠르게 검사한다.

## 포함된 검사

- 문서 첫 부분에 필수 메타데이터 항목이 있는지 확인
- Markdown 상대 링크가 실제 파일을 가리키는지 확인
- 문서 제목이 `# ` 헤더로 시작하는지 확인
- bootstrap 스크립트가 신규/기존 프로젝트 모드에서 핵심 문서, 하네스 오버레이, core 복사본을 생성하는지 확인
- bootstrap manifest 에 하네스별 global snippet 후보 정보가 포함되는지 확인
- 하네스 스캐폴드 스크립트가 새 하네스 starter 문서를 생성하는지 확인
- 하네스 export 스크립트가 dist 산출물, manifest, zip 파일을 생성하는지 확인
- 기본 export manifest 가 minimal deployment profile 과 버전 정보를 기록하는지 확인
- 기본 export 가 source docs 와 global snippets 를 제외하는지 확인
- 기본 export archive 가 `PACKAGE_CONTENTS.md`, `APPLY_GUIDE.md` 를 포함하는지 확인
- `validation-plan` 프로토타입이 예시 프로젝트에서 기대한 분류와 검증 수준을 출력하는지 확인
- `session-start` 프로토타입이 세션 기준선 복원과 구조화된 실패 경로를 유지하는지 확인
- `doc-sync` 프로토타입이 영향 문서 후보, 검토 순서, follow-up action 과 구조화된 실패 경로를 유지하는지 확인
- `merge-doc-reconcile` 프로토타입이 병합 후 재확정 포인트와 구조화된 실패 경로를 유지하는지 확인
- `backlog-update` 프로토타입이 보수적 상태 추천, draft entry, 구조화 error 경로를 유지하는지 확인
- `create_backlog_entry` 프로토타입이 draft entry 구조와 공통 output contract 를 유지하는지 확인
- `code-index-update` 프로토타입이 예시 프로젝트에서 색인 문서 후보와 stale 경고를 출력하는지 확인
- `examples/output_samples/` 아래 JSON 샘플이 README 링크와 일치하고 유효한 JSON 인지 확인
- demo runner 가 상위 `orchestration_plan`, `workflow_summary`, `source_context` 를 유지하는지 확인
- demo runner 가 하위 step 실패를 top-level `workflow_step_failed` 로 감싸는지 확인
- 대표 JSON 샘플이 공통 필드뿐 아니라 skill/runner 계약의 핵심 필드도 유지하는지 확인
- `schemas/output_sample_contracts.json` 과 Python 런타임 계약 맵이 서로 어긋나지 않는지 확인
- error payload 의 주요 `source_context` shape 가 런타임 계약, 정적 계약, generated JSON Schema 에서 어긋나지 않는지 확인
- 기존 프로젝트 bootstrap 산출물을 입력으로 받아 onboarding runner 가 session-start, validation-plan, code-index-update 를 연결하는지 확인
- onboarding runner 가 누락 입력 문서 시 top-level 구조화 error JSON 을 반환하는지 확인
- quickstart/README 문서를 입력으로 받아 stale 링크 경고와 핵심 진입 문서 누락을 감지하는지 확인
- read-only MCP bundle manifest 가 tool별 input schema 와 payload example 을 노출하는지 확인
- read-only MCP bundle manifest 가 tool별 generated JSON Schema 를 runtime output contract 에서 직접 노출하는지 확인
- read-only MCP bundle 이 draft transport tool descriptor 를 노출하는지 확인
- read-only JSON-RPC bridge 가 `initialize`, initialize capability shape validation, notification lifecycle, stdio session initialize gating, `tools/list`, `tools/call`, malformed JSON, invalid request, tool-call error mapping 을 유지하는지 확인
- optional official MCP SDK candidate 가 runtime availability metadata 와 미설치 환경 오류 경계를 유지하는지 확인
- official MCP SDK client 가 stdio server candidate 와 실제 `initialize -> tools/list -> tools/call` 왕복을 수행할 수 있는지 확인
- read-only JSON-RPC fixture 가 runtime bridge 결과와 같은 request/response envelope 를 유지하는지 확인
- read-only MCP transport 승격 기준 문서가 fixture 기준선과 유지할 descriptor 계약을 놓치지 않는지 확인
- read-only transport descriptor 체크인 산출물과 생성 스크립트가 registry 결과와 같은지 확인
- read-only harness MCP 예시가 descriptor 대상, tool 목록, `transport_ready=false`, manual-review-only 원칙을 유지하는지 확인
- read-only MCP bundle entrypoint 자체 오류가 `read_only_entrypoint` output family 계약을 따르는지 확인
- read-only MCP bundle entrypoint 가 필수 payload 누락을 schema 단계에서 실패시키는지 확인
- generated JSON Schema draft 파일과 생성 스크립트 출력이 런타임 계약과 같은지 확인
- generated JSON Schema draft 가 실제 대표 sample JSON 을 받아들일 수 있는지 확인

## 실행 방법

- 저장소 루트에서 `for t in tests/check_*.py; do python3 "$t" || exit 1; done`
- 저장소 루트에서 `python3 tests/check_docs.py`
- 저장소 루트에서 `python3 tests/check_bootstrap.py`
- 저장소 루트에서 `python3 tests/check_scaffold_harness.py`
- 저장소 루트에서 `python3 tests/check_export_harness_package.py`
- 저장소 루트에서 `python3 tests/check_session_start.py`
- 저장소 루트에서 `python3 tests/check_doc_sync.py`
- 저장소 루트에서 `python3 tests/check_merge_doc_reconcile.py`
- 저장소 루트에서 `python3 tests/check_validation_plan.py`
- 저장소 루트에서 `python3 tests/check_backlog_update.py`
- 저장소 루트에서 `python3 tests/check_create_backlog_entry.py`
- 저장소 루트에서 `python3 tests/check_code_index_update.py`
- 저장소 루트에서 `python3 tests/check_output_samples.py`
- 저장소 루트에서 `python3 tests/check_demo_workflow.py`
- 저장소 루트에서 `python3 tests/check_existing_project_onboarding.py`
- 저장소 루트에서 `python3 tests/check_quickstart_stale_links.py`
- 저장소 루트에서 `python3 tests/check_read_only_mcp_server.py`
- 저장소 루트에서 `python3 tests/check_read_only_jsonrpc_bridge.py`
- 저장소 루트에서 `python3 tests/check_read_only_jsonrpc_fixtures.py`
- 저장소 루트에서 `python3 tests/check_read_only_transport_promotion_spec.py`
- 저장소 루트에서 `python3 tests/check_read_only_mcp_sdk_candidate.py`
- 저장소 루트에서 `python3 tests/check_read_only_mcp_sdk_stdio.py`
- 저장소 루트에서 `python3 tests/check_read_only_transport_descriptors.py`
- 저장소 루트에서 `python3 tests/check_read_only_harness_mcp_examples.py`
- 저장소 루트에서 `python3 tests/check_output_json_schema.py`
- 저장소 루트에서 `python3 tests/check_generated_schema_validation.py`

## 권장 실행 순서

- 전체 회귀를 빠르게 볼 때는 `for t in tests/check_*.py; do python3 "$t" || exit 1; done`
- 문서/계약 변경 직후에는 `check_docs.py`, `check_output_samples.py`, `check_quickstart_stale_links.py` 를 먼저 본다.
- bootstrap 또는 하네스 변경 직후에는 `check_bootstrap.py`, `check_scaffold_harness.py`, `check_export_harness_package.py` 를 먼저 본다.
- skill 구현 변경 직후에는 `check_session_start.py`, `check_doc_sync.py`, `check_merge_doc_reconcile.py`, `check_backlog_update.py` 를 먼저 본다.
- runner/orchestration 변경 직후에는 `check_demo_workflow.py`, `check_existing_project_onboarding.py` 를 먼저 본다.
- skill 분류/추천 로직 변경 직후에는 `check_validation_plan.py`, `check_code_index_update.py` 를 먼저 본다.
- backlog-update 변경 직후에는 `check_backlog_update.py` 를 먼저 본다.
- create-backlog-entry 변경 직후에는 `check_create_backlog_entry.py` 를 먼저 본다.
- read-only MCP bundle 변경 직후에는 `check_read_only_mcp_server.py` 를 먼저 본다.
- read-only JSON-RPC bridge 변경 직후에는 `check_read_only_jsonrpc_bridge.py` 를 먼저 본다.
- read-only JSON-RPC fixture 변경 직후에는 `check_read_only_jsonrpc_fixtures.py` 를 먼저 본다.
- official MCP SDK candidate 변경 직후에는 `check_read_only_mcp_sdk_candidate.py`, `check_read_only_mcp_sdk_stdio.py` 를 먼저 본다.
- read-only MCP transport 승격 기준 변경 직후에는 `check_read_only_transport_promotion_spec.py` 를 먼저 본다.
- read-only transport descriptor export 변경 직후에는 `check_read_only_transport_descriptors.py` 를 먼저 본다.
- read-only harness MCP 예시 변경 직후에는 `check_read_only_harness_mcp_examples.py` 를 먼저 본다.
- output contract / generated schema 변경 직후에는 `check_output_samples.py`, `check_output_json_schema.py`, `check_generated_schema_validation.py` 를 먼저 본다.
- JSON Schema 생성/승격 작업 직후에는 `check_output_json_schema.py`, `check_generated_schema_validation.py` 를 먼저 본다.

## 실패 분류 가이드

- `check_docs.py` 실패:
  메타데이터 누락, 문서 제목 형식, 상대 링크 깨짐을 먼저 의심한다.
- `check_output_samples.py` 실패:
  `examples/output_samples/`, `schemas/output_sample_contracts.json`, `workflow_kit/common/output_contracts.py` 셋 중 하나가 어긋난 경우가 많다. error sample 이라면 `error_field_shapes` 와 `source_context` required key 를 함께 확인한다.
- `check_demo_workflow.py` 실패:
  `scripts/run_demo_workflow.py` 의 step 조립, top-level error wrapping, `runner_inputs`, `execution_trace`, `workflow_summary` 필드 구성을 먼저 본다.
- `check_existing_project_onboarding.py` 실패:
  `scripts/run_existing_project_onboarding.py` 의 입력 경로 처리, latest backlog fallback, `runner_inputs`, `execution_trace`, `source_context` 유지 여부를 먼저 본다.
- `check_bootstrap.py` 실패:
  `scripts/bootstrap_workflow_kit.py` 의 생성 경로, 생성 문서 목록, adoption mode 분기를 먼저 본다.
- `check_scaffold_harness.py` 또는 `check_export_harness_package.py` 실패:
  `harnesses/` 문서, overlay manifest, dist 산출물 경로를 먼저 본다.
- `check_validation_plan.py` 실패:
  `skills/validation-plan/scripts/run_validation_plan.py` 와 `workflow_kit/common/planning.py` 또는 change type 분류를 먼저 본다.
- `check_session_start.py` 실패:
  `skills/session-start/scripts/run_session_start.py` 와 `workflow_kit/common/project_docs.py`, `workflow_kit/common/session_outputs.py` 를 먼저 본다.
- `check_doc_sync.py` 실패:
  `skills/doc-sync/scripts/run_doc_sync.py` 와 `workflow_kit/common/doc_sync.py` 의 후보 추천 규칙과 입력 경로 처리를 먼저 본다.
- `check_merge_doc_reconcile.py` 실패:
  `skills/merge-doc-reconcile/scripts/run_merge_doc_reconcile.py`, `workflow_kit/common/reconcile.py`, `workflow_kit/common/session_outputs.py` 를 먼저 본다.
- `check_backlog_update.py` 실패:
  `skills/backlog-update/scripts/run_backlog_update.py` 의 입력 경로 처리, 보수적 상태 추천, draft entry 조립, 구조화 error 반환을 먼저 본다.
- `check_create_backlog_entry.py` 실패:
  `mcp_servers/create-backlog-entry/scripts/run_create_backlog_entry.py` 의 draft entry 라인 조립과 공통 output contract 정합성을 먼저 본다.
- `check_code_index_update.py` 실패:
  `skills/code-index-update/scripts/run_code_index_update.py` 와 index candidate 추론 규칙을 먼저 본다.
- `check_quickstart_stale_links.py` 실패:
  quickstart/README 문서의 핵심 진입 링크와 상대 경로 무결성을 먼저 본다.
- `check_read_only_mcp_server.py` 실패:
  `workflow_kit/server/read_only_registry.py` 의 input/output schema 와 transport descriptor, `workflow_kit/server/read_only_entrypoint.py` 의 payload 검증, 하위 script 의 CLI 인자 이름 정합성을 먼저 본다.
- `check_read_only_jsonrpc_bridge.py` 실패:
  `workflow_kit/server/read_only_jsonrpc.py`, `workflow_kit/server/read_only_entrypoint.py`, read-only descriptor 의 tool 이름과 error mapping 을 먼저 본다.
- `check_read_only_jsonrpc_fixtures.py` 실패:
  `workflow_kit/server/read_only_jsonrpc.py`, `scripts/generate_read_only_jsonrpc_fixtures.py`, `schemas/read_only_jsonrpc_fixtures.json` 셋 중 하나가 어긋난 경우가 많다.
- `check_read_only_transport_promotion_spec.py` 실패:
  `core/read_only_mcp_transport_promotion.md` 가 fixture 이름, 유지할 descriptor 계약, 변경 가능한 envelope 설명을 놓친 경우가 많다.
- `check_read_only_mcp_sdk_candidate.py` 실패:
  `workflow_kit/server/read_only_mcp_sdk.py` 의 optional import 경계, CLI runtime metadata, SDK 미설치 fallback 오류를 먼저 본다.
- `check_read_only_mcp_sdk_stdio.py` 실패:
  `workflow_kit/server/read_only_mcp_sdk.py` 의 low-level server tool 등록, stdio loop, entrypoint 구조화 error payload 보존 여부를 먼저 본다.
- `check_read_only_transport_descriptors.py` 실패:
  `workflow_kit/server/read_only_registry.py`, `scripts/generate_read_only_transport_descriptors.py`, `schemas/read_only_transport_descriptors.json` 셋 중 하나가 어긋난 경우가 많다.
- `check_read_only_harness_mcp_examples.py` 실패:
  `scripts/generate_read_only_harness_mcp_examples.py`, `schemas/read_only_harness_mcp_examples.json`, read-only descriptor target/tool 목록 중 하나가 어긋난 경우가 많다.
- `check_output_json_schema.py` 실패:
  `workflow_kit/common/output_contracts.py`, `scripts/generate_output_json_schema.py`, `schemas/generated_output_schemas.json` 셋 중 하나가 어긋난 경우가 많다.
- `check_generated_schema_validation.py` 실패:
  generated schema 가 실제 sample payload 보다 과하게 엄격하거나, sample JSON 이 계약과 어긋난 경우가 많다.

## CI 읽기 포인트

- GitHub Actions smoke job 은 실패한 `tests/check_*.py` 파일 목록을 Step Summary 에 다시 적는다.
- CI 에서 실패한 파일만 로컬에서 그대로 재실행해 원인을 좁히는 흐름을 권장한다.
- runner 계열 실패는 먼저 개별 runner 스크립트를 직접 실행해 JSON payload 를 확인하는 편이 빠르다.
- 파일럿 적용 전후 기록은 [../templates/pilot_adoption_record_template.md](../templates/pilot_adoption_record_template.md) 형식으로 남겨두면 회귀와 운영 피드백 비교가 쉬워진다.
- 파일럿 대상 저장소는 [../templates/pilot_candidate_checklist.md](../templates/pilot_candidate_checklist.md) 기준으로 먼저 후보를 거르는 편이 안전하다.

## 향후 확장 후보

- 템플릿 placeholder 누락 검사
- 예시 문서와 템플릿 구조 차이 비교
- skill/MCP/agent 구현 추가 시 실행 가능한 smoke test 확장
