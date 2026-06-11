"""Path dataclass and harness overlay path helpers.

The ``Paths`` dataclass captures every well-known file the bootstrap
script touches inside a generated kit. The harness-specific
``*_agents_path`` / ``*_config_path`` helpers turn those into the exact
path a given harness's overlay or MCP config should land at.

These helpers are intentionally self-contained: they take already-resolved
``Paths`` instances so the higher-level ``__main__`` can compose the
plan and the lower-level writers/renderers can stay focused on their
own jobs.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path

#: Directories ignored when discovering project files via
#: :func:`bootstrap_lib.discovery.iter_repo_files`.
IGNORED_DIRS: frozenset[str] = frozenset(
    {
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
)


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
        profile_path=target_root / "docs" / "PROJECT_PROFILE.md",
        state_path=memory_dir / "state.json",
        handoff_path=memory_dir / "session_handoff.md",
        backlog_index_path=memory_dir / "work_backlog.md",
        daily_backlog_path=backlog_dir / f"{args.today}.md",
        assessment_path=memory_dir / "repository_assessment.md",
        status_assessment_path=memory_dir / "project_status_assessment.md",
    )


# ---------------------------------------------------------------------------
# Harness overlay paths
# ---------------------------------------------------------------------------


def codex_agents_path(paths: Paths) -> Path:
    return paths.target_root / "AGENTS.md"


def gemini_cli_agents_path(paths: Paths) -> Path:
    return paths.target_root / "GEMINI.md"


def codex_config_example_path(paths: Paths) -> Path:
    return paths.target_root / ".codex" / "config.toml.example"


def opencode_config_path(paths: Paths) -> Path:
    return paths.target_root / "opencode.json"


def opencode_skill_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode" / "skills" / "standard-ai-workflow" / "SKILL.md"


def opencode_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode" / "agents" / "workflow-orchestrator.md"


def opencode_worker_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode" / "agents" / "workflow-worker.md"


def opencode_doc_worker_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode" / "agents" / "workflow-doc-worker.md"


def opencode_code_worker_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode" / "agents" / "workflow-code-worker.md"


def opencode_validation_worker_agent_path(paths: Paths) -> Path:
    return paths.target_root / ".opencode" / "agents" / "workflow-validation-worker.md"


def antigravity_agents_path(paths: Paths) -> Path:
    return paths.target_root / "ANTIGRAVITY.md"


def minimax_agents_path(paths: Paths) -> Path:
    """Path to the MiniMax Code harness entry file (project root ``MiniMax.md``)."""
    return paths.target_root / "MiniMax.md"


__all__ = [
    "HarnessDefinition",
    "IGNORED_DIRS",
    "Paths",
    "antigravity_agents_path",
    "codex_agents_path",
    "codex_config_example_path",
    "gemini_cli_agents_path",
    "make_paths",
    "minimax_agents_path",
    "opencode_agent_path",
    "opencode_code_worker_agent_path",
    "opencode_config_path",
    "opencode_doc_worker_agent_path",
    "opencode_skill_path",
    "opencode_validation_worker_agent_path",
    "opencode_worker_agent_path",
]
