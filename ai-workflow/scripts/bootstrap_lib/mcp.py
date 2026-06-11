"""MCP config renderers for the generated kit.

The bootstrap script can optionally emit per-harness MCP config
snippets (``--enable-mcp``). Each harness has its own config dialect
(TOML for Codex, JSON for the rest), so the renderers live in their
own module and are dispatched through :data:`MCP_CONFIG_RENDERERS`.

Adding a new harness only requires:

1. Writing a ``render_<harness>_mcp_config(args, paths) -> str`` function
2. Adding an entry to :data:`MCP_CONFIG_RENDERERS`
3. Adding a branch in :func:`write_mcp_config_files` that picks the right
   output path
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Callable

from bootstrap_lib.paths import Paths
from bootstrap_lib.writes import write_text


MCP_SERVER_ALIAS = "standardAiWorkflowReadOnly"
MCP_TOOL_NAME = "workflow_kit.read_only"


def _mcp_server_command(bridge: str) -> list[str]:
    """Return the ``command + args`` that the per-harness MCP config should spawn.

    ``command`` is always ``python3`` (the same Python that runs the
    bootstrap script) and ``args`` points at the entry point module so the
    harness can launch it without a shell. ``PYTHONPATH`` is set in
    ``env`` so the entry point can locate ``workflow_kit``.
    """
    if bridge == "stdio-sdk":
        return ["python3", "-m", "workflow_kit.server.read_only_mcp_sdk", "--stdio-sdk"]
    return ["python3", "-m", "workflow_kit.server.read_only_jsonrpc", "--stdio-lines"]


def _mcp_server_env(workspace_root: Path) -> dict[str, str]:
    """Return the per-harness MCP server ``env`` block.

    ``STANDARD_AI_WORKFLOW_ROOT`` lets the server locate the kit root
    even when invoked from a non-default cwd.

    ``PYTHONPATH`` is only set when the bootstrap itself is running
    from a source repo checkout (SOURCE_ROOT is available). In a wheel
    install the server should resolve ``workflow_kit`` from the wheel's
    site-packages, so the bootstrap does NOT preprend
    ``workflow-source`` — that path would otherwise shadow the installed
    wheel and break protocolVersion / smart-update fixes.
    """
    from bootstrap_lib import __main__ as _bm  # local import to avoid cycle

    env = {"STANDARD_AI_WORKFLOW_ROOT": str(workspace_root.resolve())}
    if _bm.SOURCE_ROOT is not None:
        env["PYTHONPATH"] = "workflow-source"
    return env


def render_codex_mcp_config(args: argparse.Namespace, paths: Paths) -> str:
    """Return a Codex ``.codex/config.toml`` snippet that registers the MCP server.

    Codex accepts TOML with ``[mcp_servers.<alias>]`` sections. We keep
    this as a *snippet* (not a full config) so users can paste it into
    their existing ``~/.codex/config.toml`` without losing other entries.
    """
    bridge = getattr(args, "mcp_bridge", "jsonrpc-bridge")
    cmd = _mcp_server_command(bridge)
    env = _mcp_server_env(paths.target_root)
    args_inline = ", ".join(json.dumps(part) for part in cmd[1:])
    env_block = "\n".join(f"{key} = {json.dumps(value)}" for key, value in env.items())
    return f"""# Codex MCP server snippet for the Standard AI Workflow kit.
# Drop this into ~/.codex/config.toml under the [mcp_servers] table, or keep
# the bootstrap-generated .codex/config.toml.example as a starting point.

[mcp_servers.{MCP_SERVER_ALIAS}]
command = "{cmd[0]}"
args = [{args_inline}]
{env_block}
startup_timeout_sec = 15
tool_timeout_sec = 30

