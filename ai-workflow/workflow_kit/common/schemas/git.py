"""Pydantic models for git-conflict-resolver skill."""

from __future__ import annotations

from enum import Enum
from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class ResolutionStrategy(str, Enum):
    OURS = "ours"
    THEIRS = "theirs"
    MERGE = "merge"
    MANUAL = "manual"


class ConflictPoint(BaseModel):
    file_path: str = Field(..., description="Path to the file containing the conflict")
    line_number: int | None = Field(None, description="Approximate line number of the conflict")
    our_content: str = Field(..., description="Content from the current branch (OURS)")
    their_content: str = Field(..., description="Content from the incoming branch (THEIRS)")
    resolution_strategy: ResolutionStrategy = Field(..., description="Strategy used: 'ours', 'theirs', 'merge', 'manual'")
    resolution_note: str = Field(..., description="Rationale for the chosen resolution")


class GitConflictResolverOutput(BaseOutput):
    """Output contract for the git-conflict-resolver skill."""
    status: Status = Status.OK
    conflict_count: int = Field(..., description="Total number of conflicts found")
    resolved_count: int = Field(..., description="Number of conflicts successfully resolved")
    resolution_summary: str = Field(..., description="Human-readable summary of resolutions")
    conflicts: list[ConflictPoint] = Field(default_factory=list)
    unresolved_conflicts: list[dict[str, str]] = Field(default_factory=list)
    source_context: dict[str, str] = Field(default_factory=dict)
