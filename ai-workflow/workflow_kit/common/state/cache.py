"""Logic for managing the workflow state JSON cache."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from workflow_kit.common.paths import (
    project_workspace_root,
    workflow_branch_dir,
    workflow_memory_dir,
)
from workflow_kit.common.state.builder import build_workflow_state_payload

def build_state_cache_refresh_hint(
    *,
    project_profile_path: Path,
    latest_backlog_path: Path | None = None,
    repository_assessment_path: Path | None = None,
) -> dict[str, str]:
    workspace_root = project_workspace_root(project_profile_path)
    memory_dir = workflow_memory_dir(project_profile_path)
    branch_dir = workflow_branch_dir(project_profile_path)
    generator_path = workspace_root / "workflow-source" / "scripts" / "generate_workflow_state.py"
    state_path = branch_dir / "state.json"
    command_parts = [
        f"python3 {generator_path}",
        f"--project-profile-path {project_profile_path}",
        f"--session-handoff-path {branch_dir / 'session_handoff.md'}",
        f"--work-backlog-index-path {memory_dir / 'work_backlog.md'}",
        f"--output-path {state_path}",
    ]
    if latest_backlog_path:
        command_parts.append(f"--latest-backlog-path {latest_backlog_path}")
    if repository_assessment_path:
        command_parts.append(f"--repository-assessment-path {repository_assessment_path}")
    return {
        "state_path": str(state_path),
        "refresh_command": " ".join(str(part) for part in command_parts),
    }

def refresh_workflow_state_cache(
    *,
    project_profile_path: Path,
    session_handoff_path: Path | None = None,
    work_backlog_index_path: Path | None = None,
    latest_backlog_path: Path | None = None,
    repository_assessment_path: Path | None = None,
    output_path: Path | None = None,
    generated_at: str,
    workspace_root: Path | None = None,
) -> dict[str, Any]:
    resolved_project_profile_path = project_profile_path.resolve()
    memory_dir = workflow_memory_dir(resolved_project_profile_path)
    branch_dir = workflow_branch_dir(resolved_project_profile_path)
    actual_root = workspace_root or project_workspace_root(resolved_project_profile_path)
    resolved_session_handoff_path = (session_handoff_path or (branch_dir / "session_handoff.md")).resolve()
    resolved_work_backlog_index_path = (work_backlog_index_path or (memory_dir / "work_backlog.md")).resolve()
    resolved_latest_backlog_path = latest_backlog_path.resolve() if latest_backlog_path else None
    resolved_repository_assessment_path = repository_assessment_path.resolve() if repository_assessment_path else None

    missing_paths: list[str] = []
    for required_path in (
        resolved_project_profile_path,
        resolved_session_handoff_path,
        resolved_work_backlog_index_path,
    ):
        if not required_path.exists():
            missing_paths.append(str(required_path))

    state_path = (output_path or (branch_dir / "state.json")).resolve()
    refresh_hint = build_state_cache_refresh_hint(
        project_profile_path=resolved_project_profile_path,
        latest_backlog_path=resolved_latest_backlog_path,
        repository_assessment_path=resolved_repository_assessment_path,
    )
    if missing_paths:
        return {
            "status": "skipped",
            "state_path": str(state_path),
            "refresh_command": refresh_hint["refresh_command"],
            "missing_paths": missing_paths,
        }

    payload = build_workflow_state_payload(
        project_profile_path=resolved_project_profile_path,
        session_handoff_path=resolved_session_handoff_path,
        work_backlog_index_path=resolved_work_backlog_index_path,
        latest_backlog_path=resolved_latest_backlog_path,
        repository_assessment_path=resolved_repository_assessment_path,
        generated_at=generated_at,
        workspace_root=actual_root,
    )
    state_path.parent.mkdir(parents=True, exist_ok=True)
    state_path.write_text(json.dumps(payload, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return {
        "status": "refreshed",
        "state_path": str(state_path),
        "refresh_command": refresh_hint["refresh_command"],
        "missing_paths": [],
    }
