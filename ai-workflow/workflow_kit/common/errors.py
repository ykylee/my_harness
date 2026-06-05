"""Shared error payload helpers for workflow kit scripts."""

from __future__ import annotations

from typing import Any


def build_error_result(
    *,
    tool_version: str,
    error: str,
    error_code: str,
    warnings: list[str],
    source_context: dict[str, Any],
    recovery_hint: str | None = None,
) -> dict[str, Any]:
    result = {
        "status": "error",
        "tool_version": tool_version,
        "error": error,
        "error_code": error_code,
        "warnings": warnings,
        "source_context": source_context,
    }
    if recovery_hint:
        result["recovery_hint"] = recovery_hint
    return result
