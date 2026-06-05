#!/usr/bin/env python3
"""Prototype runner for the code-index-update skill."""

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
from workflow_kit.common.change_types import classify_index_change_kinds, dedupe
from workflow_kit.common.exploration_scope import filter_project_scope_paths, is_workflow_meta_path
from workflow_kit.common.errors import build_error_result
from workflow_kit.common.markdown import rel_link_from_doc
from workflow_kit.common.paths import (
    declared_doc_path_from_profile,
    path_exists_from_profile,
    project_workspace_root,
    resolve_existing_path,
)
from workflow_kit.common.project_docs import parse_project_profile_core
from workflow_kit.common.workflow_state import refresh_workflow_state_cache
from workflow_kit.common.workflow_writes import append_unique_bullets_under_heading, update_next_documents_section


def infer_missing_index_targets(changed_files: list[str]) -> list[str]:
    targets: list[str] = []
    for changed in changed_files:
        if is_workflow_meta_path(changed):
            continue
        lower = changed.lower()
        if lower.startswith("docs/operations/runbooks/"):
            targets.append("docs/operations/README.md")
        if "/reports/" in lower or "release-report" in lower:
            targets.append("docs/evals/README.md")
        if "/dataset" in lower or "manifest" in lower:
            targets.append("docs/evals/README.md")
        if lower.endswith("readme.md"):
            targets.append("README.md")
    return dedupe(targets)


