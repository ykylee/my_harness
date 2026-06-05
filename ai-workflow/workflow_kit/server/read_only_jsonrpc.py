#!/usr/bin/env python3
"""Draft JSON-RPC bridge for the read-only tool bundle.

This is intentionally a small fixture layer, not a full MCP SDK server.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[3]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit.server.read_only_entrypoint import invoke_tool
from workflow_kit.server.read_only_registry import READ_ONLY_SERVER_NAME, build_transport_tool_descriptors


JSONRPC_VERSION = "2.0"
SERVER_NOT_INITIALIZED_CODE = -32002


def jsonrpc_result(request_id: object, result: dict[str, Any]) -> dict[str, Any]:
    return {"jsonrpc": JSONRPC_VERSION, "id": request_id, "result": result}


def jsonrpc_error(request_id: object, code: int, message: str, data: dict[str, Any] | None = None) -> dict[str, Any]:
    error: dict[str, Any] = {"code": code, "message": message}
    if data is not None:
        error["data"] = data
    return {"jsonrpc": JSONRPC_VERSION, "id": request_id, "error": error}


def parse_request_json(raw_json: str) -> tuple[dict[str, Any] | None, dict[str, Any] | None]:
    try:
        request = json.loads(raw_json)
    except json.JSONDecodeError as exc:
        return None, jsonrpc_error(
            None,
            -32700,
            "Parse error",
            {"reason": "request must be valid JSON", "position": exc.pos},
        )
    if not isinstance(request, dict):
        return None, jsonrpc_error(None, -32600, "Invalid Request", {"reason": "request must be a JSON object"})
    return request, None


def build_initialize_result() -> dict[str, Any]:
    descriptors = build_transport_tool_descriptors()
    return {
        "serverInfo": {
            "name": READ_ONLY_SERVER_NAME,
            "version": descriptors["tool_version"],
        },
        "capabilities": {
            "tools": {
                "listChanged": False,
            },
        },
        "_meta": {
            "transport_ready": False,
            "bridge_phase": "jsonrpc_draft_fixture",
            "descriptor_target": descriptors["descriptor_target"],
        },
    }


def invalid_initialize_params(request_id: object, reason: str) -> dict[str, Any]:
    return jsonrpc_error(request_id, -32602, "Invalid params", {"reason": reason})


@dataclass
class JsonRpcSessionState:
    initialized: bool = False
    client_initialized: bool = False


def is_valid_jsonrpc_id(value: object) -> bool:
    if value is None:
        return True
    if isinstance(value, bool):
        return False
    return isinstance(value, (int, float, str))


def validate_optional_capability_object(
    request_id: object,
    capabilities: dict[str, Any],
    key: str,
) -> dict[str, Any] | None:
    value = capabilities.get(key)
    if value is not None and not isinstance(value, dict):
        return invalid_initialize_params(request_id, f"initialize params.capabilities.{key} must be an object")
    return None


def validate_optional_capability_bool(
    request_id: object,
    capability: dict[str, Any],
    capability_name: str,
    key: str,
) -> dict[str, Any] | None:
    value = capability.get(key)
    if value is not None and not isinstance(value, bool):
        return invalid_initialize_params(
            request_id,
            f"initialize params.capabilities.{capability_name}.{key} must be a boolean",
        )
    return None


def validate_initialize_params(request_id: object, params: Any) -> dict[str, Any] | None:
    if params is None:
        return None
    if not isinstance(params, dict):
        return invalid_initialize_params(request_id, "initialize params must be an object")
    capabilities = params.get("capabilities")
    if capabilities is not None and not isinstance(capabilities, dict):
        return invalid_initialize_params(request_id, "initialize params.capabilities must be an object")
    if isinstance(capabilities, dict):
        for key in ("tools", "roots", "sampling", "elicitation", "experimental"):
            error = validate_optional_capability_object(request_id, capabilities, key)
            if error is not None:
                return error
        tools = capabilities.get("tools")
        if isinstance(tools, dict):
            error = validate_optional_capability_bool(request_id, tools, "tools", "listChanged")
            if error is not None:
                return error
        roots = capabilities.get("roots")
        if isinstance(roots, dict):
            error = validate_optional_capability_bool(request_id, roots, "roots", "listChanged")
            if error is not None:
                return error
    client_info = params.get("clientInfo")
    if client_info is not None and not isinstance(client_info, dict):
        return invalid_initialize_params(request_id, "initialize params.clientInfo must be an object")
    protocol_version = params.get("protocolVersion")
    if protocol_version is not None and not isinstance(protocol_version, str):
        return invalid_initialize_params(request_id, "initialize params.protocolVersion must be a string")
    return None


def build_tools_list_result() -> dict[str, Any]:
    descriptors = build_transport_tool_descriptors()
    return {
        "tools": descriptors["tools"],
        "_meta": {
            "transport_ready": descriptors["transport_ready"],
            "descriptor_target": descriptors["descriptor_target"],
            "tool_count": descriptors["tool_count"],
        },
    }


def build_tools_call_result(name: str, arguments: dict[str, Any]) -> tuple[int, dict[str, Any]]:
    returncode, payload = invoke_tool(name, json.dumps(arguments, ensure_ascii=False))
    if returncode != 0:
        return returncode, payload
    text_representation: str
    if name == "smart_context_reader" and "extracted_content" in payload:
        text_representation = "\n\n".join(payload["extracted_content"])
    else:
        text_representation = json.dumps(payload, ensure_ascii=False, sort_keys=True)

    return 0, {
        "content": [
            {
                "type": "text",
                "text": text_representation,
            }
        ],
        "structuredContent": payload,
        "_meta": {
            "transport_ready": False,
            "bridge_phase": "jsonrpc_draft_fixture",
            "tool": name,
        },
    }


def server_not_initialized_error(request_id: object, method: str) -> dict[str, Any]:
    return jsonrpc_error(
        request_id,
        SERVER_NOT_INITIALIZED_CODE,
        "Server not initialized",
        {"reason": f"initialize must be called before {method} in stdio session mode"},
    )


def handle_jsonrpc_request(
    request: dict[str, Any],
    session_state: JsonRpcSessionState | None = None,
) -> dict[str, Any] | None:
    request_id = request.get("id")
    if "id" in request and not is_valid_jsonrpc_id(request_id):
        return jsonrpc_error(None, -32600, "Invalid Request", {"reason": "id must be a string, number, or null"})
    if request.get("jsonrpc") != JSONRPC_VERSION:
        return jsonrpc_error(request_id, -32600, "Invalid Request", {"reason": "jsonrpc must be 2.0"})

    method = request.get("method")
    if not isinstance(method, str):
        return jsonrpc_error(request_id, -32600, "Invalid Request", {"reason": "method is required"})

    if method.startswith("notifications/"):
        if "id" in request:
            return jsonrpc_error(request_id, -32600, "Invalid Request", {"reason": "notifications must not include id"})
        if session_state is not None and method == "notifications/initialized" and session_state.initialized:
            session_state.client_initialized = True
        return None
    if method == "initialize":
        if session_state is not None and session_state.initialized:
            return jsonrpc_error(
                request_id,
                -32600,
                "Invalid Request",
                {"reason": "initialize may only be called once per stdio session"},
            )
        error = validate_initialize_params(request_id, request.get("params"))
        if error is not None:
            return error
        if session_state is not None:
            session_state.initialized = True
            session_state.client_initialized = False
        return jsonrpc_result(request_id, build_initialize_result())
    if method == "tools/list":
        if session_state is not None and not session_state.initialized:
            return server_not_initialized_error(request_id, method)
        return jsonrpc_result(request_id, build_tools_list_result())
    if method == "tools/call":
        if session_state is not None and not session_state.initialized:
            return server_not_initialized_error(request_id, method)
        params = request.get("params")
        if not isinstance(params, dict):
            return jsonrpc_error(request_id, -32602, "Invalid params", {"reason": "params must be an object"})
        name = params.get("name")
        arguments = params.get("arguments", {})
        if not isinstance(name, str):
            return jsonrpc_error(request_id, -32602, "Invalid params", {"reason": "params.name must be a string"})
        if not isinstance(arguments, dict):
            return jsonrpc_error(request_id, -32602, "Invalid params", {"reason": "params.arguments must be an object"})
        returncode, result = build_tools_call_result(name, arguments)
        if returncode != 0:
            return jsonrpc_error(request_id, -32000, "Tool call failed", result)
        return jsonrpc_result(request_id, result)

    return jsonrpc_error(request_id, -32601, "Method not found", {"method": method})


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Draft JSON-RPC bridge for the read-only bundle.")
    parser.add_argument("--request-json", help="Single JSON-RPC request object to handle and print.")
    parser.add_argument(
        "--stdio-lines",
        action="store_true",
        help="Read newline-delimited JSON-RPC request objects from stdin and print responses.",
    )
    return parser.parse_args()


def print_response(response: dict[str, Any] | None) -> None:
    if response is not None:
        print(json.dumps(response, ensure_ascii=False, sort_keys=True), flush=True)


def main() -> int:
    args = parse_args()
    if args.request_json:
        request, error = parse_request_json(args.request_json)
        if error is not None:
            print_response(error)
            return 1
        assert request is not None
        print_response(handle_jsonrpc_request(request))
        return 0
    if args.stdio_lines:
        session_state = JsonRpcSessionState()
        for line in sys.stdin:
            stripped = line.strip()
            if not stripped:
                continue
            request, error = parse_request_json(stripped)
            if error is not None:
                print_response(error)
                continue
            assert request is not None
            print_response(handle_jsonrpc_request(request, session_state))
        return 0

    print_response(
        jsonrpc_error(None, -32600, "Invalid Request", {"reason": "--request-json or --stdio-lines is required"})
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
