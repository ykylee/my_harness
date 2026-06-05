"""Output contracts for read-only exploration and check tools."""

from __future__ import annotations
from workflow_kit.common.contracts.base import OutputFieldShape

READ_ONLY_SUCCESS_CONTRACTS: dict[str, frozenset[str]] = {
    "latest_backlog": frozenset({"latest_backlog_path", "candidates"}),
    "check_doc_metadata": frozenset({"checked_files", "missing_metadata"}),
    "check_doc_links": frozenset({"checked_files", "broken_links"}),
    "check_quickstart_stale_links": frozenset(
        {"checked_files", "missing_expected_links", "stale_link_warnings", "reasoning_notes"}
    ),
    "create_session_handoff_draft": frozenset({"draft_handoff", "source_context"}),
    "create_environment_record_stub": frozenset({"draft_record", "source_context"}),
    "suggest_impacted_docs": frozenset({"impacted_documents", "reasoning_notes"}),
    "smart_context_reader": frozenset({"extracted_content", "not_found_symbols"}),
}

READ_ONLY_FIELD_SHAPES: dict[str, dict[str, OutputFieldShape]] = {
    "latest_backlog": {
        "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
        "candidates": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "check_doc_metadata": {
        "checked_files": OutputFieldShape(kind="list", item_kind="string"),
        "missing_metadata": OutputFieldShape(
            kind="list",
            item_kind="object",
            required_keys=frozenset({"path", "missing_fields"}),
        ),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "check_doc_links": {
        "checked_files": OutputFieldShape(kind="list", item_kind="string"),
        "broken_links": OutputFieldShape(
            kind="list",
            item_kind="object",
            required_keys=frozenset({"path", "broken_links"}),
        ),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "suggest_impacted_docs": {
        "impacted_documents": OutputFieldShape(kind="list", item_kind="string"),
        "reasoning_notes": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "check_quickstart_stale_links": {
        "checked_files": OutputFieldShape(kind="list", item_kind="string"),
        "broken_links": OutputFieldShape(
            kind="list",
            item_kind="object",
            required_keys=frozenset({"path", "broken_links"}),
        ),
        "missing_expected_links": OutputFieldShape(
            kind="list",
            item_kind="object",
            required_keys=frozenset({"path", "missing_targets"}),
        ),
        "stale_link_warnings": OutputFieldShape(kind="list", item_kind="string"),
        "reasoning_notes": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
    "create_session_handoff_draft": {
        "draft_handoff": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"latest_backlog_path"}),
            properties={
                "latest_backlog_path": OutputFieldShape(kind="string", allow_null=True),
            },
        ),
    },
    "create_environment_record_stub": {
        "draft_record": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
        "source_context": OutputFieldShape(
            kind="object",
            required_keys=frozenset({"hostname", "os_type"}),
            properties={
                "hostname": OutputFieldShape(kind="string"),
                "os_type": OutputFieldShape(kind="string"),
            },
        ),
    },
    "smart_context_reader": {
        "extracted_content": OutputFieldShape(kind="list", item_kind="string"),
        "not_found_symbols": OutputFieldShape(kind="list", item_kind="string"),
        "warnings": OutputFieldShape(kind="list", item_kind="string"),
    },
}
