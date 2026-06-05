"""Pydantic models for code-index-update skill."""

from __future__ import annotations

from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class IndexUpdateSourceContext(BaseModel):
    project_profile_path: str
    project_name: str | None = None
    changed_files: list[str] = Field(default_factory=list)
    change_summary: str | None = None


class IndexUpdateOutput(BaseOutput):
    """Output contract for the code-index-update skill."""
    status: Status = Status.OK
    index_update_candidates: list[str] = Field(default_factory=list)
    priority_index_candidates: list[str] = Field(default_factory=list)
    stale_index_warnings: list[str] = Field(default_factory=list)
    reasoning_notes: list[str] = Field(default_factory=list)
    suggested_index_actions: list[str] = Field(default_factory=list)
    document_structure_signals: list[str] = Field(default_factory=list)
    missing_index_candidates: list[str] = Field(default_factory=list)
    confidence_notes: list[str] = Field(default_factory=list)
    source_context: IndexUpdateSourceContext
    apply_status: str | None = None
    written_paths: list[str] = Field(default_factory=list)
