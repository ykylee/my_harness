# Per-Harness MCP Installation Guide (D-75 Plan A-2)

- 문서 목적: my_harness 의 5 MCP 프로토타입을 외부 harness (codex, opencode) 에 설치하는 절차와 transport 비교를 안내한다.
- 범위: harness 별 MCP config schema, transport 비교, 6 troubleshooting, 후속 TASK 링크
- 대상 독자: harness 운영자, MCP 구현자, AI agent 설계자
- 상태: draft (D-75 Plan A-2 도입)
- 최종 수정일: 2026-06-11
- 관련 문서: [../mcp_servers/README.md](../mcp_servers/README.md), [workflow_mcp_candidate_catalog.md](workflow_mcp_candidate_catalog.md), [read_only_mcp_transport_promotion.md](read_only_mcp_transport_promotion.md)
- 원본 참조: standard_ai_workflow v0.5.1 `workflow-source/core/mcp_installation_by_harness.md` (5.1 의 transport 비교 + 6 troubleshooting 의 my_harness 적응)

## 1. my_harness 의 5 MCP 프로토타입 (5.1 의 1 server-multi-tools 와 다른 패턴)

my_harness 는 `standardAiWorkflowReadOnly` 같은 단일 server (multi-tool) 가 아닌, **5 separate prototype** 패턴:

| Prototype | 역할 | read/write |
|---|---|---|
| `latest-backlog` | 최신 날짜 backlog 문서 찾기 | read-only |
| `check-doc-metadata` | markdown 필수 메타데이터 누락 점검 | read-only |
| `check-doc-links` | 상대 링크 무결성 검사 | read-only |
| `create-backlog-entry` | backlog entry 초안 JSON 생성 | write (dry-run 권장) |
| `suggest-impacted-docs` | 변경 파일 기준 영향 문서 후보 추천 | read-only |

각 프로토타입은 `ai-workflow/mcp_servers/<name>/MCP.md` + entrypoint. 단일 server 가 아니므로 harness 의 MCP config 에는 **5 server entry** 가 들어감 (또는 사용자 선택 subset).

## 2. Transport 비교 (5.1 의 jsonrpc-bridge vs stdio-sdk 동등)

| 항목 | jsonrpc-bridge (안정, default) | stdio-sdk (실험적) |
|---|---|---|
| 안정성 | stable (1.0+) | experimental |
| 의존성 | stdlib only | mcp SDK (1.27+) |
| sandbox 호환 | bridge 가 PYTHONPATH/ROOT anchor 필요 | SDK 가 native anchor |
| transport | subprocess stdin/stdout + JSON-RPC | subprocess stdio + SDK |
| 적합 | production / CI | 빠른 local dev |

**권장**: codex/opencode 모두 `jsonrpc-bridge` 로 시작. stdio-sdk 는 mcp SDK 안정화 후 재검토.

## 3. Harness 별 config schema

my_harness 가 실제로 사용하는 harness = **codex + opencode** (2 개). 나머지 (gemini-cli, antigravity, minimax-code) 는 my_harness 운영 scope 밖 → 5.1 의 6 example 중 4 는 적응 불요.

### 3.1 codex (TOML)

codex 의 MCP config 는 `~/.codex/config.toml` 의 `[mcp_servers.<name>]` 섹션. 예시는 [examples/mcp_config_examples/codex-mcp.toml](../examples/mcp_config_examples/codex-mcp.toml) 참조.

핵심:
- 5 server entry 각각이 `[mcp_servers.<name>]` block.
- `command` = `/usr/bin/env python3` (또는 my_harness 의 venv)
- `args` = `["-m", "ai_workflow.mcp_servers.<name>"]` 또는 entrypoint script path
- `env` = `PYTHONPATH=<my_harness repo root>/ai-workflow` (절대경로 권장)
- my_harness 의 ROOT anchor: `${workspaceFolder}` 또는 절대경로

### 3.2 opencode (JSON)

opencode 의 MCP config 는 `~/.opencode/mcp.json` 또는 workspace `.opencode/mcp.json` 의 `mcp` 키. 예시는 [examples/mcp_config_examples/opencode-mcp.json](../examples/mcp_config_examples/opencode-mcp.json) 참조.

