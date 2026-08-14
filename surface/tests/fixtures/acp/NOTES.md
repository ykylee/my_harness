# ACP fixtures (D-140 S4a, grok 1.0.3)

Live handshake 2026-08-14 against `grok agent -m minimax --plugin-dir <plugin> stdio`.

| 항목 | 실측 |
| --- | --- |
| framing | **NDJSON** (한 줄 = 한 JSON-RPC 메시지). Content-Length 는 parse error. |
| argv | `grok agent -m <model> --plugin-dir <dir> stdio` (`-m`/`--plugin-dir` 는 `agent` 위. `stdio` 서브커맨드는 그 플래그를 거절) |
| protocolVersion | `1` |
| initialize | request id=1 → result `{protocolVersion, agentCapabilities, authMethods, _meta}` |
| session/new | request id=2, params `{cwd, mcpServers:[]}` → result `{sessionId, models, _meta}` |
| session/update | notification `{sessionId, update, _meta}` (handshake 중 availableCommands) |
| authenticate | `cached_token` 은 handshake 없이 session/new 가 성공. 선택. |
| session/prompt | handshake 에서 미관측. S4b 가 보냄. |
| session/request_permission | handshake 에서 미관측. 툴 호출 시 가설 유지. |
| 무시 | `_x.ai/*` 알림 (models/update, settings, announcements, mcp/*, session_notification) |

픽스처 JSON 은 시크릿·MCP args/env·모델 카탈로그를 넣지 않는다. 라이브 덤프를 그대로 커밋하지 말 것.

재현: `cargo run --manifest-path surface/Cargo.toml -- engine acp-probe --out /tmp/acp.json`
