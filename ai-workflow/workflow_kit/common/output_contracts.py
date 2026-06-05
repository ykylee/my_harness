"""Main entry point for output contracts, using Pydantic as the single source of truth.

This module exposes both the modern Pydantic-driven API (``PYDANTIC_MODEL_REGISTRY``,
``output_json_schema_bundle``) and a backwards-compatible legacy contract surface
(``COMMON_REQUIRED_KEYS``, ``SUCCESS_PATH_CONTRACTS``, ``ERROR_PATH_CONTRACTS``,
``output_field_shapes_schema``, ``output_error_field_shapes_schema``).

The legacy surface is derived from the Pydantic models so that downstream
consumers (sample validators, read-only MCP descriptors, schema generation
scripts) keep working after the Phase 9 Pydantic refactor.
"""

from __future__ import annotations

import copy
from pathlib import Path
from typing import Any, Type

from pydantic import BaseModel, ValidationError

from workflow_kit.common.schemas import (
    BacklogUpdateOutput,
    CreateBacklogEntryOutput,
    SessionStartOutput,
    DocSyncOutput,
    ReconcileOutput,
    IndexUpdateOutput,
    ValidationPlanOutput,
    GitConflictResolverOutput,
    OnboardingOutput,
    DemoWorkflowOutput,
    WorkerTask,
    WorkerResponse,
    WorkflowLinterOutput,
    LatestBacklogOutput,
    CheckDocMetadataOutput,
    CheckDocLinksOutput,
    SuggestImpactedDocsOutput,
    CheckQuickstartStaleLinksOutput,
    CreateSessionHandoffDraftOutput,
    CreateEnvironmentRecordStubOutput,
    SmartContextReaderOutput,
    ErrorOutput,
)


# ---------------------------------------------------------------------------
# Legacy compatibility surface (Phase 8 and earlier)
# ---------------------------------------------------------------------------

#: Field names that every output payload (success or error) must include.
COMMON_REQUIRED_KEYS: frozenset[str] = frozenset({"status", "tool_version", "warnings"})

#: Mapping from a Pydantic model class to the set of success-path required keys
#: that are NOT part of ``COMMON_REQUIRED_KEYS``.
_SUCCESS_REQUIRED_FIELDS_BY_MODEL: dict[Type[BaseModel], frozenset[str]] = {
    LatestBacklogOutput: frozenset({"latest_backlog_path", "candidates"}),
    CheckDocMetadataOutput: frozenset({"checked_files", "missing_metadata"}),
    CheckDocLinksOutput: frozenset({"checked_files", "broken_links"}),
    SuggestImpactedDocsOutput: frozenset({"impacted_documents", "reasoning_notes"}),
    CheckQuickstartStaleLinksOutput: frozenset({
        "checked_files",
        "broken_links",
        "missing_expected_links",
        "stale_link_warnings",
        "reasoning_notes",
    }),
    CreateSessionHandoffDraftOutput: frozenset({"draft_handoff", "source_context"}),
    CreateEnvironmentRecordStubOutput: frozenset({"draft_record", "source_context"}),
    SmartContextReaderOutput: frozenset({"extracted_content", "not_found_symbols"}),
    CreateBacklogEntryOutput: frozenset({"draft_entry"}),
    BacklogUpdateOutput: frozenset({
        "operation_type",
        "target_backlog_path",
        "task_id",
        "task_found",
        "draft_entry",
        "status_recommendation",
        "state_cache_status",
        "apply_status",
        "written_paths",
        "created_paths",
        "updated_paths",
        "source_context",
    }),
    DocSyncOutput: frozenset({
        "impacted_documents",
        "recommended_review_order",
        "source_context",
    }),
    ReconcileOutput: frozenset({
        "reconcile_targets",
        "state_conflicts",
        "source_context",
    }),
    IndexUpdateOutput: frozenset({"index_update_candidates", "source_context"}),
    ValidationPlanOutput: frozenset({
        "recommended_validation_levels",
        "documentation_checks",
        "source_context",
    }),
    GitConflictResolverOutput: frozenset({
        "conflict_count",
        "resolved_count",
        "resolution_summary",
    }),
    OnboardingOutput: frozenset({
        "onboarding_summary",
        "orchestration_plan",
        "source_context",
    }),
    DemoWorkflowOutput: frozenset({
        "workflow_summary",
        "orchestration_plan",
        "source_context",
    }),
    SessionStartOutput: frozenset({"source_documents", "summary", "recommended_next_action"}),
    WorkflowLinterOutput: frozenset({"issues", "warnings", "summary", "source_context"}),
}


