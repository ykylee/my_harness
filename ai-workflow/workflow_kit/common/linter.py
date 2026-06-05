import json
import re
from pathlib import Path
from typing import Dict, List, Any

from workflow_kit.common.project_docs import parse_backlog, parse_handoff

def check_maturity_consistency(
    matrix_path: Path,
    roadmap_path: Path,
    project_root: Path
) -> Dict[str, Any]:
    issues = []
    warnings = []

    if not matrix_path.exists():
        return {"status": "skipped", "reason": "maturity_matrix.json not found"}

    try:
        matrix = json.loads(matrix_path.read_text(encoding="utf-8"))
    except Exception as e:
        return {"status": "error", "error_code": "matrix_json_load_failure", "description": f"Failed to load maturity_matrix.json: {e}"}

    # 1. Check test_path existence
    skills = matrix.get("skills", {})
    for skill_name, info in skills.items():
        test_path_str = info.get("test_path")
        if test_path_str:
            test_path = (project_root / test_path_str).resolve()
            if not test_path.exists():
                issues.append({
                    "type": "maturity_error",
                    "code": "missing_test_file",
                    "description": f"Skill '{skill_name}' declares test_path '{test_path_str}', but the file does not exist.",
                    "severity": "high",
                    "fix_suggestion": f"Create the missing test file at {test_path_str} or update the matrix."
                })
        elif info.get("stage") in ["beta", "stable"]:
            warnings.append(f"Skill '{skill_name}' is in stage '{info.get('stage')}' but has no test_path defined.")

    # 2. Check Roadmap alignment (Basic Check)
    if roadmap_path.exists():
        roadmap_content = roadmap_path.read_text(encoding="utf-8")
        milestones = matrix.get("milestones", {})

        # Check if current Roadmap phase matches In-Progress milestone
        in_progress_milestones = [name for name, m in milestones.items() if m.get("status") == "in_progress"]
        for m_name in in_progress_milestones:
            # Simple check: Does the roadmap mention the in-progress phase name?
            phase_name = milestones[m_name].get("name", "")
            if phase_name and phase_name not in roadmap_content:
                 issues.append({
                    "type": "maturity_error",
                    "code": "roadmap_milestone_mismatch",
                    "description": f"Milestone '{m_name}' ({phase_name}) is 'in_progress' in matrix, but not prominently mentioned as current phase in roadmap.md.",
                    "severity": "medium",
                    "fix_suggestion": "Update roadmap.md to reflect the current in-progress phase from maturity_matrix.json."
                })

    return {
        "status": "ok" if not issues else "issues_found",
        "issues": issues,
        "warnings": warnings
    }

def check_workflow_consistency(
    state_json_path: Path,
    handoff_path: Path,
    latest_backlog_path: Path
) -> Dict[str, Any]:
    issues = []
    warnings = []

    # Load data
    try:
        if not state_json_path.exists():
            return {"status": "error", "error_code": "missing_state_json", "description": "state.json file not found"}
        state = json.loads(state_json_path.read_text(encoding="utf-8"))
    except Exception as e:
        return {"status": "error", "error_code": "state_json_load_failure", "description": f"Failed to load state.json: {e}"}

    try:
        handoff = parse_handoff(handoff_path)
    except Exception as e:
        warnings.append(f"Failed to parse handoff: {e}")
        handoff = {}

    try:
        backlog = parse_backlog(latest_backlog_path)
    except Exception as e:
        warnings.append(f"Failed to parse backlog: {e}")
        backlog = {}

    # 1. Check in_progress consistency
    backlog_in_progress = {item.split()[0] for item in backlog.get("in_progress_items", []) if item.startswith("TASK-")}
    handoff_in_progress = {item.split()[0] for item in handoff.get("in_progress_items", []) if item.startswith("TASK-") and "N/A" not in item}
    state_in_progress = {item.split()[0] for item in state.get("session", {}).get("in_progress_items", []) if item.startswith("TASK-") and "N/A" not in item}

    all_tasks = backlog_in_progress | handoff_in_progress | state_in_progress
    for task in all_tasks:
        missing = []
        if task not in backlog_in_progress: missing.append("backlog")
        if task not in handoff_in_progress: missing.append("handoff")
        if task not in state_in_progress: missing.append("state.json")

        if missing:
            issues.append({
                "type": "sync_error",
                "code": "task_status_mismatch",
                "description": f"Task {task} is inconsistent. Missing in: {', '.join(missing)}",
                "severity": "medium",
                "fix_suggestion": f"Ensure {task} is listed in all three core documents."
            })

    # 2. Check for bloat in handoff
    done_items = handoff.get("recent_done_items", [])
    if len(done_items) > 10:
        issues.append({
            "type": "bloat_warning",
            "code": "handoff_bloat",
            "description": f"Handoff has {len(done_items)} recently done items.",
            "severity": "low",
            "fix_suggestion": "Move older tasks to baseline summary and keep only last 5-10 in recently done."
        })

    # 3. Check for broken links in handoff/backlog (simple regex)
    for path in [handoff_path, latest_backlog_path]:
        if not path.exists():
            continue
        content = path.read_text(encoding="utf-8")
        links = re.findall(r"\[.*?\]\((.*?)\)", content)
        for link in links:
            if link.startswith("http") or link.startswith("#") or not link.strip():
                continue
            # Handle relative links
            try:
                # Remove query or fragments if any
                clean_link = link.split("#")[0].split("?")[0]
                link_path = (path.parent / clean_link).resolve()
                if not link_path.exists():
                    issues.append({
                        "type": "broken_link",
                        "code": "file_not_found",
                        "description": f"Broken link in {path.name}: {link}",
                        "severity": "medium",
                        "fix_suggestion": f"Fix the relative path or create the missing file: {link}"
                    })
            except Exception:
                warnings.append(f"Invalid link format detected in {path.name}: {link}")

    return {
        "status": "ok" if not issues else "issues_found",
        "issues": issues,
        "warnings": warnings,
        "summary": {
            "total_issues": len(issues),
            "sync_errors": len([i for i in issues if i["type"] == "sync_error"]),
            "broken_links": len([i for i in issues if i["type"] == "broken_link"]),
            "bloat_warnings": len([i for i in issues if i["type"] == "bloat_warning"])
        }
    }
