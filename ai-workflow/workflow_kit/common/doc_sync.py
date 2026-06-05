"""Document-candidate helpers shared across doc-sync style skills."""

from __future__ import annotations

from pathlib import Path

from workflow_kit.common.change_types import classify_doc_sync_file
from workflow_kit.common.exploration_scope import filter_project_scope_paths, is_workflow_meta_path
from workflow_kit.common.normalize import dedupe_strings
from workflow_kit.common.paths import path_exists_relative


def build_doc_sync_candidates(
    *,
    project_root: Path,
    profile: dict[str, object],
    changed_files: list[str],
    session_handoff_path: Path | None,
    work_backlog_index_path: Path | None,
    latest_backlog_path: Path | None,
    change_summary: str | None,
) -> dict[str, list[str]]:
    operations_doc = path_exists_relative(project_root, profile.get("operations_path"))
    doc_home = path_exists_relative(project_root, profile.get("document_home"))
    impacted: list[str] = []
    hub_candidates: list[str] = []
    status_doc_candidates: list[str] = []
    stale_warnings: list[str] = []
    reasoning_notes: list[str] = []
    follow_up_actions: list[str] = []
    validation_doc_candidates: list[str] = []

    if session_handoff_path and session_handoff_path.exists():
        status_doc_candidates.append(str(session_handoff_path))
    if latest_backlog_path and latest_backlog_path.exists():
        status_doc_candidates.append(str(latest_backlog_path))
    if work_backlog_index_path and work_backlog_index_path.exists():
        hub_candidates.append(str(work_backlog_index_path))
    if operations_doc and operations_doc.exists():
        hub_candidates.append(str(operations_doc))
    if doc_home and doc_home.exists():
        hub_candidates.append(str(doc_home))

    for changed in changed_files:
        kind = classify_doc_sync_file(changed)
        reasoning_notes.append(f"`{changed}` 는 `{kind}` 유형 변경으로 분류됐다.")
        if is_workflow_meta_path(changed):
            reasoning_notes.append(
                f"`{changed}` 는 workflow 메타 문서 경로이므로 프로젝트 문서 탐색 후보에서는 제외한다."
            )
            continue
        if kind in {"handoff_doc", "backlog_doc", "doc"}:
            impacted.append(changed)
            if "handoff" in changed.lower() and session_handoff_path:
                status_doc_candidates.append(str(session_handoff_path))
            if "backlog" in changed.lower():
                if latest_backlog_path:
                    status_doc_candidates.append(str(latest_backlog_path))
                if work_backlog_index_path:
                    hub_candidates.append(str(work_backlog_index_path))
        if kind == "hub_doc":
            hub_candidates.append(changed)
            impacted.append(changed)
        if kind == "runbook_doc":
            impacted.append(changed)
            if operations_doc:
                hub_candidates.append(str(operations_doc))
            follow_up_actions.append("runbook 링크가 운영 허브에 반영됐는지 확인한다.")
        if kind in {"code", "config"}:
            if session_handoff_path:
                status_doc_candidates.append(str(session_handoff_path))
            if latest_backlog_path:
                status_doc_candidates.append(str(latest_backlog_path))
            if operations_doc:
                hub_candidates.append(str(operations_doc))
            stale_warnings.append(f"{changed} 변경이 운영 문서에 반영됐는지 아직 확인되지 않았다.")
            follow_up_actions.append("코드/설정 변경이 관련 운영 문서에 반영됐는지 확인한다.")

    summary_lower = (change_summary or "").lower()
    if "runbook" in summary_lower:
        follow_up_actions.append("runbook 내용과 허브 링크 최신성을 함께 점검한다.")
        if operations_doc:
            hub_candidates.append(str(operations_doc))
    if "handoff" in summary_lower and session_handoff_path:
        status_doc_candidates.append(str(session_handoff_path))
    if "backlog" in summary_lower and latest_backlog_path:
        status_doc_candidates.append(str(latest_backlog_path))

    if any(classify_doc_sync_file(item) in {"code", "config"} for item in changed_files):
        validation_doc_candidates.extend(dedupe_strings(status_doc_candidates))
        if not status_doc_candidates:
            stale_warnings.append("코드 변경은 있었지만 상태 문서 후보를 찾지 못했다.")

    impacted_documents = dedupe_strings(filter_project_scope_paths(impacted))
    hub_update_candidates = dedupe_strings(filter_project_scope_paths(hub_candidates))
    status_doc_candidates = dedupe_strings(status_doc_candidates)
    follow_up_actions = dedupe_strings(follow_up_actions)

    # 자기 자신(session_handoff.md)은 검토 추천 목록에서 제외
    raw_review_order = [*impacted_documents, *status_doc_candidates, *hub_update_candidates]
    recommended_review_order = [
        p for p in dedupe_strings(raw_review_order)
        if session_handoff_path is None or Path(p).resolve() != session_handoff_path.resolve()
    ]

    confidence_notes = []
    if stale_warnings:
        confidence_notes.append("현재 출력에는 추정 기반 후보가 포함되어 있어 수동 검토가 필요하다.")
    else:
        confidence_notes.append("입력된 변경 파일 기준으로 직접 영향 문서를 우선 정리했다.")

    return {
        "impacted_documents": impacted_documents,
        "hub_update_candidates": hub_update_candidates,
        "status_doc_candidates": status_doc_candidates,
        "validation_doc_candidates": dedupe_strings(validation_doc_candidates),
        "stale_warnings": dedupe_strings(stale_warnings),
        "warnings": dedupe_strings(stale_warnings),
        "reasoning_notes": dedupe_strings(reasoning_notes),
        "recommended_review_order": recommended_review_order,
        "follow_up_actions": follow_up_actions,
        "confidence_notes": confidence_notes,
    }