def _build_contract_dicts() -> tuple[
    dict[str, frozenset[str]],
    dict[str, frozenset[str]],
    dict[str, dict[str, Any]],
    dict[str, dict[str, Any]],
]:
    """Derive the legacy contract dicts from the Pydantic models."""
    success: dict[str, frozenset[str]] = {}
    field_shapes: dict[str, dict[str, Any]] = {}
    for family, model_cls in PYDANTIC_MODEL_REGISTRY.items():
        success[family] = _SUCCESS_REQUIRED_FIELDS_BY_MODEL.get(model_cls, frozenset())
        field_shapes[family] = _build_flat_field_shapes(model_cls)

    # Error path: every family uses ErrorOutput; required keys beyond the
    # common ones are ``error`` and ``error_code``. ``source_context`` is
    # required for backwards compatibility with v0.5.0-beta contracts.
    error_required = {"error", "error_code", "source_context"}
    error_paths: dict[str, frozenset[str]] = {
        family: frozenset(error_required) for family in PYDANTIC_MODEL_REGISTRY
    }
    # Also expose the entrypoint family so legacy harness descriptors keep working.
    error_paths["read_only_entrypoint"] = frozenset(error_required)
    success.setdefault("read_only_entrypoint", frozenset())
    error_shape = _build_flat_field_shapes(ErrorOutput)
    field_shapes.setdefault("read_only_entrypoint", error_shape)

    error_field_shapes: dict[str, dict[str, Any]] = {
        family: error_shape for family in error_paths
    }
    return success, error_paths, field_shapes, error_field_shapes


def _build_flat_field_shapes(model_cls: Type[BaseModel]) -> dict[str, Any]:
    """Build a flat ``{field_name: shape}`` descriptor for a Pydantic model.

    The legacy v0.5.0-beta ``field_shapes`` contract stored per-field
    descriptors directly under the family key, e.g.
    ``field_shapes["latest_backlog"]["latest_backlog_path"]`` rather than
    nesting them under ``properties``. This helper preserves that shape so
    existing tests and downstream descriptors keep working.
    """
    flat: dict[str, Any] = {}
    for name, field in model_cls.model_fields.items():
        flat[name] = _field_descriptor(field.annotation, field.default)
    return flat


def _field_descriptor(annotation: Any, default: Any) -> dict[str, Any]:
    """Best-effort legacy ``field_shapes`` descriptor for a Pydantic field."""
    import types
    import typing

    origin = getattr(annotation, "__origin__", None)
    if annotation is type(None) or annotation is None:
        return {"kind": "null", "allow_null": True}
    if origin is typing.Union or (types.UnionType is not None and isinstance(annotation, types.UnionType)):
        args = [arg for arg in annotation.__args__ if arg is not type(None)]
        if len(args) == 1:
            return _field_descriptor(args[0], default)
        return {"kind": "any", "allow_null": True}
    if origin is list or origin is tuple:
        item_annotation = annotation.__args__[0] if annotation.__args__ else Any
        return {
            "kind": "list",
            "item_kind": _scalar_kind(item_annotation),
            "allow_null": False,
        }
    if origin is dict:
        return {
            "kind": "object",
            "item_kind": "string",
            "allow_null": False,
        }
    if annotation is Any:
        return {"kind": "any", "allow_null": True}
    if hasattr(annotation, "__members__"):  # Enum subclass
        return {"kind": "string", "allow_null": False}
    return {"kind": _scalar_kind(annotation), "allow_null": default is None}


def _scalar_kind(annotation: Any) -> str:
    name = getattr(annotation, "__name__", str(annotation))
    if name in {"str", "string"}:
        return "string"
    if name in {"int", "integer"}:
        return "integer"
    if name in {"float", "number"}:
        return "number"
    if name in {"bool", "boolean"}:
        return "boolean"
    return name or "any"


# Registry mapping family names to Pydantic models
PYDANTIC_MODEL_REGISTRY: dict[str, Type[BaseModel]] = {
    "backlog_update": BacklogUpdateOutput,
    "create_backlog_entry": CreateBacklogEntryOutput,
    "session_start": SessionStartOutput,
    "doc_sync": DocSyncOutput,
    "merge_doc_reconcile": ReconcileOutput,
    "code_index_update": IndexUpdateOutput,
    "validation_plan": ValidationPlanOutput,
    "git_conflict_resolver": GitConflictResolverOutput,
    "existing_project_onboarding": OnboardingOutput,
    "demo_workflow": DemoWorkflowOutput,
    "worker_task": WorkerTask,
    "worker_response": WorkerResponse,
    "workflow_linter": WorkflowLinterOutput,
    "latest_backlog": LatestBacklogOutput,
    "check_doc_metadata": CheckDocMetadataOutput,
    "check_doc_links": CheckDocLinksOutput,
    "suggest_impacted_docs": SuggestImpactedDocsOutput,
    "check_quickstart_stale_links": CheckQuickstartStaleLinksOutput,
    "create_session_handoff_draft": CreateSessionHandoffDraftOutput,
    "create_environment_record_stub": CreateEnvironmentRecordStubOutput,
    "smart_context_reader": SmartContextReaderOutput,
}