핵심:
- top-level `mcp` 키 + `{ "<name>": { "type": "stdio", "command": [...], "env": {...} } }` 구조
- 5 server entry 각각이 키-값
- `command` 배열 = `["python3", "-m", "ai_workflow.mcp_servers.<name>"]`
- `env.PYTHONPATH` = 절대경로
- opencode 의 mcp key 의 `type` = `"stdio"` 명시

## 4. 글로벌 vs 로컬 (5.1 의 2 가지 install mode)

- **글로벌 (`~/.codex/config.toml`, `~/.opencode/mcp.json`)**: 모든 workspace 에서 사용. CI / multiple project.
- **로컬 (`<workspace>/.codex/config.toml`, `<workspace>/.opencode/mcp.json`)**: 해당 workspace 만. dev/test 격리.

my_harness 권장 = **로컬** (commit 가능, 재현 가능). 글로벌은 fallback.

## 5. 6 Troubleshooting 항목 (5.1 패턴 적응)

1. **command not found**: `python3` 가 시스템 기본 → pydantic/requests 없는 python resolve. 해결: `command` 를 `sys.executable` 또는 venv path 로 override.
2. **PYTHONPATH 상대경로 fail**: harness 가 다른 cwd 에서 spawn. 해결: `env.PYTHONPATH` 를 절대경로.
3. **server hang / no response**: bridge 가 initialize 응답 안 함. 해결: bridge log 확인 + `STANDARD_AI_WORKFLOW_ROOT` (또는 my_harness 의 `MYHARNESS_ROOT`) env 확인.
4. **tools/list empty**: server 는 응답하지만 tool 0 개. 해결: entrypoint script 의 `--list-tools` dry-run 으로 확인.
5. **permission denied on config write**: `~/.codex/config.toml` 또는 `~/.opencode/mcp.json` 가 read-only. 해결: `chmod 600` 후 retry.
6. **MCP server conflict with existing entry**: 동일 `<name>` 이 이미 있음. 해결: my_harness 의 5 prototype 의 `<name>` 은 unique (`latest-backlog`, `check-doc-metadata` 등) → 충돌 시 prefix 추가 (예: `myharness-latest-backlog`).

## 6. 후속 TASK 링크 (5.1 의 TASK-V051-005/006 동등)

- **TASK-005-2 MCP-1**: 5 prototype 의 stdio-sdk round-trip smoke (5.1 의 TASK-V051-006 동등).
- **TASK-005-2 MCP-2**: per-harness install auto-emit (`bootstrap_workflow_kit.py` 의 `--enable-mcp` 동등). my_harness 의 2204 lines bootstrap script 의 분기는 별도 sub-task.
- **TASK-005-2 MCP-3**: stdio-sdk 안정화 후 jsonrpc-bridge → stdio-sdk 마이그레이션 (5.1 의 TASK-V051-005 debug 동등).

## 7. 검증 (acceptance criteria)

- codex 의 `~/.codex/config.toml` 에 5 entry 가 모두 들어가고, codex restart 후 `tools/list` 에 5 tool 이 보임.
- opencode 의 `~/.opencode/mcp.json` (또는 `.opencode/mcp.json`) 에 5 entry 가 모두 들어가고, opencode 가 5 server 를 spawn 가능.
- 6 troubleshooting 항목의 각 scenario 에서 복구 절차가 동작.
- 5.1 의 TASK-V051-005/006 동등 항목 (MCP-1, MCP-2, MCP-3) 의 TODO 가 my_harness 의 TASK 백로그에 등록.

## 다음에 읽을 문서

- mcp 허브: [../mcp_servers/README.md](../mcp_servers/README.md)
- MCP 카탈로그: [workflow_mcp_candidate_catalog.md](workflow_mcp_candidate_catalog.md)
- 읽기 전용 bundle 초안: [../mcp_servers/read_only_bundle.md](../mcp_servers/read_only_bundle.md)
- 5.1 원본: standard_ai_workflow v0.5.1 `workflow-source/core/mcp_installation_by_harness.md`
