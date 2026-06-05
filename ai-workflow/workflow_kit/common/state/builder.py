"""Logic for building the workflow state payload from various sources."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from workflow_kit.common.normalize import (
    dedupe_normalized_backticked,
    dedupe_strings as _dedupe_strings_base,
    dedupe_work_items,
)
from workflow_kit.common.paths import project_workspace_root, safe_relpath
from workflow_kit.common.project_docs import (
    find_latest_backlog_path,
    parse_backlog,
    parse_handoff,
    parse_project_profile_core,
    parse_project_profile_validation,
)

def is_meaningful_text(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and not value.strip().startswith("TODO:")

def build_workflow_state_payload(
    *,
    project_profile_path: Path,
    session_handoff_path: Path,
    work_backlog_index_path: Path,
    latest_backlog_path: Path | None = None,
    repository_assessment_path: Path | None = None,
    generated_at: str,
    workspace_root: Path | None = None,
) -> dict[str, Any]:
    actual_root = workspace_root or project_workspace_root(project_profile_path)
    resolved_latest_backlog_path = latest_backlog_path or find_latest_backlog_path(work_backlog_index_path)
    if resolved_latest_backlog_path is not None and not resolved_latest_backlog_path.exists():
        resolved_latest_backlog_path = None

    profile_core = parse_project_profile_core(project_profile_path)
    profile_validation = parse_project_profile_validation(project_profile_path)
    handoff = parse_handoff(session_handoff_path)
    backlog = parse_backlog(resolved_latest_backlog_path) if resolved_latest_backlog_path else {
        "tasks": [],
        "in_progress_items": [],
        "blocked_items": [],
        "done_items": [],
    }

    in_progress_items = dedupe_work_items(
        [item for item in list(handoff.get("in_progress_items", [])) if is_meaningful_text(item)]
        + list(backlog.get("in_progress_items", []))
    )
    blocked_items = dedupe_work_items(
        [item for item in list(handoff.get("blocked_items", [])) if is_meaningful_text(item)]
        + list(backlog.get("blocked_items", []))
    )
    recent_done_items = dedupe_work_items(
        [item for item in list(handoff.get("recent_done_items", [])) if is_meaningful_text(item)]
        + list(backlog.get("done_items", []))
    )[:10]

    next_documents = _dedupe_strings_base(
        [
            safe_relpath(project_profile_path, actual_root),
            safe_relpath(session_handoff_path, actual_root),
            safe_relpath(work_backlog_index_path, actual_root),
            safe_relpath(resolved_latest_backlog_path, actual_root) if resolved_latest_backlog_path else "",
            *[safe_relpath(path, actual_root) for path in handoff.get("next_documents", []) if isinstance(path, Path) and path.exists()],
        ]
    )

    current_focus = in_progress_items[0] if in_progress_items else (blocked_items[0] if blocked_items else None)
    if current_focus is None:
        tasks = list(backlog.get("tasks", []))
        if tasks:
            first_task = tasks[0]
            current_focus = f"{first_task['task_id']} {first_task['title']}"

    return {
        "schema_version": "1",
        "generated_at": generated_at,
        "source_of_truth": {
            "project_profile_path": safe_relpath(project_profile_path, actual_root),
            "session_handoff_path": safe_relpath(session_handoff_path, actual_root),
            "work_backlog_index_path": safe_relpath(work_backlog_index_path, actual_root),
            "latest_backlog_path": safe_relpath(resolved_latest_backlog_path, actual_root) if resolved_latest_backlog_path else None,
            "repository_assessment_path": safe_relpath(repository_assessment_path, actual_root) if repository_assessment_path else None,
        },
        "project": {
            "project_name": profile_core.get("project_name"),
            "document_home": profile_core.get("document_home"),
            "operations_path": profile_core.get("operations_path"),
            "backlog_path": profile_core.get("backlog_path"),
            "handoff_path": profile_core.get("handoff_path"),
            "environment_path": profile_core.get("environment_path"),
        },
        "commands": {
            "quick_tests": profile_validation.get("quick_tests"),
            "isolated_tests": profile_validation.get("isolated_tests"),
            "runtime_checks": profile_validation.get("runtime_checks"),
        },
        "session": {
            "current_baseline": handoff.get("current_baseline"),
            "current_axis": handoff.get("current_axis"),
            "current_focus": current_focus,
            "in_progress_items": in_progress_items,
            "blocked_items": blocked_items,
            "recent_done_items": recent_done_items,
            "environment_constraints": dedupe_normalized_backticked(
                [item for item in [handoff.get("constraints")] if is_meaningful_text(item)]
            ),
        },
        "backlog": {
            "latest_backlog_path": safe_relpath(resolved_latest_backlog_path, actual_root) if resolved_latest_backlog_path else None,
            "task_count": len(list(backlog.get("tasks", []))),
            "in_progress_items": list(backlog.get("in_progress_items", [])),
            "blocked_items": list(backlog.get("blocked_items", [])),
            "done_items": list(backlog.get("done_items", [])),
        },
        "next_documents": next_documents,
        "repository_assessment": {
            "path": safe_relpath(repository_assessment_path, actual_root) if repository_assessment_path else None,
            "present": bool(repository_assessment_path and repository_assessment_path.exists()),
        },
    }
