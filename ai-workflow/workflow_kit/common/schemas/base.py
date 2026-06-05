"""Strict Pydantic models for standard AI workflow output contracts."""

from __future__ import annotations

from enum import Enum
from typing import Any, Generic, TypeVar

from pydantic import BaseModel, Field


class Status(str, Enum):
    OK = "ok"
    WARNING = "warning"
    ERROR = "error"


class BaseOutput(BaseModel):
    """Base fields present in all workflow tool outputs."""
    status: Status = Field(..., description="Execution status: ok, warning, or error")
    tool_version: str = Field(..., description="Version of the tool that generated this output")
    warnings: list[str] = Field(default_factory=list, description="List of non-fatal warnings encountered")


class ErrorOutput(BaseOutput):
    """Standard payload for failed tool executions."""
    status: Status = Status.ERROR
    error: str = Field(..., description="Human-readable error message")
    error_code: str = Field(..., description="Stable machine-readable error code")
    source_context: dict[str, Any] = Field(
        ...,
        description="Arbitrary context data to help diagnose the error",
    )


T = TypeVar("T")


class SuccessOutput(BaseOutput, Generic[T]):
    """Generic wrapper for successful tool outputs with specific data payloads."""
    # Note: Specific tools will subclass this or add their own fields.
    pass
