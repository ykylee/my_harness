#!/usr/bin/env python3
"""Bootstrap a reusable standard AI workflow kit into a target repository."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
import re
from dataclasses import dataclass
from datetime import date
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
SCRIPT_DIR = REPO_ROOT / "workflow-source" / "scripts"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from workflow_kit.common.workflow_state import build_workflow_state_payload
from bootstrap_harnesses import (
    HARNESS_FILE_BUILDERS,
    HARNESS_SPECS,
    SUPPORTED_HARNESSES,
    register_harness_builder,
    spec_for,
)

DEFAULT_CORE_DOCS = [
    "global_workflow_standard.md",
    "workflow_skill_catalog.md",
    "workflow_mcp_candidate_catalog.md",
    "workflow_agent_topology.md",
    "output_schema_guide.md",
    "workflow_adoption_entrypoints.md",
    "workflow_harness_distribution.md",
]
# Base dependencies for any workflow integration
COMMON_PYTHON_DEPS = ["mcp_servers", "pytest", "pytest-asyncio"]
COMMON_NODE_DEPS = ["@modelcontextprotocol/sdk"]

# Harness-specific optional dependencies
HARNESS_PYTHON_DEPS = {
    "gemini-cli": [],
    "codex": [],
    "opencode": [],
    "antigravity": [],
}
HARNESS_NODE_DEPS = {
    "gemini-cli": [],
    "codex": [],
}

DEFAULT_CORE_SUPPORT_PATHS = [
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
IGNORED_DIRS = {
    ".git",
    ".hg",
    ".svn",
    ".idea",
    ".vscode",
    "node_modules",
    ".next",
    ".turbo",
    "dist",
    "build",
    "coverage",
    "__pycache__",
    ".venv",
    "venv",
}
SUPPORTED_HARNESSES = SUPPORTED_HARNESSES  # re-exported from bootstrap_harnesses


@dataclass(frozen=True)
class Paths:
    target_root: Path
    kit_root: Path
    core_dir: Path
    memory_dir: Path
    backlog_dir: Path
    readme_path: Path
    profile_path: Path
    state_path: Path
    handoff_path: Path
    backlog_index_path: Path
    daily_backlog_path: Path
    assessment_path: Path
    status_assessment_path: Path


@dataclass(frozen=True)
class HarnessDefinition:
    name: str
    description: str


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
        help="Generate harness-specific overlay files for the selected target.",
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
        "--force",
        action="store_true",
        help="Overwrite existing generated files when the destination already exists.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the generation plan without writing files.",
    )
    return parser.parse_args()


def make_paths(args: argparse.Namespace) -> Paths:
    target_root = Path(args.target_root).resolve()
    kit_root = target_root / args.kit_dir
    memory_dir = kit_root / "memory"
    backlog_dir = memory_dir / "backlog"
    return Paths(
        target_root=target_root,
        kit_root=kit_root,
        core_dir=kit_root / "core",
        memory_dir=memory_dir,
        backlog_dir=backlog_dir,
        readme_path=kit_root / "README.md",
        profile_path=target_root / "docs"/ "PROJECT_PROFILE.md",
        state_path=memory_dir / "state.json",
        handoff_path=memory_dir / "session_handoff.md",
        backlog_index_path=memory_dir / "work_backlog.md",
        daily_backlog_path=backlog_dir / f"{args.today}.md",
        assessment_path=memory_dir / "repository_assessment.md",
        status_assessment_path=memory_dir / "project_status_assessment.md",
    )


def write_text(path: Path, content: str, *, force: bool) -> None:
    if path.exists() and not force:
        raise FileExistsError(f"Destination already exists: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def copy_core_docs(paths: Paths, *, force: bool) -> list[str]:
    copied: list[str] = []
    paths.core_dir.mkdir(parents=True, exist_ok=True)
    for name in DEFAULT_CORE_DOCS:
        source = SOURCE_ROOT/ "core"/ name
        destination = paths.core_dir / name
        if destination.exists() and not force:
            raise FileExistsError(f"Destination already exists: {destination}")
        shutil.copyfile(source, destination)
        copied.append(str(destination))
    for raw_relative_path in DEFAULT_CORE_SUPPORT_PATHS:
        relative_path = Path(raw_relative_path)
        source = SOURCE_ROOT/ relative_path
        destination = paths.kit_root / relative_path
        if source.is_dir():
            for file_path in sorted(source.rglob("*")):
                if not file_path.is_file():
                    continue
                nested_relative = file_path.relative_to(SOURCE_ROOT)
                nested_destination = paths.kit_root / nested_relative
                if nested_destination.exists() and not force:
                    raise FileExistsError(f"Destination already exists: {nested_destination}")
                nested_destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(file_path, nested_destination)
            continue
        if destination.exists() and not force:
            raise FileExistsError(f"Destination already exists: {destination}")
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    return copied


def rel(path: Path, base: Path) -> str:
    return path.relative_to(base).as_posix()


def selected_harnesses(args: argparse.Namespace) -> list[str]:
    return sorted(dict.fromkeys(args.harnesses))


def update_dependencies(paths: Paths, context: dict[str, object], harnesses: list[str]) -> list[str]:
    updated_files: list[str] = []
    primary_stack = context.get("primary_stack", "unknown")
    target_root = paths.target_root

    # 1. Handle Python dependencies (requirements.txt)
    req_file = target_root / "requirements.txt"
    is_python = primary_stack == "python"or primary_stack == "unspecified"

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

            if "# Standard AI Workflow Dependencies"not in existing_content:
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
                dep for dep in needed_node
                if dep not in dev_deps and dep not in deps
            ]

            if to_add_node:
                if "devDependencies"not in payload:
                    payload["devDependencies"] = {}
                for dep in to_add_node:
                    payload["devDependencies"][dep] = "latest"

                payload["devDependencies"] = dict(sorted(payload["devDependencies"].items()))
                package_json.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
                updated_files.append(rel(package_json, target_root))
        except (json.JSONDecodeError, KeyError):
            context["analysis_summary"].append("Warning: Could not parse package.json for dependency updates.")

    # 3. Handle pyproject.toml / other managers with hints
    if (target_root / "pyproject.toml").exists():
        needed_python = sorted(list(set(COMMON_PYTHON_DEPS + [d for h in harnesses for d in HARNESS_PYTHON_DEPS.get(h, [])])))
        context["analysis_summary"].append(
            f"Note: `pyproject.toml` detected. Recommended: `uv add {' '.join(needed_python)}` or equivalent."
        )

    return updated_files


HARNESS_DEFINITIONS = {
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
}


def global_snippet_sources() -> dict[str, dict[str, str]]:
    return {
        "codex": {
            "readme": str((SOURCE_ROOT/ "global-snippets"/ "codex"/ "README.md").resolve()),
            "snippet": str((SOURCE_ROOT/ "global-snippets"/ "codex"/ "config.toml.snippet").resolve()),
            "target": "~/.codex/config.toml",
            "policy": "additive_only",
        },
        "opencode": {
            "readme": str((SOURCE_ROOT/ "global-snippets"/ "opencode"/ "README.md").resolve()),
            "snippet": str((SOURCE_ROOT/ "global-snippets"/ "opencode"/ "opencode.global.jsonc").resolve()),
            "target": "~/.config/opencode/opencode.json",
            "policy": "additive_only",
        },
    }


def iter_repo_files(root: Path, *, max_depth: int = 3, ignore_dirs: set[str] | None = None) -> list[Path]:
    results: list[Path] = []
    combined_ignore = IGNORED_DIRS.copy()
    if ignore_dirs:
        combined_ignore.update(ignore_dirs)

    for current_root, dirs, files in os.walk(root):
        current_path = Path(current_root)
        try:
            relative = current_path.relative_to(root)
            depth = len(relative.parts)
        except ValueError:
            depth = 0
        dirs[:] = sorted(
            name
            for name in dirs
            if name not in combined_ignore and depth < max_depth
        )
        for file_name in sorted(files):
            results.append(current_path / file_name)
    return results


def detect_package_scripts(target_root: Path) -> dict[str, str]:
    package_json = target_root / "package.json"
    if not package_json.exists():
        return {}
    try:
        payload = json.loads(package_json.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return {}
    scripts = payload.get("scripts")
    if isinstance(scripts, dict):
        return {str(key): str(value) for key, value in scripts.items()}
    return {}


def guess_run_command(target_root: Path, package_scripts: dict[str, str]) -> str:
    for script_name in ("dev", "start", "serve"):
        if script_name in package_scripts:
            return f"npm run {script_name}"
    for candidate in ("app/main.py", "main.py"):
        if (target_root / candidate).exists():
            return f"python {candidate}"
    return "TODO: 로컬 실행 명령 입력"


def infer_project_context(args: argparse.Namespace, paths: Paths) -> dict[str, object]:
    # Basic exploration for both "new"and "existing"modes
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
    # Check root and subdirectories for stack indicators
    search_paths = [paths.target_root] + [p for p in paths.target_root.iterdir() if p.is_dir() and p.name not in IGNORED_DIRS]
    
    for p in search_paths:
        if (p / "package.json").exists():
            stack_labels.append("node")
        if (p / "pyproject.toml").exists() or (p / "requirements.txt").exists() or (p / "setup.py").exists():
            stack_labels.append("python")
        if (p / "Cargo.toml").exists():
            stack_labels.append("rust")
        if (p / "go.mod").exists():
            stack_labels.append("go")
        if (p / "Gemfile").exists():
            stack_labels.append("ruby")
        if (p / "CMakeLists.txt").exists() or (p / "Makefile").exists():
            stack_labels.append("cpp")

    # Remove duplicates and prioritize
    stack_labels = sorted(list(set(stack_labels)))
    primary_stack = stack_labels[0] if stack_labels else "unknown"

    package_scripts = detect_package_scripts(paths.target_root)

    # Resolve paths: prefer CLI args if provided, then inferred, then defaults
    if args.doc_home and args.doc_home != "README.md":
        doc_home = args.doc_home
    elif docs_dirs and (paths.target_root / docs_dirs[0] / "README.md").exists():
        doc_home = f"{docs_dirs[0]}/README.md"
    else:
        doc_home = "README.md"

    # Resolve operations_dir
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

    # Refine commands based on stack (Priority: Makefile -> Primary Stack)
    if (paths.target_root / "Makefile").exists():
        if not install_command: install_command = "make install"
        if not run_command: run_command = "make run"
        if not quick_test_command: quick_test_command = "make test"
        if not smoke_check_command: smoke_check_command = "make smoke"

    # Fallback to stack-specific defaults if still TODO
    if primary_stack == "node":
        if not install_command or "TODO" in install_command: install_command = "npm install"
        if not run_command or "TODO" in run_command: run_command = guess_run_command(paths.target_root, package_scripts)
        if not quick_test_command or "TODO" in quick_test_command:
            quick_test_command = "npm test" if "test" in package_scripts else ("npm run lint" if "lint" in package_scripts else "TODO: 빠른 테스트 명령 입력")
        if not isolated_test_command or "TODO" in isolated_test_command:
            isolated_test_command = "npm run test:unit" if "test:unit" in package_scripts else ("npm run test:ci" if "test:ci" in package_scripts else "TODO: 격리 테스트 명령 입력")
        if not smoke_check_command or "TODO" in smoke_check_command:
            smoke_check_command = "npm run test:smoke" if "test:smoke" in package_scripts else "TODO: 실행 확인 명령 입력"
    elif primary_stack == "python":
        if not install_command or "TODO" in install_command:
            install_command = "pip install -r requirements.txt" if (paths.target_root / "requirements.txt").exists() else "pip install ."
        if not run_command or "TODO" in run_command: run_command = guess_run_command(paths.target_root, {})
        if not quick_test_command or "TODO" in quick_test_command:
            quick_test_command = "pytest" if test_dirs else "TODO: 빠른 테스트 명령 입력"
        if not isolated_test_command or "TODO" in isolated_test_command:
            isolated_test_command = "pytest tests/unit" if (paths.target_root / "tests/unit").exists() else "TODO: 격리 테스트 명령 입력"
    elif primary_stack == "rust":
        if not install_command or "TODO" in install_command: install_command = "cargo fetch"
        if not run_command or "TODO" in run_command: run_command = "cargo run"
        if not quick_test_command or "TODO" in quick_test_command: quick_test_command = "cargo test"
        if not isolated_test_command or "TODO" in isolated_test_command: isolated_test_command = "cargo test --lib"
    elif primary_stack == "go":
        if not install_command or "TODO" in install_command: install_command = "go mod download"
        if not run_command or "TODO" in run_command: run_command = "go run ./..."
        if not quick_test_command or "TODO" in quick_test_command: quick_test_command = "go test ./..."
        if not isolated_test_command or "TODO" in isolated_test_command: isolated_test_command = "go test ./... -run TestSmoke"

    # Fallback for all stacks
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

    # "existing"mode additional info
    files = iter_repo_files(paths.target_root, ignore_dirs={args.kit_dir})
    rel_files = [path.relative_to(paths.target_root).as_posix() for path in files]

    analysis_summary = [
        f"상위 디렉터리 기준으로 `{', '.join(top_level_entries[:10])}` 구조를 확인했다."if top_level_entries else "상위 디렉터리 항목이 거의 비어 있다.",
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


def value_or_inferred(explicit: str | None, fallback: str) -> str:
    if explicit is None or explicit.startswith("TODO:"):
        return fallback
    return explicit


def render_readme(args: argparse.Namespace, context: dict[str, object]) -> str:
    if args.copy_core_docs:
        core_docs = "\n".join(
            f"- [core/{name}](./core/{name})"for name in DEFAULT_CORE_DOCS
        )
    else:
        core_docs = "- core 문서는 `--copy-core-docs` 옵션을 사용하면 함께 복사할 수 있다."
    generated_assessment = ""
    mode_summary = "신규 프로젝트용 기본 문서 세트를 생성했다."
    harness_lines = "\n".join(
        f"- `{name}` 하네스용 오버레이 파일 생성"
        for name in selected_harnesses(args)
    ) or "- 선택한 하네스 없음"
    if args.adoption_mode == "existing":
        generated_assessment = "- [ai-workflow/memory/repository_assessment.md](./ai-workflow/memory/repository_assessment.md)"
        mode_summary = "기존 프로젝트 분석 결과를 반영한 문서 초안과 평가 문서를 생성했다."
    return f"""# Standard AI Workflow Kit

