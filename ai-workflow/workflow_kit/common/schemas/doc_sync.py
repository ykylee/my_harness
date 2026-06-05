"""Pydantic models for doc-sync skill."""

from __future__ import annotations

from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class DocSyncSourceContext(BaseModel):
    project_profile_path: str
    changed_files: list[str]
    change_summary: str | None = None
    session_handoff_path: str | None = None
    work_backlog_index_path: str | None = None
    latest_backlog_path: str | None = None


class DocSyncOutput(BaseOutput):
    """Output contract for the doc-sync skill."""
    status: Status = Status.OK
    impacted_documents: list[str] = Field(default_factory=list)
    hub_update_candidates: list[str] = Field(default_factory=list)
    status_doc_candidates: list[str] = Field(default_factory=list)
    validation_doc_candidates: list[str] = Field(default_factory=list)
    stale_warnings: list[str] = Field(default_factory=list)
    reasoning_notes: list[str] = Field(default_factory=list)
    recommended_review_order: list[str] = Field(default_factory=list)
    follow_up_actions: list[str] = Field(default_factory=list)
    confidence_notes: list[str] = Field(default_factory=list)
    source_context: DocSyncSourceContext
    apply_status: str | None = None
    written_paths: list[str] = Field(default_factory=list)
