"""Bootstrap CLI entry point.

This module is the ``python -m bootstrap_lib`` entry point. It composes
the smaller modules in :mod:`bootstrap_lib` and the harness renderers
from :mod:`bootstrap_lib.harnesses.renderers` into the full bootstrap
flow:

1. Parse CLI args (:func:`parse_args`)
2. Resolve well-known output paths (:func:`make_paths`)
3. Infer the project context from the target repo (:func:`infer_project_context`)
4. Render every doc artefact (README, profile, handoff, backlog, ...)
5. Write harness overlay files for each selected harness
6. Optionally write per-harness MCP config snippets
7. Optionally update ``requirements.txt`` / ``package.json``
8. Emit a JSON manifest describing everything that was generated
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from datetime import date
from pathlib import Path

# ---------------------------------------------------------------------------
# Path setup (must run before importing workflow_kit or bootstrap_lib submodules)
# ---------------------------------------------------------------------------
# This file lives at workflow-source/scripts/bootstrap_lib/__main__.py in the
# source repo, and at site-packages/bootstrap_lib/__main__.py when installed
# via pip. We resolve SOURCE_ROOT by walking up from this file's parent
# until we find a directory containing core/global_workflow_standard.md; if
# none is found, SOURCE_ROOT is None and the bootstrap gracefully skips
# core-docs copy (wheel install without bundled data).
_THIS_FILE = Path(__file__).resolve()
_BOOTSTRAP_LIB_PARENT = _THIS_FILE.parent
SOURCE_ROOT: Path | None = None
for _candidate in _BOOTSTRAP_LIB_PARENT.parents:
    if (_candidate / "core" / "global_workflow_standard.md").exists():
        SOURCE_ROOT = _candidate
        break
REPO_ROOT = SOURCE_ROOT.parent if SOURCE_ROOT is not None else None
SCRIPT_DIR = SOURCE_ROOT / "scripts" if SOURCE_ROOT is not None else None
if SOURCE_ROOT is not None and str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

# Test/operational override: when set, skip SOURCE_ROOT auto-detection
# and treat as a wheel install (core docs / global snippets unavailable).
if os.environ.get("BOOTSTRAP_LIB_NO_SOURCE"):
    SOURCE_ROOT = None
    REPO_ROOT = None
    SCRIPT_DIR = None
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from workflow_kit.common.workflow_state import build_workflow_state_payload  # noqa: E402

from bootstrap_lib.discovery import (  # noqa: E402
    COMMON_NODE_DEPS,
    COMMON_PYTHON_DEPS,
    HARNESS_NODE_DEPS,
    HARNESS_PYTHON_DEPS,
    IGNORED_DIRS,
    detect_package_scripts,
    global_snippet_sources as _global_snippet_sources,
    guess_run_command,
    iter_repo_files,
    value_or_inferred,
)
from bootstrap_lib.harnesses import (  # noqa: E402
    HARNESS_FILE_BUILDERS,
    HARNESS_SPECS,
    SUPPORTED_HARNESSES,
    register_harness_builder,
    spec_for,
)
from bootstrap_lib.harnesses.renderers import (  # noqa: E402
    render_antigravity_agents,
    render_codex_agents,
    render_codex_config_example,
    render_gemini_cli_agents,
    render_minimax_code_worker,
    render_minimax_config_example,
    render_minimax_doc_worker,
    render_minimax_orchestrator,
    render_minimax_validation_worker,
    render_minimax_worker,
    render_minimax_agents,
    render_opencode_agent,
    render_opencode_code_worker_agent,
    render_opencode_config,
    render_opencode_doc_worker_agent,
    render_opencode_skill,
    render_opencode_validation_worker_agent,
    render_opencode_worker_agent,
    write_antigravity_harness_files,
    write_codex_harness_files,
    write_gemini_cli_harness_files,
    write_minimax_code_harness_files,
    write_opencode_harness_files,
    write_pi_dev_harness_files,
)
from bootstrap_lib.mcp import (  # noqa: E402
    write_mcp_config_files,
)
from bootstrap_lib.paths import (  # noqa: E402
    HarnessDefinition,
    Paths,
    antigravity_agents_path,
    codex_agents_path,
    gemini_cli_agents_path,
    make_paths,
    minimax_agents_path,
)
from bootstrap_lib.renderers import (  # noqa: E402
    load_template,
    render_assessment,
    render_backlog_index,
    render_daily_backlog,
    render_project_profile,
    render_project_status_assessment,
    render_readme,
    render_session_handoff,
)
from bootstrap_lib.writes import (  # noqa: E402
    build_manifest,
    copy_core_docs,
    drain_file_actions,
    rel,
    write_text,
)


# ---------------------------------------------------------------------------
# Module-level constants
# ---------------------------------------------------------------------------
DEFAULT_CORE_DOCS: list[str] = [
    "global_workflow_standard.md",
    "workflow_skill_catalog.md",
    "workflow_mcp_candidate_catalog.md",
    "workflow_agent_topology.md",
    "output_schema_guide.md",
    "workflow_adoption_entrypoints.md",
    "workflow_harness_distribution.md",
]

DEFAULT_CORE_SUPPORT_PATHS: list[str] = [
    "core/existing_project_onboarding_contract.md",
    "core/project_status_assessment.md",
    "core/prototype_promotion_scope.md",
    "core/read_only_mcp_transport_promotion.md",
    "core/workflow_release_spec.md",
    "core/workflow_configuration_layers.md",
    "core/workflow_global_injection_policy.md",
    "core/workflow_kit_roadmap.md",
    "core/session_start_skill_spec.md",
    "core/backlog_update_skill_spec.md",
    "core/doc_sync_skill_spec.md",
    "core/merge_doc_reconcile_skill_spec.md",
    "core/validation_plan_skill_spec.md",
    "core/code_index_update_skill_spec.md",
    "templates/project_workflow_profile_template.md",
    "templates/session_handoff_template.md",
    "templates/work_backlog_template.md",
    "templates/daily_backlog_template.md",
    "templates/release_note_template.md",
    "templates/pilot_candidate_checklist.md",
    "templates/pilot_adoption_record_template.md",
    "schemas/output_sample_contracts.json",
    "schemas/generated_output_schemas.json",
    "examples/output_samples",
    "examples/README.md",
    "examples/end_to_end_skill_demo.md",
    "examples/end_to_end_mcp_demo.md",
    "examples/bootstrap_output_samples.md",
    "examples/pilot_adoption_open_git_client_example.md",
    "skills/README.md",
    "skills/prototype_layout.md",
    "skills/session-start/SKILL.md",
    "skills/session-start/scripts/run_session_start.py",
    "skills/backlog-update/SKILL.md",
    "skills/backlog-update/scripts/run_backlog_update.py",
    "skills/doc-sync/SKILL.md",
    "skills/doc-sync/scripts/run_doc_sync.py",
    "skills/merge-doc-reconcile/SKILL.md",
    "skills/merge-doc-reconcile/scripts/run_merge_doc_reconcile.py",
    "skills/validation-plan/SKILL.md",
    "skills/validation-plan/scripts/run_validation_plan.py",
    "skills/code-index-update/SKILL.md",
    "skills/code-index-update/scripts/run_code_index_update.py",
    "mcp_servers/README.md",
    "mcp_servers/prototype_layout.md",
    "mcp_servers/read_only_bundle.md",
    "mcp_servers/latest-backlog/MCP.md",
    "mcp_servers/latest-backlog/scripts/run_latest_backlog.py",
    "mcp_servers/check-doc-metadata/MCP.md",
    "mcp_servers/check-doc-metadata/scripts/run_check_doc_metadata.py",
    "mcp_servers/check-doc-links/MCP.md",
    "mcp_servers/check-doc-links/scripts/run_check_doc_links.py",
    "mcp_servers/create-backlog-entry/MCP.md",
    "mcp_servers/create-backlog-entry/scripts/run_create_backlog_entry.py",
    "mcp_servers/suggest-impacted-docs/MCP.md",
    "mcp_servers/suggest-impacted-docs/scripts/run_suggest_impacted_docs.py",
    "mcp_servers/check-quickstart-stale-links/MCP.md",
    "mcp_servers/check-quickstart-stale-links/scripts/run_check_quickstart_stale_links.py",
    "tests/README.md",
    "tests/check_docs.py",
    "tests/check_bootstrap.py",
    "tests/check_validation_plan.py",
    "tests/check_code_index_update.py",
    "tests/check_existing_project_onboarding.py",
    "tests/check_quickstart_stale_links.py",
    "harnesses/_template/README.md",
    "harnesses/README.md",
    "harnesses/codex/apply_guide.md",
    "harnesses/opencode/apply_guide.md",
    "global-snippets/README.md",
    "scripts/README.md",
    "scripts/apply_harness_update.py",
    "scripts/apply_workflow_upgrade.py",
    "scripts/bootstrap_workflow_kit.py",
    "scripts/export_harness_package.py",
    "scripts/generate_workflow_state.py",
    "scripts/scaffold_harness.py",
    "scripts/run_demo_workflow.py",
    "scripts/run_existing_project_onboarding.py",
    "workflow_kit",
]

#: Deprecated alias kept for backwards compatibility.
#:
#: The single source of truth for harness metadata is
#: :data:`bootstrap_lib.harnesses.HARNESS_SPECS`. This dict only mirrors the
#: subset of fields that older consumers depended on (``name`` + ``description``)
#: and is currently missing the ``pi-dev`` overlay that ``HARNESS_SPECS``
#: already declares. New code should import ``HARNESS_SPECS`` from
#: :mod:`bootstrap_lib.harnesses` directly; this dict will be removed in a
#: future release (target: 0.6.x). See ``workflow_harness_distribution.md``
#: for the migration note.
HARNESS_DEFINITIONS: dict[str, HarnessDefinition] = {
    "codex": HarnessDefinition(
        name="codex",
        description="Generate AGENTS.md and a Codex config example.",
    ),
    "opencode": HarnessDefinition(
        name="opencode",
        description="Generate opencode.json and project-local OpenCode overlays.",
    ),
    "gemini-cli": HarnessDefinition(
        name="gemini-cli",
        description="Generate GEMINI.md for the Gemini CLI.",
    ),
    "antigravity": HarnessDefinition(
        name="antigravity",
        description="Generate ANTIGRAVITY.md for the Antigravity agent.",
    ),
    "minimax-code": HarnessDefinition(
        name="minimax-code",
        description="Generate AGENTS.md + MiniMax.md + minimax_config_example.json + orchestrator/worker split.",
    ),
}


# ---------------------------------------------------------------------------
# CLI argument parsing
# ---------------------------------------------------------------------------
def parse_args() -> argparse.Namespace:
    today = date.today().isoformat()
    parser = argparse.ArgumentParser(
        description="Scaffold a standard AI workflow kit into a target repository."
    )
    parser.add_argument("--target-root", default=".")
    parser.add_argument("--kit-dir", default="ai-workflow")
    parser.add_argument(
        "--adoption-mode",
        choices=["new", "existing"],
        default="new",
        help="Choose whether the target is a new project or an existing codebase.",
    )
    parser.add_argument("--project-slug", required=True)
    parser.add_argument("--project-name", required=True)
    parser.add_argument(
        "--project-purpose",
        default="TODO: 프로젝트 목적과 핵심 사용자 가치를 한두 문장으로 정리한다.",
    )
    parser.add_argument(
        "--stakeholders",
        default="TODO: 주요 이해관계자 목록을 정리한다.",
    )
    parser.add_argument("--doc-home", default="README.md")
    parser.add_argument("--operations-dir", default="ai-workflow/memory/")
    parser.add_argument("--backlog-dir", default="ai-workflow/memory/backlog/")
    parser.add_argument("--session-doc-path", default="ai-workflow/memory/session_handoff.md")
    parser.add_argument("--environment-dir", default="ai-workflow/memory/environments/")
    parser.add_argument("--install-command", default=None)
    parser.add_argument("--run-command", default=None)
    parser.add_argument("--quick-test-command", default=None)
    parser.add_argument("--isolated-test-command", default=None)
    parser.add_argument("--smoke-check-command", default=None)
    parser.add_argument("--today", default=today)
    parser.add_argument("--initial-task-id", default="TASK-001")
    parser.add_argument("--initial-task-name", default="표준 AI 워크플로우 초기 도입")
    parser.add_argument(
        "--initial-task-status",
        choices=["planned", "in_progress", "blocked", "done"],
        default="planned",
    )
    parser.add_argument("--initial-priority", default="high")
    parser.add_argument("--owner", default="TODO")
    parser.add_argument("--host-name", default="TODO")
    parser.add_argument("--host-ip", default="TODO")
    parser.add_argument(
        "--harness",
        action="append",
        choices=list(SUPPORTED_HARNESSES),
        dest="harnesses",
        default=[],
        help="Generate harness-specific overlay files for the selected target. "
        "When omitted in a TTY session, an interactive picker is shown; in "
        "non-TTY sessions (CI, scripts), pass --no-interactive to receive a "
        "clear error instead.",
    )
    parser.add_argument(
        "--no-interactive",
        action="store_true",
        dest="no_interactive",
        help="Disable the interactive harness picker. Use this in CI, scripts, "
        "or any non-TTY context to fail fast if --harness is missing.",
    )
    parser.add_argument(
        "--copy-core-docs",
        action="store_true",
        help="Copy selected core docs into the generated kit directory.",
    )
    parser.add_argument(
        "--update-deps",
        action="store_true",
        help="Update or create requirements.txt with recommended dependencies.",
    )
    parser.add_argument(
        "--enable-mcp",
        action="store_true",
        help=(
            "Emit a per-harness MCP server config snippet alongside the harness overlay. "
            "Use with --mcp-bridge to pick the transport (defaults to jsonrpc-bridge)."
        ),
    )
    parser.add_argument(
        "--mcp-bridge",
        choices=["jsonrpc-bridge", "stdio-sdk"],
        default="jsonrpc-bridge",
        help=(
            "Which MCP transport to wire into the per-harness config. "
            "'jsonrpc-bridge' is the draft fixture (always available). "
            "'stdio-sdk' uses the official mcp Python SDK stdio server "
            "(requires `mcp[cli]>=1.0` and known-good SDK API compatibility)."
        ),
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing generated files when the destination already exists.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the generation plan without writing files.",
    )
    args = parser.parse_args()

    # Interactive harness picker: fire only when (a) the user didn't pass
    # --harness, (b) --no-interactive wasn't requested, and (c) stdin is a
    # TTY. In every other case (CI, scripts, explicit --no-interactive) we
    # either keep an empty selection (for callers that handle it) or fail
    # fast with a clear message (see enforce_harness_selection).
    if (
        not args.harnesses
        and not args.no_interactive
        and sys.stdin.isatty()
    ):
        args.harnesses = prompt_for_harnesses()

    return args


def enforce_harness_selection(args: argparse.Namespace) -> None:
    """Fail fast if no harness is selected and we can't prompt the user.

    Called by :func:`main` after argparse, so the picker has already had a
    chance to populate ``args.harnesses``. If we get here with an empty list
    we know we are in non-TTY / --no-interactive mode and should refuse to
    silently generate zero overlay files.
    """
    if args.harnesses:
        return
    raise SystemExit(
        "ERROR: --harness is required when --no-interactive is set or when "
        "stdin is not a TTY. Re-run with one or more of: "
        + ", ".join(SUPPORTED_HARNESSES)
        + " (comma-separated via repeated --harness flags)."
    )


def selected_harnesses(args: argparse.Namespace) -> list[str]:
    return sorted(dict.fromkeys(args.harnesses))


def prompt_for_harnesses(
    selected: list[str] | None = None,
    *,
    input_stream=None,
    output_stream=None,
) -> list[str]:
    """Interactively ask the user to pick one or more harness overlays.

    The menu is built from :data:`HARNESS_SPECS` (single source of truth for
    harness metadata) and shown via plain ``input()`` so no third-party
    dependency is required. Empty input keeps the current ``selected`` list
    (defaulting to an empty list on first call). ``a`` selects every harness,
    ``q`` aborts the picker and returns the existing ``selected`` value.
    """
    from bootstrap_lib.harnesses import HARNESS_SPECS  # local import: keep top-level lean

    # Late-bind default streams so monkey-patching sys.stdin/stdout (e.g. in
    # tests) is observed at call time, not at import time.
    if input_stream is None:
        input_stream = sys.stdin
    if output_stream is None:
        output_stream = sys.stdout

    catalog = list(HARNESS_SPECS.items())
    base = sorted(dict.fromkeys(selected or []))
    chosen: set[str] = set(base)
    print(
        "Select harness overlays to generate.",
        file=output_stream,
    )
    print(
        "Enter numbers separated by commas (e.g. 1,3), 'a' for all, 'q' to "
        "keep current selection, or just Enter to keep current selection.",
        file=output_stream,
    )
    for idx, (name, spec) in enumerate(catalog, start=1):
        marker = "*" if name in chosen else " "
        print(
            f"  [{marker}] {idx}. {name} — {spec.description}",
            file=output_stream,
        )
    if chosen:
        print(
            f"Current selection: {', '.join(sorted(chosen))}",
            file=output_stream,
        )

    raw = input_stream.readline()
    if not raw:
        return sorted(chosen)
    answer = raw.strip().lower()
    if not answer or answer == "q":
        return sorted(chosen)
    if answer == "a":
        chosen = {name for name, _ in catalog}
        return sorted(chosen)

    parsed: set[str] = set()
    for token in answer.split(","):
        token = token.strip()
        if not token:
            continue
        if not token.isdigit():
            print(
                f"  ! ignoring non-numeric token: {token!r}",
                file=output_stream,
            )
            continue
        idx = int(token)
        if 1 <= idx <= len(catalog):
            parsed.add(catalog[idx - 1][0])
        else:
            print(
                f"  ! ignoring out-of-range index: {idx}",
                file=output_stream,
            )
    chosen.update(parsed)
    return sorted(chosen)


# ---------------------------------------------------------------------------
# Dependency management
# ---------------------------------------------------------------------------
def update_dependencies(
    paths: Paths,
    context: dict[str, object],
    harnesses: list[str],
) -> list[str]:
    updated_files: list[str] = []
    primary_stack = context.get("primary_stack", "unknown")
    target_root = paths.target_root

    # 1. Handle Python dependencies (requirements.txt)
    req_file = target_root / "requirements.txt"
    is_python = primary_stack == "python" or primary_stack == "unspecified"

    if is_python or req_file.exists():
        needed_python = list(COMMON_PYTHON_DEPS)
        for h in harnesses:
            needed_python.extend(HARNESS_PYTHON_DEPS.get(h, []))

        needed_python = sorted(list(set(needed_python)))
        existing_content = ""
        if req_file.exists():
            existing_content = req_file.read_text(encoding="utf-8")

        to_add = []
        for dep in needed_python:
            pattern = rf"(^|\s|[,]){re.escape(dep)}([>=<#\s]|$)"
            if not re.search(pattern, existing_content, re.MULTILINE | re.IGNORECASE):
                to_add.append(dep)

        if to_add:
            new_lines = []
            if existing_content and not existing_content.endswith("\n"):
                new_lines.append("\n")

            if "# Standard AI Workflow Dependencies" not in existing_content:
                new_lines.append("# Standard AI Workflow Dependencies\n")

            for dep in to_add:
                new_lines.append(f"{dep}\n")

            with open(req_file, "a", encoding="utf-8") as f:
                f.writelines(new_lines)
            updated_files.append(rel(req_file, target_root))

    # 2. Handle Node.js dependencies (package.json)
    package_json = target_root / "package.json"
    if package_json.exists():
        try:
            payload = json.loads(package_json.read_text(encoding="utf-8"))
            dev_deps = payload.get("devDependencies", {})
            deps = payload.get("dependencies", {})

            needed_node = list(COMMON_NODE_DEPS)
            for h in harnesses:
                needed_node.extend(HARNESS_NODE_DEPS.get(h, []))

            to_add_node = [
                dep
                for dep in needed_node
                if dep not in dev_deps and dep not in deps
            ]

            if to_add_node:
                if "devDependencies" not in payload:
                    payload["devDependencies"] = {}
                for dep in to_add_node:
                    payload["devDependencies"][dep] = "latest"

                payload["devDependencies"] = dict(
                    sorted(payload["devDependencies"].items())
                )
                package_json.write_text(
                    json.dumps(payload, indent=2, ensure_ascii=False) + "\n",
                    encoding="utf-8",
                )
                updated_files.append(rel(package_json, target_root))
        except (json.JSONDecodeError, KeyError):
            context["analysis_summary"].append(
                "Warning: Could not parse package.json for dependency updates."
            )

    # 3. Handle pyproject.toml / other managers with hints
    if (target_root / "pyproject.toml").exists():
        needed_python = sorted(
            list(
                set(
                    COMMON_PYTHON_DEPS
                    + [d for h in harnesses for d in HARNESS_PYTHON_DEPS.get(h, [])]
                )
            )
        )
        context["analysis_summary"].append(
            f"Note: `pyproject.toml` detected. Recommended: `uv add {' '.join(needed_python)}` or equivalent."
        )

    return updated_files


# ---------------------------------------------------------------------------
# Project context inference
# ---------------------------------------------------------------------------
def infer_project_context(args: argparse.Namespace, paths: Paths) -> dict[str, object]:
    """Scan the target repo and return a dict of inferred values for the renderers."""
    docs_dirs = sorted(
        {
            name
            for name in ("docs", "doc", "wiki", "handbook")
            if (paths.target_root / name).exists()
        }
    )
    test_dirs = sorted(
        {
            name
            for name in ("tests", "test", "spec", "__tests__")
            if (paths.target_root / name).exists()
        }
    )
    source_dirs = sorted(
        {
            name
            for name in ("src", "app", "apps", "services", "packages", "lib")
            if (paths.target_root / name).exists()
        }
    )

    stack_labels: list[str] = []
    search_paths = [paths.target_root] + [
        p
        for p in paths.target_root.iterdir()
        if p.is_dir() and p.name not in IGNORED_DIRS
    ]

    for p in search_paths:
        if (p / "package.json").exists():
            stack_labels.append("node")
        if (
            (p / "pyproject.toml").exists()
            or (p / "requirements.txt").exists()
            or (p / "setup.py").exists()
        ):
            stack_labels.append("python")
        if (p / "Cargo.toml").exists():
            stack_labels.append("rust")
        if (p / "go.mod").exists():
            stack_labels.append("go")
        if (p / "Gemfile").exists():
            stack_labels.append("ruby")
        if (p / "CMakeLists.txt").exists() or (p / "Makefile").exists():
            stack_labels.append("cpp")

    stack_labels = sorted(list(set(stack_labels)))
    primary_stack = stack_labels[0] if stack_labels else "unknown"

    package_scripts = detect_package_scripts(paths.target_root)

    # Resolve doc_home
    if args.doc_home and args.doc_home != "README.md":
        doc_home = args.doc_home
    elif docs_dirs and (paths.target_root / docs_dirs[0] / "README.md").exists():
        doc_home = f"{docs_dirs[0]}/README.md"
    else:
        doc_home = "README.md"

    operations_dir = args.operations_dir
    backlog_dir = f"{operations_dir.rstrip('/')}/backlog/"
    session_doc_path = f"{operations_dir.rstrip('/')}/session_handoff.md"
    environment_dir = f"{operations_dir.rstrip('/')}/environments/"

    # Initial commands from args
    install_command = args.install_command
    run_command = args.run_command
    quick_test_command = args.quick_test_command
    isolated_test_command = args.isolated_test_command
    smoke_check_command = args.smoke_check_command

    # Refine commands based on Makefile
    if (paths.target_root / "Makefile").exists():
        if not install_command:
            install_command = "make install"
        if not run_command:
            run_command = "make run"
        if not quick_test_command:
            quick_test_command = "make test"
        if not smoke_check_command:
            smoke_check_command = "make smoke"

    # Fallback to stack-specific defaults
    if primary_stack == "node":
        if not install_command or "TODO" in install_command:
            install_command = "npm install"
        if not run_command or "TODO" in run_command:
            run_command = guess_run_command(paths.target_root, package_scripts)
        if not quick_test_command or "TODO" in quick_test_command:
            quick_test_command = (
                "npm test"
                if "test" in package_scripts
                else "npm run lint"
                if "lint" in package_scripts
                else "TODO: 빠른 테스트 명령 입력"
            )
        if not isolated_test_command or "TODO" in isolated_test_command:
            isolated_test_command = (
                "npm run test:unit"
                if "test:unit" in package_scripts
                else "npm run test:ci"
                if "test:ci" in package_scripts
                else "TODO: 격리 테스트 명령 입력"
            )
        if not smoke_check_command or "TODO" in smoke_check_command:
            smoke_check_command = (
                "npm run test:smoke"
                if "test:smoke" in package_scripts
                else "TODO: 실행 확인 명령 입력"
            )
    elif primary_stack == "python":
        if not install_command or "TODO" in install_command:
            install_command = (
                "pip install -r requirements.txt"
                if (paths.target_root / "requirements.txt").exists()
                else "pip install ."
            )
        if not run_command or "TODO" in run_command:
            run_command = guess_run_command(paths.target_root, {})
        if not quick_test_command or "TODO" in quick_test_command:
            quick_test_command = "pytest" if test_dirs else "TODO: 빠른 테스트 명령 입력"
        if not isolated_test_command or "TODO" in isolated_test_command:
            isolated_test_command = (
                "pytest tests/unit"
                if (paths.target_root / "tests/unit").exists()
                else "TODO: 격리 테스트 명령 입력"
            )
    elif primary_stack == "rust":
        if not install_command or "TODO" in install_command:
            install_command = "cargo fetch"
        if not run_command or "TODO" in run_command:
            run_command = "cargo run"
        if not quick_test_command or "TODO" in quick_test_command:
            quick_test_command = "cargo test"
        if not isolated_test_command or "TODO" in isolated_test_command:
            isolated_test_command = "cargo test --lib"
    elif primary_stack == "go":
        if not install_command or "TODO" in install_command:
            install_command = "go mod download"
        if not run_command or "TODO" in run_command:
            run_command = "go run ./..."
        if not quick_test_command or "TODO" in quick_test_command:
            quick_test_command = "go test ./..."
        if not isolated_test_command or "TODO" in isolated_test_command:
            isolated_test_command = "go test ./... -run TestSmoke"

    # Final fallback
    install_command = install_command or "TODO: 설치 명령 입력"
    run_command = run_command or "TODO: 로컬 실행 명령 입력"
    quick_test_command = quick_test_command or "TODO: 빠른 테스트 명령 입력"
    isolated_test_command = isolated_test_command or "TODO: 격리 테스트 명령 입력"
    smoke_check_command = smoke_check_command or "TODO: 실행 확인 명령 입력"

    top_level_entries = sorted(
        path.name for path in paths.target_root.iterdir() if path.name != args.kit_dir
    )

    if args.adoption_mode == "new":
        return {
            "top_level_entries": top_level_entries,
            "source_dirs": source_dirs,
            "docs_dirs": docs_dirs,
            "test_dirs": test_dirs,
            "stack_labels": stack_labels,
            "primary_stack": primary_stack,
            "package_scripts": package_scripts,
            "existing_docs_detected": bool(docs_dirs),
            "has_existing_tests": bool(test_dirs),
            "doc_home": doc_home,
            "operations_dir": operations_dir,
            "backlog_dir": backlog_dir,
            "session_doc_path": session_doc_path,
            "environment_dir": environment_dir,
            "install_command": install_command,
            "run_command": run_command,
            "quick_test_command": quick_test_command,
            "isolated_test_command": isolated_test_command,
            "smoke_check_command": smoke_check_command,
            "analysis_summary": [
                "신규 프로젝트 모드이지만 기본 구조 분석을 통해 명령어를 추론했다.",
                "필요한 경우 생성된 profile 문서에서 명령어를 추가로 보정할 수 있다.",
            ],
        }

    # "existing" mode: deeper scan
    files = iter_repo_files(paths.target_root, ignore_dirs={args.kit_dir})
    rel_files = [path.relative_to(paths.target_root).as_posix() for path in files]

    analysis_summary = [
        f"상위 디렉터리 기준으로 `{', '.join(top_level_entries[:10])}` 구조를 확인했다."
        if top_level_entries
        else "상위 디렉터리 항목이 거의 비어 있다.",
        f"추정 기본 스택은 `{primary_stack}` 이며 감지된 스택 라벨은 `{', '.join(stack_labels) or '없음'}` 이다.",
        f"문서 디렉터리는 `{', '.join(docs_dirs) or '없음'}`, 테스트 디렉터리는 `{', '.join(test_dirs) or '없음'}` 으로 감지됐다.",
        f"package script 는 `{', '.join(sorted(package_scripts)[:8]) or '없음'}` 으로 확인됐다.",
    ]

    return {
        "top_level_entries": top_level_entries,
        "source_dirs": source_dirs,
        "docs_dirs": docs_dirs,
        "test_dirs": test_dirs,
        "stack_labels": stack_labels,
        "primary_stack": primary_stack,
        "package_scripts": package_scripts,
        "existing_docs_detected": bool(docs_dirs),
        "has_existing_tests": bool(test_dirs),
        "doc_home": doc_home,
        "operations_dir": operations_dir,
        "backlog_dir": backlog_dir,
        "session_doc_path": session_doc_path,
        "environment_dir": environment_dir,
        "install_command": install_command,
        "run_command": run_command,
        "quick_test_command": quick_test_command,
        "isolated_test_command": isolated_test_command,
        "smoke_check_command": smoke_check_command,
        "analysis_summary": analysis_summary,
        "sample_paths": rel_files[:20],
    }


# ---------------------------------------------------------------------------
# Harness dispatcher
# ---------------------------------------------------------------------------
def write_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    """Write all harness-specific overlay files and return a {key: path} map.

    The harness overlay files are written in two passes:

    1. **Agents entry files** (``AGENTS.md`` / ``GEMINI.md`` / ``ANTIGRAVITY.md`` /
       ``MiniMax.md``) are written directly here because the renderer and the
       path are 1:1 with the harness name. The same ``codex_agents_path`` is
       reused for both ``codex``, ``opencode``, and ``pi-dev`` (the latter two
       share the root ``AGENTS.md`` location).
    2. **Per-harness config builders** registered in
       :data:`bootstrap_lib.harnesses.HARNESS_FILE_BUILDERS` add the rest:
       config examples, skill files, worker overlays, etc.

    When ``--enable-mcp`` is set, per-harness MCP server config snippets are
    also written and merged into the result.
    """
    generated: dict[str, str] = {}
    actions: list[dict[str, str]] = []
    harnesses = selected_harnesses(args)

    def _w(path, content):
        actions.append(
            write_text(path, content, force=args.force, rel_to=paths.target_root)
        )

    if "codex" in harnesses or "opencode" in harnesses:
        codex_agents = codex_agents_path(paths)
        _w(codex_agents, render_codex_agents(args, paths, context))
        generated["codex_agents"] = str(codex_agents)

    if "pi-dev" in harnesses:
        # Pi Coding Agent uses AGENTS.md at the root, same path as codex
        # If both are selected, Pi's version will overwrite or vice versa.
        # Usually only one harness is selected for a project.
        from bootstrap_lib.harnesses.renderers import render_pi_dev_agents

        pi_agents = codex_agents_path(paths)
        _w(pi_agents, render_pi_dev_agents(args, context))
        generated["pi_dev_agents"] = str(pi_agents)

    if "gemini-cli" in harnesses:
        gemini_agents = gemini_cli_agents_path(paths)
        _w(gemini_agents, render_gemini_cli_agents(args, paths, context))
        generated["gemini_cli_agents"] = str(gemini_agents)

    if "antigravity" in harnesses:
        antigravity_agents = antigravity_agents_path(paths)
        _w(antigravity_agents, render_antigravity_agents(args, paths, context))
        generated["antigravity_agents"] = str(antigravity_agents)

    if "minimax-code" in harnesses:
        # MiniMax Code shares AGENTS.md with codex/opencode but writes its own
        # MiniMax.md entry file. We do NOT route through the codex/opencode
        # block above because the AGENTS.md text differs slightly.
        minimax_agents = minimax_agents_path(paths)
        _w(minimax_agents, render_minimax_agents(args, paths, context))
        generated["minimax_code_agents"] = str(minimax_agents)

    for harness in harnesses:
        builder = HARNESS_FILE_BUILDERS[harness]
        generated.update(builder(args, paths, context))

    if getattr(args, "enable_mcp", False):
        generated.update(write_mcp_config_files(args, paths, harnesses))

    return generated, actions


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main() -> int:
    args = parse_args()
    enforce_harness_selection(args)
    paths = make_paths(args)
    context = infer_project_context(args, paths)

    harness_files: dict[str, str] = {}
    dependency_files: list[str] = []
    core_docs: list[str] = []
    file_actions: list[dict[str, str]] = []

    # Records each smart-update action into the manifest.
    def _record_write(
        path: Path,
        content: str,
        *,
        force: bool = False,
        src_version: str | None = None,
    ) -> None:
        file_actions.append(
            write_text(
                path,
                content,
                force=force,
                src_version=src_version,
                rel_to=paths.target_root,
            )
        )

    if not args.dry_run:
        # 1. Write core docs if requested
        if args.copy_core_docs:
            if SOURCE_ROOT is None:
                # Wheel install without bundled data: skip core-docs copy
                # but record it as a manifest warning so users notice.
                context.setdefault("warnings", []).append(
                    "--copy-core-docs requested but core docs are not bundled "
                    "in the installed wheel. To copy them, install from source: "
                    "git clone … && pip install -e workflow-source/"
                )
            else:
                core_actions = copy_core_docs(
                    paths,
                    force=args.force,
                    default_core_docs=DEFAULT_CORE_DOCS,
                    default_core_support_paths=DEFAULT_CORE_SUPPORT_PATHS,
                    source_root=SOURCE_ROOT,
                )
                file_actions.extend(core_actions)
                # copied_core_docs is the manifest field that historically
                # lists only default_core_docs, not the support sub-tree.
                core_doc_names = set(DEFAULT_CORE_DOCS)
                core_docs = [
                    a["rel"] for a in core_actions
                    if a.get("action") in ("create", "updated")
                    and Path(a["rel"]).name in core_doc_names
                ]

        # 2. Render + write all generated docs
        _record_write(paths.readme_path, render_readme(args, context, default_core_docs=DEFAULT_CORE_DOCS), force=args.force)
        _record_write(paths.profile_path, render_project_profile(args, context), force=args.force)
        _record_write(paths.handoff_path, render_session_handoff(args, context), force=args.force)
        _record_write(paths.backlog_index_path, render_backlog_index(args), force=args.force)
        _record_write(paths.daily_backlog_path, render_daily_backlog(args, context), force=args.force)
        _record_write(paths.status_assessment_path, render_project_status_assessment(args), force=args.force)
        if args.adoption_mode == "existing":
            _record_write(paths.assessment_path, render_assessment(args, context), force=args.force)

        # 3. Write harness overlay files (includes MCP config snippets if --enable-mcp)
        harness_files, harness_actions = write_harness_files(args, paths, context)
        file_actions.extend(harness_actions)

        # 4. Optionally update dependency manifests
        if args.update_deps:
            dependency_files = update_dependencies(paths, context, selected_harnesses(args))

        # 6. Write workflow state payload
        state_payload = build_workflow_state_payload(
            project_profile_path=paths.profile_path,
            session_handoff_path=paths.handoff_path,
            work_backlog_index_path=paths.backlog_index_path,
            latest_backlog_path=paths.daily_backlog_path,
            repository_assessment_path=(
                paths.assessment_path if args.adoption_mode == "existing" else None
            ),
            generated_at=args.today,
            workspace_root=paths.target_root,
        )
        paths.state_path.parent.mkdir(parents=True, exist_ok=True)
        paths.state_path.write_text(
            json.dumps(state_payload, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
        )

    # 7. Build + emit the manifest
    all_file_actions = file_actions + drain_file_actions()
    manifest = build_manifest(
        args=args,
        paths=paths,
        core_docs=core_docs,
        context=context,
        harness_files=harness_files,
        dependency_files=dependency_files,
        selected_harnesses=selected_harnesses(args),
        global_snippet_sources_fn=lambda: _global_snippet_sources(SOURCE_ROOT),
        file_actions=all_file_actions,
    )
    print(json.dumps(manifest, ensure_ascii=False, indent=2))
    return 0


# ---------------------------------------------------------------------------
# Harness builder registration
# ---------------------------------------------------------------------------
# Registration happens at the bottom of
# :mod:`bootstrap_lib.harnesses.renderers` (it lives next to the
# ``write_*_harness_files`` implementations so the two stay in sync).
# Importing that module — which we do above — is enough to populate
# :data:`bootstrap_lib.harnesses.HARNESS_FILE_BUILDERS`.


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------
if __name__ == "__main__":
    raise SystemExit(main())


__all__ = [
    "DEFAULT_CORE_DOCS",
    "DEFAULT_CORE_SUPPORT_PATHS",
    "HARNESS_DEFINITIONS",
    "build_manifest",
    "infer_project_context",
    "main",
    "make_paths",
    "parse_args",
    "selected_harnesses",
    "update_dependencies",
    "write_harness_files",
]