- 문서 목적: `{args.project_name}` 저장소에 표준 AI 워크플로우 기본 문서 세트를 도입할 수 있도록 bootstrap 결과를 안내한다.
- 범위: 공통 코어 문서 위치, 프로젝트 상태 문서 세트, 도입 모드별 후속 작업
- 대상 독자: 개발자, 운영자, AI agent, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `docs/PROJECT_PROFILE.md`, `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`

## 1. 도입 모드

- 선택한 도입 모드: `{args.adoption_mode}`
- 요약:
- {mode_summary}

## 2. 생성된 파일

- [docs/PROJECT_PROFILE.md](../docs/PROJECT_PROFILE.md)
- [ai-workflow/memory/state.json](./memory/state.json)
- [ai-workflow/memory/session_handoff.md](./memory/session_handoff.md)
- [ai-workflow/memory/work_backlog.md](./memory/work_backlog.md)
- [ai-workflow/memory/backlog/{args.today}.md](./memory/backlog/{args.today}.md)
{generated_assessment}

## 3. 코어 문서

{core_docs}

## 4. 하네스 오버레이

{harness_lines}

## 5. 도입 직후 해야 할 일

1. `PROJECT_PROFILE.md` 에 프로젝트 목적, 명령, 검증 규칙을 실제 값으로 채운다.
2. `state.json`, `session_handoff.md`, 오늘 날짜 backlog 를 현재 진행 작업 기준으로 갱신한다.
3. 기존 프로젝트 모드였다면 `repository_assessment.md` 의 추정값을 실제 저장소 규칙과 대조해 수정한다.
4. 선택한 하네스가 있으면 생성된 overlay 파일을 각 하네스 실행 경로에 맞게 검토한다.
5. 이후 표준 skill/MCP 도입 범위는 `core/` 문서를 기준으로 결정한다.

## 6. 언어와 컨텍스트 운영 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, handoff/backlog 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 중간 분류는 모델이 가장 효율적인 형태로 처리하고, 사용자에게는 필요한 결론만 짧게 전달한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 7. 프로젝트 실제 문서 경로 설정값

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- 백로그 위치: `{context['backlog_dir']}`
- 세션 인계 문서 위치: `{context['session_doc_path']}`
- 환경 기록 위치: `{context['environment_dir']}`

## 다음에 읽을 문서

