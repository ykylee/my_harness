"""Shared state comparison helpers for workflow kit skills."""

from __future__ import annotations

from workflow_kit.common.normalize import dedupe_normalized_backticked


def compare_state_lists(handoff_items: list[str], backlog_items: list[str], label: str) -> list[str]:
    warnings: list[str] = []
    handoff_set = set(dedupe_normalized_backticked(handoff_items))
    backlog_set = set(dedupe_normalized_backticked(backlog_items))
    if handoff_set != backlog_set:
        warnings.append(
            f"{label} 항목이 handoff 와 backlog 사이에서 다를 수 있으므로 수동 재확인이 필요하다."
        )
    return warnings


def explain_state_conflicts(handoff_items: list[str], backlog_items: list[str], label: str) -> list[str]:
    handoff_set = set(dedupe_normalized_backticked(handoff_items))
    backlog_set = set(dedupe_normalized_backticked(backlog_items))
    conflicts: list[str] = []
    if handoff_set != backlog_set:
        handoff_view = ", ".join(sorted(handoff_set)) or "없음"
        backlog_view = ", ".join(sorted(backlog_set)) or "없음"
        conflicts.append(f"{label} 항목이 다르다. handoff: {handoff_view} / backlog: {backlog_view}")
    return conflicts

