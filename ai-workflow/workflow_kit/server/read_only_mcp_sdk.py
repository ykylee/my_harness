#!/usr/bin/env python3
"""Optional official MCP Python SDK stdio server candidate for the read-only bundle."""

from __future__ import annotations

import argparse
import asyncio
import importlib
import json
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[3]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit import __version__ as TOOL_VERSION
from workflow_kit.server.read_only_entrypoint import invoke_tool
from workflow_kit.server.read_only_registry import READ_ONLY_SERVER_NAME, build_transport_tool_descriptors


SDK_IMPORT_TARGETS = (
    "mcp.types",
    "mcp.server.stdio",
    "mcp.server.lowlevel",
    "mcp.server.models",
)


@dataclass(frozen=True)
class OfficialSdkModules:
    types: Any
    stdio: Any
    lowlevel: Any
    models: Any


def _import_sdk_modules() -> OfficialSdkModules:
    return OfficialSdkModules(
        types=importlib.import_module("mcp.types"),
        stdio=importlib.import_module("mcp.server.stdio"),
        lowlevel=importlib.import_module("mcp.server.lowlevel"),
        models=importlib.import_module("mcp.server.models"),
    )


def sdk_runtime_status() -> dict[str, object]:
    missing_modules: list[str] = []
    resolved_modules: dict[str, str] = {}
    for module_name in SDK_IMPORT_TARGETS:
        try:
            module = importlib.import_module(module_name)
        except ModuleNotFoundError:
            missing_modules.append(module_name)
            continue
        module_path = getattr(module, "__file__", None)
        resolved_modules[module_name] = module_path or "<namespace>"

    descriptors = build_transport_tool_descriptors()
    return {
        "status": "ok",
        "server_name": READ_ONLY_SERVER_NAME,
        "tool_version": TOOL_VERSION,
        "transport_ready": False,
        "sdk_candidate_phase": "official_sdk_optional_candidate",
        "sdk_available": not missing_modules,
        "sdk_import_targets": list(SDK_IMPORT_TARGETS),
        "missing_modules": missing_modules,
        "resolved_modules": resolved_modules,
        "tool_count": descriptors["tool_count"],
        "descriptor_target": descriptors["descriptor_target"],
        "candidate_module": "workflow_kit.server.read_only_mcp_sdk",
    }


def _text_content_from_payload(sdk_types: Any, name: str, payload: dict[str, Any]) -> Any:
    text_representation: str
    if name == "smart_context_reader" and "extracted_content" in payload:
        text_representation = "\n\n".join(payload["extracted_content"])
    else:
        text_representation = json.dumps(payload, ensure_ascii=False, sort_keys=True)
    return sdk_types.TextContent(type="text", text=text_representation)


def _call_tool_result_for_payload(sdk_types: Any, name: str, payload: dict[str, Any]) -> Any:
    return sdk_types.CallToolResult(
        content=[_text_content_from_payload(sdk_types, name, payload)],
        structuredContent=payload,
        isError=payload.get("status") == "error",
        _meta={
            "transport_ready": False,
            "sdk_candidate_phase": "official_sdk_optional_candidate",
            "tool": name,
        },
    )


def build_lowlevel_server() -> Any:
    sdk = _import_sdk_modules()
    server = sdk.lowlevel.Server(READ_ONLY_SERVER_NAME)
    descriptors = build_transport_tool_descriptors()

    @server.list_tools()
    async def list_tools() -> list[Any]:
        return [
            sdk.types.Tool(
                name=descriptor["name"],
                description=descriptor["description"],
                inputSchema=descriptor["inputSchema"],
                outputSchema=descriptor["outputSchema"],
                annotations=descriptor["annotations"],
            )
            for descriptor in descriptors["tools"]
        ]

    @server.call_tool(validate_input=False)
    async def call_tool(name: str, arguments: dict[str, Any]) -> Any:
        returncode, payload = invoke_tool(name, json.dumps(arguments, ensure_ascii=False))
        result = _call_tool_result_for_payload(sdk.types, name, payload)
        if returncode != 0:
            result.isError = True
        return result

    return server


async def run_stdio_server() -> None:
    sdk = _import_sdk_modules()
    server = build_lowlevel_server()
    async with sdk.stdio.stdio_server() as (read_stream, write_stream):
        await server.run(
            read_stream,
            write_stream,
            sdk.models.InitializationOptions(
                server_name=READ_ONLY_SERVER_NAME,
                server_version=TOOL_VERSION,
                capabilities=server.get_capabilities(
                    notification_options=sdk.lowlevel.NotificationOptions(),
                    experimental_capabilities={},
                ),
            ),
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Optional official MCP Python SDK stdio server candidate.")
    parser.add_argument(
        "--print-sdk-runtime",
        action="store_true",
        help="Print SDK availability/runtime metadata as JSON.",
    )
    parser.add_argument(
        "--stdio-sdk",
        action="store_true",
        help="Run the official MCP Python SDK stdio server when the SDK is installed.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.print_sdk_runtime:
        print(json.dumps(sdk_runtime_status(), ensure_ascii=False, indent=2, sort_keys=True))
        return 0
    if args.stdio_sdk:
        status = sdk_runtime_status()
        if not status["sdk_available"]:
            print(
                json.dumps(
                    {
                        "status": "error",
                        "error_code": "missing_official_mcp_sdk",
                        "error": "Official MCP Python SDK is not installed in this environment.",
                        "source_context": status,
                    },
                    ensure_ascii=False,
                    indent=2,
                    sort_keys=True,
                )
            )
            return 1
        asyncio.run(run_stdio_server())
        return 0

    print(
        json.dumps(
            {
                "status": "error",
                "error_code": "missing_sdk_action",
                "error": "--print-sdk-runtime or --stdio-sdk is required",
            },
            ensure_ascii=False,
            indent=2,
            sort_keys=True,
        )
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
