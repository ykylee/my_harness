"""Pydantic models for read-only exploration skills."""

from __future__ import annotations

from typing import Any
from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class LatestBacklogOutput(BaseOutput):
    status: Status = Status.OK
    latest_backlog_path: str | None = None
    candidates: list[str] = Field(default_factory=list)


class MissingMetadata(BaseModel):
    path: str
    missing_fields: list[str]


class CheckDocMetadataOutput(BaseOutput):
    status: Status = Status.OK
    checked_files: list[str] = Field(default_factory=list)
    missing_metadata: list[MissingMetadata] = Field(default_factory=list)


class BrokenLinks(BaseModel):
    path: str
    broken_links: list[str]


class CheckDocLinksOutput(BaseOutput):
    status: Status = Status.OK
    checked_files: list[str] = Field(default_factory=list)
    broken_links: list[BrokenLinks] = Field(default_factory=list)


class ImpactedDoc(BaseModel):
    path: str
    reasoning: str | None = None


class SuggestImpactedDocsOutput(BaseOutput):
    status: Status = Status.OK
    impacted_documents: list[str] = Field(default_factory=list)
    reasoning_notes: list[str] = Field(default_factory=list)


class StaleLinkWarning(BaseModel):
    path: str
    missing_targets: list[str]


class CheckQuickstartStaleLinksOutput(BaseOutput):
    status: Status = Status.OK
    checked_files: list[str] = Field(default_factory=list)
    broken_links: list[BrokenLinks] = Field(default_factory=list)
    missing_expected_links: list[StaleLinkWarning] = Field(default_factory=list)
    stale_link_warnings: list[str] = Field(default_factory=list)
    reasoning_notes: list[str] = Field(default_factory=list)


class CreateSessionHandoffDraftOutput(BaseOutput):
    status: Status = Status.OK
    draft_handoff: list[str] = Field(default_factory=list)
    source_context: dict[str, Any]


class CreateEnvironmentRecordStubOutput(BaseOutput):
    status: Status = Status.OK
    draft_record: list[str] = Field(default_factory=list)
    source_context: dict[str, Any]


class SmartContextReaderOutput(BaseOutput):
    status: Status = Status.OK
    extracted_content: list[str] = Field(default_factory=list)
    not_found_symbols: list[str] = Field(default_factory=list)
