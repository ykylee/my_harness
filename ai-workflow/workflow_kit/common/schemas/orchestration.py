"""Pydantic models for orchestration and demo skills."""

from __future__ import annotations

from typing import Any
from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class OrchestrationPlan(BaseModel):
    recommended_pattern: str
    model_split: dict[str, str]
    worker_assignments: list[dict[str, Any]] = Field(default_factory=list)
    integration_notes: list[str] = Field(default_factory=list)


class OnboardingSummary(BaseModel):
    review_assessment_first: bool
    primary_stack: str | None = None
    inferred_commands: dict[str, str | None]
    recommended_next_steps: list[str] = Field(default_factory=list)


class OnboardingOutput(BaseOutput):
    """Output contract for the existing-project-onboarding skill."""
    status: Status = Status.OK
    orchestration_plan: OrchestrationPlan
    onboarding_summary: OnboardingSummary
    source_context: dict[str, Any]


class DemoWorkflowOutput(BaseOutput):
    """Output contract for the demo-workflow skill."""
    status: Status = Status.OK
    orchestration_plan: OrchestrationPlan
    workflow_summary: dict[str, Any]
    source_context: dict[str, Any]
