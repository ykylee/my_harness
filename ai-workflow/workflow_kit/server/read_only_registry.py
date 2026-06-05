"""Registry for the first read-only MCP server bundle."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from workflow_kit import __version__ as TOOL_VERSION
from workflow_kit.common.output_contracts import (
    ERROR_PATH_CONTRACTS,
    SUCCESS_PATH_CONTRACTS,
    output_field_shapes_schema,
    output_json_schema_for_family,
)


REPO_ROOT = Path(__file__).resolve().parents[3]
SOURCE_ROOT = REPO_ROOT / "workflow-source"


@dataclass(frozen=True)
class ReadOnlyToolFieldSpec:
    name: str
    cli_flag: str
    value_type: str
    description: str
    required: bool = False
    repeated: bool = False


@dataclass(frozen=True)
class ReadOnlyToolSpec:
    name: str
    description: str
    script_path: Path
    input_fields: tuple[ReadOnlyToolFieldSpec, ...]
    requires_any_of: tuple[str, ...] = ()
    payload_example: dict[str, object] | None = None


READ_ONLY_SERVER_NAME = "workflow_read_only_bundle"
READ_ONLY_TRANSPORT_DESCRIPTOR_TARGET = "mcp_tools_list_draft"

READ_ONLY_TOOL_SPECS: tuple[ReadOnlyToolSpec, ...] = (
    ReadOnlyToolSpec(
        name="latest_backlog",
        description="Locate the latest dated backlog document from an index or backlog directory.",
        script_path=SOURCE_ROOT / "mcp_servers" / "latest-backlog" / "scripts" / "run_latest_backlog.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="backlog_dir_path",
                cli_flag="--backlog-dir-path",
                value_type="path",
                description="Fallback backlog directory to scan for dated markdown files.",
            ),
            ReadOnlyToolFieldSpec(
                name="work_backlog_index_path",
                cli_flag="--work-backlog-index-path",
                value_type="path",
                description="Backlog index markdown file whose links point to dated backlog files.",
            ),
        ),
        requires_any_of=("backlog_dir_path", "work_backlog_index_path"),
        payload_example={"work_backlog_index_path": str(REPO_ROOT / "work_backlog.md")},
    ),
    ReadOnlyToolSpec(
        name="check_doc_metadata",
        description="Inspect markdown files and report missing required metadata fields.",
        script_path=SOURCE_ROOT / "mcp_servers" / "check-doc-metadata" / "scripts" / "run_check_doc_metadata.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="doc_dir_path",
                cli_flag="--doc-dir-path",
                value_type="path",
                description="Root directory whose markdown files will be scanned.",
                required=True,
            ),
        ),
        payload_example={"doc_dir_path": str(REPO_ROOT / "examples" / "acme_delivery_platform")},
    ),
    ReadOnlyToolSpec(
        name="check_doc_links",
        description="Inspect markdown relative links and report broken targets.",
        script_path=SOURCE_ROOT / "mcp_servers" / "check-doc-links" / "scripts" / "run_check_doc_links.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="doc_dir_path",
                cli_flag="--doc-dir-path",
                value_type="path",
                description="Root directory whose markdown files will be scanned for broken links.",
                required=True,
            ),
        ),
        payload_example={"doc_dir_path": str(REPO_ROOT / "examples" / "acme_delivery_platform")},
    ),
    ReadOnlyToolSpec(
        name="suggest_impacted_docs",
        description="Suggest impacted workflow documents from changed files and summary input.",
        script_path=SOURCE_ROOT / "mcp_servers" / "suggest-impacted-docs" / "scripts" / "run_suggest_impacted_docs.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="changed_files",
                cli_flag="--changed-file",
                value_type="string",
                description="Changed file paths that should be classified into impacted document candidates.",
                required=True,
                repeated=True,
            ),
            ReadOnlyToolFieldSpec(
                name="session_handoff_path",
                cli_flag="--session-handoff-path",
                value_type="path",
                description="Optional session handoff document to include as an impacted state document.",
            ),
            ReadOnlyToolFieldSpec(
                name="latest_backlog_path",
                cli_flag="--latest-backlog-path",
                value_type="path",
                description="Optional latest backlog document to include as an impacted state document.",
            ),
            ReadOnlyToolFieldSpec(
                name="work_backlog_index_path",
                cli_flag="--work-backlog-index-path",
                value_type="path",
                description="Optional backlog index document to include as an impacted state document.",
            ),
        ),
        payload_example={
            "changed_files": ["workflow-source/workflow_kit/server/read_only_entrypoint.py", "workflow-source/tests/check_read_only_mcp_server.py"],
            "latest_backlog_path": str(REPO_ROOT / "backlog" / "2026-04-22.md"),
        },
    ),
    ReadOnlyToolSpec(
        name="create_backlog_entry",
        description="Generate a draft backlog entry JSON for a new task.",
        script_path=SOURCE_ROOT / "mcp_servers" / "create-backlog-entry" / "scripts" / "run_create_backlog_entry.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="task_id",
                cli_flag="--task-id",
                value_type="string",
                description="Unique identifier for the task (e.g., TASK-001).",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="task_name",
                cli_flag="--task-name",
                value_type="string",
                description="Short descriptive name of the task.",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="request_date",
                cli_flag="--request-date",
                value_type="string",
                description="Date the task was requested (YYYY-MM-DD).",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="status",
                cli_flag="--status",
                value_type="string",
                description="Initial status of the task (default: planned).",
            ),
            ReadOnlyToolFieldSpec(
                name="priority",
                cli_flag="--priority",
                value_type="string",
                description="Priority of the task (default: high).",
            ),
        ),
        payload_example={
            "task_id": "TASK-009",
            "task_name": "MCP Server Promotion",
            "request_date": "2026-04-26",
        },
    ),
    ReadOnlyToolSpec(
        name="create_session_handoff_draft",
        description="Generate a draft session handoff document from the latest backlog.",
        script_path=SOURCE_ROOT / "mcp_servers" / "create-session-handoff-draft" / "scripts" / "run_create_session_handoff_draft.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="latest_backlog_path",
                cli_flag="--latest-backlog-path",
                value_type="path",
                description="Latest dated backlog document to extract task status from.",
            ),
            ReadOnlyToolFieldSpec(
                name="git_summary",
                cli_flag="--git-summary",
                value_type="string",
                description="Optional git summary text to include in the handoff.",
            ),
        ),
        payload_example={
            "latest_backlog_path": str(SOURCE_ROOT / "project" / "backlog" / "2026-04-26.md"),
            "git_summary": "### Git Summary\n- feat: some change",
        },
    ),
    ReadOnlyToolSpec(
        name="create_environment_record_stub",
        description="Generate a draft environment record stub for the current host.",
        script_path=SOURCE_ROOT / "mcp_servers" / "create-environment-record-stub" / "scripts" / "run_create_environment_record_stub.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="hostname",
                cli_flag="--hostname",
                value_type="string",
                description="Current host name.",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="os_type",
                cli_flag="--os-type",
                value_type="string",
                description="Current OS type (e.g., darwin, linux, windows).",
                required=True,
            ),
        ),
        payload_example={
            "hostname": "local-dev",
            "os_type": "darwin",
        },
    ),
    ReadOnlyToolSpec(
        name="check_quickstart_stale_links",
        description="Check quickstart and README entry docs for stale or missing links.",
        script_path=SOURCE_ROOT / "mcp_servers" / "check-quickstart-stale-links" / "scripts" / "run_check_quickstart_stale_links.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="quickstart_paths",
                cli_flag="--quickstart-path",
                value_type="path",
                description="One or more quickstart or README entry documents to inspect.",
                required=True,
                repeated=True,
            ),
            ReadOnlyToolFieldSpec(
                name="project_profile_path",
                cli_flag="--project-profile-path",
                value_type="path",
                description="Optional project profile document expected to be linked from entry docs.",
            ),
            ReadOnlyToolFieldSpec(
                name="session_handoff_path",
                cli_flag="--session-handoff-path",
                value_type="path",
                description="Optional session handoff document expected to be linked from entry docs.",
            ),
            ReadOnlyToolFieldSpec(
                name="work_backlog_index_path",
                cli_flag="--work-backlog-index-path",
                value_type="path",
                description="Optional backlog index document expected to be linked from entry docs.",
            ),
            ReadOnlyToolFieldSpec(
                name="agents_path",
                cli_flag="--agents-path",
                value_type="path",
                description="Optional AGENTS or harness guidance document expected to be linked from entry docs.",
            ),
        ),
        payload_example={
            "quickstart_paths": [str(REPO_ROOT / "README.md")],
            "work_backlog_index_path": str(REPO_ROOT / "work_backlog.md"),
        },
    ),
    ReadOnlyToolSpec(
        name="summarize_git_history",
        description="Summarize git commit history into categories and markdown for handoff.",
        script_path=SOURCE_ROOT / "mcp_servers" / "git-history-summarizer" / "scripts" / "run_git_history_summarizer.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="repo_path",
                cli_flag="--repo-path",
                value_type="path",
                description="Path to the git repository.",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="commit_range",
                cli_flag="--range",
                value_type="string",
                description="Commit range to summarize (e.g., 'HEAD~5..HEAD').",
                required=True,
            ),
        ),
        payload_example={
            "repo_path": ".",
            "commit_range": "HEAD~3..HEAD",
        },
    ),
    ReadOnlyToolSpec(
        name="rotate_workflow_logs",
        description="Rotate old done items from handoff into baseline to prevent bloat.",
        script_path=SOURCE_ROOT / "mcp_servers" / "rotate-workflow-logs" / "scripts" / "run_rotate_workflow_logs.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="handoff_path",
                cli_flag="--handoff-path",
                value_type="path",
                description="Path to the session handoff document.",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="max_done_items",
                cli_flag="--max-done-items",
                value_type="string",
                description="Maximum number of done items to keep in 'recently done' (default: 10).",
            ),
        ),
        payload_example={
            "handoff_path": "ai-workflow/memory/session_handoff.md",
            "max_done_items": "5",
        },
    ),
    ReadOnlyToolSpec(
        name="assess_milestone_progress",
        description="Analyze milestone progress based on maturity matrix and backlog.",
        script_path=SOURCE_ROOT / "mcp_servers" / "milestone-progress" / "scripts" / "run_assess_milestone_progress.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="matrix_path",
                cli_flag="--matrix-path",
                value_type="path",
                description="Path to the maturity matrix JSON.",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="backlog_path",
                cli_flag="--backlog-path",
                value_type="path",
                description="Path to the current backlog document.",
                required=True,
            ),
        ),
        payload_example={
            "matrix_path": "workflow-source/core/maturity_matrix.json",
            "backlog_path": "ai-workflow/memory/codex/phase6/backlog/2026-04-27.md",
        },
    ),
    ReadOnlyToolSpec(
        name="smart_context_reader",
        description="Extract specific function or class blocks from a Python file to reduce LLM context bloat.",
        script_path=SOURCE_ROOT / "mcp_servers" / "smart-context-reader" / "scripts" / "run_smart_reader.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="file_path",
                cli_flag="--file-path",
                value_type="path",
                description="Path to the Python file.",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="symbols",
                cli_flag="--symbols",
                value_type="string",
                description="List of function or class names to extract. If empty, all are extracted.",
                repeated=True,
            ),
        ),
        payload_example={
            "file_path": "src/main.py",
            "symbols": ["calculate_total", "UserContext"],
        },
    ),
    ReadOnlyToolSpec(
        name="apply_robust_patch",
        description="Apply a robust Search-Replace block patch to a file with fuzzy matching and syntax validation.",
        script_path=SOURCE_ROOT / "mcp_servers" / "apply_robust_patch" / "scripts" / "run_apply_robust_patch.py",
        input_fields=(
            ReadOnlyToolFieldSpec(
                name="file_path",
                cli_flag="--file-path",
                value_type="path",
                description="Target file to patch.",
                required=True,
            ),
            ReadOnlyToolFieldSpec(
                name="patch_content",
                cli_flag="--patch-content",
                value_type="string",
                description="The SEARCH/REPLACE block content (using <<<<<<< SEARCH, =======, >>>>>>> REPLACE).",
                required=True,
            ),
        ),
        payload_example={
            "file_path": "src/main.py",
            "patch_content": "<<<<<<< SEARCH\ndef old():\n    pass\n=======\ndef new():\n    print('fixed')\n>>>>>>> REPLACE",
        },
    ),
)


def get_tool_spec(tool_name: str) -> ReadOnlyToolSpec | None:
    for spec in READ_ONLY_TOOL_SPECS:
        if spec.name == tool_name:
            return spec
    return None


def input_json_schema_for_spec(spec: ReadOnlyToolSpec) -> dict[str, object]:
    properties: dict[str, object] = {}
    required: list[str] = []
    for field in spec.input_fields:
        field_schema: dict[str, object]
        if field.repeated:
            field_schema = {
                "type": "array",
                "items": {"type": "string"},
                "description": field.description,
            }
        else:
            field_schema = {
                "type": "string",
                "description": field.description,
            }
        properties[field.name] = field_schema
        if field.required:
            required.append(field.name)

    schema: dict[str, object] = {
        "type": "object",
        "properties": properties,
        "required": sorted(required),
        "additionalProperties": False,
    }
    if spec.requires_any_of:
        schema["anyOf"] = [{"required": [field_name]} for field_name in spec.requires_any_of]
    return schema


def build_transport_tool_descriptor(spec: ReadOnlyToolSpec) -> dict[str, object]:
    return {
        "name": spec.name,
        "description": spec.description,
        "inputSchema": input_json_schema_for_spec(spec),
        "outputSchema": output_json_schema_for_family(spec.name),
        "annotations": {
            "readOnlyHint": True,
        },
        "_meta": {
            "transport_ready": False,
            "bundle_phase": "direct_call_adapter",
            "adapter": "workflow_kit.server.read_only_tools.invoke_read_only_tool",
            "descriptor_target": READ_ONLY_TRANSPORT_DESCRIPTOR_TARGET,
        },
    }


def build_transport_tool_descriptors() -> dict[str, object]:
    descriptors = [build_transport_tool_descriptor(spec) for spec in READ_ONLY_TOOL_SPECS]
    return {
        "status": "ok",
        "tool_version": TOOL_VERSION,
        "server_name": READ_ONLY_SERVER_NAME,
        "descriptor_target": READ_ONLY_TRANSPORT_DESCRIPTOR_TARGET,
        "transport_ready": False,
        "tool_count": len(descriptors),
        "tools": descriptors,
    }


def build_server_manifest() -> dict[str, object]:
    return {
        "status": "ok",
        "tool_version": TOOL_VERSION,
        "server_name": READ_ONLY_SERVER_NAME,
        "tool_count": len(READ_ONLY_TOOL_SPECS),
        "transport": {
            "descriptor_target": READ_ONLY_TRANSPORT_DESCRIPTOR_TARGET,
            "transport_ready": False,
            "descriptor_source": "workflow_kit.server.read_only_registry.build_transport_tool_descriptors",
        },
        "tools": [
            {
                "name": spec.name,
                "description": spec.description,
                "script_path": str(spec.script_path),
                "transport_ready": False,
                "bundle_phase": "direct_call_adapter",
                "input_schema": {
                    "type": "object",
                    "fields": [
                        {
                            "name": field.name,
                            "cli_flag": field.cli_flag,
                            "value_type": field.value_type,
                            "required": field.required,
                            "repeated": field.repeated,
                            "description": field.description,
                        }
                        for field in spec.input_fields
                    ],
                    "requires_any_of": list(spec.requires_any_of),
                },
                "output_schema": {
                    "success_required_keys": sorted(SUCCESS_PATH_CONTRACTS.get(spec.name, frozenset())),
                    "error_required_keys": sorted(ERROR_PATH_CONTRACTS.get(spec.name, frozenset())),
                    "field_shapes": output_field_shapes_schema().get(spec.name, {}),
                    "json_schema_draft": "2020-12",
                    "json_schema_source": "workflow_kit.common.output_contracts.output_json_schema_for_family",
                    "json_schema": output_json_schema_for_family(spec.name),
                },
                "transport_descriptor": build_transport_tool_descriptor(spec),
                "payload_example": spec.payload_example,
            }
            for spec in READ_ONLY_TOOL_SPECS
        ],
    }