- 프로젝트 프로파일: [../docs/PROJECT_PROFILE.md](../docs/PROJECT_PROFILE.md)
- 빠른 상태 요약: [./memory/state.json](./memory/state.json)
- 세션 인계 문서: [./memory/session_handoff.md](./memory/session_handoff.md)
- 작업 백로그 인덱스: [./memory/work_backlog.md](./memory/work_backlog.md)
"""


def load_template(name: str) -> str:
    template_path = SOURCE_ROOT/ "templates"/ name
    if not template_path.exists():
        return f"MISSING TEMPLATE: {name}"
    return template_path.read_text(encoding="utf-8")


def render_project_profile(args: argparse.Namespace, context: dict[str, object]) -> str:
    content = load_template("project_workflow_profile_template.md")
    install_command = value_or_inferred(args.install_command, str(context["install_command"]))
    run_command = value_or_inferred(args.run_command, str(context["run_command"]))
    quick_test_command = value_or_inferred(args.quick_test_command, str(context["quick_test_command"]))
    isolated_test_command = value_or_inferred(args.isolated_test_command, str(context["isolated_test_command"]))
    smoke_check_command = value_or_inferred(args.smoke_check_command, str(context["smoke_check_command"]))

    replacements = {
        "<Project Name>": args.project_name,
        "<핵심 사용자 가치 및 목표>": value_or_inferred(args.project_purpose, "TODO: 프로젝트 목적 정리"),
        "<협업 부서 및 담당자>": value_or_inferred(args.stakeholders, "TODO: 주요 이해관계자 정리"),
        "<README.md>": str(context["doc_home"]),
        "<docs/operations/>": str(context["operations_dir"]),
        "<ai-workflow/memory/backlog/>": str(context["backlog_dir"]),
        "<ai-workflow/memory/session_handoff.md>": str(context["session_doc_path"]),
        "<ai-workflow/memory/repository_assessment.md>": str(context["environment_dir"]),
        "<설치 및 가상환경 구성 명령>": install_command,
        "<어플리케이션 실행 명령>": run_command,
        "<단위 테스트 및 Lint 명령>": quick_test_command,
        "<Docker 또는 독립 환경 테스트 명령>": isolated_test_command,
        "<상태 체크 및 E2E 확인 명령>": smoke_check_command,
        "YYYY-MM-DD": args.today,
    }
    for key, val in replacements.items():
        content = content.replace(key, val)
    return content


def render_session_handoff(args: argparse.Namespace, context: dict[str, object]) -> str:
    content = load_template("session_handoff_template.md")

    current_focus = "TODO: Summarize the current session focus."
    in_progress = f"{args.initial_task_id} {args.initial_task_name}"
    blocked = "N/A"
    completed = "N/A"
    key_change = "Initial workflow docs generated."
    next_action = "Review and refine generated workflow docs."
    risk_or_blocker = "N/A"

    if args.adoption_mode == "existing":
        current_focus = f"Existing codebase onboarding completed; inferred primary stack: {context['primary_stack']}."
        completed = "Repository scan completed"
        key_change = "Generated initial workflow docs from the existing repository scan."
        next_action = "Validate generated profile, handoff, and backlog against the repository."

    replacements = {
        "<CURRENT_FOCUS>": current_focus,
        "<IN_PROGRESS_ITEM>": in_progress,
        "<BLOCKED_ITEM>": blocked,
        "<DONE_ITEM>": completed,
        "<KEY_CHANGE>": key_change,
        "<NEXT_ACTION>": next_action,
        "<RISK_OR_BLOCKER>": risk_or_blocker,
        "YYYY-MM-DD": args.today,
    }
    for key, val in replacements.items():
        content = content.replace(key, val)
    return content


def render_backlog_index(args: argparse.Namespace) -> str:
    content = load_template("work_backlog_template.md")
    replacements = {
        "YYYY-MM-DD": args.today,
    }
    for key, val in replacements.items():
        content = content.replace(key, val)
    return content


def render_daily_backlog(args: argparse.Namespace, context: dict[str, object]) -> str:
    content = load_template("daily_backlog_template.md")

    task_goal = "TODO: 작업 목표"
    done_criteria = "TODO: 완료 기준"
    progress = f"`{args.today} 09:00` bootstrap 초기 생성"

    if args.adoption_mode == "existing":
        task_goal = "기존 프로젝트 분석 및 워크플로우 도입"
        done_criteria = "profile/handoff/backlog 초안 생성 및 검토 완료"
        progress = f"`{args.today} 09:00` 기존 저장소 분석 및 문서 생성 완료"

    replacements = {
        "TASK-XXX": args.initial_task_id,
        "<작업명>": args.initial_task_name,
        "planned | in_progress | done | blocked": args.initial_task_status,
        "high | medium | low": args.initial_priority,
        "<name>": args.owner,
        "<file_paths>": f"{context['session_doc_path']}, {context['backlog_dir']}",
        "TODO: 작업 목표": task_goal,
        "TODO: 완료 기준": done_criteria,
        "YYYY-MM-DD": args.today,
    }
    for key, val in replacements.items():
        content = content.replace(key, val)
    # Append the progress note as it's not a direct placeholder in the simple template
    content = content.replace("- 진행 현황:", f"- 진행 현황: {progress}")
    return content


def render_project_status_assessment(args: argparse.Namespace) -> str:
    content = load_template("project_status_assessment_template.md")
    return content.replace("<Project Name>", args.project_name).replace("<YYYY-MM-DD>", args.today)


def render_assessment(args: argparse.Namespace, context: dict[str, object]) -> str:
    if args.adoption_mode != "existing":
        return ""
    top_entries = ", ".join(context["top_level_entries"]) or "없음"
    docs_dirs = ", ".join(context["docs_dirs"]) or "없음"
    test_dirs = ", ".join(context["test_dirs"]) or "없음"
    source_dirs = ", ".join(context["source_dirs"]) or "없음"
    stack_labels = ", ".join(context["stack_labels"]) or "없음"
    scripts = ", ".join(sorted(context["package_scripts"])) or "없음"
    sample_paths = "\n".join(f"- `{item}`"for item in context.get("sample_paths", []))
    return f"""# Repository Assessment

- 문서 목적: 기존 프로젝트에 표준 AI 워크플로우를 도입하기 전에 현재 코드베이스와 문서 구조를 빠르게 진단한다.
- 범위: 저장소 구조, 추정 기술 스택, 문서 위치, 테스트 흔적, 초기 워크플로우 도입 포인트
- 대상 독자: 개발자, 운영자, AI agent, 프로젝트 온보딩 담당자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `./PROJECT_PROFILE.md`, `./session_handoff.md`, `../core/workflow_adoption_entrypoints.md`

## 1. 요약

- 분석 대상 프로젝트:
- `{args.project_name}`
- 분석 모드:
- `existing`
- 추정 기본 스택:
- `{context['primary_stack']}`
- 감지된 스택 라벨:
- `{stack_labels}`

## 2. 저장소 구조 관찰

- 상위 디렉터리 항목:
- `{top_entries}`
- 소스 디렉터리 후보:
- `{source_dirs}`
- 문서 디렉터리 후보:
- `{docs_dirs}`
- 테스트 디렉터리 후보:
- `{test_dirs}`

## 3. 추정 명령

- 설치:
- `{context['install_command']}`
- 로컬 실행:
- `{context['run_command']}`
- 빠른 테스트:
- `{context['quick_test_command']}`
- 격리 테스트:
- `{context['isolated_test_command']}`
- 실행 확인:
- `{context['smoke_check_command']}`

## 4. package script 및 경로 샘플

- package script 목록:
- `{scripts}`
- 분석 중 확인한 경로 샘플:
{sample_paths or '- 없음'}

## 5. 워크플로우 도입 초안

- 추천 문서 위키 홈:
- `{context['doc_home']}`
- 추천 운영 문서 위치:
- `{context['operations_dir']}`
- 추천 backlog 위치:
- `{context['backlog_dir']}`
- 추천 session handoff 위치:
- `{context['session_doc_path']}`

## 6. 자동 분석 기반 다음 작업

- 현재 추정 명령과 실제 운영 명령이 일치하는지 확인한다.
- 기존 문서 체계가 있으면 운영 문서 위치를 그대로 따를지, 별도 워크플로우 디렉터리로 분리할지 결정한다.
- 빠른 테스트와 실행 확인 기준이 약하면 우선 profile 문서에서 검증 규칙을 먼저 보강한다.

## 다음에 읽을 문서

- 프로젝트 프로파일: [./PROJECT_PROFILE.md](./PROJECT_PROFILE.md)
- 세션 인계 문서: [./session_handoff.md](./session_handoff.md)
- 도입 분기 가이드: [../core/workflow_adoption_entrypoints.md](../core/workflow_adoption_entrypoints.md)
"""


def codex_agents_path(paths: Paths) -> Path:
    return paths.target_root / "AGENTS.md"


def gemini_cli_agents_path(paths: Paths) -> Path:
    return paths.target_root / "GEMINI.md"


def codex_config_example_path(paths: Paths) -> Path:
    return paths.target_root / ".codex"/ "config.toml.example"


def opencode_config_path(paths: Paths) -> Path:
    return paths.target_root / "opencode.json"


def opencode_skill_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode"/ "skills"/ "standard-ai-workflow"/ "SKILL.md"


def opencode_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode"/ "agents"/ "workflow-orchestrator.md"


def opencode_worker_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode"/ "agents"/ "workflow-worker.md"


def opencode_doc_worker_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode"/ "agents"/ "workflow-doc-worker.md"


def opencode_code_worker_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode"/ "agents"/ "workflow-code-worker.md"


def opencode_validation_worker_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode"/ "agents"/ "workflow-validation-worker.md"


def render_gemini_cli_agents(args: argparse.Namespace, paths: Paths, context: dict[str, object]) -> str:
    harness_note = (
        "기존 코드베이스 분석 결과를 반영한 초안이다. 추정 명령과 문서 경로는 실제 저장소 기준으로 수정할 수 있다."
        if args.adoption_mode == "existing"
        else "신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다."
    )
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in smoke_check:
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""# GEMINI.md

- 문서 목적: Gemini CLI 가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙과 기본 작업 원칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령
- 대상 독자: Gemini CLI, 저장소 관리자, workflow 설계자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `ai-workflow/memory/PROJECT_PROFILE.md`

## 목적

이 저장소에서는 표준 AI 워크플로우를 기준으로 작업한다. 세션 시작, backlog 갱신, 문서 동기화, 세션 종료는 `ai-workflow/` 아래 문서를 우선 기준으로 삼는다.

## 항상 먼저 읽을 문서

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

`ai-workflow/` 는 세션 복원과 workflow 상태 관리용 메타 레이어다. 프로젝트 코드나 프로젝트 문서를 탐색할 때는 이 경로를 기본 탐색 범위에 넣지 말고, workflow 문서 자체를 갱신하거나 현재 세션 상태를 복원할 때만 예외적으로 참조한다.

## 작업 원칙

- 작업을 시작하기 전에 목적, 범위, 영향 문서를 짧게 정리한다.
- 작업 상태는 `planned`, `in_progress`, `blocked`, `done` 중 하나로 관리한다.
- 검증하지 않은 결과는 완료로 확정하지 않는다.
- 세션 종료 전에는 `state.json`, `session_handoff.md`, 최신 backlog 를 갱신한다.

## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 프로젝트 실행 기본값

- 설치: `{context['install_command']}`
- 로컬 실행: `{context['run_command']}`
- 빠른 테스트: `{context['quick_test_command']}`
- 격리 테스트: `{context['isolated_test_command']}`
- 실행 확인: `{smoke_check}`

## 문서 작업 기준

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- backlog 위치: `{context['backlog_dir']}`
- session handoff 위치: `{context['session_doc_path']}`

## Gemini CLI 전용 메모

- Gemini CLI 는 프로젝트 루트의 `GEMINI.md` 를 읽으므로, 상세 정책은 본 문서에서 시작하고 세부 운영 기준은 `ai-workflow/` 문서를 참조한다.
- `GEMINI.md` 에 기재된 지침은 시스템 프롬프트보다 우선하는 강력한 지침으로 취급한다.
- 가능한 경우 메인 에이전트는 조정과 통합에 집중하고, bounded scope 의 읽기/쓰기/검증 작업은 서브 에이전트(`invoke_agent`)로 분리하는 패턴을 권장한다.
- 서브 에이전트에게는 책임 범위와 종료 조건을 명확히 넘기고, 메인 에이전트에는 핵심 사실과 결과만 다시 모은다.
- {harness_note}
"""


