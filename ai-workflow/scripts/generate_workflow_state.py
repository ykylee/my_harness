#!/usr/bin/env python3
"""Generate a compact state.json cache from workflow markdown documents."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import date
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit.common.workflow_state import refresh_workflow_state_cache


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate workflow state.json from workflow docs.")
    parser.add_argument("--project-profile-path", required=True)
    parser.add_argument("--session-handoff-path", required=True)
    parser.add_argument("--work-backlog-index-path", required=True)
    parser.add_argument("--output-path", required=True)
    parser.add_argument("--latest-backlog-path")
    parser.add_argument("--repository-assessment-path")
    parser.add_argument("--workspace-root")
    parser.add_argument("--generated-at", default=date.today().isoformat())
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    output_path = Path(args.output_path).resolve()
    refresh_result = refresh_workflow_state_cache(
        project_profile_path=Path(args.project_profile_path).resolve(),
        session_handoff_path=Path(args.session_handoff_path).resolve(),
        work_backlog_index_path=Path(args.work_backlog_index_path).resolve(),
        latest_backlog_path=Path(args.latest_backlog_path).resolve() if args.latest_backlog_path else None,
        repository_assessment_path=Path(args.repository_assessment_path).resolve() if args.repository_assessment_path else None,
        output_path=output_path,
        generated_at=args.generated_at,
        workspace_root=Path(args.workspace_root).resolve() if args.workspace_root else None,
    )
    print(
        json.dumps(
            {
                "status": "ok",
                "output_path": str(output_path),
                "state_cache_status": refresh_result["status"],
            },
            ensure_ascii=False,
            indent=2,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
