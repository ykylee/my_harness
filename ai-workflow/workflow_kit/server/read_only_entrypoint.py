#!/usr/bin/env python3
"""Draft entrypoint for the first read-only MCP tool bundle."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit import __version__ as TOOL_VERSION
from workflow_kit.common.errors import build_error_result
from workflow_kit.common.output_contracts import validate_output_payload
from workflow_kit.server.read_only_registry import build_server_manifest, build_transport_tool_descriptors, get_tool_spec
from workflow_kit.server.read_only_tools import invoke_read_only_tool


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Draft entrypoint for the read-only MCP bundle.")
    parser.add_argument("--list-tools", action="store_true", help="Print the bundled tool manifest as JSON.")
    parser.add_argument(
        "--list-transport-tools",
        action="store_true",
        help="Print draft MCP-style tool descriptors as JSON.",
    )
    parser.add_argument("--tool", help="Tool name from the read-only bundle registry.")
    parser.add_argument(
        "--payload-json",
        help="JSON object that will be converted into repeated CLI flags for the selected tool script.",
    )
    return parser.parse_args()


def build_entrypoint_error_result(
    *,
    error: str,
    error_code: str,
    warnings: list[str],
    source_context: dict[str, Any],
) -> dict[str, Any]:
    return build_error_result(
        tool_version=TOOL_VERSION,
        error=error,
        error_code=error_code,
        warnings=warnings,
        source_context=source_context,
    )


def validate_payload(spec: Any, payload: dict[str, Any]) -> list[str]:
    field_by_name = {field.name: field for field in spec.input_fields}
    errors: list[str] = []

    for key in payload:
        if key not in field_by_name:
            errors.append(f"`{key}` 는 `{spec.name}` tool schema 에 없는 field 이다.")

    for field in spec.input_fields:
        value = payload.get(field.name)
        if value is None:
            if field.required:
                errors.append(f"`{field.name}` field 는 필수다.")
            continue

        if field.repeated:
            if not isinstance(value, list):
                errors.append(f"`{field.name}` field 는 list 형태여야 한다.")
                continue
            if field.required and not value:
                errors.append(f"`{field.name}` field 는 최소 한 개 이상의 값을 포함해야 한다.")
                continue
            if any(isinstance(item, (dict, list, bool)) or item is None for item in value):
                errors.append(f"`{field.name}` field list 항목은 문자열로 변환 가능한 scalar 여야 한다.")
            continue

        if isinstance(value, (dict, list, bool)):
            errors.append(f"`{field.name}` field 는 단일 scalar 값이어야 한다.")

    if spec.requires_any_of and not any(payload.get(name) is not None for name in spec.requires_any_of):
        joined = ", ".join(f"`{name}`" for name in spec.requires_any_of)
        errors.append(f"{joined} 중 하나는 제공해야 한다.")

    return errors


def invoke_tool(tool_name: str, payload_json: str | None) -> tuple[int, dict[str, Any]]:
    spec = get_tool_spec(tool_name)
    source_context = {"action": "tool", "tool": tool_name, "payload_json": payload_json}
    if spec is None:
        return 1, build_entrypoint_error_result(
            error="알 수 없는 읽기 전용 MCP tool 이다.",
            error_code="unknown_read_only_tool",
            warnings=["등록된 tool 이름을 다시 확인해야 한다."],
            source_context=source_context,
        )
    if not payload_json:
        return 1, build_entrypoint_error_result(
            error="선택한 tool 을 실행하려면 payload JSON 이 필요하다.",
            error_code="missing_tool_payload",
            warnings=["`--payload-json` 에 JSON object 문자열을 전달해야 한다."],
            source_context=source_context,
        )
    try:
        payload = json.loads(payload_json)
    except json.JSONDecodeError:
        return 1, build_entrypoint_error_result(
            error="payload JSON 을 해석할 수 없다.",
            error_code="invalid_tool_payload",
            warnings=["`--payload-json` 은 JSON object 형태여야 한다."],
            source_context=source_context,
        )
    if not isinstance(payload, dict):
        return 1, build_entrypoint_error_result(
            error="payload JSON 은 object 형태여야 한다.",
            error_code="invalid_tool_payload_shape",
            warnings=["배열이나 문자열이 아니라 key/value object 를 전달해야 한다."],
            source_context=source_context,
        )
    validation_errors = validate_payload(spec, payload)
    if validation_errors:
        return 1, build_entrypoint_error_result(
            error="payload JSON 이 read-only bundle input schema 와 맞지 않는다.",
            error_code="invalid_tool_payload_schema",
            warnings=validation_errors,
            source_context=source_context | {"allowed_fields": [field.name for field in spec.input_fields]},
        )
    try:
        result = invoke_read_only_tool(tool_name=tool_name, payload=payload, tool_version=TOOL_VERSION)
        output_errors = validate_output_payload(result, family=tool_name)
        if output_errors:
            return 1, build_entrypoint_error_result(
                error="read-only tool 이 output contract 와 맞지 않는 payload 를 반환했다.",
                error_code="invalid_tool_output_contract",
                warnings=output_errors,
                source_context=source_context,
            )
        return 0, result
    except FileNotFoundError as exc:
        return 1, build_entrypoint_error_result(
            error="payload 에 포함된 경로를 찾을 수 없다.",
            error_code="missing_tool_input_path",
            warnings=["존재하지 않는 문서/디렉터리 경로가 전달됐다."],
            source_context=source_context | {"missing_path": str(exc)},
        )
    except Exception as exc:
        return 1, build_entrypoint_error_result(
            error="read-only tool direct call 실행 중 예기치 않은 오류가 발생했다.",
            error_code="read_only_tool_runtime_error",
            warnings=["tool direct-call adapter 구현과 payload 계약을 함께 확인해야 한다."],
            source_context=source_context | {"exception_type": type(exc).__name__, "exception": str(exc)},
        )


def main() -> int:
    args = parse_args()
    if args.list_tools:
        print(json.dumps(build_server_manifest(), ensure_ascii=False, indent=2))
        return 0
    if args.list_transport_tools:
        print(json.dumps(build_transport_tool_descriptors(), ensure_ascii=False, indent=2))
        return 0
    if args.tool:
        returncode, payload = invoke_tool(args.tool, args.payload_json)
        print(json.dumps(payload, ensure_ascii=False, indent=2))
        return returncode

    result = build_entrypoint_error_result(
        error="실행할 동작이 지정되지 않았다.",
        error_code="missing_server_action",
        warnings=["`--list-tools` 또는 `--tool <name> --payload-json '{...}'` 중 하나가 필요하다."],
        source_context={"action": "none", "tool": None, "payload_json": None},
    )
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
