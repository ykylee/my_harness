#!/usr/bin/env python3
"""Prototype runner for the merge-doc-reconcile skill."""

from __future__ import annotations

import argparse
import json
import sys
from datetime import date
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit import __version__ as TOOL_VERSION
from workflow_kit.common.change_types import classify_doc_sync_file
from workflow_kit.common.exploration_scope import filter_project_scope_paths
from workflow_kit.common.errors import build_error_result
from workflow_kit.common.normalize import dedupe_strings
from workflow_kit.common.paths import project_workspace_root, resolve_existing_path
from workflow_kit.common.project_docs import (
    parse_backlog,
    parse_handoff,
    parse_project_profile_merge,
)
from workflow_kit.common.reconcile import explain_state_conflicts
from workflow_kit.common.session_outputs import build_reconcile_notes
from workflow_kit.common.workflow_state import build_state_cache_refresh_hint, refresh_workflow_state_cache
from workflow_kit.common.workflow_writes import append_unique_bullets_under_heading


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the merge-doc-reconcile prototype.")
    parser.add_argument("--project-profile-path", required=True)
    parser.add_argument("--merge-result-summary", required=True)
    parser.add_argument("--session-handoff-path")
    parser.add_argument("--work-backlog-index-path")
    parser.add_argument("--latest-backlog-path")
    parser.add_argument("--changed-file", action="append", dest="changed_files", default=[])
    parser.add_argument("--validation-result")
    parser.add_argument("--apply", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    source_context = {
        "project_profile_path": args.project_profile_path,
        "merge_result_summary": args.merge_result_summary,
        "session_handoff_path": args.session_handoff_path,
        "work_backlog_index_path": args.work_backlog_index_path,
        "latest_backlog_path": args.latest_backlog_path,
        "changed_files": args.changed_files,
    }

    try:
        profile_path = resolve_existing_path(args.project_profile_path)
        profile_data = parse_project_profile_merge(profile_path)
        project_root = project_workspace_root(profile_path)

        warnings: list[str] = []
        if "warnings" in profile_data:
            warnings.extend(profile_data["warnings"])
        state_conflicts: list[str] = []
        reconfirmation_points: list[str] = []
        reconcile_targets: list[str] = []
        draft_reconcile_notes: list[str] = []

        session_handoff_path = resolve_existing_path(args.session_handoff_path) if args.session_handoff_path else None
        work_backlog_index_path = (
            resolve_existing_path(args.work_backlog_index_path) if args.work_backlog_index_path else None
        )
        latest_backlog_path = resolve_existing_path(args.latest_backlog_path) if args.latest_backlog_path else None

        handoff: dict[str, Any] = {
            "in_progress_items": [],
            "blocked_items": [],
            "recent_done_items": [],
            "next_documents": [],
        }
        backlog: dict[str, Any] = {"in_progress_items": [], "blocked_items": [], "done_items": []}

        if session_handoff_path:
            handoff = parse_handoff(session_handoff_path)
            reconcile_targets.append(str(session_handoff_path))
            if "warnings" in handoff:
                warnings.extend(handoff["warnings"])
        else:
            warnings.append("handoff 경로가 없어 세션 요약 상태를 직접 비교하지 못했다.")

        if latest_backlog_path:
            backlog = parse_backlog(latest_backlog_path)
            reconcile_targets.append(str(latest_backlog_path))
            if "warnings" in backlog:
                warnings.extend(backlog["warnings"])
        else:
            warnings.append("최신 backlog 경로가 없어 작업 단위 상태를 직접 비교하지 못했다.")

        if work_backlog_index_path:
            reconcile_targets.append(str(work_backlog_index_path))

        operations_doc = None
        if profile_data.get("operations_path"):
            operations_doc = (project_root / str(profile_data["operations_path"])).resolve()
            if operations_doc.exists():
                reconcile_targets.append(str(operations_doc))

        state_conflicts.extend(
            explain_state_conflicts(handoff.get("in_progress_items", []), backlog.get("in_progress_items", []), "in_progress")
        )
        state_conflicts.extend(
            explain_state_conflicts(handoff.get("blocked_items", []), backlog.get("blocked_items", []), "blocked")
        )

        if backlog.get("done_items") and not args.validation_result:
            warnings.append("병합 후 검증 결과가 없어 done 상태를 재확정할 수 없다.")
            reconfirmation_points.append("병합 후 완료 처리된 작업의 검증 근거를 다시 확인한다.")

        filtered_changed_files = filter_project_scope_paths(args.changed_files)
        for changed in filtered_changed_files:
            kind = classify_doc_sync_file(changed)
            if kind in {"runbook_doc", "hub_doc"}:
                reconfirmation_points.append(f"{changed} 링크와 허브 반영 여부를 다시 확인한다.")
            if kind in {"code", "config"}:
                reconfirmation_points.append(f"{changed} 변경이 handoff/backlog 설명과 일치하는지 확인한다.")
            if kind in {"handoff_doc", "backlog_doc"}:
                reconfirmation_points.append(f"{changed} 의 병합 후 상태 요약을 재확정한다.")

        draft_reconcile_notes.extend(build_reconcile_notes(profile_data, filtered_changed_files))
        apply_result = {
            "status": "skipped",
            "written_paths": [],
            "warnings": [],
        }
        if args.apply and session_handoff_path:
            applied = append_unique_bullets_under_heading(
                doc_path=session_handoff_path,
                heading="현재 세션 운영 메모",
                bullets=[f"[merge-doc-reconcile] {note}" for note in draft_reconcile_notes],
            )
            if applied:
                apply_result["status"] = "applied"
                apply_result["written_paths"].append(str(session_handoff_path))
            else:
                apply_result["warnings"].append("handoff 운영 메모 섹션을 찾지 못했거나 추가할 재정리 노트가 없었다.")
        elif args.apply:
            apply_result["warnings"].append("session_handoff.md 경로가 없어 merge-doc-reconcile apply 모드를 건너뛰었다.")

        state_cache_update = build_state_cache_refresh_hint(
            project_profile_path=profile_path,
            latest_backlog_path=latest_backlog_path,
        )
        state_cache_refresh = refresh_workflow_state_cache(
            project_profile_path=profile_path,
            session_handoff_path=session_handoff_path,
            work_backlog_index_path=work_backlog_index_path,
            latest_backlog_path=latest_backlog_path,
            generated_at=date.today().isoformat(),
        )
        if state_cache_refresh["status"] == "refreshed":
            apply_result["status"] = "applied"
            state_json_path = state_cache_refresh["state_path"]
            if state_json_path not in apply_result["written_paths"]:
                apply_result["written_paths"].append(state_json_path)

        warnings.extend(apply_result["warnings"])

        if state_conflicts:
            reconfirmation_points.append("handoff 와 backlog 의 충돌 항목을 우선 정리한다.")

        recommended_review_order = dedupe_strings(
            [
                str(latest_backlog_path) if latest_backlog_path else "",
                str(session_handoff_path) if session_handoff_path else "",
                str(work_backlog_index_path) if work_backlog_index_path else "",
                str(operations_doc) if operations_doc and operations_doc.exists() else "",
            ]
        )
        reconcile_targets = dedupe_strings(reconcile_targets)
        reconfirmation_points = dedupe_strings(reconfirmation_points)
        draft_reconcile_notes = dedupe_strings(draft_reconcile_notes)

        result = {
            "status": "ok",
            "tool_version": TOOL_VERSION,
            "reconcile_targets": reconcile_targets,
            "state_conflicts": state_conflicts,
            "reconfirmation_points": reconfirmation_points,
            "draft_reconcile_notes": draft_reconcile_notes,
            "recommended_review_order": recommended_review_order,
            "warnings": warnings,
            "handoff_update_note": (
                "handoff 의 현재 주 작업 축과 상태 요약을 병합 후 기준으로 재작성할지 확인한다." if session_handoff_path else None
            ),
            "backlog_update_note": (
                "최신 backlog 와 backlog index 의 정합성을 함께 확인한다." if latest_backlog_path or work_backlog_index_path else None
            ),
            "hub_update_note": (
                "허브 문서 링크와 설명이 병합 후 구조를 반영하는지 확인한다." if operations_doc and operations_doc.exists() else None
            ),
            "state_cache_update_note": (
                f"현재 source-of-truth 문서를 기준으로 `{state_cache_update['state_path']}` 를 자동 재생성했다."
                if state_cache_refresh["status"] == "refreshed"
                else f"source-of-truth 문서가 아직 부족해 `{state_cache_update['state_path']}` 자동 재생성을 건너뛰었다."
            ),
            "state_cache_refresh_command": state_cache_update["refresh_command"],
            "state_cache_status": state_cache_refresh["status"],
            "state_cache_missing_paths": state_cache_refresh["missing_paths"],
            "apply_status": apply_result["status"],
            "written_paths": apply_result["written_paths"],
            "validation_follow_up": args.validation_result or "병합 후 별도 검증 결과가 없으면 상태 재확정 전에 확인이 필요하다.",
            "source_context": {
                "project_profile_path": str(profile_path),
                "merge_result_summary": args.merge_result_summary,
                "changed_files": filtered_changed_files,
            },
        }
    except FileNotFoundError as exc:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error="merge-doc-reconcile 에 필요한 입력 문서를 읽을 수 없다.",
            error_code="missing_required_document",
            warnings=["프로젝트 프로파일 또는 선택 입력 경로를 다시 확인해야 한다."],
            source_context=source_context | {"missing_path_detail": str(exc)},
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1
    except Exception as exc:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error="merge-doc-reconcile 실행 중 예기치 않은 오류가 발생했다.",
            error_code="merge_doc_reconcile_runtime_error",
            warnings=["입력 문서 형식과 병합 후 정합성 로직을 점검한 뒤 다시 실행해야 한다."],
            source_context=source_context | {"exception_type": type(exc).__name__},
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1

    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
