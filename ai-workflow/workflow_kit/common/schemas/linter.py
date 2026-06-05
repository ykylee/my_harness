"""Pydantic models for workflow-linter skill."""

from __future__ import annotations

from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class LinterIssue(BaseModel):
    type: str = Field(..., description="Category of the issue (e.g., 'sync_error', 'broken_link')")
    code: str = Field(..., description="Specific error code")
    description: str = Field(..., description="Detailed description of the issue")
    severity: str = Field(..., description="Severity level: 'low', 'medium', 'high'")
    fix_suggestion: str | None = Field(None, description="Recommended action to resolve the issue")


class LinterSummary(BaseModel):
    total_issues: int
    sync_errors: int
    broken_links: int
    bloat_warnings: int


class WorkflowLinterOutput(BaseOutput):
    """Output contract for the workflow-linter skill."""
    status: Status = Status.OK
    issues: list[LinterIssue] = Field(default_factory=list)
    warnings: list[str] = Field(default_factory=list)
    summary: LinterSummary
    source_context: dict[str, str] = Field(default_factory=dict)

    @property
    def linter_status(self) -> str:
        """Legacy ``linter_status`` key for downstream tests."""
        return "issues_found" if self.issues else "ok"
