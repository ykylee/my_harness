"""Definition of capabilities for different AI harnesses."""

from __future__ import annotations

from enum import Enum
from typing import Any
from pydantic import BaseModel


class MultiAgentStrategy(str, Enum):
    DYNAMIC_DELEGATION = "dynamic_delegation" # Native sub-agents (Antigravity)
    SEQUENTIAL_PARTITIONING = "sequential_partitioning" # One agent, multiple logical phases (Codex)
    NONE = "none"


class HarnessCapability(BaseModel):
    name: str
    multi_agent_strategy: MultiAgentStrategy
    has_browser: bool = False
    has_artifacts: bool = False
    has_native_subagents: bool = False


HARNESS_CAPABILITIES = {
    "antigravity": HarnessCapability(
        name="antigravity",
        multi_agent_strategy=MultiAgentStrategy.DYNAMIC_DELEGATION,
        has_browser=True,
        has_artifacts=True,
        has_native_subagents=True,
    ),
    "codex": HarnessCapability(
        name="codex",
        multi_agent_strategy=MultiAgentStrategy.SEQUENTIAL_PARTITIONING,
        has_browser=False,
        has_artifacts=False,
        has_native_subagents=False,
    ),
}


def get_harness_capability(harness_name: str) -> HarnessCapability:
    """Return the capability record for a given harness, defaulting to Codex-like limitations."""
    return HARNESS_CAPABILITIES.get(harness_name.lower(), HARNESS_CAPABILITIES["codex"])
