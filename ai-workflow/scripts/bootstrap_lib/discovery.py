"""Project discovery helpers used to seed the generated kit.

The bootstrap script needs three things from the *existing* project (if
any) to make the generated kit feel native:

- A list of files inside the project root (to seed the project profile).
- The contents of ``package.json`` (to seed the ``run_command`` heuristic).
- The recommended global snippet paths (so operators can copy them into
  their harness-wide config without a second bootstrap invocation).

These helpers don't depend on argparse, paths, or any other bootstrap
module — they operate purely on the filesystem.

The implementation mirrors the original ``bootstrap_workflow_kit.py``
behaviour exactly (the bootstrap regression test ``check_bootstrap.py``
asserts on the exact structure of the produced output).
"""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any


# Base dependencies for any workflow integration
COMMON_PYTHON_DEPS: list[str] = ["mcp_servers", "pytest", "pytest-asyncio"]
COMMON_NODE_DEPS: list[str] = ["@modelcontextprotocol/sdk"]

# Harness-specific optional dependencies
HARNESS_PYTHON_DEPS: dict[str, list[str]] = {
    "gemini-cli": [],
    "codex": [],
    "opencode": [],
    "antigravity": [],
}
HARNESS_NODE_DEPS: dict[str, list[str]] = {
    "gemini-cli": [],
    "codex": [],
}


#: Directories ignored when walking the target repo.
IGNORED_DIRS: set[str] = {
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


def global_snippet_sources(
    source_root: Path | None,
) -> dict[str, dict[str, str]]:
    """Return the recommended non-invasive global snippet paths to copy.

    The keys are short identifiers used in the generated README and apply guide;
    the values point to the snippet templates that ship with the kit, together
    with the recommended target file path and the merge policy.

    When ``source_root`` is ``None`` (e.g. wheel install without bundled data),
    an empty dict is returned; the bootstrap surfaces a manifest warning so
    operators can install from source to access the snippets.
    """
    if source_root is None:
        return {}
    return {
        "codex": {
            "readme": str((source_root / "global-snippets" / "codex" / "README.md").resolve()),
            "snippet": str((source_root / "global-snippets" / "codex" / "config.toml.snippet").resolve()),
            "target": "~/.codex/config.toml",
            "policy": "additive_only",
        },
        "opencode": {
            "readme": str((source_root / "global-snippets" / "opencode" / "README.md").resolve()),
            "snippet": str((source_root / "global-snippets" / "opencode" / "opencode.global.jsonc").resolve()),
            "target": "~/.config/opencode/opencode.json",
            "policy": "additive_only",
        },
    }


def iter_repo_files(
    root: Path,
    *,
    max_depth: int = 3,
    ignore_dirs: set[str] | None = None,
) -> list[Path]:
    """Walk ``root`` up to ``max_depth`` and return a sorted list of files."""
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
    """Detect npm/pnpm/yarn script entries in ``package.json`` if present."""
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
    """Guess a sensible default ``run_command`` for the target project."""
    for script_name in ("dev", "start", "serve"):
        if script_name in package_scripts:
            return f"npm run {script_name}"
    for candidate in ("app/main.py", "main.py"):
        if (target_root / candidate).exists():
            return f"python {candidate}"
    return "TODO: 로컬 실행 명령 입력"


def value_or_inferred(explicit: str | None, fallback: str) -> str:
    """Return the explicit value unless it is empty or a ``TODO:`` placeholder."""
    if not explicit:
        return fallback
    if explicit.strip().startswith("TODO"):
        return fallback
    return explicit


__all__ = [
    "COMMON_NODE_DEPS",
    "COMMON_PYTHON_DEPS",
    "HARNESS_NODE_DEPS",
    "HARNESS_PYTHON_DEPS",
    "IGNORED_DIRS",
    "detect_package_scripts",
    "global_snippet_sources",
    "guess_run_command",
    "iter_repo_files",
    "value_or_inferred",
]
