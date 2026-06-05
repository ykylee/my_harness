"""Output contracts for error payloads across all tool families."""

from __future__ import annotations
from workflow_kit.common.contracts.base import OutputFieldShape

ERROR_SUCCESS_CONTRACTS: dict[str, frozenset[str]] = {
    "session_start": frozenset({"error", "error_code", "source_context"}),
    "backlog_update": frozenset({"error", "error_code", "source_context"}),
    "doc_sync": frozenset({"error", "error_code", "source_context"}),
    "merge_doc_reconcile": frozenset({"error", "error_code", "source_context"}),
    "validation_plan": frozenset({"error", "error_code", "source_context"}),
    "code_index_update": frozenset({"error", "error_code", "source_context"}),
    "latest_backlog": frozenset({"error", "error_code", "source_context"}),
    "check_doc_metadata": frozenset({"error", "error_code", "source_context"}),
    "check_doc_links": frozenset({"error", "error_code", "source_context"}),
    "check_quickstart_stale_links": frozenset({"error", "error_code", "source_context"}),
    "create_backlog_entry": frozenset({"error", "error_code", "source_context"}),
    "create_session_handoff_draft": frozenset({"error", "error_code", "source_context"}),
    "create_environment_record_stub": frozenset({"error", "error_code", "source_context"}),
    "suggest_impacted_docs": frozenset({"error", "error_code", "source_context"}),
    "read_only_entrypoint": frozenset({"error", "error_code", "source_context"}),
    "demo_workflow": frozenset({"error", "error_code", "source_context"}),
    "existing_project_onboarding": frozenset({"error", "error_code", "source_context"}),
}

