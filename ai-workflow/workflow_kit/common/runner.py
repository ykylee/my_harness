"""Shared subprocess runner helpers for workflow kit orchestration scripts."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path
from typing import Any

from workflow_kit.common.errors import build_error_result


class WorkflowStepError(RuntimeError):
    """Raised when an orchestration runner cannot obtain a usable child payload."""

    def __init__(
        self,
        *,
        step_name: str,
        error: str,
        error_code: str,
        command: list[str],
        payload: dict[str, Any] | None = None,
    ) -> None:
        super().__init__(error)
        self.step_name = step_name
        self.error = error
        self.error_code = error_code
        self.command = command
        self.payload = payload


def _trim_text(value: str, *, limit: int = 400) -> str:
    text = value.strip()
    if len(text) <= limit:
        return text
    return f"{text[:limit].rstrip()}..."


def _json_safe_value(value: Any) -> Any:
    if isinstance(value, Path):
        return str(value)
    if isinstance(value, list):
        return [_json_safe_value(item) for item in value]
    if isinstance(value, dict):
        return {key: _json_safe_value(item) for key, item in value.items()}
    return value


def run_json_command(cmd: list[str], cwd: Path, *, step_name: str) -> dict[str, Any]:
    try:
        completed = subprocess.run(
            cmd,
            cwd=cwd,
            capture_output=True,
            text=True,
            check=True,
        )
    except FileNotFoundError as exc:
        raise WorkflowStepError(
            step_name=step_name,
            error=f"{step_name} 실행에 필요한 명령을 찾을 수 없다: {exc.filename}",
            error_code="step_command_missing",
            command=cmd,
        ) from exc
    except subprocess.CalledProcessError as exc:
        if exc.stdout:
            try:
                payload = json.loads(exc.stdout)
            except json.JSONDecodeError:
                payload = None
            if isinstance(payload, dict) and payload.get("status") == "error":
                raise WorkflowStepError(
                    step_name=step_name,
                    error=str(payload.get("error") or f"{step_name} 가 오류 상태를 반환했다."),
                    error_code=str(payload.get("error_code") or "step_reported_error"),
                    command=cmd,
                    payload=payload,
                ) from exc
        stderr = _trim_text(exc.stderr or "")
        stdout = _trim_text(exc.stdout or "")
        detail = stderr or stdout or "하위 step 이 비정상 종료했지만 stderr/stdout 을 확인하지 못했다."
        raise WorkflowStepError(
            step_name=step_name,
            error=f"{step_name} 실행이 실패했다. {detail}",
            error_code="step_process_failed",
            command=cmd,
        ) from exc

    try:
        payload = json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise WorkflowStepError(
            step_name=step_name,
            error=f"{step_name} 출력이 유효한 JSON 이 아니다.",
            error_code="step_invalid_json",
            command=cmd,
        ) from exc

    if payload.get("status") == "error":
        raise WorkflowStepError(
            step_name=step_name,
            error=str(payload.get("error") or f"{step_name} 가 오류 상태를 반환했다."),
            error_code=str(payload.get("error_code") or "step_reported_error"),
            command=cmd,
            payload=payload,
        )

    return payload


def current_python_executable() -> str:
    return sys.executable


def repeated_flag_args(flag: str, values: list[str]) -> list[str]:
    return [item for value in values for item in [flag, value]]


def collect_step_warnings(*payloads: dict[str, Any]) -> list[str]:
    warnings: list[str] = []
    for payload in payloads:
        for warning in payload.get("warnings", []):
            if warning not in warnings:
                warnings.append(warning)
    return warnings


def build_step_error_context(
    *,
    step_error: WorkflowStepError,
    source_context: dict[str, Any],
) -> dict[str, Any]:
    context = dict(source_context)
    context["failed_step"] = step_error.step_name
    context["failed_command"] = step_error.command
    context["upstream_error_code"] = step_error.error_code
    if step_error.payload is not None:
        context["upstream_status"] = step_error.payload.get("status")
    return context


def build_worker_assignment(
    *,
    worker: str,
    model: str,
    responsibilities: list[str],
    backing_steps: list[str],
) -> dict[str, Any]:
    return {
        "worker": worker,
        "model": model,
        "responsibilities": responsibilities,
        "backing_steps": backing_steps,
    }


def build_execution_trace_step(
    *,
    step: str,
    status: str,
    command: list[str] | None = None,
    used_inputs: dict[str, Any] | None = None,
    produced_keys: list[str] | None = None,
) -> dict[str, Any]:
    return {
        "step": step,
        "status": status,
        "command": _json_safe_value(command),
        "used_inputs": _json_safe_value(used_inputs or {}),
        "produced_keys": _json_safe_value(produced_keys or []),
    }


def build_orchestration_plan(
    *,
    model_split: dict[str, str],
    worker_assignments: list[dict[str, Any]],
    integration_notes: list[str],
    recommended_pattern: str = "main_orchestrator_with_specialized_workers",
) -> dict[str, Any]:
    return {
        "recommended_pattern": recommended_pattern,
        "model_split": model_split,
        "worker_assignments": worker_assignments,
        "integration_notes": integration_notes,
    }


def run_latest_backlog_step(
    *,
    python: str,
    repo_root: Path,
    latest_backlog_script: Path,
    work_backlog_index_path: str,
    backlog_dir_path: str,
    direct_latest_backlog_path: str | None,
    tool_version: str,
) -> tuple[dict[str, Any], Path | None]:
    """Resolve latest backlog either from a direct path or by calling the MCP prototype."""

    if direct_latest_backlog_path:
        latest_backlog_path = Path(direct_latest_backlog_path).resolve()
        return (
            {
                "status": "ok",
                "tool_version": tool_version,
                "latest_backlog_path": str(latest_backlog_path),
                "candidates": [str(latest_backlog_path)],
                "warnings": [],
            },
            latest_backlog_path,
        )

    latest_backlog_data = run_json_command(
        [
            python,
            str(latest_backlog_script),
            "--work-backlog-index-path",
            work_backlog_index_path,
            "--backlog-dir-path",
            backlog_dir_path,
        ],
        repo_root,
        step_name="latest_backlog",
    )
    raw_latest_backlog = latest_backlog_data.get("latest_backlog_path")
    latest_backlog_path = Path(str(raw_latest_backlog)).resolve() if raw_latest_backlog else None
    return latest_backlog_data, latest_backlog_path


def optional_path_flag(flag: str, path: Path | None) -> list[str]:
    """Return a CLI flag pair only when the optional path is available."""

    if path is None:
        return []
    return [flag, str(path)]


def build_top_level_step_error_result(
    *,
    tool_version: str,
    step_error: WorkflowStepError,
    source_context: dict[str, Any],
) -> dict[str, Any]:
    """Wrap a child step error payload into the top-level runner error contract."""

    return build_error_result(
        tool_version=tool_version,
        error=step_error.error,
        error_code="workflow_step_failed",
        warnings=list(step_error.payload.get("warnings", [])) if step_error.payload else [],
        source_context=build_step_error_context(step_error=step_error, source_context=source_context),
    )


def build_runner_success_result(
    *,
    tool_version: str,
    warnings: list[str],
    orchestration_plan: dict[str, Any],
    source_context: dict[str, Any],
    extra_fields: dict[str, Any],
    written_paths: list[str] | None = None,
    runner_inputs: dict[str, Any] | None = None,
    execution_trace: list[dict[str, Any]] | None = None,
) -> dict[str, Any]:
    """Build the shared top-level success payload shape for orchestration runners."""

    return {
        "status": "ok",
        "tool_version": tool_version,
        "warnings": warnings,
        "orchestration_plan": orchestration_plan,
        "written_paths": written_paths or [],
        "runner_inputs": runner_inputs or {},
        "execution_trace": execution_trace or [],
        **extra_fields,
        "source_context": source_context,
    }
