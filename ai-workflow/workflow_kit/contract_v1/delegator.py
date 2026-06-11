"""§6.1 / §6.3 / §6.5 Delegator for Orchestrator ↔ Sub-agent Contract v1.

Decides role mapping per §6.1 (MUST delegate), rejects §6.3 (MUST NOT
delegate) actions, and (v0.5.7) supports multi-component fan-out via
`choose_roles` and model_tier recommendation via `recommend_model_tier`.

Used by the orchestrator side to auto-enforce the contract on every task it
considers doing itself (P0 from v0.5.5 pilot findings).

Reference: workflow-source/core/orchestrator_subagent_contract_v1.md §6
"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from typing import Any

# §3.1~§3.5 role names (must match contract v1 spec)
ALLOWED_ROLES = ("doc-worker", "code-worker", "validation-worker", "workflow-worker")
# §3 model_tier enum
ALLOWED_MODEL_TIERS = ("small", "main")
# §4 task_type enum
ALLOWED_TASK_TYPES = ("doc_draft", "code_change", "validation_run", "bounded_research")

# §6.1 mapping (deterministic): task_type → role
TASK_TYPE_TO_ROLE: dict[str, str] = {
    "doc_draft": "doc-worker",
    "code_change": "code-worker",
    "validation_run": "validation-worker",
    "bounded_research": "doc-worker",
}

# §6.3 MUST-NOT-delegate actions (7 in v0.5.6, +2 in v0.5.7 → 9 total).
# Substring match against `task.brief` and `task.expected_outputs.primary_artifact`.
# Each entry is (marker, human-readable reason).
MUST_NOT_DELEGATE_MARKERS: tuple[tuple[str, str], ...] = (
    ("handoff", "handoff 갱신은 orchestrator 직접 처리 (contract v1 §6.3)"),
    ("backlog", "backlog 갱신은 orchestrator 직접 처리 (contract v1 §6.3)"),
    ("state.json", "state.json 갱신은 orchestrator 직접 처리 (contract v1 §6.3)"),
    ("ask_user", "ask_user 호출은 orchestrator 직접 처리 (contract v1 §6.3)"),
    ("우선순위", "우선순위 결정은 orchestrator 직접 처리 (contract v1 §6.3)"),
    ("통합/리뷰", "sub-agent 출력 통합/리뷰는 orchestrator 직접 처리 (contract v1 §6.3)"),
    ("PR 본문", "PR 본문 작성은 orchestrator 직접 처리 (contract v1 §6.3)"),
    # v0.5.7 신규: cross-ref 갱신은 orchestrator 의 단일 진실 공급원 책임
    (
        "cross-ref",
        "cross-ref 갱신은 orchestrator 직접 처리 (contract v1 §6.3, v0.5.7)",
    ),
    # v0.5.7 신규: fan-in 통합 보고 작성을 sub-agent 가 시도하는 것도 거부
    (
        "fan-in 통합",
        "fan-in 통합 보고는 orchestrator 직접 처리 (contract v1 §6.3, v0.5.7)",
    ),
)

# v0.5.7: model_tier main 승격 keyword (case-insensitive substring match
# against task.brief, task.constraints, task.must_include).
# If any keyword matches, delegator.recommend_model_tier returns "main".
MAIN_TIER_KEYWORDS: tuple[str, ...] = (
    "아키텍처",
    "정책",
    "5+ 파일",
    "cross-cutting",
    "5+ source",
)


@dataclass
class DelegationDecision:
    """The orchestrator's delegation decision for a given task.

    `must_not_delegate` is True iff the task matches a §6.3 marker — in which
    case `rejected_reason` is set and `role` is None. Otherwise `role` is set
    per §6.1 and a fresh `delegation_id` is generated.
    """
    role: str | None
    must_not_delegate: bool
    rejected_reason: str | None
    warnings: list[str] = field(default_factory=list)
    delegation_id: str | None = None
    task_type: str | None = None
    recommended_model_tier: str | None = None  # v0.5.7
    sub_id: str | None = None  # v0.5.7: fan-out sub-task id


class DelegationRejected(ValueError):
    """Raised by strict-mode helpers when §6.3 is violated."""

    def __init__(self, decision: DelegationDecision) -> None:
        super().__init__(decision.rejected_reason or "delegation rejected")
        self.decision = decision


def _generate_delegation_id() -> str:
    """Generate a unique delegation_id per the §4 format `del-YYYY-MM-DD-NNN`.

    Default 8-char uuid4 hex suffix; callers may append their own suffix via
    `_generate_delegation_id_with_suffix` for sub-task fan-out.
    """
    from datetime import date
    today = date.today().isoformat()
    suffix = uuid.uuid4().hex[:8]
    return f"del-{today}-{suffix}"


def _generate_delegation_id_with_suffix(extra: str) -> str:
    """Generate `del-YYYY-MM-DD-<base>-<extra>` for sub-task fan-out (v0.5.7).

    Deprecated in v0.5.10. ``choose_roles`` now issues sub delegation_ids
    in the spec-declared ``{parent_delegation_id}-st-N`` format directly
    so that ``validate_fanin_output``'s prefix check (parent
    delegation_id must be a prefix of every sub_results[].delegation_id)
    is satisfied end-to-end. The random-base + sub_id-suffix format this
    helper produced did not respect that contract, so the v0.5.7.1
    verifier's actual walkthrough surfaced the gap. Kept for any external
    callers that imported it; new code should not use this helper.
    """
    from datetime import date
    today = date.today().isoformat()
    base = uuid.uuid4().hex[:6]
    safe_extra = "".join(c for c in extra if c.isalnum() or c in "-_")[:24]
    return f"del-{today}-{base}-{safe_extra}"


def _looks_like_must_not_delegate(task: dict[str, Any]) -> tuple[bool, str | None]:
    """Inspect task.brief and primary_artifact for §6.3 markers.

    Returns (matched, reason). The marker is substring-matched; matches are
    case-insensitive. We intentionally do NOT match "handoff" inside e.g.
    "handoff_path" — only the bare marker word to reduce false positives.

    v0.5.7: also inspects sub_tasks[].brief when present.
    """
    brief = str(task.get("brief", ""))
    primary = str(
        task.get("expected_outputs", {}).get("primary_artifact", "")
        if isinstance(task.get("expected_outputs"), dict)
        else ""
    )
    haystack = f"{brief}\n{primary}".lower()
    # v0.5.7: also scan sub_tasks brief
    for sub in task.get("sub_tasks", []) or []:
        if isinstance(sub, dict):
            haystack += "\n" + str(sub.get("brief", "")).lower()
    for marker, reason in MUST_NOT_DELEGATE_MARKERS:
        if marker.lower() in haystack:
            return True, reason
    return False, None


def _validate_sub_task(sub: Any, parent_delegation_id: str | None) -> None:
    """Validate a sub-task dict per §4.2 (v0.5.7).

    Raises ValueError on schema violation. When `parent_delegation_id` is
    provided AND `sub.parent_delegation_id` is set, they must match. If
    `sub.parent_delegation_id` is missing, the caller (choose_roles) will
    auto-fill it.
    """
    if not isinstance(sub, dict):
        raise ValueError(f"sub_task must be a dict, got {type(sub).__name__}")
    for field_name in ("sub_id", "task_type", "brief", "primary_artifact", "artifact_kind"):
        if field_name not in sub:
            raise ValueError(f"sub_task missing required field: {field_name}")
    if sub["task_type"] not in ALLOWED_TASK_TYPES:
        raise ValueError(
            f"sub_task.task_type must be one of {ALLOWED_TASK_TYPES}, "
            f"got {sub['task_type']!r}"
        )
    if sub["artifact_kind"] not in (
        "markdown", "python", "json", "toml", "text", "code", "other",
    ):
        raise ValueError(
            f"sub_task.artifact_kind invalid: {sub['artifact_kind']!r}"
        )
    if parent_delegation_id is not None and sub.get("parent_delegation_id") is not None:
        if sub["parent_delegation_id"] != parent_delegation_id:
            raise ValueError(
                f"sub_task.parent_delegation_id={sub.get('parent_delegation_id')!r} "
                f"does not match parent delegation_id={parent_delegation_id!r}"
            )


def recommend_model_tier(task: dict[str, Any]) -> str:
    """v0.5.7: 자동 model_tier 결정.

    Returns "main" if the task brief/constraints/must_include matches any
    main-tier keyword (architecture, policy, 5+ file cross-cutting, etc.).
    Otherwise returns "small".

    Respects explicit `task.required_model_tier` if set ("small" or "main").

    Args:
        task: A task dict matching contract v1 §4. Inspects `brief`,
            `constraints`, `expected_outputs.must_include`, and
            `required_model_tier`.

    Returns:
        "small" or "main".
    """
    explicit = task.get("required_model_tier")
    if explicit in ALLOWED_MODEL_TIERS:
        return explicit
    haystack_parts: list[str] = []
    haystack_parts.append(str(task.get("brief", "")))
    for c in task.get("constraints", []) or []:
        if isinstance(c, str):
            haystack_parts.append(c)
    expected = task.get("expected_outputs", {})
    if isinstance(expected, dict):
        for item in expected.get("must_include", []) or []:
            if isinstance(item, str):
                haystack_parts.append(item)
    haystack = "\n".join(haystack_parts).lower()
    for kw in MAIN_TIER_KEYWORDS:
        if kw.lower() in haystack:
            return "main"
    return "small"


def choose_role(task: dict[str, Any], *, strict: bool = False) -> DelegationDecision:
    """Decide which role a task should be delegated to, per contract v1 §6.

    Args:
        task: A task dict matching contract v1 §4 input schema. Inspects
            `task_type`, `brief`, `expected_outputs.primary_artifact`,
            `sub_tasks` (v0.5.7), and `required_model_tier` (v0.5.7).
        strict: If True, raise DelegationRejected on §6.3 violation instead
            of returning a `must_not_delegate=True` decision. Default False
            (the orchestrator's normal-mode "warn, do not delegate" path).

    Returns:
        DelegationDecision. On §6.3 violation, `must_not_delegate=True` and
        `rejected_reason` is set; orchestrator should NOT call the worker in
        that case. On valid §6.1 task, `role` and `delegation_id` are set.

    Raises:
        DelegationRejected: only if `strict=True` and §6.3 is violated.
        ValueError: if `task.task_type` is not a known enum value, or
            `sub_tasks` schema invalid.
    """
    if not isinstance(task, dict):
        raise ValueError(f"task must be a dict, got {type(task).__name__}")

    task_type = task.get("task_type")
    if task_type not in ALLOWED_TASK_TYPES:
        raise ValueError(
            f"task.task_type must be one of {ALLOWED_TASK_TYPES}, got {task_type!r}"
        )

    matched, reason = _looks_like_must_not_delegate(task)
    if matched:
        decision = DelegationDecision(
            role=None,
            must_not_delegate=True,
            rejected_reason=reason,
            warnings=[],
            delegation_id=None,
            task_type=task_type,
            recommended_model_tier=None,
        )
        if strict:
            raise DelegationRejected(decision)
        return decision

    role = TASK_TYPE_TO_ROLE[task_type]
    warnings: list[str] = []
    # §3 model_tier hint (best-effort, not strict) — v0.5.5
    if isinstance(task.get("constraints"), list):
        for c in task["constraints"]:
            if isinstance(c, str) and "main" in c.lower() and "model" in c.lower():
                warnings.append("task mentions 'main model' — orchestrator may promote model_tier")

    # v0.5.7: 자동 model_tier 결정
    recommended_tier = recommend_model_tier(task)

    return DelegationDecision(
        role=role,
        must_not_delegate=False,
        rejected_reason=None,
        warnings=warnings,
        delegation_id=_generate_delegation_id(),
        task_type=task_type,
        recommended_model_tier=recommended_tier,
    )


def choose_roles(
    task: dict[str, Any],
    *,
    strict: bool = False,
) -> list[DelegationDecision]:
    """v0.5.7: 멀티 컴포넌트 fan-out 일괄 위임 결정.

    - `task.sub_tasks` 가 비어있거나 없으면 단일 결과 (`[choose_role(task)]`)
    - `task.sub_tasks` 가 있으면 각 sub-task 별로 `choose_role` 호출하되
      `sub_id` 를 DelegationDecision 에 박고, delegation_id 는
      `<parent>-<sub_id>` 형식으로 생성한다.
    - 부모 task 자체는 별도 decision 으로 앞에 포함 (parent delegation_id).
    - §6.3 매치 시 (부모 또는 sub) 즉시 `must_not_delegate=True` decision 반환
      (전체 fan-out 거부, strict=True 면 raise).

    Args:
        task: A task dict matching contract v1 §4 + §4.2.
        strict: 부모 또는 sub 가 §6.3 위반 시 raise DelegationRejected.

    Returns:
        list[DelegationDecision]. 첫 항목이 부모, 이후가 sub-task 순서.

    Raises:
        DelegationRejected, ValueError (sub-task schema invalid).
    """
    if not isinstance(task, dict):
        raise ValueError(f"task must be a dict, got {type(task).__name__}")

    sub_tasks = task.get("sub_tasks") or []
    if not isinstance(sub_tasks, list):
        raise ValueError(f"task.sub_tasks must be a list, got {type(sub_tasks).__name__}")

    # 부모 task 에 대한 decision
    parent_decision = choose_role(task, strict=False)
    # strict=True 면 부모도 enforce
    if parent_decision.must_not_delegate and strict:
        raise DelegationRejected(parent_decision)

    parent_delegation_id = parent_decision.delegation_id

    if not sub_tasks:
        # 단일 위임 모드 — fan-out 아님
        return [parent_decision]

    # fan-out: 각 sub-task 별 decision
    decisions: list[DelegationDecision] = [parent_decision]
    for sub_index, sub in enumerate(sub_tasks, start=1):
        _validate_sub_task(sub, parent_delegation_id)
        # sub-task 를 단일 task 로 위장해서 choose_role 호출
        sub_as_task: dict[str, Any] = {
            "task_type": sub["task_type"],
            "brief": sub.get("brief", ""),
            "expected_outputs": {
                "primary_artifact": sub.get("primary_artifact", ""),
                "artifact_kind": sub.get("artifact_kind", "other"),
            },
        }
        sub_decision = choose_role(sub_as_task, strict=False)
        sub_decision.sub_id = sub["sub_id"]
        # v0.5.10 spec 정합 (v0.5.7.1 verifier actual walkthrough 가 surface):
        # sub delegation_id 는 반드시 parent delegation_id 의 prefix 여야
        # 한다 (v0.5.7 spec §4.2 + §5.2 의 fanin validator 가 enforce). 형식:
        # `{parent_delegation_id}-st-{i}` (1-based, e.g.
        # `del-2026-06-08-c6cc8da7-st-1`). sub_id 가 무엇이든
        # (`"a"`, `"st-1"`, `"core-build"` 등) 무관하게 st-N 형식 사용 —
        # spec 본체 (orchestrator_subagent_contract_v1.md §4.2) 의 권장
        # 형식과 회귀 test `_ok_payload` 의 정답 builder 와 정합.
        # v0.5.7.1 에서는 `_generate_delegation_id_with_suffix(sub_id)` 가
        # random parent prefix 를 발급해 fanin validator 가 거절했음.
        sub_decision.delegation_id = f"{parent_delegation_id}-st-{sub_index}"
        # sub 의 parent_delegation_id 자동 채움 (caller 가 미리 채웠다면 그대로 둠)
        if sub.get("parent_delegation_id") is None:
            sub["parent_delegation_id"] = parent_delegation_id
        decisions.append(sub_decision)
        if sub_decision.must_not_delegate and strict:
            raise DelegationRejected(sub_decision)

    return decisions
