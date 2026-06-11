"""§5 Output schema validator for Orchestrator ↔ Sub-agent Contract v1.

Validates a sub-agent output payload against contract v1 §5 schema. Used by the
orchestrator to auto-enforce the contract on every sub-agent response (P0 from
v0.5.5 pilot findings).

Reference: workflow-source/core/orchestrator_subagent_contract_v1.md §5
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

# Constants pulled from the contract spec. Keep in sync with §5 of
# workflow-source/core/orchestrator_subagent_contract_v1.md.
CONTRACT_VERSION = "1.0"
ALLOWED_STATUS = ("ok", "partial", "failed")
ALLOWED_ROLES = ("doc-worker", "code-worker", "validation-worker", "workflow-worker")
ALLOWED_MODEL_TIERS = ("small", "main")
ALLOWED_ARTIFACT_KINDS = ("markdown", "python", "json", "toml", "text", "code", "other")

REQUIRED_TOP_FIELDS = ("contract_version", "delegation_id", "completed_at", "worker", "result")
REQUIRED_WORKER_FIELDS = ("session_id", "role", "model_tier")
REQUIRED_RESULT_FIELDS = ("status", "summary", "artifacts", "written_paths", "next_step")
# v0.5.7: fan-in sub_result 최소 필드
REQUIRED_SUB_RESULT_FIELDS = ("sub_id", "delegation_id", "status", "summary", "written_paths")
# v0.5.7: artifacts action enum (created/modified/deleted)
ALLOWED_ARTIFACT_ACTIONS = ("created", "modified", "deleted")


@dataclass
class OutputValidationError:
    """A single field-level validation error."""
    field: str
    reason: str

    def __str__(self) -> str:  # pragma: no cover - trivial
        return f"{self.field}: {self.reason}"


@dataclass
class OutputValidationResult:
    """Result of validating a sub-agent output payload.

    `is_valid` is True iff `errors` is empty. `warnings` are non-fatal hints
    (e.g., `result.summary` is short, `artifacts` list is empty).
    """
    is_valid: bool
    errors: list[OutputValidationError] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    def raise_if_invalid(self) -> None:
        if not self.is_valid:
            joined = "; ".join(str(err) for err in self.errors)
            raise ValueError(f"Contract v1 §5 output schema violation: {joined}")


def _ensure_dict(payload: Any, field: str) -> dict[str, Any]:
    if not isinstance(payload, dict):
        raise ValueError(f"{field} must be a dict, got {type(payload).__name__}")
    return payload


def enforce_subagent_response(
    payload: Any,
    *,
    expected_delegation_id: str | None = None,
) -> None:
    """P0 enforcement: validate sub-agent response; raise ValueError on violation.

    Thin wrapper over ``validate_output`` + ``OutputValidationResult.raise_if_invalid``.
    See ``orchestrator_subagent_contract_v1.md`` §6.5 for the full policy.
    """
    result = validate_output(payload, expected_delegation_id=expected_delegation_id)
    result.raise_if_invalid()


def enforce_fanin_response(
    payload: Any,
    *,
    expected_parent_delegation_id: str | None = None,
) -> None:
    """P0 enforcement: validate fan-in payload; raise ValueError on violation.

    Thin wrapper over ``validate_fanin_output`` + ``OutputValidationResult.raise_if_invalid``.
    See ``orchestrator_subagent_contract_v1.md`` §6.5 for the full policy.
    """
    result = validate_fanin_output(
        payload, expected_parent_delegation_id=expected_parent_delegation_id
    )
    result.raise_if_invalid()


def validate_output(
    payload: Any,
    expected_delegation_id: str | None = None,
) -> OutputValidationResult:
    """Validate a sub-agent output payload against contract v1 §5.

    Args:
        payload: The deserialized JSON object a sub-agent produced.
        expected_delegation_id: If provided, the validator also asserts the
            payload's `delegation_id` matches this expected value. This is
            what orchestrators should pass to detect cross-delegation leaks.

    Returns:
        OutputValidationResult with `is_valid`, `errors`, `warnings`. Does not
        raise on validation failure — callers decide policy (reject, retry,
        escalate) per contract v1 §7.4.
    """
    errors: list[OutputValidationError] = []
    warnings: list[str] = []

    # Top-level shape
    if not isinstance(payload, dict):
        errors.append(OutputValidationError("payload", f"must be a dict, got {type(payload).__name__}"))
        return OutputValidationResult(is_valid=False, errors=errors)

    for field_name in REQUIRED_TOP_FIELDS:
        if field_name not in payload:
            errors.append(OutputValidationError(field_name, "missing required field"))

    if errors:
        return OutputValidationResult(is_valid=False, errors=errors)

    # contract_version
    version = payload["contract_version"]
    if version != CONTRACT_VERSION:
        errors.append(OutputValidationError(
            "contract_version", f"must be {CONTRACT_VERSION!r}, got {version!r}"
        ))

    # delegation_id match
    delegation_id = payload.get("delegation_id")
    if expected_delegation_id is not None and delegation_id != expected_delegation_id:
        errors.append(OutputValidationError(
            "delegation_id",
            f"expected {expected_delegation_id!r}, got {delegation_id!r}",
        ))

    # worker
    worker = _ensure_dict(payload["worker"], "worker")
    for field_name in REQUIRED_WORKER_FIELDS:
        if field_name not in worker:
            errors.append(OutputValidationError(f"worker.{field_name}", "missing required field"))
    if "role" in worker and worker["role"] not in ALLOWED_ROLES:
        errors.append(OutputValidationError(
            "worker.role", f"must be one of {ALLOWED_ROLES}, got {worker['role']!r}"
        ))
    if "model_tier" in worker and worker["model_tier"] not in ALLOWED_MODEL_TIERS:
        errors.append(OutputValidationError(
            "worker.model_tier", f"must be one of {ALLOWED_MODEL_TIERS}, got {worker['model_tier']!r}"
        ))

    # result
    result = _ensure_dict(payload["result"], "result")
    for field_name in REQUIRED_RESULT_FIELDS:
        if field_name not in result:
            errors.append(OutputValidationError(f"result.{field_name}", "missing required field"))
    if "status" in result and result["status"] not in ALLOWED_STATUS:
        errors.append(OutputValidationError(
            "result.status", f"must be one of {ALLOWED_STATUS}, got {result['status']!r}"
        ))
    if "summary" in result and not isinstance(result["summary"], str):
        errors.append(OutputValidationError("result.summary", "must be a string"))
    if "summary" in result and isinstance(result["summary"], str) and len(result["summary"]) < 3:
        warnings.append("result.summary is very short (< 3 chars); may be insufficient for orchestrator review")
    if "artifacts" in result and not isinstance(result["artifacts"], list):
        errors.append(OutputValidationError("result.artifacts", "must be a list"))
    if "artifacts" in result and isinstance(result["artifacts"], list):
        for idx, artifact in enumerate(result["artifacts"]):
            if not isinstance(artifact, dict):
                errors.append(OutputValidationError(
                    f"result.artifacts[{idx}]", f"must be a dict, got {type(artifact).__name__}"
                ))
                continue
            if "path" not in artifact or "kind" not in artifact:
                errors.append(OutputValidationError(
                    f"result.artifacts[{idx}]", "missing required 'path' or 'kind'"
                ))
            elif artifact["kind"] not in ALLOWED_ARTIFACT_KINDS:
                errors.append(OutputValidationError(
                    f"result.artifacts[{idx}].kind",
                    f"must be one of {ALLOWED_ARTIFACT_KINDS}, got {artifact['kind']!r}",
                ))
            # v0.5.7: action enum (optional, but validated if present)
            if "action" in artifact and artifact["action"] not in ALLOWED_ARTIFACT_ACTIONS:
                errors.append(OutputValidationError(
                    f"result.artifacts[{idx}].action",
                    f"must be one of {ALLOWED_ARTIFACT_ACTIONS}, got {artifact['action']!r}",
                ))
            # v0.5.7: action 통계 (added/removed/total) — optional, validated if present
            for stat_field in ("added", "removed", "total"):
                if stat_field in artifact and not isinstance(artifact[stat_field], int):
                    errors.append(OutputValidationError(
                        f"result.artifacts[{idx}].{stat_field}",
                        "must be an int",
                    ))
    if "written_paths" in result and not isinstance(result["written_paths"], list):
        errors.append(OutputValidationError("result.written_paths", "must be a list"))
    if "next_step" in result:
        if not isinstance(result["next_step"], str):
            errors.append(OutputValidationError("result.next_step", "must be a string"))
        elif not result["next_step"].strip():
            errors.append(OutputValidationError("result.next_step", "must be non-empty"))

    # Optional but validated when present
    if "validation_result" in result:
        vr = result["validation_result"]
        if not isinstance(vr, dict):
            errors.append(OutputValidationError("result.validation_result", "must be a dict"))
        else:
            if "ran" in vr and not isinstance(vr["ran"], bool):
                errors.append(OutputValidationError("result.validation_result.ran", "must be a bool"))
            if vr.get("ran") is True:
                if "status" not in vr:
                    errors.append(OutputValidationError(
                        "result.validation_result.status", "required when ran=true"
                    ))
                elif vr["status"] not in ("pass", "fail", "skipped"):
                    errors.append(OutputValidationError(
                        "result.validation_result.status",
                        f"must be one of ('pass', 'fail', 'skipped'), got {vr['status']!r}",
                    ))

    return OutputValidationResult(
        is_valid=not errors,
        errors=errors,
        warnings=warnings,
    )


# v0.5.7: sub_result status → 부모 status 자동 계산
SUB_RESULT_TO_PARENT_STATUS = {
    # (any_failed, any_partial) → parent status
    (False, False): "ok",
    (False, True): "partial",
    (True, False): "failed",
    (True, True): "failed",
}


def _validate_sub_result_entry(
    sub: Any,
    idx: int,
    expected_parent: str | None,
) -> tuple[list[OutputValidationError], str | None]:
    """Validate a single sub_result entry. Returns (errors, status)."""
    errs: list[OutputValidationError] = []
    if not isinstance(sub, dict):
        errs.append(OutputValidationError(
            f"result.sub_results[{idx}]",
            f"must be a dict, got {type(sub).__name__}",
        ))
        return errs, None
    for field_name in REQUIRED_SUB_RESULT_FIELDS:
        if field_name not in sub:
            errs.append(OutputValidationError(
                f"result.sub_results[{idx}].{field_name}",
                "missing required field",
            ))
    if "delegation_id" in sub and expected_parent is not None:
        # sub_result delegation_id 가 parent delegation_id 로 시작하는지 (prefix check)
        if not str(sub["delegation_id"]).startswith(expected_parent):
            errs.append(OutputValidationError(
                f"result.sub_results[{idx}].delegation_id",
                f"must start with parent delegation_id {expected_parent!r}, "
                f"got {sub['delegation_id']!r}",
            ))
    if "status" in sub and sub["status"] not in ALLOWED_STATUS:
        errs.append(OutputValidationError(
            f"result.sub_results[{idx}].status",
            f"must be one of {ALLOWED_STATUS}, got {sub['status']!r}",
        ))
    if "written_paths" in sub and not isinstance(sub["written_paths"], list):
        errs.append(OutputValidationError(
            f"result.sub_results[{idx}].written_paths",
            "must be a list",
        ))
    return errs, sub.get("status") if isinstance(sub, dict) else None


def _compute_aggregated_status(sub_statuses: list[str]) -> str:
    """Compute aggregated parent status from sub_result statuses (§5.2.1)."""
    any_failed = any(s == "failed" for s in sub_statuses)
    any_partial = any(s == "partial" for s in sub_statuses)
    return SUB_RESULT_TO_PARENT_STATUS[(any_failed, any_partial)]


def validate_fanin_output(
    payload: Any,
    expected_parent_delegation_id: str | None = None,
) -> OutputValidationResult:
    """v0.5.7: Validate a fan-in payload (§5.2).

    Runs the standard §5 schema validation first, then enforces the
    fan-in-specific rules:
    1. `parent_delegation_id` matches expected (when provided)
    2. `sub_results` is a list; each entry has the 5 required fields
    3. Each sub_result.delegation_id starts with the parent delegation_id
    4. `result.status` is consistent with sub_results aggregation
       (when sub_results is present)

    Args:
        payload: A sub-agent fan-in report dict (§5.2).
        expected_parent_delegation_id: If provided, asserts
            `payload.parent_delegation_id` matches AND that all
            `sub_results[].delegation_id` start with this value.

    Returns:
        OutputValidationResult. Aggregated status mismatch is reported
        as an error (not warning) — it indicates a fan-in bug.
    """
    # First, run standard §5 validation
    base = validate_output(payload, expected_delegation_id=None)
    errors: list[OutputValidationError] = list(base.errors)
    warnings: list[str] = list(base.warnings)

    if not base.is_valid:
        # base validate_output already reported; do not attempt fan-in logic
        # on a malformed payload (it may not be a dict).
        return OutputValidationResult(
            is_valid=False, errors=errors, warnings=warnings,
        )

    # parent_delegation_id field
    parent_id = payload.get("parent_delegation_id")
    if expected_parent_delegation_id is not None:
        if parent_id is None:
            errors.append(OutputValidationError(
                "parent_delegation_id",
                f"required when sub_results present, expected "
                f"{expected_parent_delegation_id!r}",
            ))
        elif parent_id != expected_parent_delegation_id:
            errors.append(OutputValidationError(
                "parent_delegation_id",
                f"expected {expected_parent_delegation_id!r}, got {parent_id!r}",
            ))

    result = payload.get("result", {})
    if not isinstance(result, dict):
        return OutputValidationResult(
            is_valid=not errors, errors=errors, warnings=warnings,
        )

    sub_results = result.get("sub_results")
    if sub_results is None:
        # Not a fan-in payload — base validation suffices
        return OutputValidationResult(
            is_valid=not errors, errors=errors, warnings=warnings,
        )

    # parent_delegation_id 가 있는데 sub_results 가 있으면 fan-in 모드
    if parent_id is None:
        errors.append(OutputValidationError(
            "parent_delegation_id",
            "required when result.sub_results is present",
        ))

    if not isinstance(sub_results, list):
        errors.append(OutputValidationError(
            "result.sub_results", "must be a list",
        ))
        return OutputValidationResult(
            is_valid=not errors, errors=errors, warnings=warnings,
        )

    if len(sub_results) == 0:
        errors.append(OutputValidationError(
            "result.sub_results", "must contain at least 1 sub-result",
        ))

    sub_statuses: list[str] = []
    for idx, sub in enumerate(sub_results):
        sub_errs, sub_status = _validate_sub_result_entry(
            sub, idx, expected_parent_delegation_id
        )
        errors.extend(sub_errs)
        if sub_status in ALLOWED_STATUS:
            sub_statuses.append(sub_status)

    # Aggregated status consistency check
    if sub_statuses and "status" in result:
        computed = _compute_aggregated_status(sub_statuses)
        declared = result["status"]
        if computed != declared:
            errors.append(OutputValidationError(
                "result.status",
                f"declared {declared!r} but sub_results aggregate to {computed!r} "
                f"(any_failed={any(s=='failed' for s in sub_statuses)}, "
                f"any_partial={any(s=='partial' for s in sub_statuses)})",
            ))

    return OutputValidationResult(
        is_valid=not errors,
        errors=errors,
        warnings=warnings,
    )
