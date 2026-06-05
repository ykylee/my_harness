"""Pydantic models for session-start skill."""

from __future__ import annotations

from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class SessionStartSourceDocs(BaseModel):
    session_handoff_path: str
    work_backlog_index_path: str
    project_profile_path: str


class SessionStartOutput(BaseOutput):
    """Output contract for the session-start skill."""
    status: Status = Status.OK
    summary: list[str] = Field(default_factory=list)
    in_progress_items: list[str] = Field(default_factory=list)
    blocked_items: list[str] = Field(default_factory=list)
    latest_backlog_path: str | None = None
    next_documents: list[str] = Field(default_factory=list)
    recommended_next_action: str = ""
    validation_notes: list[str] = Field(default_factory=list)
    environment_constraints: list[str] = Field(default_factory=list)
    source_documents: SessionStartSourceDocs

    @property
    def primary_summary(self) -> str:
        """Backwards-compatible string view of the summary list."""
        if not self.summary:
            return ""
        return self.summary[0]
