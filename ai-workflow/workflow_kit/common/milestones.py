"""Logic for tracking milestone progress and suggesting rotations."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Dict, List, Any

from workflow_kit.common.project_docs import parse_backlog

def assess_milestone_progress(
    matrix_path: Path,
    backlog_path: Path
) -> Dict[str, Any]:
    if not matrix_path.exists() or not backlog_path.exists():
        return {"status": "error", "message": "Required files missing"}

    try:
        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
        backlog_data = parse_backlog(backlog_path)
    except Exception as e:
        return {"status": "error", "message": f"Parsing failed: {e}"}

    milestones = matrix.get("milestones", {})
    current_milestone_id = next((k for k, v in milestones.items() if v.get("status") == "in_progress"), None)

    if not current_milestone_id:
        return {"status": "ok", "message": "No in-progress milestone found", "progress": 0}

    current_milestone = milestones[current_milestone_id]

    # Heuristic: Count tasks related to the current phase (THREAD-003 is Phase 5)
    # Mapping phases to threads (simplified for demo)
    phase_thread_map = {
        "Phase 1": "THREAD-001",
        "Phase 2": "THREAD-001",
        "Phase 3": "THREAD-001",
        "Phase 4": "THREAD-002",
        "Phase 5": "THREAD-003"
    }

    target_thread = phase_thread_map.get(current_milestone_id)

    done_tasks = backlog_data.get("done_items", [])
    in_progress_tasks = backlog_data.get("in_progress_items", [])

    if target_thread:
        relevant_done = [t for t in done_tasks if target_thread in t]
        relevant_ip = [t for t in in_progress_tasks if target_thread in t]
    else:
        relevant_done = done_tasks
        relevant_ip = in_progress_tasks

    total_relevant = len(relevant_done) + len(relevant_ip)
    progress = (len(relevant_done) / total_relevant * 100) if total_relevant > 0 else 0

    suggestion = None
    if progress >= 80:
        suggestion = f"Milestone '{current_milestone['name']}' is {progress:.1f}% complete. Consider rotating to the next phase."

    return {
        "status": "ok",
        "milestone_id": current_milestone_id,
        "milestone_name": current_milestone["name"],
        "progress": progress,
        "done_count": len(relevant_done),
        "total_count": total_relevant,
        "suggestion": suggestion
    }
