"""Pydantic models for backlog-related skills."""

from __future__ import annotations

from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class StatusRecommendation(BaseModel):
    value: str = Field(..., description="Recommended status value")
    reason: str = Field(..., description="Reason for the recommendation")


class BacklogUpdateSourceContext(BaseModel):
    daily_backlog_exists: bool
    existing_task_count: int
    project_profile_path: str


class BacklogUpdateOutput(BaseOutput):
    """Output contract for the backlog-update skill."""
    status: Status = Status.OK
    operation_type: str = Field(..., description="Classified operation type (create_entry, update_entry, ...)")
    target_backlog_path: str = Field(..., description="Path to the updated backlog file")
    task_id: str = Field(..., description="ID of the task that was updated")
    task_found: bool = Field(False, description="Whether the requested task_id was found in the daily backlog")
    draft_entry: list[str] = Field(..., description="Generated markdown lines for the backlog entry")
    status_recommendation: StatusRecommendation = Field(..., description="Conservative status recommendation")
    fields_requiring_confirmation: list[str] = Field(default_factory=list)
    handoff_update_note: str | None = None
    index_update_note: str | None = None
    validation_note: str | None = None
    state_cache_update_note: str | None = None
    state_cache_refresh_command: str | None = None
    state_cache_status: str | None = None
    state_cache_missing_paths: list[str] = Field(default_factory=list)
    apply_status: str | None = None
    written_paths: list[str] = Field(default_factory=list)
    created_paths: list[str] = Field(default_factory=list)
    updated_paths: list[str] = Field(default_factory=list)
    source_context: BacklogUpdateSourceContext


class CreateBacklogEntryOutput(BaseOutput):
    """Output contract for the create-backlog-entry skill."""
    status: Status = Status.OK
    draft_entry: list[str] = Field(..., description="Generated markdown lines for the backlog entry")