SUCCESS_PATH_CONTRACTS: dict[str, frozenset[str]]
ERROR_PATH_CONTRACTS: dict[str, frozenset[str]]
_FIELD_SHAPES: dict[str, dict[str, Any]]
_ERROR_FIELD_SHAPES: dict[str, dict[str, Any]]


def _ensure_legacy_contracts_built() -> None:
    """Populate the legacy contract module-level dicts on first access."""
    global SUCCESS_PATH_CONTRACTS, ERROR_PATH_CONTRACTS, _FIELD_SHAPES, _ERROR_FIELD_SHAPES
    if "_FIELD_SHAPES" in globals() and _FIELD_SHAPES:
        return
    success, error_paths, field_shapes, error_field_shapes = _build_contract_dicts()
    SUCCESS_PATH_CONTRACTS = success
    ERROR_PATH_CONTRACTS = error_paths
    _FIELD_SHAPES = field_shapes
    _ERROR_FIELD_SHAPES = error_field_shapes


# Eagerly populate the legacy contract dicts at import time so that consumers
# can use them as module-level constants (the historical pattern).
_ensure_legacy_contracts_built()


def output_field_shapes_schema() -> dict[str, dict[str, Any]]:
    """Return a per-family shape descriptor (legacy ``field_shapes`` contract)."""
    _ensure_legacy_contracts_built()
    return _FIELD_SHAPES


def output_error_field_shapes_schema() -> dict[str, dict[str, Any]]:
    """Return a per-family error shape descriptor (legacy contract)."""
    _ensure_legacy_contracts_built()
    return _ERROR_FIELD_SHAPES


def detect_sample_family(path: Path | str) -> str | None:
    """Infer the contract family from a sample filename."""
    name = Path(path).name.lower()
    # Normalize filename to family name (e.g., backlog-update.json -> backlog_update)
    family = name.split(".")[0].replace("-", "_")
    if family in PYDANTIC_MODEL_REGISTRY:
        return family
    if family == "read_only_entrypoint":
        return "read_only_entrypoint"
    return None


def validate_output_payload(payload: dict[str, object], *, family: str) -> list[str]:
    """Validate a payload against the Pydantic models."""
    errors: list[str] = []
    status = str(payload.get("status") or "")

    if status == "error":
        model_cls = ErrorOutput
    else:
        model_cls = PYDANTIC_MODEL_REGISTRY.get(family)
        if not model_cls:
            return [f"Unknown family: {family}"]

    try:
        model_cls.model_validate(payload)
    except ValidationError as e:
        for error in e.errors():
            loc = " -> ".join(str(l) for l in error["loc"])
            errors.append(f"[{loc}] {error['msg']}")

    return errors


def output_json_schema_bundle() -> dict[str, dict[str, object]]:
    """Return a bundle of all output and error schemas generated from Pydantic."""
    outputs = {}
    errors = {}

    error_schema = _inline_defs(ErrorOutput.model_json_schema())

    for family, model_cls in PYDANTIC_MODEL_REGISTRY.items():
        outputs[family] = _inline_defs(model_cls.model_json_schema())
        # For legacy compatibility in tests, provide the same error schema for each family
        errors[family] = error_schema

    return {
        "outputs": outputs,
        "errors": errors,
    }


def output_json_schema_for_family(family: str) -> dict[str, object]:
    """Return a single JSON schema representing the full output for a family.

    For families that do not have a dedicated Pydantic model (for example,
    legacy ``summarize_git_history`` or ``rotate_workflow_logs`` descriptors
    that predate the Phase 9 Pydantic refactor) a generic ``{"type": "object"}``
    schema is returned so descriptor generation keeps working.

    The returned schema has its ``$defs`` inlined so downstream descriptors and
    tests can inspect ``properties.status.enum`` directly without resolving
    ``$ref`` pointers. The v0.5.0-beta contract explicitly required that the
    JSON Schema draft and inline enums stay stable for harness consumers.
    """
    model_cls = PYDANTIC_MODEL_REGISTRY.get(family)
    if not model_cls:
        return {"type": "object", "description": f"Generic schema for unmodeled family '{family}'."}
    return _inline_defs(model_cls.model_json_schema())


def _inline_defs(schema: dict[str, object]) -> dict[str, object]:
    """Resolve ``$ref`` entries against ``$defs`` so the schema is self-contained."""
    defs = schema.get("$defs", {})
    if not defs:
        return schema

    def _resolve(node: object) -> object:
        if isinstance(node, dict):
            if "$ref" in node and isinstance(node["$ref"], str):
                ref_name = node["$ref"].rsplit("/", 1)[-1]
                target = defs.get(ref_name)
                if target is not None:
                    return _resolve(copy.deepcopy(target))
            return {key: _resolve(value) for key, value in node.items()}
        if isinstance(node, list):
            return [_resolve(item) for item in node]
        return node

    inlined = _resolve(schema)
    if isinstance(inlined, dict) and "$defs" in inlined:
        inlined = {key: value for key, value in inlined.items() if key != "$defs"}
    return inlined
