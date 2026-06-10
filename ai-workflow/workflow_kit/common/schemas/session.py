"""Pydantic models for session-start skill."""

from __future__ import annotations

from typing import Any

from pydantic import BaseModel, ConfigDict, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class SessionStartSourceDocs(BaseModel):
    """Paths to the canonical three source documents."""

    session_handoff_path: str
    work_backlog_index_path: str
    project_profile_path: str


class WikiLintSummary(BaseModel):
    """Subset of wiki-lint result exposed via session-start (D-71)."""

    errors: int = 0
    warns: int = 0
    infos: int = 0
    pages_scanned: int = 0
    last_report: str | None = None
    extra: dict[str, Any] = Field(default_factory=dict)


class WikiVaultStatus(BaseModel):
    """Status of the LLM Wiki vault (D-71), if discovered.

    - `exists=False` 면 vault 미설치. 다른 필드는 무시.
    - `last_log_entry` 는 log.md 의 가장 최근 `## [YYYY-MM-DD] ...` 한 줄.
    - `lint` 는 wiki-lint 실행 결과 요약 (오래 걸리면 skip 가능).
    - `wiki_page_count` 는 `wiki/{concepts,entities,topics,sources,comparisons,query,meta}/` 의 `*.md` 총 개수.
    - `raw_entry_count` 는 `raw/_manifest.md` 의 `## [YYYY-MM-DD]` 항목 수.
    """

    model_config = ConfigDict(extra="allow")

    path: str
    exists: bool = False
    log_md_exists: bool = False
    last_log_entry: str | None = None
    wiki_page_count: int = 0
    raw_entry_count: int = 0
    lint: WikiLintSummary | None = None
    warnings: list[str] = Field(default_factory=list)
    notes: list[str] = Field(default_factory=list)


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
    wiki_vault: WikiVaultStatus | None = None

    @property
    def primary_summary(self) -> str:
        """Backwards-compatible string view of the summary list."""
        if not self.summary:
            return ""
        return self.summary[0]