def antigravity_agents_path(paths: Paths) -> Path:
    return paths.target_root / "ANTIGRAVITY.md"


def minimax_agents_path(paths: Paths) -> Path:
    """Return the MiniMax Code entry file path (project root ``MiniMax.md``)."""
    return paths.target_root / "MiniMax.md"


def render_antigravity_agents(args: argparse.Namespace, paths: Paths, context: dict[str, object]) -> str:
    harness_note = (
        "기존 코드베이스 분석 결과를 반영한 초안이다. 추정 명령과 문서 경로는 실제 저장소 기준으로 수정할 수 있다."
        if args.adoption_mode == "existing"
        else "신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다."
    )
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in smoke_check:
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""# ANTIGRAVITY.md

- 문서 목적: Antigravity 가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙과 기본 작업 원칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령
- 대상 독자: Antigravity, 저장소 관리자, workflow 설계자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `ai-workflow/memory/PROJECT_PROFILE.md`

## 목적

이 저장소에서는 표준 AI 워크플로우를 기준으로 작업한다. 세션 시작, backlog 갱신, 문서 동기화, 세션 종료는 `ai-workflow/` 아래 문서를 우선 기준으로 삼는다.

## 항상 먼저 읽을 문서

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

`ai-workflow/` 는 세션 복원과 workflow 상태 관리용 메타 레이어다. 프로젝트 코드나 프로젝트 문서를 탐색할 때는 이 경로를 기본 탐색 범위에 넣지 말고, workflow 문서 자체를 갱신하거나 현재 세션 상태를 복원할 때만 예외적으로 참조한다.

## 작업 원칙

- 작업을 시작하기 전에 목적, 범위, 영향 문서를 짧게 정리한다.
- 작업 상태는 `planned`, `in_progress`, `blocked`, `done` 중 하나로 관리한다.
- 검증하지 않은 결과는 완료로 확정하지 않는다.
- 세션 종료 전에는 `state.json`, `session_handoff.md`, 최신 backlog 를 갱신한다.

## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 프로젝트 실행 기본값

- 설치: `{context['install_command']}`
- 로컬 실행: `{context['run_command']}`
- 빠른 테스트: `{context['quick_test_command']}`
- 격리 테스트: `{context['isolated_test_command']}`
- 실행 확인: `{smoke_check}`

## Antigravity 전용 작업 원칙

### 1. Artifacts (작업 증빙) 활용
Antigravity 에이전트는 모든 주요 의사결정과 작업 결과를 Artifacts 로 관리한다.
- **Implementation Plan**: 복잡한 수정 전에는 반드시 계획 문서를 작성하여 의도를 공유한다.
- **Task List**: 작업 단위를 쪼개어 실시간 진행 상황을 기록한다.
- **Walkthrough**: 작업 완료 후 변경 사항과 검증 결과를 요약하여 제출한다.

### 2. 브라우저 통합 및 서브 에이전트
UI 검증이나 외부 환경 조작이 필요한 경우, 직접 도구를 사용하는 대신 전용 **브라우저 서브 에이전트**를 활용하여 스크린샷과 녹화본을 증빙으로 확보한다.

### 3. 워크플로우 Skills 연동
`ai-workflow/skills/` 및 `scripts/` 아래의 도구들은 Antigravity 의 **Specialized Skills** 로 간주한다. 복잡한 상태 갱신이나 백로그 동기화는 직접 파일을 수정하기보다 이 도구들을 호출하여 수행하는 것을 권장한다.

## 문서 작업 기준

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- backlog 위치: `{context['backlog_dir']}`
- session handoff 위치: `{context['session_doc_path']}`

## Antigravity 전용 메모

- Antigravity 는 프로젝트 루트의 `ANTIGRAVITY.md` 를 읽으므로, 상세 정책은 본 문서에서 시작하고 세부 운영 기준은 `ai-workflow/` 문서를 참조한다.
- `ANTIGRAVITY.md` 에 기재된 지침은 시스템 프롬프트보다 우선하는 강력한 지침으로 취급한다.
- 가능한 경우 메인 에이전트는 조정과 통합에 집중하고, bounded scope 의 읽기/쓰기/검증 작업은 브라우저 서브 에이전트 등 적절한 서브 에이전트로 분리하는 패턴을 권장한다.
- 서브 에이전트에게는 책임 범위와 종료 조건을 명확히 넘기고, 메인 에이전트에는 핵심 사실과 결과만 다시 모은다.
- {harness_note}
"""


def render_minimax_agents(args: argparse.Namespace, paths: Paths, context: dict[str, object]) -> str:
    """Render ``MiniMax.md`` — the MiniMax Code harness entry file."""
    harness_note = (
        "기존 코드베이스 분석 결과를 반영한 초안이다. 추정 명령과 문서 경로는 실제 저장소 기준으로 수정할 수 있다."
        if args.adoption_mode == "existing"
        else "신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다."
    )
    smoke_check = context['smoke_check_command']
    if "TODO" in smoke_check:
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""# MiniMax.md

- 문서 목적: MiniMax Code(Mavis / 미니맥스 코드) 하네스가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령, 오케스트레이터/워커 운영 원칙
- 대상 독자: MiniMax Code, 저장소 관리자, 멀티 에이전트 운영자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `ai-workflow/memory/PROJECT_PROFILE.md`, `AGENTS.md`

## 목적

이 저장소에서는 **Standard AI Workflow**를 기준으로 작업한다. 세션 시작, backlog 갱신, 문서 동기화, 세션 종료는 `ai-workflow/` 아래 문서를 우선 기준으로 삼는다. MiniMax Code는 메인 orchestrator로 동작하고, doc/code/validation worker에 bounded scope 작업을 위임해 컨텍스트를 절약한다.

## 항상 먼저 읽을 문서

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- `AGENTS.md` (워크플로우 규칙 요약)

`ai-workflow/` 는 세션 복원과 workflow 상태 관리용 메타 레이어다. 프로젝트 코드나 프로젝트 문서를 탐색할 때는 이 경로를 기본 탐색 범위에 넣지 말고, workflow 문서 자체를 갱신하거나 현재 세션 상태를 복원할 때만 예외적으로 참조한다.

## 작업 원칙

- 작업을 시작하기 전에 목적, 범위, 영향 문서를 짧게 정리한다.
- 작업 상태는 `planned`, `in_progress`, `blocked`, `done` 중 하나로 관리한다.
- 검증하지 않은 결과는 완료로 확정하지 않는다.
- 세션 종료 전에는 `state.json`, `session_handoff.md`, 최신 backlog 를 갱신한다.
- 가능한 한 메인 orchestrator는 조정과 통합에 집중하고, 도구 호출/탐색/수정은 `.MiniMax/agents/workflow-*.md` 워커에 위임한다.

## 오케스트레이터 / 워커 운영 원칙 (Multi-Agent Topology)

- **Orchestrator (Mavis / 미니맥스 코드 메인 에이전트)**: 사용자 직접 소통, 작업 분해, 워커 호출/통합, `state.json`/`session_handoff`/`work_backlog` 동기화 전담. 도구 호출을 직접 떠안지 않는다.
- **doc-worker**: 문서 링크/메타데이터/카탈로그 정합성 작업. `ai-workflow/skills/doc-sync`, `merge-doc-reconcile`, `workflow-linter` 호출.
- **code-worker**: 코드 수정/리팩토링 작업. `ai-workflow/skills/code-index-update`, `robust-patcher` 호출. 출력 파일 범위는 `output_files` 명시.
- **validation-worker**: 테스트/스모크 실행 및 결과 기록. `ai-workflow/skills/validation-plan`, `ai-workflow/tests/check_*.py` 호출.

워커에 작업을 위임할 때는 `WorkerTask` (worker_id, task_description, input_files, output_files, constraints, context_summary) 형식으로 의도와 책임 경계를 명확히 적는다. 결과는 `WorkerResponse` (status, summary, produced_artifacts, risks_identified, suggested_follow_up) 형식으로 받는다.

## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 프로젝트 실행 기본값

- 설치: `{context['install_command']}`
- 로컬 실행: `{context['run_command']}`
- 빠른 테스트: `{context['quick_test_command']}`
- 격리 테스트: `{context['isolated_test_command']}`
- 실행 확인: `{smoke_check}`

## 문서 작업 기준

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- backlog 위치: `{context['backlog_dir']}`
- session handoff 위치: `{context['session_doc_path']}`

## MiniMax Code 전용 메모

- MiniMax Code는 `MiniMax.md` 와 `AGENTS.md` 모두를 진입점으로 활용한다. 시스템 정책과 충돌할 경우 MiniMax.md 가 우선하되, 두 문서가 같은 사실을 가리키는 방향으로 동기화한다.
- `minimax_config_example.json` 는 사용자 환경 설정(`~/.MiniMax/config.json` 또는 프로젝트 로컬 `.MiniMax/config.json`)에 복사해 사용한다. 서버 토큰 등은 직접 채워 넣는다.
- 워커 호출 시 위험한 외부 작업(예: 데이터베이스 마이그레이션, 프로덕션 배포, 시크릿 회전)은 사용자 명시적 승인을 먼저 받는다.
- {harness_note}
"""


