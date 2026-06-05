"""Document metadata helpers for workflow kit scripts."""

from __future__ import annotations

from pathlib import Path


REQUIRED_METADATA_FIELDS = [
    "문서 목적",
    "범위",
    "대상 독자",
    "상태",
    "최종 수정일",
    "관련 문서",
]

ENGLISH_METADATA_FIELDS = [
    "Purpose",
    "Scope",
    "Audience",
    "Status",
    "Updated",
    "Related docs",
]


def missing_metadata_fields(path: Path, required_fields: list[str] | None = None) -> list[str]:
    fields = required_fields or REQUIRED_METADATA_FIELDS
    lines = path.read_text(encoding="utf-8").splitlines()[:20]
    if required_fields is None and _has_metadata_set(lines, ENGLISH_METADATA_FIELDS):
        fields = ENGLISH_METADATA_FIELDS
    missing: list[str] = []
    for field in fields:
        prefix = f"- {field}:"
        if not any(line.startswith(prefix) for line in lines):
            missing.append(field)
    if not lines or not lines[0].startswith("# "):
        missing.append("제목 헤더")
    return missing


def _has_metadata_set(lines: list[str], fields: list[str]) -> bool:
    return all(any(line.startswith(f"- {field}:") for line in lines) for field in fields)
