"""Logic for preventing document bloat by rotating old tasks."""

from __future__ import annotations

import re
from pathlib import Path
from typing import List, Dict, Any

def rotate_handoff_tasks(
    handoff_path: Path,
    max_done_items: int = 10
) -> Dict[str, Any]:
    if not handoff_path.exists():
        return {"status": "skipped", "reason": "handoff file not found"}

    content = handoff_path.read_text(encoding="utf-8")
    lines = content.splitlines()

    recent_done_start = -1
    recent_done_end = -1
    baseline_line_idx = -1

    # 1. Locate sections
    for i, line in enumerate(lines):
        if line.startswith("## 5. 최근 완료 작업"):
            recent_done_start = i
        elif line.startswith("## 6. 잔여 작업"):
            recent_done_end = i
        elif "현재 기준선:" in line:
            baseline_line_idx = i

    if recent_done_start == -1 or recent_done_end == -1:
        return {"status": "error", "message": "Could not locate 'recent_done_items' section"}

    # 2. Extract items
    done_section_lines = lines[recent_done_start+1 : recent_done_end]
    items = [l for l in done_section_lines if l.strip().startswith("- TASK-")]

    if len(items) <= max_done_items:
        return {"status": "ok", "message": f"Done items ({len(items)}) within limit ({max_done_items})", "rotated": False}

    # 3. Rotate old items
    to_rotate = items[:-max_done_items]
    remaining = items[-max_done_items:]

    # Create summary for baseline
    rotate_summary = ", ".join([it.strip("- ").strip() for it in to_rotate])

    # Update baseline
    if baseline_line_idx != -1:
        old_baseline = lines[baseline_line_idx]
        if "N/A" in old_baseline:
            lines[baseline_line_idx] = f"- 현재 기준선: {rotate_summary}"
        else:
            lines[baseline_line_idx] = f"{old_baseline}, {rotate_summary}"

    # Update recent_done section
    new_content_lines = lines[:recent_done_start+1]
    new_content_lines.append("")
    new_content_lines.extend(remaining)
    new_content_lines.append("")
    new_content_lines.extend(lines[recent_done_end:])

    new_content = "\n".join(new_content_lines)
    handoff_path.write_text(new_content, encoding="utf-8")

    return {
        "status": "ok",
        "rotated": True,
        "rotated_count": len(to_rotate),
        "remaining_count": len(remaining),
        "rotated_items": to_rotate
    }