def build_index_plan(
    *,
    project_profile_path: Path,
    repo_root: Path,
    profile: dict[str, Any],
    changed_files: list[str],
    work_backlog_index_path: Path | None,
    session_handoff_path: Path | None,
    change_summary: str | None,
) -> dict[str, Any]:
    index_candidates: list[str] = []
    priority_candidates: list[str] = []
    stale_warnings: list[str] = []
    reasoning_notes: list[str] = []
    suggested_actions: list[str] = []
    confidence_notes: list[str] = []
    structure_signals: list[str] = []
    missing_index_candidates: list[str] = []

    root_readme = repo_root / "README.md"
    if root_readme.exists():
        index_candidates.append(str(root_readme))

    declared_document_home = declared_doc_path_from_profile(project_profile_path, profile.get("document_home"))
    document_home = path_exists_from_profile(project_profile_path, profile.get("document_home"))
    if declared_document_home:
        index_candidates.append(declared_document_home)
        if not document_home:
            missing_index_candidates.append(declared_document_home)

    declared_operations_root = declared_doc_path_from_profile(project_profile_path, profile.get("operations_path"))
    operations_path = path_exists_from_profile(project_profile_path, profile.get("operations_path"))
    if declared_operations_root:
        index_candidates.append(declared_operations_root)
        if not operations_path:
            missing_index_candidates.append(declared_operations_root)
        operations_readme = str((Path(declared_operations_root) / "README.md").resolve())
        index_candidates.append(operations_readme)
        if not Path(operations_readme).exists():
            missing_index_candidates.append(operations_readme)

    if work_backlog_index_path and work_backlog_index_path.exists():
        index_candidates.append(str(work_backlog_index_path))
    if session_handoff_path and session_handoff_path.exists():
        index_candidates.append(str(session_handoff_path))

    doc_change_detected = False
    structure_change_detected = False
    code_change_detected = False

    for changed in changed_files:
        kinds = classify_index_change_kinds(changed)
        reasoning_notes.append(f"`{changed}` 는 `{', '.join(sorted(kinds))}` 신호로 분류됐다.")

        if "doc" in kinds:
            doc_change_detected = True
        if "code" in kinds or "config" in kinds:
            code_change_detected = True
        if "nested_doc" in kinds or "root_or_hub_readme" in kinds:
            structure_change_detected = True

        if "root_or_hub_readme" in kinds:
            priority_candidates.append(changed)
            suggested_actions.append("README 또는 허브 문서 자체가 바뀌었으므로 링크와 문서 목록을 다시 확인한다.")

        if "hub_child_doc" in kinds:
            structure_signals.append(f"{changed} 변경은 상위 허브 문서 stale 가능성을 만든다.")
            if declared_operations_root:
                priority_candidates.append(str((Path(declared_operations_root) / "README.md").resolve()))
            elif declared_document_home:
                priority_candidates.append(declared_document_home)
            suggested_actions.append("하위 문서 변경이 허브 링크나 설명에 반영됐는지 확인한다.")

        if "backlog_doc" in kinds and work_backlog_index_path:
            priority_candidates.append(str(work_backlog_index_path))
            suggested_actions.append("날짜별 backlog 변경이 backlog index 설명과 최신 링크에 반영됐는지 확인한다.")

        if "handoff_doc" in kinds and session_handoff_path:
            priority_candidates.append(str(session_handoff_path))

        if "nested_doc" in kinds:
            structure_change_detected = True
            missing_index_candidates.extend(infer_missing_index_targets([changed]))

    summary_lower = (change_summary or "").lower()
    if any(token in summary_lower for token in ["new doc", "새 문서", "문서 추가", "rename", "move", "이동"]):
        structure_change_detected = True
        structure_signals.append("변경 요약에서 문서 구조 변경 신호가 감지됐다.")
    if any(token in summary_lower for token in ["runbook", "report", "dataset", "prompt", "허브"]):
        suggested_actions.append("변경 요약에 허브성 문서 신호가 있어 상위 index 문서를 우선 검토한다.")

    if code_change_detected:
        if declared_operations_root:
            index_candidates.append(str((Path(declared_operations_root) / "README.md").resolve()))
        if work_backlog_index_path:
            index_candidates.append(str(work_backlog_index_path))
        stale_warnings.append("코드/설정 변경이 문서 허브 설명과 최신 구조에 반영됐는지 확인이 필요하다.")

    if doc_change_detected and not any(path.endswith(".md") or path.endswith("README.md") for path in index_candidates):
        stale_warnings.append("문서 변경은 있었지만 재검토할 색인 문서 후보를 찾지 못했다.")

    if structure_change_detected:
        if declared_document_home:
            priority_candidates.append(declared_document_home)
        if root_readme.exists():
            priority_candidates.append(str(root_readme))
        suggested_actions.append("문서 구조 변경 가능성이 있어 루트 README 와 문서 홈을 함께 점검한다.")

    missing_index_candidates = [
        candidate
        for candidate in dedupe(missing_index_candidates)
        if not (repo_root / candidate).exists()
    ]
    if missing_index_candidates:
        stale_warnings.append("변경 경로 기준으로 추정한 색인 문서 일부가 현재 저장소에서 확인되지 않는다.")

    confidence_notes.append("현재 출력은 변경 경로와 프로젝트 프로파일만 사용한 보수적 추천이다.")
    if not structure_change_detected:
        confidence_notes.append("rename 또는 신규 문서 생성 여부는 git diff 없이 확정하지 않았다.")

    return {
        "index_update_candidates": dedupe(index_candidates),
        "priority_index_candidates": dedupe(priority_candidates),
        "stale_index_warnings": dedupe(stale_warnings),
        "warnings": dedupe(stale_warnings),
        "reasoning_notes": dedupe(reasoning_notes),
        "suggested_index_actions": dedupe(suggested_actions),
        "document_structure_signals": dedupe(structure_signals),
        "missing_index_candidates": missing_index_candidates,
        "confidence_notes": dedupe(confidence_notes),
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the code-index-update prototype.")
    parser.add_argument("--project-profile-path", required=True)
    parser.add_argument("--changed-file", action="append", dest="changed_files", default=[])
    parser.add_argument("--work-backlog-index-path")
    parser.add_argument("--session-handoff-path")
    parser.add_argument("--change-summary")
    parser.add_argument("--apply", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    source_context = {
        "project_profile_path": args.project_profile_path,
        "changed_files": args.changed_files,
        "change_summary": args.change_summary,
        "work_backlog_index_path": args.work_backlog_index_path,
        "session_handoff_path": args.session_handoff_path,
    }

    if not args.changed_files and not args.change_summary:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error="변경 입력이 없어 code-index-update 를 구성할 수 없다.",
            error_code="missing_change_input",
            warnings=["최소 하나의 changed file 또는 change summary 가 필요하다."],
            source_context=source_context,
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1

    try:
        project_profile_path = resolve_existing_path(args.project_profile_path)
        profile_data = parse_project_profile_core(project_profile_path)
        repo_root = project_workspace_root(project_profile_path)

        work_backlog_index_path = (
            resolve_existing_path(args.work_backlog_index_path) if args.work_backlog_index_path else None
        )
        session_handoff_path = resolve_existing_path(args.session_handoff_path) if args.session_handoff_path else None

        filtered_changed_files = filter_project_scope_paths(args.changed_files)
        plan_details = build_index_plan(
            project_profile_path=project_profile_path,
            repo_root=repo_root,
            profile=profile_data,
            changed_files=filtered_changed_files,
            work_backlog_index_path=work_backlog_index_path,
            session_handoff_path=session_handoff_path,
            change_summary=args.change_summary,
        )

        if "warnings" in profile_data:
            plan_details["warnings"] = dedupe(plan_details.get("warnings", []) + profile_data["warnings"])

        result = {
            "status": "ok",
            "tool_version": TOOL_VERSION,
            **plan_details,
            "source_context": {
                "project_profile_path": str(project_profile_path),
                "project_name": profile_data.get("project_name"),
                "changed_files": filtered_changed_files,
                "change_summary": args.change_summary,
            },
        }

        apply_result = {
            "status": "skipped",
            "written_paths": [],
            "warnings": [],
        }

        if args.apply and session_handoff_path:
            # 1. '다음에 읽을 문서' 섹션 갱신
            # 중요도 높은 후보와 일반 후보 병합
            all_candidates = dedupe(plan_details.get("priority_index_candidates", []) + plan_details.get("index_update_candidates", []))
            links = []
            for path_str in all_candidates:
                target_path = Path(path_str)
                if not target_path.is_absolute():
                    target_path = (repo_root / target_path).resolve()

                if not target_path.exists():
                    continue

                # 자기 자신(session_handoff.md)은 제외
                if target_path.resolve() == session_handoff_path.resolve():
                    continue

                rel_link = rel_link_from_doc(session_handoff_path, target_path)
                label = target_path.name
                links.append(f"[{label}]({rel_link})")

            if links:
                if update_next_documents_section(doc_path=session_handoff_path, links=links):
                    apply_result["status"] = "applied"
                    apply_result["written_paths"].append(str(session_handoff_path))

            # 2. '현재 세션 운영 메모'에 권장 조치 추가
            actions = plan_details.get("suggested_index_actions", [])
            if actions:
                bullets = [f"[code-index-update] {action}" for action in actions]
                if append_unique_bullets_under_heading(
                    doc_path=session_handoff_path, heading="현재 세션 운영 메모", bullets=bullets
                ):
                    apply_result["status"] = "applied"
                    if str(session_handoff_path) not in apply_result["written_paths"]:
                        apply_result["written_paths"].append(str(session_handoff_path))

            # 3. state.json 갱신
            latest_backlog_path = None # 여기서 정확히 알기 어려우면 None으로 전달
            state_refresh = refresh_workflow_state_cache(
                project_profile_path=project_profile_path,
                session_handoff_path=session_handoff_path,
                work_backlog_index_path=work_backlog_index_path,
                latest_backlog_path=None,
                generated_at=date.today().isoformat(),
            )
            if state_refresh["status"] == "refreshed":
                apply_result["status"] = "applied"
                state_json_path = state_refresh["state_path"]
                if state_json_path not in apply_result["written_paths"]:
                    apply_result["written_paths"].append(state_json_path)

            result["apply_status"] = apply_result["status"]
            result["written_paths"] = apply_result["written_paths"]
        elif args.apply:
            result["apply_status"] = "skipped"
            result["warnings"].append("session_handoff.md 경로가 없어 code-index-update apply 모드를 건너뛰었다.")

    except FileNotFoundError as exc:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error="code-index-update 에 필요한 입력 문서를 읽을 수 없다.",
            error_code="missing_required_document",
            warnings=["프로젝트 프로파일 또는 선택 입력 경로를 다시 확인해야 한다."],
            source_context=source_context | {"missing_path_detail": str(exc)},
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1
    except Exception as exc:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error="code-index-update 실행 중 예기치 않은 오류가 발생했다.",
            error_code="code_index_update_runtime_error",
            warnings=["프로파일 형식, 변경 경로, 또는 추천 로직을 점검한 뒤 다시 실행해야 한다."],
            source_context=source_context | {"exception_type": type(exc).__name__},
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1

    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
