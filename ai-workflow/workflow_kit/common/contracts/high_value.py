"""Output contracts for high-value orchestration and state update tools."""

from __future__ import annotations
from workflow_kit.common.contracts.base import OutputFieldShape

HIGH_VALUE_SUCCESS_CONTRACTS: dict[str, frozenset[str]] = {
    "session_start": frozenset({"summary", "recommended_next_action"}),
    "backlog_update": frozenset({"operation_type", "target_backlog_path", "source_context"}),
    "doc_sync": frozenset({"impacted_documents", "recommended_review_order", "source_context"}),
    "merge_doc_reconcile": frozenset({"reconcile_targets", "state_conflicts", "source_context"}),
    "validation_plan": frozenset({"recommended_validation_levels", "documentation_checks", "source_context"}),
    "code_index_update": frozenset({"index_update_candidates", "source_context"}),
    "create_backlog_entry": frozenset({"draft_entry"}),
    "demo_workflow": frozenset({"orchestration_plan", "workflow_summary", "source_context"}),
    "existing_project_onboarding": frozenset({"orchestration_plan", "onboarding_summary", "source_context"}),
}

HIGH_VALUE_FIELD_SHAPES: dict[str, dict[str, OutputFieldShape]] = {
    "session_start": {
        "summary": OutputFieldShape(kind="list", item_kind="string"),
        "in_progress_items": OutputFieldShape(kind="list", item_kind="string"),
        "blocked_items": OutputFieldShape(kind="list", item_kind="string"),
        "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
        "next_documents": OutputFieldShape(kind="list", item_kind="string"),
        "validation_notes": OutputFieldShape(kind="list", item_kind="string"),
        "environment_constraints": OutputFieldShape(kind="list", item_kind="string"),
        "source_documents": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"session_handoff_path", "work_backlog_index_path", "project_profile_path"}),
            properties={
                "session_handoff_path": OutputFieldShape(kind="string"),
                "work_backlog_index_path": OutputFieldShape(kind="string"),
                "project_profile_path": OutputFieldShape(kind="string"),
            },
        ),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "backlog_update": {
        "target_backlog_path": OutputFieldShape(kind="string"),
        "task_id": OutputFieldShape(kind="string"),
        "draft_entry": OutputFieldShape(kind="list", item_kind="string"),
        "status_recommendation": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"value", "reason"}),
            properties={
                "value": OutputFieldShape(kind="string"),
                "reason": OutputFieldShape(kind="string"),
            },
        ),
        "fields_requiring_confirmation": OutputFieldShape(kind="list", item_kind="string"),
        "index_update_note": OutputFieldShape(kind="string", allow_null=True),
        "handoff_update_note": OutputFieldShape(kind="string", allow_null=True),
        "validation_note": OutputFieldShape(kind="string", allow_null=True),
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"project_profile_path", "daily_backlog_exists", "existing_task_count"}),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "daily_backlog_exists": OutputFieldShape(kind="boolean"),
                "existing_task_count": OutputFieldShape(kind="integer"),
            },
        ),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "create_backlog_entry": {
        "draft_entry": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "doc_sync": {
        "impacted_documents": OutputFieldShape(kind="list", item_kind="string"),
        "hub_update_candidates": OutputFieldShape(kind="list", item_kind="string"),
        "status_doc_candidates": OutputFieldShape(kind="list", item_kind="string"),
        "validation_doc_candidates": OutputFieldShape(kind="list", item_kind="string"),
        "stale_warnings": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
        "reasoning_notes": OutputFieldShape(kind="list", item_kind="string"),
        "recommended_review_order": OutputFieldShape(kind="list", item_kind="string"),
        "follow_up_actions": OutputFieldShape(kind="list", item_kind="string"),
        "confidence_notes": OutputFieldShape(kind="list", item_kind="string"),
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"project_profile_path", "changed_files", "change_summary"}),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "change_summary": OutputFieldShape(kind="string", allow_null=True),
            },
        ),
    },
    "merge_doc_reconcile": {
        "reconcile_targets": OutputFieldShape(kind="list", item_kind="string"),
        "state_conflicts": OutputFieldShape(kind="list", item_kind="string"),
        "reconfirmation_points": OutputFieldShape(kind="list", item_kind="string"),
        "draft_reconcile_notes": OutputFieldShape(kind="list", item_kind="string"),
        "recommended_review_order": OutputFieldShape(kind="list", item_kind="string"),
        "handoff_update_note": OutputFieldShape(kind="string", allow_null=True),
        "backlog_update_note": OutputFieldShape(kind="string", allow_null=True),
        "hub_update_note": OutputFieldShape(kind="string", allow_null=True),
        "validation_follow_up": OutputFieldShape(kind="string", allow_null=True),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"project_profile_path", "merge_result_summary", "changed_files"}),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "merge_result_summary": OutputFieldShape(kind="string"),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
            },
        ),
    },
    "code_index_update": {
        "index_update_candidates": OutputFieldShape(kind="list", item_kind="string"),
        "priority_index_candidates": OutputFieldShape(kind="list", item_kind="string"),
        "stale_index_warnings": OutputFieldShape(kind="list", item_kind="string"),
        "document_structure_signals": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"project_profile_path", "changed_files", "change_summary"}),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "project_name": OutputFieldShape(kind="string", allow_null=True),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "change_summary": OutputFieldShape(kind="string", allow_null=True),
                "work_backlog_index_path": OutputFieldShape(kind="string", allow_null=True),
                "session_handoff_path": OutputFieldShape(kind="string", allow_null=True),
            },
        ),
    },
    "validation_plan": {
        "detected_change_types": OutputFieldShape(kind="list", item_kind="string"),
        "recommended_validation_levels": OutputFieldShape(kind="list", item_kind="string"),
        "recommended_commands": OutputFieldShape(
            kind="list",
            item_kind="object",
            required_keys=frozenset({"command", "reason"}),
            item_properties={
                "command": OutputFieldShape(kind="string"),
                "reason": OutputFieldShape(kind="string"),
            },
        ),
        "commands_requiring_confirmation": OutputFieldShape(
            kind="list",
            item_kind="object",
            required_keys=frozenset({"command", "reason"}),
            item_properties={
                "command": OutputFieldShape(kind="string"),
                "reason": OutputFieldShape(kind="string"),
            },
        ),
        "documentation_checks": OutputFieldShape(kind="list", item_kind="string"),
        "deferred_validation_items": OutputFieldShape(
            kind="list",
            item_kind="object",
            required_keys=frozenset({"item", "suggested_record_path"}),
            item_properties={
                "item": OutputFieldShape(kind="string"),
                "suggested_record_path": OutputFieldShape(kind="string"),
            },
        ),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
        "confidence_notes": OutputFieldShape(kind="list", item_kind="string"),
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"project_profile_path", "project_name", "changed_files", "change_summary"}),
            properties={
                "project_profile_path": OutputFieldShape(kind="string"),
                "project_name": OutputFieldShape(kind="string"),
                "changed_files": OutputFieldShape(kind="list", item_kind="string"),
                "change_summary": OutputFieldShape(kind="string"),
            },
        ),
    },
    "demo_workflow": {
        "orchestration_plan": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"recommended_pattern", "model_split", "worker_assignments", "integration_notes"}),
            properties={
                "recommended_pattern": OutputFieldShape(kind="string"),
                "model_split": OutputFieldShape(
                    kind="object",
                    required_keys=frozenset({"orchestrator", "doc_worker", "validation_worker"}),
                    properties={
                        "orchestrator": OutputFieldShape(kind="string"),
                        "doc_worker": OutputFieldShape(kind="string"),
                        "code_worker": OutputFieldShape(kind="string"),
                        "validation_worker": OutputFieldShape(kind="string"),
                    },
                ),
                "worker_assignments": OutputFieldShape(
                    kind="list",
                    item_kind="object",
                    required_keys=frozenset({"worker", "model", "responsibilities", "backing_steps"}),
                    item_properties={
                        "worker": OutputFieldShape(kind="string"),
                        "model": OutputFieldShape(kind="string"),
                        "responsibilities": OutputFieldShape(kind="list", item_kind="string"),
                        "backing_steps": OutputFieldShape(kind="list", item_kind="string"),
                    },
                ),
                "integration_notes": OutputFieldShape(kind="list", item_kind="string"),
            },
        ),
        "workflow_summary": OutputFieldShape(
            kind="object",
            required_keys=frozenset(
                {
                    "current_baseline",
                    "target_backlog_path",
                    "primary_impacted_documents",
                    "recommended_validation_levels",
                    "priority_index_candidates",
                    "reconcile_targets",
                }
            ),
            properties={
                "current_baseline": OutputFieldShape(kind="list", item_kind="string"),
                "target_backlog_path": OutputFieldShape(kind="string"),
                "primary_impacted_documents": OutputFieldShape(kind="list", item_kind="string"),
                "recommended_validation_levels": OutputFieldShape(kind="list", item_kind="string"),
                "priority_index_candidates": OutputFieldShape(kind="list", item_kind="string"),
                "reconcile_targets": OutputFieldShape(kind="list", item_kind="string"),
            },
        ),
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
            },
        ),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "existing_project_onboarding": {
        "orchestration_plan": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"recommended_pattern", "model_split", "worker_assignments", "integration_notes"}),
            properties={
                "recommended_pattern": OutputFieldShape(kind="string"),
                "model_split": OutputFieldShape(
                    kind="object",
                    required_keys=frozenset({"orchestrator", "doc_worker", "validation_worker"}),
                    properties={
                        "orchestrator": OutputFieldShape(kind="string"),
                        "doc_worker": OutputFieldShape(kind="string"),
                        "code_worker": OutputFieldShape(kind="string"),
                        "validation_worker": OutputFieldShape(kind="string"),
                    },
                ),
                "worker_assignments": OutputFieldShape(
                    kind="list",
                    item_kind="object",
                    required_keys=frozenset({"worker", "model", "responsibilities", "backing_steps"}),
                    item_properties={
                        "worker": OutputFieldShape(kind="string"),
                        "model": OutputFieldShape(kind="string"),
                        "responsibilities": OutputFieldShape(kind="list", item_kind="string"),
                        "backing_steps": OutputFieldShape(kind="list", item_kind="string"),
                    },
                ),
                "integration_notes": OutputFieldShape(kind="list", item_kind="string"),
            },
        ),
        "onboarding_summary": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"review_assessment_first", "primary_stack", "inferred_commands", "recommended_next_steps"}),
            properties={
                "review_assessment_first": OutputFieldShape(kind="boolean"),
                "primary_stack": OutputFieldShape(kind="string", allow_null=True),
                "inferred_commands": OutputFieldShape(
                    kind="object",
                    required_keys=frozenset({"install", "run", "quick_test", "isolated_test", "smoke_check"}),
                    properties={
                        "install": OutputFieldShape(kind="string", allow_null=True),
                        "run": OutputFieldShape(kind="string", allow_null=True),
                        "quick_test": OutputFieldShape(kind="string", allow_null=True),
                        "isolated_test": OutputFieldShape(kind="string", allow_null=True),
                        "smoke_check": OutputFieldShape(kind="string", allow_null=True),
                    },
                ),
                "recommended_next_steps": OutputFieldShape(kind="list", item_kind="string"),
            },
        ),
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
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
}
