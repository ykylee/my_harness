"""Pydantic models for merge-doc-reconcile skill."""

from __future__ import annotations

from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class ReconcileSourceContext(BaseModel):
    project_profile_path: str
    merge_result_summary: str
    changed_files: list[str] = Field(default_factory=list)


class ReconcileOutput(BaseOutput):
    """Output contract for the merge-doc-reconcile skill."""
    status: Status = Status.OK
    reconcile_targets: list[str] = Field(default_factory=list)
    state_conflicts: list[str] = Field(default_factory=list)
    reconfirmation_points: list[str] = Field(default_factory=list)
    draft_reconcile_notes: list[str] = Field(default_factory=list)
    recommended_review_order: list[str] = Field(default_factory=list)
    handoff_update_note: str | None = None
    backlog_update_note: str | None = None
    hub_update_note: str | None = None
    state_cache_update_note: str | None = None
    state_cache_refresh_command: str | None = None
    state_cache_status: str | None = None
    state_cache_missing_paths: list[str] = Field(default_factory=list)
    apply_status: str | None = None
    written_paths: list[str] = Field(default_factory=list)
    validation_follow_up: str | None = None
    source_context: ReconcileSourceContext