# Tool description shown in the Codex tool picker
[mcp_servers.{MCP_SERVER_ALIAS}.tool_descriptions]
{MCP_TOOL_NAME} = "Read-only MCP tools (latest_backlog, check_doc_metadata, ...) for the Standard AI Workflow kit."
"""


def render_opencode_mcp_config(args: argparse.Namespace, paths: Paths) -> str:
    """Return an OpenCode MCP config block to embed in ``opencode.json``.

    OpenCode expects ``"mcp": { "<name>": { ... } }`` at the top level. The
    bootstrap writes a standalone ``mcp.opencode.json`` that can be merged
    or symlinked into the project ``opencode.json``.
    """
    bridge = getattr(args, "mcp_bridge", "jsonrpc-bridge")
    return json.dumps(
        {
            "mcp": {
                MCP_SERVER_ALIAS: {
                    "type": "local",
                    "command": _mcp_server_command(bridge)[0],
                    "args": _mcp_server_command(bridge)[1:],
                    "env": _mcp_server_env(paths.target_root),
                    "timeout": 30000,
                }
            }
        },
        ensure_ascii=False,
        indent=2,
    ) + "\n"


def render_gemini_cli_mcp_config(args: argparse.Namespace, paths: Paths) -> str:
    """Return a Gemini CLI ``.gemini/settings.json`` snippet."""
    bridge = getattr(args, "mcp_bridge", "jsonrpc-bridge")
    return json.dumps(
        {
            "mcpServers": {
                MCP_SERVER_ALIAS: {
                    "command": _mcp_server_command(bridge)[0],
                    "args": _mcp_server_command(bridge)[1:],
                    "env": _mcp_server_env(paths.target_root),
                    "trust": True,
                    "includeTools": [
                        "latest_backlog",
                        "check_doc_metadata",
                        "check_doc_links",
                        "suggest_impacted_docs",
                    ],
                }
            }
        },
        ensure_ascii=False,
        indent=2,
    ) + "\n"


def render_antigravity_mcp_config(args: argparse.Namespace, paths: Paths) -> str:
    """Return an Antigravity MCP snippet.

    Antigravity shares the JSON ``mcpServers`` shape with Gemini CLI. The
    bootstrap writes the file as ``.antigravity/mcp.json`` (project-local),
    following the same dot-dir convention as ``.codex/``, ``.gemini/``,
    and ``.MiniMax/``.
    """
    bridge = getattr(args, "mcp_bridge", "jsonrpc-bridge")
    return json.dumps(
        {
            "mcpServers": {
                MCP_SERVER_ALIAS: {
                    "type": "stdio",
                    "command": _mcp_server_command(bridge)[0],
                    "args": _mcp_server_command(bridge)[1:],
                    "env": _mcp_server_env(paths.target_root),
                }
            }
        },
        ensure_ascii=False,
        indent=2,
    ) + "\n"


def render_minimax_code_mcp_config(args: argparse.Namespace, paths: Paths) -> str:
    """Return a MiniMax Code ``.MiniMax/mcp.json`` config.

    The bootstrap writes the file as ``.MiniMax/mcp.json`` (project-local).
    Users can symlink it into their ``~/.MiniMax/mcp.json`` or copy the
    ``mcp_servers`` block into the global config.
    """
    bridge = getattr(args, "mcp_bridge", "jsonrpc-bridge")
    descriptor = {
        MCP_SERVER_ALIAS: {
            "command": _mcp_server_command(bridge)[0],
            "args": _mcp_server_command(bridge)[1:],
            "env": _mcp_server_env(paths.target_root),
            "transport_ready": False,
            "transport": bridge,
            "description": (
                "Read-only MCP tools for the Standard AI Workflow kit. "
                "Draft JSON-RPC bridge by default; switch to stdio-sdk once "
                "check_read_only_mcp_sdk_stdio.py is green."
            ),
            "tools": [
                "latest_backlog",
                "check_doc_metadata",
                "check_doc_links",
                "suggest_impacted_docs",
                "create_backlog_entry",
                "create_session_handoff_draft",
                "create_environment_record_stub",
                "check_quickstart_stale_links",
                "summarize_git_history",
                "smart_context_reader",
            ],
        }
    }
    return json.dumps({"mcp_servers": descriptor}, ensure_ascii=False, indent=2) + "\n"


def write_mcp_config_files(
    args: argparse.Namespace,
    paths: Paths,
    harnesses: list[str],
) -> dict[str, str]:
    """Emit per-harness MCP config snippets when ``--enable-mcp`` is set."""
    generated: dict[str, str] = {}

    if "codex" in harnesses or "opencode" in harnesses:
        codex_path = paths.target_root / ".codex" / "mcp.toml"
        write_text(codex_path, render_codex_mcp_config(args, paths), force=args.force, rel_to=paths.target_root)
        generated["codex_mcp_config"] = str(codex_path)

    if "opencode" in harnesses:
        opencode_path = paths.target_root / "mcp.opencode.json"
        write_text(opencode_path, render_opencode_mcp_config(args, paths), force=args.force, rel_to=paths.target_root)
        generated["opencode_mcp_config"] = str(opencode_path)

    if "gemini-cli" in harnesses:
        gemini_path = paths.target_root / ".gemini" / "mcp.json"
        write_text(gemini_path, render_gemini_cli_mcp_config(args, paths), force=args.force, rel_to=paths.target_root)
        generated["gemini_cli_mcp_config"] = str(gemini_path)

    if "antigravity" in harnesses:
        antigravity_path = paths.target_root / ".antigravity" / "mcp.json"
        write_text(antigravity_path, render_antigravity_mcp_config(args, paths), force=args.force, rel_to=paths.target_root)
        generated["antigravity_mcp_config"] = str(antigravity_path)

    if "minimax-code" in harnesses:
        minimax_path = paths.target_root / ".MiniMax" / "mcp.json"
        write_text(minimax_path, render_minimax_code_mcp_config(args, paths), force=args.force, rel_to=paths.target_root)
        generated["minimax_code_mcp_config"] = str(minimax_path)

    return generated


#: Dispatch table from harness name to its MCP config renderer.
#: Adding a new harness only requires an entry here and a branch in
#: :func:`write_mcp_config_files`.
MCP_CONFIG_RENDERERS: dict[str, Callable[[argparse.Namespace, Paths], str]] = {
    "codex": render_codex_mcp_config,
    "opencode": render_opencode_mcp_config,
    "gemini-cli": render_gemini_cli_mcp_config,
    "antigravity": render_antigravity_mcp_config,
    "minimax-code": render_minimax_code_mcp_config,
}


__all__ = [
    "MCP_CONFIG_RENDERERS",
    "MCP_SERVER_ALIAS",
    "MCP_TOOL_NAME",
    "render_antigravity_mcp_config",
    "render_codex_mcp_config",
    "render_gemini_cli_mcp_config",
    "render_minimax_code_mcp_config",
    "render_opencode_mcp_config",
    "write_mcp_config_files",
]