def render_minimax_config_example() -> str:
    """Render a ``MiniMax_config.example.json`` snippet for the user to copy.

    The values are intentionally placeholders; users fill in their own MCP
    server tokens, project name, and harness-specific options.
    """
    return """{
  "$schema": "https://MiniMax.dev/schema/config.json",
  "project_name": "Standard AI Workflow Project",
  "language": "ko-KR",
  "agents": {
    "workflow-orchestrator": {
      "file": ".MiniMax/agents/workflow-orchestrator.md",
      "role": "orchestrator"
    },
    "workflow-worker": {
      "file": ".MiniMax/agents/workflow-worker.md",
      "role": "worker"
    },
    "workflow-doc-worker": {
      "file": ".MiniMax/agents/workflow-doc-worker.md",
      "role": "doc-worker"
    },
    "workflow-code-worker": {
      "file": ".MiniMax/agents/workflow-code-worker.md",
      "role": "code-worker"
    },
    "workflow-validation-worker": {
      "file": ".MiniMax/agents/workflow-validation-worker.md",
      "role": "validation-worker"
    }
  },
  "mcp_servers": {
    "standard-ai-workflow-readonly": {
      "command": "python3",
      "args": ["-m", "workflow_kit.server.read_only_jsonrpc", "--stdio-lines"],
      "env": {
        "PYTHONPATH": "./workflow-source"
      },
      "transport_ready": false,
      "description": "Read-only MCP draft fixture for the standard workflow kit. See workflow-source/schemas/read_only_transport_descriptors.json."
    }
  },
  "workflow": {
    "memory_dir": "ai-workflow/memory",
    "session_handoff_path": "ai-workflow/memory/session_handoff.md",
    "work_backlog_index_path": "ai-workflow/memory/work_backlog.md",
    "project_profile_path": "docs/PROJECT_PROFILE.md",
    "state_json_path": "ai-workflow/memory/state.json"
  },
  "session_protocol": {
    "language": "ko-KR",
    "auto_refresh_state": true,
    "require_handoff_before_done": true
  }
}
"""


