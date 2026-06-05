"""Helpers for keeping workflow metadata out of normal project exploration."""

from __future__ import annotations

from pathlib import Path


def is_workflow_meta_path(raw_path: str | Path | None) -> bool:
    if raw_path is None:
        return False
    normalized = str(raw_path).replace("\\", "/")
    return "/ai-workflow/" in normalized or normalized.startswith("ai-workflow/")


def filter_project_scope_paths(paths: list[str]) -> list[str]:
    return [path for path in paths if not is_workflow_meta_path(path)]
