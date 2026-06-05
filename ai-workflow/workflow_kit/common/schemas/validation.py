"""Pydantic models for validation-related skills."""

from __future__ import annotations

from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class ValidationCommand(BaseModel):
    command: str
    reason: str


class DeferredValidationItem(BaseModel):
    item: str
    suggested_record_path: str


class ValidationPlanSourceContext(BaseModel):
    project_profile_path: str
    project_name: str | None = None
    changed_files: list[str] = Field(default_factory=list)
    change_summary: str | None = None


class ValidationPlanOutput(BaseOutput):
    """Output contract for the validation-plan skill."""
    status: Status = Status.OK
    detected_change_types: list[str] = Field(default_factory=list)
    recommended_validation_levels: list[str] = Field(default_factory=list)
    recommended_commands: list[ValidationCommand] = Field(default_factory=list)
    commands_requiring_confirmation: list[ValidationCommand] = Field(default_factory=list)
    documentation_checks: list[str] = Field(default_factory=list)
    evidence_expectations: list[str] = Field(default_factory=list)
    deferred_validation_items: list[DeferredValidationItem] = Field(default_factory=list)
    confidence_notes: list[str] = Field(default_factory=list)
    scaffold_status: str | None = None
    scaffold_path: str | None = None
    written_paths: list[str] = Field(default_factory=list)
    source_context: ValidationPlanSourceContext
