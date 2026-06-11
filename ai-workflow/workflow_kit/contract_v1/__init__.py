"""Orchestrator ↔ Sub-agent Contract v1 enforcement modules.

Two enforcement helpers introduced in v0.5.6 (P0 from v0.5.5 pilot findings):
- `output_validator`: validates a sub-agent output payload against §5 of the contract.
- `delegator.choose_role`: decides role mapping per §6.1 + rejects §6.3 actions.

v0.5.7 extends with multi-component fan-out/in (P1 from v0.5.5 pilot):
- `output_validator.validate_fanin_output`: validates fan-in payload + sub_results
- `delegator.choose_roles`: batch delegation decisions for `task.sub_tasks`
- `delegator.recommend_model_tier`: auto small/main decision per keyword rules

v0.5.11 introduces the P0 enforcement hook (contract §6.5):
- `output_validator.enforce_subagent_response` / `enforce_fanin_response`:
  thin wrappers that call ``validate_output``/``validate_fanin_output`` and
  raise ``ValueError`` on the first invalid result. Mavis / Mavis callers
  should wire these in at the orchestrator boundary (see
  ``orchestrator_contract_v1_wire_guide.md`` §2/§3).

Both modules are pure Python (no external deps beyond the standard library) so
they can be imported from any orchestrator runtime (Mavis, mavis, OpenCode,
Gemini CLI, etc.) and from sub-agent runtimes that already depend on the
standard_ai_workflow kit.

Reference: workflow-source/core/orchestrator_subagent_contract_v1.md
"""

from .output_validator import (
    OutputValidationError,
    OutputValidationResult,
    enforce_fanin_response,
    enforce_subagent_response,
    validate_output,
    validate_fanin_output,
)
from .delegator import (
    DelegationDecision,
    DelegationRejected,
    choose_role,
    choose_roles,
    recommend_model_tier,
)

__all__ = [
    "OutputValidationError",
    "OutputValidationResult",
    "validate_output",
    "validate_fanin_output",
    "enforce_subagent_response",
    "enforce_fanin_response",
    "DelegationDecision",
    "DelegationRejected",
    "choose_role",
    "choose_roles",
    "recommend_model_tier",
]
