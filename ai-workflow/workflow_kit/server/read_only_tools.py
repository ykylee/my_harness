"""Direct-call adapter for the first read-only MCP bundle."""

from __future__ import annotations

from typing import Any

from workflow_kit.common.read_only_bundle import (
    check_doc_links_payload,
    check_doc_metadata_payload,
    check_quickstart_stale_links_payload,
    create_backlog_entry_payload,
    create_environment_record_stub_payload,
    create_session_handoff_draft_payload,
    latest_backlog_payload,
    suggest_impacted_docs_payload,
    summarize_git_history_payload,
    rotate_workflow_logs_payload,
    assess_milestone_progress_payload,
    smart_context_reader_payload,
)
from workflow_kit.common.writing_bundle import (
    apply_robust_patch_payload,
)


def invoke_read_only_tool(*, tool_name: str, payload: dict[str, Any], tool_version: str) -> dict[str, Any]:
    if tool_name == "latest_backlog":
        return latest_backlog_payload(
            backlog_dir_path=payload.get("backlog_dir_path"),
            work_backlog_index_path=payload.get("work_backlog_index_path"),
            tool_version=tool_version,
        )
    if tool_name == "check_doc_metadata":
        return check_doc_metadata_payload(doc_dir_path=str(payload["doc_dir_path"]), tool_version=tool_version)
    if tool_name == "check_doc_links":
        return check_doc_links_payload(doc_dir_path=str(payload["doc_dir_path"]), tool_version=tool_version)
    if tool_name == "create_backlog_entry":
        return create_backlog_entry_payload(
            task_id=str(payload["task_id"]),
            task_name=str(payload["task_name"]),
            request_date=str(payload["request_date"]),
            status=payload.get("status"),
            priority=payload.get("priority"),
            tool_version=tool_version,
        )
    if tool_name == "create_session_handoff_draft":
        return create_session_handoff_draft_payload(
            latest_backlog_path=payload.get("latest_backlog_path"),
            git_summary=payload.get("git_summary"),
            tool_version=tool_version,
        )
    if tool_name == "create_environment_record_stub":
        return create_environment_record_stub_payload(
            hostname=str(payload["hostname"]),
            os_type=str(payload["os_type"]),
            tool_version=tool_version,
        )
    if tool_name == "suggest_impacted_docs":
        return suggest_impacted_docs_payload(
            changed_files=[str(item) for item in payload["changed_files"]],
            session_handoff_path=payload.get("session_handoff_path"),
            latest_backlog_path=payload.get("latest_backlog_path"),
            work_backlog_index_path=payload.get("work_backlog_index_path"),
            tool_version=tool_version,
        )
    if tool_name == "check_quickstart_stale_links":
        return check_quickstart_stale_links_payload(
            quickstart_paths=[str(item) for item in payload["quickstart_paths"]],
            project_profile_path=payload.get("project_profile_path"),
            session_handoff_path=payload.get("session_handoff_path"),
            work_backlog_index_path=payload.get("work_backlog_index_path"),
            agents_path=payload.get("agents_path"),
            tool_version=tool_version,
        )
    if tool_name == "summarize_git_history":
        return summarize_git_history_payload(
            repo_path=str(payload["repo_path"]),
            commit_range=str(payload["commit_range"]),
            tool_version=tool_version,
        )
    if tool_name == "rotate_workflow_logs":
        return rotate_workflow_logs_payload(
            handoff_path=str(payload["handoff_path"]),
            max_done_items=int(payload.get("max_done_items", 10)),
            tool_version=tool_version,
        )
    if tool_name == "assess_milestone_progress":
        return assess_milestone_progress_payload(
            matrix_path=str(payload["matrix_path"]),
            backlog_path=str(payload["backlog_path"]),
            tool_version=tool_version,
        )
    if tool_name == "smart_context_reader":
        return smart_context_reader_payload(
            file_path=str(payload["file_path"]),
            symbols=payload.get("symbols"),
            tool_version=tool_version,
        )
    if tool_name == "apply_robust_patch":
        return apply_robust_patch_payload(
            file_path=str(payload["file_path"]),
            patch_content=str(payload["patch_content"]),
            tool_version=tool_version,
        )
    raise KeyError(tool_name)
