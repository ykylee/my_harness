"""Pydantic models for multi-agent delegation and worker communication."""

from __future__ import annotations

from typing import Any
from pydantic import BaseModel, Field
from workflow_kit.common.schemas.base import BaseOutput, Status


class WorkerTask(BaseModel):
    """Payload sent by the orchestrator to a sub-agent (worker)."""
    worker_id: str = Field(..., description="ID of the target worker (e.g., 'doc-worker')")
    task_description: str = Field(..., description="High-level description of what the worker should do")
    input_files: list[str] = Field(default_factory=list, description="List of files the worker should read")
    output_files: list[str] = Field(default_factory=list, description="List of files the worker is expected to modify or create")
    constraints: list[str] = Field(default_factory=list, description="Specific limitations or rules for the task")
    context_summary: str | None = Field(None, description="Short summary of why this task is being delegated")


class WorkerResponse(BaseOutput):
    """Payload returned by a sub-agent (worker) to the orchestrator."""
    status: Status = Status.OK
    summary: str = Field(..., description="Human-readable summary of the work performed")
    produced_artifacts: list[str] = Field(default_factory=list, description="Paths to files created or modified")
    risks_identified: list[str] = Field(default_factory=list, description="Potential issues or technical debt identified during execution")
    suggested_follow_up: list[str] = Field(default_factory=list, description="Next steps recommended by the worker")
    raw_worker_output: str | None = Field(None, description="Optional raw output or logs from the worker execution")
