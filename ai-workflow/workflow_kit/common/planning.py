"""Planning helpers shared across workflow kit skills."""

from __future__ import annotations

from workflow_kit.common.normalize import dedupe_strings


def collect_validation_levels(change_types: list[str]) -> list[str]:
    levels: list[str] = []
    if change_types == ["docs"]:
        levels.append("documentation")
    if any(item in change_types for item in ["code", "config"]):
        levels.append("standard")
    if "ui" in change_types:
        levels.append("ui_extended")
    if "ops" in change_types:
        levels.append("release_sensitive")
    if "prompt_or_eval" in change_types:
        levels.append("artifact_sensitive")
    if not levels:
        levels.append("light_review")
    return dedupe_strings(levels)


def determine_conservative_task_status(
    requested_status: str | None,
    validation_result: str | None,
    operation_type: str,
) -> tuple[str, list[str]]:
    warnings: list[str] = []
    status = requested_status or ("planned" if operation_type == "create_entry" else "in_progress")
    if status not in {"planned", "in_progress", "blocked", "done"}:
        warnings.append(f"알 수 없는 상태 `{status}` 는 사용할 수 없어 `planned` 로 대체한다.")
        status = "planned"
    if status == "done" and not validation_result:
        warnings.append("검증 결과가 없으므로 `done` 상태는 초안에서 `in_progress` 로 낮춘다.")
        status = "in_progress"
    return status, warnings