ERROR_FIELD_SHAPES: dict[str, dict[str, OutputFieldShape]] = {
    "session_start": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {"session_handoff_path", "work_backlog_index_path", "project_profile_path", "latest_backlog_path"}
            ),
            properties={
                "session_handoff_path": OutputFieldShape(kind="string"),
                "work_backlog_index_path": OutputFieldShape(kind="string"),
                "project_profile_path": OutputFieldShape(kind="string"),
                "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
                "missing_path_detail": OutputFieldShape(kind="string"),
                "exception_type": OutputFieldShape(kind="string"),
            },
        ),
    },
    "backlog_update": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {"project_profile_path", "task_name", "task_brief", "daily_backlog_path", "target_date", "task_id", "mode"}
            ),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "task_name": OutputFieldShape(kind="string"),
                "task_brief": OutputFieldShape(kind="string"),
                "daily_backlog_path": OutputFieldShape(kind="string", allow_null=True),
                "target_date": OutputFieldShape(kind="string", allow_null=True),
                "task_id": OutputFieldShape(kind="string", allow_null=True),
                "mode": OutputFieldShape(kind="string"),
                "missing_path_detail": OutputFieldShape(kind="string"),
                "exception_type": OutputFieldShape(kind="string"),
            },
        ),
    },
    "doc_sync": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {
                    "project_profile_path",
                    "changed_files",
                    "change_summary",
                    "session_handoff_path",
                    "work_backlog_index_path",
                    "latest_backlog_path",
                }
            ),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "change_summary": OutputFieldShape(kind="string", allow_null=True),
                "session_handoff_path": OutputFieldShape(kind="string", allow_null=True),
                "work_backlog_index_path": OutputFieldShape(kind="string", allow_null=True),
                "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
                "missing_path_detail": OutputFieldShape(kind="string"),
                "exception_type": OutputFieldShape(kind="string"),
            },
        ),
    },
    "merge_doc_reconcile": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {
                    "project_profile_path",
                    "merge_result_summary",
                    "session_handoff_path",
                    "work_backlog_index_path",
                    "latest_backlog_path",
                    "changed_files",
                }
            ),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "merge_result_summary": OutputFieldShape(kind="string"),
                "session_handoff_path": OutputFieldShape(kind="string", allow_null=True),
                "work_backlog_index_path": OutputFieldShape(kind="string", allow_null=True),
                "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "missing_path_detail": OutputFieldShape(kind="string"),
                "exception_type": OutputFieldShape(kind="string"),
            },
        ),
    },
    "validation_plan": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {"project_profile_path", "changed_files", "change_summary", "session_handoff_path", "latest_backlog_path"}
            ),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "change_summary": OutputFieldShape(kind="string", allow_null=True),
                "session_handoff_path": OutputFieldShape(kind="string", allow_null=True),
                "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
                "missing_path_detail": OutputFieldShape(kind="string"),
                "exception_type": OutputFieldShape(kind="string"),
            },
        ),
    },
    "code_index_update": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {"project_profile_path", "changed_files", "change_summary", "work_backlog_index_path", "session_handoff_path"}
            ),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "change_summary": OutputFieldShape(kind="string", allow_null=True),
                "work_backlog_index_path": OutputFieldShape(kind="string", allow_null=True),
                "session_handoff_path": OutputFieldShape(kind="string", allow_null=True),
                "missing_path_detail": OutputFieldShape(kind="string"),
                "exception_type": OutputFieldShape(kind="string"),
            },
        ),
    },
    "demo_workflow": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {
                    "example_project",
                    "project_profile_path",
                    "session_handoff_path",
                    "work_backlog_index_path",
                    "backlog_dir_path",
                    "latest_backlog_path",
                    "task_id",
                    "task_name",
                    "task_brief",
                    "task_status",
                    "changed_files",
                    "merge_result_summary",
                    "failed_step",
                    "failed_command",
                    "upstream_error_code",
                }
            ),
            properties={
                "example_project": OutputFieldShape(kind="string"),
                "project_profile_path": OutputFieldShape(kind="string"),
                "session_handoff_path": OutputFieldShape(kind="string"),
                "work_backlog_index_path": OutputFieldShape(kind="string"),
                "backlog_dir_path": OutputFieldShape(kind="string"),
                "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
                "task_id": OutputFieldShape(kind="string"),
                "task_name": OutputFieldShape(kind="string"),
                "task_brief": OutputFieldShape(kind="string"),
                "task_status": OutputFieldShape(kind="string"),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "merge_result_summary": OutputFieldShape(kind="string"),
                "failed_step": OutputFieldShape(kind="string"),
                "failed_command": OutputFieldShape(kind="list", item_kind="string"),
                "upstream_error_code": OutputFieldShape(kind="string"),
                "upstream_status": OutputFieldShape(kind="string"),
                "exception_type": OutputFieldShape(kind="string"),
            },
        ),
    },
    "existing_project_onboarding": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {
                    "project_profile_path",
                    "session_handoff_path",
                    "work_backlog_index_path",
                    "backlog_dir_path",
                    "repository_assessment_path",
                    "latest_backlog_path",
                    "changed_files",
                    "change_summary",
                }
            ),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "session_handoff_path": OutputFieldShape(kind="string"),
                "work_backlog_index_path": OutputFieldShape(kind="string"),
                "backlog_dir_path": OutputFieldShape(kind="string"),
                "repository_assessment_path": OutputFieldShape(kind="string", allow_null=True),
                "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "change_summary": OutputFieldShape(kind="string"),
            },
        ),
    },
    "read_only_entrypoint": {
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"action", "tool", "payload_json"}),
            properties={
                "action": OutputFieldShape(kind="string"),
                "tool": OutputFieldShape(kind="string", allow_null=True),
                "payload_json": OutputFieldShape(kind="string", allow_null=True),
                "allowed_fields": OutputFieldShape(kind="list", item_kind="string"),
                "missing_path": OutputFieldShape(kind="string"),
                "exception_type": OutputFieldShape(kind="string"),
                "exception": OutputFieldShape(kind="string"),
            },
        ),
    },
}