def render_minimax_orchestrator(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the orchestrator overlay describing the orchestrator's role."""
    return f"""# workflow-orchestrator

- 문서 목적: MiniMax Code 메인 orchestrator 페르소나의 책임/경계/산출물을 정의한다.
- 범위: 작업 분해, 워커 위임, handoff/state 동기화, 사용자 보고
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `../../../MiniMax.md`, `../../../AGENTS.md`, `workflow-worker.md`

## 책임

1. 사용자 요청을 받아 bounded-scope 작업 단위로 분해한다.
2. 각 작업을 `WorkerTask` 형식으로 워커(doc/code/validation)에 위임한다.
3. 워커의 `WorkerResponse` 를 모아서 `state.json` / `session_handoff.md` / 최신 `backlog` 를 갱신한다.
4. 사용자에게 한국어로 짧은 진행 보고와 다음 행동을 안내한다.

## 절대 하지 말 것

- 직접 `read_file` / `edit_file` 로 프로젝트 코드를 수정하지 않는다. (code-worker에 위임)
- 직접 `bash` 로 테스트/스모크를 실행하지 않는다. (validation-worker에 위임)
- 워커가 보고한 사실 외에 추측성 결론을 추가하지 않는다.

## 종료 조건

- 모든 위임 작업이 `WorkerResponse.status == "ok"` 또는 명시적 blocked 사유와 함께 반환됨
- `state.json` 의 `session.last_orchestrator_action` 이 이번 세션의 최종 행동으로 갱신됨
- `session_handoff.md` 의 "다음 세션 시작 포인트" 가 한 문장으로 갱신됨
"""


def render_minimax_worker(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the generic worker overlay (sub-agent operating contract)."""
    return f"""# workflow-worker

- 문서 목적: MiniMax Code sub-worker 의 공통 운영 계약을 정의한다.
- 범위: 입력, 책임, 산출물, 통신 형식
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `../../../workflow-source/core/workflow_agent_topology.md`, `../../../workflow-source/prompts/code_worker_prompt.md`, `../../../workflow-source/prompts/doc_worker_prompt.md`, `../../../workflow-source/prompts/validation_worker_prompt.md`

## 입력

- orchestrator 가 위임한 `WorkerTask` (worker_id, task_description, input_files, output_files, constraints, context_summary)

## 책임

1. `output_files` 명시 범위 내에서만 변경한다.
2. 변경 후 `produced_artifacts`, `risks_identified`, `suggested_follow_up` 을 함께 보고한다.
3. 정적 검증 실패나 외부 시스템 호출이 필요하면 validation-worker에 협업 위임한다.

## 절대 하지 말 것

- 다른 워커의 `output_files` 를 수정하지 않는다.
- 명시되지 않은 의존성 추가/제거를 하지 않는다.

## 산출물

- `WorkerResponse` (status, summary, produced_artifacts, risks_identified, suggested_follow_up, raw_worker_output)
"""


def render_minimax_doc_worker(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the doc-worker overlay."""
    return f"""# workflow-doc-worker

- 문서 목적: MiniMax Code doc-worker 페르소나의 책임/산출물을 정의한다.
- 범위: 문서 정합성, 메타데이터, 링크, 카탈로그 동기화
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `workflow-worker.md`, `../../../workflow-source/prompts/doc_worker_prompt.md`

## 책임

1. `doc-sync` 스킬로 변경된 코드/문서가 영향 받는 문서를 식별하고 recommended review order 를 만든다.
2. `merge-doc-reconcile` 스킬로 충돌한 handoff/state/backlog 를 정리한다.
3. `workflow-linter` 스킬로 메타데이터/링크/카탈로그 정합성을 검사하고 복구한다.
4. 결과는 `output_files` 안의 문서들에 한정해 직접 수정한다.

## 금지

- 코드를 수정하지 않는다 (code-worker 영역)
- `backlog-update` 로 상태를 갱신할 때는 orchestrator 에게 명시적 위임을 요청한다
"""


def render_minimax_code_worker(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the code-worker overlay."""
    return f"""# workflow-code-worker

- 문서 목적: MiniMax Code code-worker 페르소나의 책임/산출물을 정의한다.
- 범위: 코드 구현, 정밀 리팩토링, 회귀 수정
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `workflow-worker.md`, `../../../workflow-source/prompts/code_worker_prompt.md`

## 책임

1. orchestrator 가 위임한 bounded scope 안에서만 코드를 수정한다.
2. `code-index-update` 스킬로 코드 인덱스/카탈로그를 동기화한다.
3. `robust_patcher` 스킬로 정밀 패치를 적용한다.
4. 변경 후 `produced_artifacts` 에 실제 변경한 파일 목록을 남긴다.

## 금지

- 명시되지 않은 파일을 수정하지 않는다.
- 의존성 추가/제거는 orchestrator 의 명시적 승인 없이 하지 않는다.
"""


def render_minimax_validation_worker(args: argparse.Namespace, context: dict[str, object]) -> str:
    """Render the validation-worker overlay."""
    return f"""# workflow-validation-worker

- 문서 목적: MiniMax Code validation-worker 페르소나의 책임/산출물을 정의한다.
- 범위: 테스트/스모크 실행, 결과 기록
- 대상 독자: MiniMax Code, 멀티 에이전트 운영자
- 상태: stable
- 최종 수정일: {args.today}
- 관련 문서: `workflow-worker.md`, `../../../workflow-source/prompts/validation_worker_prompt.md`

## 책임

1. `validation-plan` 스킬로 변경 사항에 적합한 검증 단계를 결정한다.
2. `ai-workflow/tests/check_*.py` 와 같은 스모크/테스트 스크립트를 실행한다.
3. 실행 결과를 `passed`, `failed`, `skipped` 로 명확히 분류하고, 실패 시 raw stderr 를 `risks_identified` 에 첨부한다.

## 금지

- 코드/문서를 직접 수정하지 않는다.
- 외부 시스템 호출은 orchestrator 의 명시적 승인을 받는다.
"""


def render_codex_agents(args: argparse.Namespace, paths: Paths, context: dict[str, object]) -> str:
    harness_note = (
        "기존 코드베이스 분석 결과를 반영한 초안이다. 추정 명령과 문서 경로는 실제 저장소 기준으로 수정할 수 있다."
        if args.adoption_mode == "existing"
        else "신규 프로젝트 기준 초안이다. 프로젝트 고유의 실행 명령과 문서 구조가 정확한지 확인해야 한다."
    )
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in smoke_check:
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""# AGENTS.md

- 문서 목적: Codex 가 이 저장소에서 먼저 읽어야 할 workflow 진입 규칙과 기본 작업 원칙을 제공한다.
- 범위: 세션 복원, workflow state docs 참조 순서, 사용자 보고 언어, 기본 실행/검증 명령
- 대상 독자: Codex, 저장소 관리자, workflow 설계자
- 상태: draft
- 최종 수정일: {args.today}
- 관련 문서: `ai-workflow/memory/state.json`, `ai-workflow/memory/session_handoff.md`, `ai-workflow/memory/work_backlog.md`, `ai-workflow/memory/PROJECT_PROFILE.md`

## 목적

이 저장소에서는 표준 AI 워크플로우를 기준으로 작업한다. 세션 시작, backlog 갱신, 문서 동기화, 세션 종료는 `ai-workflow/` 아래 문서를 우선 기준으로 삼는다.

## 항상 먼저 읽을 문서

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

`ai-workflow/` 는 세션 복원과 workflow 상태 관리용 메타 레이어다. 프로젝트 코드나 프로젝트 문서를 탐색할 때는 이 경로를 기본 탐색 범위에 넣지 말고, workflow 문서 자체를 갱신하거나 현재 세션 상태를 복원할 때만 예외적으로 참조한다.

## 작업 원칙

- 작업을 시작하기 전에 목적, 범위, 영향 문서를 짧게 정리한다.
- 작업 상태는 `planned`, `in_progress`, `blocked`, `done` 중 하나로 관리한다.
- 검증하지 않은 결과는 완료로 확정하지 않는다.
- 세션 종료 전에는 `state.json`, `session_handoff.md`, 최신 backlog 를 갱신한다.

## 언어와 컨텍스트 원칙

- 사용자에게 직접 보이는 작업 보고, 상태 요약, 문서 갱신 문안은 기본적으로 한국어로 작성한다.
- 코드, 명령어, 파일 경로, 설정 key, 외부 시스템 고유 명칭은 필요할 때 원문 그대로 유지한다.
- 내부 사고 과정과 임시 분류는 모델이 가장 효율적인 방식으로 처리하되, 사용자에게는 필요한 결론과 다음 행동만 짧게 전달한다.
- 장문의 중간 reasoning, 중복 요약, 불필요한 자기 설명을 피한다.
- handoff 와 backlog 에는 다음 세션에 필요한 핵심 사실만 남겨 불필요한 컨텍스트 누적을 줄인다.

## 프로젝트 실행 기본값

- 설치: `{context['install_command']}`
- 로컬 실행: `{context['run_command']}`
- 빠른 테스트: `{context['quick_test_command']}`
- 격리 테스트: `{context['isolated_test_command']}`
- 실행 확인: `{smoke_check}`

## 문서 작업 기준

- 문서 위키 홈: `{context['doc_home']}`
- 운영 문서 위치: `{context['operations_dir']}`
- backlog 위치: `{context['backlog_dir']}`
- session handoff 위치: `{context['session_doc_path']}`

## Codex 전용 메모

- Codex 는 프로젝트 루트의 `AGENTS.md` 를 읽으므로, 상세 정책은 본 문서에서 시작하고 세부 운영 기준은 `ai-workflow/` 문서를 참조한다.
- OpenAI 관련 질문이 나오면 OpenAI 문서 MCP 를 우선 사용하는 구성을 권장한다.
- 가능한 경우 메인 에이전트는 조정과 통합에 집중하고, bounded scope 의 읽기/쓰기/검증 작업은 worker 성격의 서브 에이전트로 분리하는 패턴을 권장한다.
- worker 에게는 책임 파일과 종료 조건을 명확히 넘기고, 메인 에이전트에는 핵심 사실과 결과만 다시 모은다.
- `main`/`small` 모델을 함께 운영한다면, 메인 에이전트는 난도 높은 판단과 통합에, worker 는 bounded scope 탐색/초안/검증에 우선 배치하는 편이 효율적이다.
- {harness_note}
"""


def render_codex_config_example() -> str:
    return """# Merge this into ~/.codex/config.toml if you want Codex-wide defaults.

[mcp_servers.openaiDeveloperDocs]
url = "https://developers.openai.com/mcp"
default_tools_approval_mode = "approve"
supports_parallel_tool_calls = true
"""


def render_opencode_config(args: argparse.Namespace, paths: Paths) -> str:
    instructions = [
        "AGENTS.md",
        f"{rel(paths.state_path, paths.target_root)}",
        f"{rel(paths.profile_path, paths.target_root)}",
        f"{rel(paths.handoff_path, paths.target_root)}",
        f"{rel(paths.backlog_index_path, paths.target_root)}",
    ]
    return json.dumps(
        {
            "$schema": "https://opencode.ai/config.json",
            "instructions": instructions,
            "agent": {
                "workflow-orchestrator": {
                    "mode": "primary",
                    "description": "Standard AI workflow orchestrator for this project",
                    "prompt": "{file:.opencode/agents/workflow-orchestrator.md}",
                    "permission": {
                        "task": {
                            "*": "deny",
                            "workflow-*": "allow",
                        }
                    },
                },
                "workflow-worker": {
                    "description": "Scoped worker for implementation, draft writing, and verification tasks",
                    "prompt": "{file:.opencode/agents/workflow-worker.md}",
                },
                "workflow-doc-worker": {
                    "description": "Scoped worker for document reading, comparison, and draft updates",
                    "prompt": "{file:.opencode/agents/workflow-doc-worker.md}",
                },
                "workflow-code-worker": {
                    "description": "Scoped worker for bounded code edits and implementation tasks",
                    "prompt": "{file:.opencode/agents/workflow-code-worker.md}",
                },
                "workflow-validation-worker": {
                    "description": "Scoped worker for checks, logs, and validation evidence collection",
                    "prompt": "{file:.opencode/agents/workflow-validation-worker.md}",
                }
            },
            "mcp_servers": {
                "openaiDeveloperDocs": {
                    "type": "remote",
                    "url": "https://developers.openai.com/mcp",
                }
            },
        },
        ensure_ascii=False,
        indent=2,
    ) + "\n"


def render_opencode_skill() -> str:
    return """---
name: standard-ai-workflow
description: Load the project workflow docs before starting or updating work in this repository.
---

# Standard AI Workflow

Use this skill when you need to start a session, update backlog state, sync documents, or prepare a handoff.

Always read:

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

If the repository is still in adoption, also read:

- `ai-workflow/memory/repository_assessment.md`

Follow these rules:

- Write user-facing status updates, work reports, and document drafts in Korean by default.
- Keep code, commands, file paths, config keys, and external product names in their original form when needed.
- Brief the task before editing files.
- Keep task status aligned with backlog records.
- Do not mark work done without validation evidence.
- Update `state.json`, the handoff, and the latest backlog before ending a session.
- Keep internal reasoning and intermediate classification compact, and avoid long repeated explanations to the user.
- Leave only essential facts in handoff/backlog so session context stays lean.
- Treat `ai-workflow/` as workflow metadata only. Ignore it during normal project document exploration unless the task is explicitly about workflow docs or session state.
"""


def render_opencode_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in str(smoke_check):
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""---
description: Orchestrates the standard AI workflow for this repository
mode: primary
permission:
  edit: deny
  bash: deny
  webfetch: deny
---

You are the workflow orchestrator for this repository.

Start each substantial task by reading:

- `AGENTS.md`
- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`

Treat `ai-workflow/` as a workflow metadata layer, not part of the normal project work scope. After session restoration, ignore it during project code or project document exploration unless the task explicitly asks for workflow doc maintenance.

You may directly read only the minimum session-restoration set and tiny triage inputs:

- `ai-workflow/memory/state.json`
- `ai-workflow/memory/session_handoff.md`
- `ai-workflow/memory/work_backlog.md`
- `ai-workflow/memory/PROJECT_PROFILE.md`
- one clearly bounded file or path for tiny triage

Project defaults:

- Install: `{context['install_command']}`
- Run: `{context['run_command']}`
- Quick test: `{context['quick_test_command']}`
- Isolated test: `{context['isolated_test_command']}`
- Smoke check: `{smoke_check}`

When the repo is in adoption mode, review `ai-workflow/memory/repository_assessment.md` before trusting inferred commands.

User-facing workflow rules:

- Write visible work reports, summaries, and document drafts in Korean by default.
- Keep code, commands, file paths, config keys, and external system names in their original form when useful.
- Use concise progress updates and avoid long repeated reasoning in user-visible messages.
- Keep internal processing compact and preserve only the facts needed for the next step or next session.
- Do not call direct tools yourself. Use only task delegation for repository exploration, comparisons, implementation, checks, and draft generation.
- Use sub-agents aggressively for file exploration, comparisons, log inspection, and draft generation when that helps reduce context pollution.
- Keep the main orchestrator focused on coordination, prioritization, integration, and the final user-facing report.
- Separate broad read-heavy exploration from write tasks when possible so one stream of work does not pollute another stream's context.
- Treat this agent as a read-mostly coordinator with task-only execution: delegate edits, scans, log review, and validation to sub-agents instead of making exceptions for direct tool use.
- Keep direct read narrow: after the session-restoration set, only tiny single-file or single-path triage reads stay local; broader reading goes to workers.
- Ask the user only when a missing decision is genuinely blocking or a risky external action needs confirmation; otherwise make the smallest reasonable assumption and continue through a worker.
- When delegating, give each worker a bounded scope, clear output, and a concise completion contract.
- Prefer `workflow-doc-worker` for large document reads and draft updates, `workflow-code-worker` for bounded implementation, config edits, and build-oriented tasks, and `workflow-validation-worker` for checks and evidence collection.
- If your harness supports per-agent model selection, prefer the main model for this orchestrator and a smaller model for the worker agents by default.
- Do not treat `ai-workflow/` as part of normal project document discovery. Use it only for workflow-state restoration or explicit workflow-maintenance tasks.
"""


def render_opencode_worker_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in str(smoke_check):
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""---
description: Executes bounded workflow tasks for this repository
mode: subagent
permission:
  edit: allow
  bash: allow
  webfetch: allow
---

You are a workflow worker for this repository.

You are not the main orchestrator. Your role is to execute a tightly scoped task and return only the essential result.

Before starting, read only the minimum relevant context:

- `AGENTS.md`
- `ai-workflow/memory/state.json` when it helps restore the current task baseline quickly
- the specific `ai-workflow/memory/` document or file paths that match your assigned scope

Project defaults:

- Install: `{context['install_command']}`
- Run: `{context['run_command']}`
- Quick test: `{context['quick_test_command']}`
- Isolated test: `{context['isolated_test_command']}`
- Smoke check: `{smoke_check}`

Worker rules:

- Stay within the assigned file or task scope.
- Prefer doing the actual bounded work instead of producing long plans.
- Summarize only the key facts, edits, risks, and follow-up items needed by the orchestrator.
- Avoid pasting large raw outputs when a short summary is enough.
- If you edit files, keep changes narrow and do not expand into unrelated cleanup.
- If you run checks, report only the command intent and the result that matters.
- Write user-facing drafts in Korean by default unless the assigned task clearly requires another language.
- Minimize asks during execution. Proceed with the smallest reasonable assumption unless the orchestrator explicitly requested a decision point.
- Ignore `ai-workflow/` during normal project document or source exploration unless the assigned task explicitly targets workflow docs or session-state updates.
"""


def render_opencode_doc_worker_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    return f"""---
description: Executes bounded document-focused workflow tasks for this repository
mode: subagent
permission:
  edit: allow
  bash: allow
  webfetch: allow
---

You are a document-focused workflow worker for this repository.

Your role is to read, compare, summarize, and update a tightly scoped set of documents without pulling unrelated context into the main orchestrator.

Before starting, read only the minimum relevant context:

- `AGENTS.md`
- `ai-workflow/memory/state.json` when it helps restore the current task baseline quickly
- the assigned `ai-workflow/memory/` documents or directly named doc paths

Worker rules:

- Stay within the assigned document scope.
- Prefer concise comparisons, change notes, and draft text over long quotations.
- Return only the facts, inconsistencies, draft wording, and follow-up items needed by the orchestrator.
- Keep user-facing drafts in Korean by default.
- Minimize asks during execution and resolve obvious document-structure choices locally when risk is low.
- If your harness supports per-agent model selection, this worker is a good default target for a smaller model.
- Ignore `ai-workflow/` when looking for project documentation unless the assigned task is explicitly about workflow docs or session-state maintenance.
"""


def render_opencode_code_worker_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in str(smoke_check):
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""---
description: Executes bounded implementation and build-focused workflow tasks for this repository
mode: subagent
permission:
  edit: allow
  bash: allow
  webfetch: allow
---

You are an implementation and build-focused workflow worker for this repository.

Your role is to implement a tightly scoped code or config change, run the minimum relevant build-oriented checks when needed, and report only the essential result back to the orchestrator.

Before starting, read only the minimum relevant context:

- `AGENTS.md`
- `ai-workflow/memory/state.json` when it helps restore the current task baseline quickly
- the specific source files, tests, and workflow docs tied to your assigned scope

Project defaults:

- Install: `{context['install_command']}`
- Run: `{context['run_command']}`
- Quick test: `{context['quick_test_command']}`
- Isolated test: `{context['isolated_test_command']}`
- Smoke check: `{smoke_check}`

Worker rules:

- Stay within the assigned write scope.
- Prefer shipping the bounded change over expanding into adjacent cleanup.
- Treat build, compile, package, or asset-generation commands as part of your default scope when they are the shortest path to proving the implementation still holds.
- If you run checks, report what matters: pass/fail, key regression risk, build impact, and any deferred follow-up.
- Avoid broad repository exploration unless explicitly assigned.
- Minimize asks during execution. Make bounded implementation choices locally unless the change would alter product behavior or ownership boundaries.
- If your harness supports per-agent model selection, use a smaller model for routine edits and reserve the main model for unusually risky or architectural code tasks.
- Ignore `ai-workflow/` during normal implementation-context discovery unless the assigned task explicitly targets workflow docs or workflow automation.
"""


def render_opencode_validation_worker_agent(args: argparse.Namespace, context: dict[str, object]) -> str:
    # Ensure smoke check has a sensible default if still TODO
    smoke_check = context['smoke_check_command']
    if "TODO"in str(smoke_check):
        if context['primary_stack'] == 'python':
            smoke_check = "python3 --version"
        elif context['primary_stack'] == 'node':
            smoke_check = "node --version"

    return f"""---
description: Executes bounded validation and evidence-collection tasks for this repository
mode: subagent
permission:
  edit: allow
  bash: allow
  webfetch: allow
---

You are a validation-focused workflow worker for this repository.

Your role is to run bounded checks, inspect logs, gather evidence, and return a compact validation summary to the orchestrator.

Before starting, read only the minimum relevant context:

- `AGENTS.md`
- `ai-workflow/memory/state.json` when it helps restore the current task baseline quickly
- the assigned validation scope, commands, and relevant backlog or handoff notes

Project defaults:

- Quick test: `{context['quick_test_command']}`
- Isolated test: `{context['isolated_test_command']}`
- Smoke check: `{smoke_check}`

Worker rules:

- Stay within the assigned validation scope and command set.
- Report only the result that matters: what ran, what failed or passed, and what evidence should be recorded.
- Avoid flooding the orchestrator with raw logs when a short summary is enough.
- Minimize asks during execution and complete the assigned checks unless the environment is genuinely blocked.
- If your harness supports per-agent model selection, this worker is usually a strong candidate for a smaller model.
- Ignore `ai-workflow/` during normal validation-context discovery unless the assigned task explicitly targets workflow docs or session-state verification.
"""


def write_codex_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    codex_config = codex_config_example_path(paths)
    write_text(codex_config, render_codex_config_example(), force=args.force)
    return {
        "codex_config_example": str(codex_config),
    }


def write_opencode_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    opencode_config = opencode_config_path(paths)
    opencode_skill = opencode_skill_path(paths)
    opencode_agent = opencode_agent_path(paths)
    opencode_worker_agent = opencode_worker_agent_path(paths)
    opencode_doc_worker_agent = opencode_doc_worker_agent_path(paths)
    opencode_code_worker_agent = opencode_code_worker_agent_path(paths)
    opencode_validation_worker_agent = opencode_validation_worker_agent_path(paths)
    write_text(opencode_config, render_opencode_config(args, paths), force=args.force)
    write_text(opencode_skill, render_opencode_skill(), force=args.force)
    write_text(opencode_agent, render_opencode_agent(args, context), force=args.force)
    write_text(opencode_worker_agent, render_opencode_worker_agent(args, context), force=args.force)
    write_text(opencode_doc_worker_agent, render_opencode_doc_worker_agent(args, context), force=args.force)
    write_text(opencode_code_worker_agent, render_opencode_code_worker_agent(args, context), force=args.force)
    write_text(
        opencode_validation_worker_agent,
        render_opencode_validation_worker_agent(args, context),
        force=args.force,
    )
    return {
        "opencode_config": str(opencode_config),
        "opencode_skill": str(opencode_skill),
        "opencode_agent": str(opencode_agent),
        "opencode_worker_agent": str(opencode_worker_agent),
        "opencode_doc_worker_agent": str(opencode_doc_worker_agent),
        "opencode_code_worker_agent": str(opencode_code_worker_agent),
        "opencode_validation_worker_agent": str(opencode_validation_worker_agent),
    }


def write_gemini_cli_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    # GEMINI.md is written in write_harness_files if selected,
    # but we can also do it here if we want to be explicit or if we change write_harness_files.
    # Currently write_harness_files writes it.
    return {}


def render_pi_dev_agents(args: argparse.Namespace, context: dict[str, object]) -> str:
    return f"""# AGENTS.md (Pi Coding Agent Profile)

- **Mandate**: 본 저장소는 'Standard AI Workflow'를 따릅니다. 모든 행동은 아래 문서의 상태를 기준으로 결정하십시오.
- **Priority Docs**:
    1. `ai-workflow/memory/state.json` (현재 세션의 진실의 원천)
    2. `ai-workflow/memory/session_handoff.md` (이전 세션 인계 사항)
    3. `ai-workflow/memory/work_backlog.md` (작업 목록)

## 1. 세션 시작 루틴 (Mandatory)
세션이 시작되면 가장 먼저 `ai-workflow/memory/state.json`을 읽고 `current_focus`와 `next_documents`를 파악하십시오. 이후 `session_handoff.md`를 읽어 중단된 지점부터 작업을 재개하십시오.

## 2. 작업 원칙 (Research -> Strategy -> Execution)
- **Research**: `grep_search`와 `read_file`을 사용하여 현재 코드와 문서 상태를 객관적으로 확인하십시오.
- **Strategy**: 변경 계획을 세우고, 작업 전후에 어떤 문서를 갱신할지 결정하십시오.
- **Execution**: `edit`, `write`, `bash` 도구를 사용하여 변경을 수행하십시오.

## 3. 워크플로우 상태 관리
- 작업 상태가 변경되면 반드시 `ai-workflow/memory/backlog/`의 해당 날짜 문서를 업데이트하십시오.
- 세션 종료 전에는 `ai-workflow/memory/state.json`과 `session_handoff.md`를 갱신하여 다음 에이전트를 위한 맥락을 보존하십시오.

## 4. 도구 사용 가이드
- 복잡한 워크플로우 제어(상태 자동 갱신 등)가 필요할 때 `python3 ai-workflow/scripts/` 아래의 도구들을 활용할 수 있습니다.
- 모든 도구 호출 결과는 구조화된 JSON으로 처리하는 것을 선호합니다.

## 5. 언어 가이드
- 사용자에게 보고하거나 문서를 작성할 때는 한국어를 사용하십시오.
- 코드와 기술적 명칭은 원문을 유지하십시오.
"""


def write_pi_dev_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    # Pi Coding Agent primarily uses AGENTS.md at the root
    # We will also create a pi-dev specific apply guide if possible
    return {}


def write_antigravity_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    return {}


#: Build the in-process harness dispatcher. Each entry maps a harness name to
#: its ``write_*_harness_files`` implementation. ``bootstrap_harnesses`` holds
#: the registry; here we just populate it.
HARNESS_FILE_BUILDERS = HARNESS_FILE_BUILDERS
register_harness_builder("codex", write_codex_harness_files)
register_harness_builder("opencode", write_opencode_harness_files)
register_harness_builder("gemini-cli", write_gemini_cli_harness_files)
register_harness_builder("pi-dev", write_pi_dev_harness_files)
register_harness_builder("antigravity", write_antigravity_harness_files)


def write_minimax_code_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    """Generate MiniMax Code harness overlay files.

    The overlay mirrors the OpenCode orchestrator + worker split but pins the
    entry files at ``AGENTS.md`` and ``MiniMax.md`` (the latter is the
    MiniMax Code-specific entry point) and emits a JSON config example so
    the user can drop the snippet into their ``~/.MiniMax/config.json`` or
    a project-local ``.MiniMax/config.json`` without further editing.
    """
    generated: dict[str, str] = {}

    minimax_root = paths.target_root / ".MiniMax"
    minimax_config = paths.target_root / "MiniMax_config.example.json"
    minimax_orchestrator = minimax_root / "agents" / "workflow-orchestrator.md"
    minimax_worker = minimax_root / "agents" / "workflow-worker.md"
    minimax_doc_worker = minimax_root / "agents" / "workflow-doc-worker.md"
    minimax_code_worker = minimax_root / "agents" / "workflow-code-worker.md"
    minimax_validation_worker = minimax_root / "agents" / "workflow-validation-worker.md"

    write_text(minimax_config, render_minimax_config_example(), force=args.force)
    generated["minimax_config_example"] = str(minimax_config)
    write_text(minimax_orchestrator, render_minimax_orchestrator(args, context), force=args.force)
    generated["minimax_orchestrator"] = str(minimax_orchestrator)
    write_text(minimax_worker, render_minimax_worker(args, context), force=args.force)
    generated["minimax_worker"] = str(minimax_worker)
    write_text(minimax_doc_worker, render_minimax_doc_worker(args, context), force=args.force)
    generated["minimax_doc_worker"] = str(minimax_doc_worker)
    write_text(minimax_code_worker, render_minimax_code_worker(args, context), force=args.force)
    generated["minimax_code_worker"] = str(minimax_code_worker)
    write_text(minimax_validation_worker, render_minimax_validation_worker(args, context), force=args.force)
    generated["minimax_validation_worker"] = str(minimax_validation_worker)
    return generated


register_harness_builder("minimax-code", write_minimax_code_harness_files)


def write_harness_files(
    args: argparse.Namespace,
    paths: Paths,
    context: dict[str, object],
) -> dict[str, str]:
    generated: dict[str, str] = {}
    harnesses = selected_harnesses(args)

    if "codex"in harnesses or "opencode"in harnesses:
        codex_agents = codex_agents_path(paths)
        write_text(codex_agents, render_codex_agents(args, paths, context), force=args.force)
        generated["codex_agents"] = str(codex_agents)

    if "pi-dev"in harnesses:
        # Pi Coding Agent uses AGENTS.md at the root, same path as codex
        # If both are selected, Pi's version will overwrite or vice versa.
        # Usually only one harness is selected for a project.
        pi_agents = codex_agents_path(paths)
        write_text(pi_agents, render_pi_dev_agents(args, context), force=args.force)
        generated["pi_dev_agents"] = str(pi_agents)

    if "gemini-cli"in harnesses:
        gemini_agents = gemini_cli_agents_path(paths)
        write_text(gemini_agents, render_gemini_cli_agents(args, paths, context), force=args.force)
        generated["gemini_cli_agents"] = str(gemini_agents)

    if "antigravity"in harnesses:
        antigravity_agents = antigravity_agents_path(paths)
        write_text(antigravity_agents, render_antigravity_agents(args, paths, context), force=args.force)
        generated["antigravity_agents"] = str(antigravity_agents)

    if "minimax-code" in harnesses:
        # MiniMax Code shares AGENTS.md with codex/opencode but writes its own
        # MiniMax.md entry file. We do NOT route through the codex/opencode
        # block above because the AGENTS.md text differs slightly.
        minimax_agents = minimax_agents_path(paths)
        write_text(minimax_agents, render_minimax_agents(args, paths, context), force=args.force)
        generated["minimax_code_agents"] = str(minimax_agents)

    for harness in harnesses:
        builder = HARNESS_FILE_BUILDERS[harness]
        generated.update(builder(args, paths, context))
    return generated


def build_manifest(
    args: argparse.Namespace,
    paths: Paths,
    core_docs: list[str],
    context: dict[str, object],
    harness_files: dict[str, str],
    dependency_files: list[str] = None,
) -> dict[str, object]:
    selected_snippets = {
        harness: global_snippet_sources()[harness]
        for harness in selected_harnesses(args)
        if harness in global_snippet_sources()
    }
    generated_files: dict[str, str] = {
        "readme": str(paths.readme_path),
        "project_profile": str(paths.profile_path),
        "workflow_state": str(paths.state_path),
        "session_handoff": str(paths.handoff_path),
        "work_backlog": str(paths.backlog_index_path),
        "daily_backlog": str(paths.daily_backlog_path),
        "project_status_assessment": str(paths.status_assessment_path),
    }
    if args.adoption_mode == "existing":
        generated_files["repository_assessment"] = str(paths.assessment_path)
    return {
        "target_root": str(paths.target_root),
        "kit_root": str(paths.kit_root),
        "project_slug": args.project_slug,
        "project_name": args.project_name,
        "adoption_mode": args.adoption_mode,
        "harnesses": selected_harnesses(args),
        "analysis_summary": context["analysis_summary"],
        "generated_files": generated_files,
        "generated_harness_files": harness_files,
        "updated_dependency_files": dependency_files or [],
        "global_snippet_candidates": selected_snippets,
        "copied_core_docs": core_docs,
        "next_steps": [
            f"Open {rel(paths.profile_path, paths.target_root)} and replace TODO placeholders.",
            f"Refresh {rel(paths.state_path, paths.target_root)} after updating workflow docs.",
            f"Update {rel(paths.handoff_path, paths.target_root)} with the current session baseline.",
            f"Register the next real task in {rel(paths.daily_backlog_path, paths.target_root)}.",
        ]
        + (
            [f"Run `{context.get('install_command')}` to install recommended dependencies."]
            if dependency_files
            else []
        )
        + (
            [f"Review {rel(paths.assessment_path, paths.target_root)} and confirm inferred commands and doc paths."]
            if args.adoption_mode == "existing"
            else []
        )
        + (
            [
                "Review the recommended global snippet before merging it into your harness-wide config."
            ]
            if selected_snippets
            else []
        ),
    }


def main() -> int:
    args = parse_args()
    paths = make_paths(args)
    context = infer_project_context(args, paths)
    core_docs = [str(paths.core_dir / name) for name in DEFAULT_CORE_DOCS] if args.copy_core_docs else []

    # Pre-calculate harness files for manifest
    harnesses = selected_harnesses(args)
    harness_files: dict[str, str] = {}
    if "codex"in harnesses or "opencode"in harnesses:
        harness_files["codex_agents"] = str(codex_agents_path(paths))
    if "gemini-cli"in harnesses:
        harness_files["gemini_cli_agents"] = str(gemini_cli_agents_path(paths))
    if "antigravity"in harnesses:
        harness_files["antigravity_agents"] = str(antigravity_agents_path(paths))

    dependency_files: list[str] = []
    if args.update_deps and not args.dry_run:
        dependency_files = update_dependencies(paths, context, harnesses)

    manifest = build_manifest(args, paths, core_docs, context, harness_files, dependency_files)

    if args.dry_run:
        print(json.dumps(manifest, ensure_ascii=False, indent=2))
        return 0

    try:
        write_text(paths.readme_path, render_readme(args, context), force=args.force)
        write_text(paths.profile_path, render_project_profile(args, context), force=args.force)
        write_text(paths.handoff_path, render_session_handoff(args, context), force=args.force)
        write_text(paths.backlog_index_path, render_backlog_index(args), force=args.force)
        write_text(paths.daily_backlog_path, render_daily_backlog(args, context), force=args.force)
        write_text(paths.status_assessment_path, render_project_status_assessment(args), force=args.force)
        if args.adoption_mode == "existing":
            write_text(paths.assessment_path, render_assessment(args, context), force=args.force)
        write_text(
            paths.state_path,
            json.dumps(
                build_workflow_state_payload(
                    project_profile_path=paths.profile_path,
                    session_handoff_path=paths.handoff_path,
                    work_backlog_index_path=paths.backlog_index_path,
                    latest_backlog_path=paths.daily_backlog_path,
                    repository_assessment_path=paths.assessment_path if args.adoption_mode == "existing"else None,
                    generated_at=args.today,
                    workspace_root=paths.target_root,
                ),
                ensure_ascii=False,
                indent=2,
            )
            + "\n",
            force=args.force,
        )
        harness_files = write_harness_files(args, paths, context)
        if args.copy_core_docs:
            core_docs = copy_core_docs(paths, force=args.force)
        manifest = build_manifest(args, paths, core_docs, context, harness_files, dependency_files)
    except FileExistsError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    print(json.dumps(manifest, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
