#!/usr/bin/env python3
"""Run the end-to-end workflow demo against the example project."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOT = REPO_ROOT / "workflow-source"

if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit import __version__ as TOOL_VERSION
from workflow_kit.common.runner import (
    build_orchestration_plan,
    build_execution_trace_step,
    build_runner_success_result,
    build_top_level_step_error_result,
    build_worker_assignment,
    collect_step_warnings,
    current_python_executable,
    optional_path_flag,
    repeated_flag_args,
    run_latest_backlog_step,
    run_json_command,
    WorkflowStepError,
)


def repo_path(*parts: str) -> Path:
    return SOURCE_ROOT.joinpath(*parts).resolve()


EXAMPLE_PRESETS = {
    "acme_delivery_platform": {
        "project_profile_path": repo_path("examples", "acme_delivery_platform", "PROJECT_PROFILE.md"),
        "session_handoff_path": repo_path("examples", "acme_delivery_platform", "session_handoff.md"),
        "work_backlog_index_path": repo_path("examples", "acme_delivery_platform", "work_backlog.md"),
        "backlog_dir_path": repo_path("examples", "acme_delivery_platform", "backlog"),
        "task_id": "TASK-021",
        "task_name": "배송 상태 동기화 실패 대응 절차 문서 정리",
        "task_brief": "runbook 및 handoff 반영 상태를 점검했다.",
        "task_status": "in_progress",
        "changed_files": [
            "app/jobs/delivery_sync.py",
            "docs/operations/runbooks/delivery-sync.md",
        ],
        "merge_result_summary": "runbook 링크와 상태 문서가 함께 수정된 브랜치 병합 후 재정리",
    },
    "research_eval_hub": {
        "project_profile_path": repo_path("examples", "research_eval_hub", "PROJECT_PROFILE.md"),
        "session_handoff_path": repo_path("examples", "research_eval_hub", "session_handoff.md"),
        "work_backlog_index_path": repo_path("examples", "research_eval_hub", "work_backlog.md"),
        "backlog_dir_path": repo_path("examples", "research_eval_hub", "backlog"),
        "task_id": "TASK-044",
        "task_name": "평가 리포트 패키지와 실험 메타데이터 정합성 점검",
        "task_brief": "release report 와 manifest 기준선을 재확인했다.",
        "task_status": "in_progress",
        "changed_files": [
            "evals/pipelines/report_builder.py",
            "docs/evals/reports/release-report-v2.md",
        ],
        "merge_result_summary": "평가 리포트와 실험 메타데이터 문서가 함께 갱신된 브랜치 병합 후 재정리",
    },
}
def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Run the workflow kit end-to-end demo.")
    parser.add_argument(
        "--example-project",
        choices=sorted(EXAMPLE_PRESETS),
        default="acme_delivery_platform",
        help="Use one of the bundled example projects as the default input set.",
    )
    parser.add_argument(
        "--project-profile-path",
        default=None,
    )
    parser.add_argument(
        "--session-handoff-path",
        default=None,
    )
    parser.add_argument(
        "--work-backlog-index-path",
        default=None,
    )
    parser.add_argument(
        "--backlog-dir-path",
        default=None,
    )
    parser.add_argument("--latest-backlog-path")
    parser.add_argument(
        "--task-id",
        default=None,
    )
    parser.add_argument(
        "--task-name",
        default=None,
    )
    parser.add_argument(
        "--task-brief",
        default=None,
    )
    parser.add_argument(
        "--task-status",
        default=None,
    )
    parser.add_argument(
        "--changed-file",
        action="append",
        dest="changed_files",
        default=None,
    )
    parser.add_argument(
        "--merge-result-summary",
        default=None,
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Run write-capable skills with --apply and --scaffold flags.",
    )
    args = parser.parse_args()
    preset = EXAMPLE_PRESETS[args.example_project]
    args.project_profile_path = args.project_profile_path or str(preset["project_profile_path"])
    args.session_handoff_path = args.session_handoff_path or str(preset["session_handoff_path"])
    args.work_backlog_index_path = args.work_backlog_index_path or str(preset["work_backlog_index_path"])
    args.backlog_dir_path = args.backlog_dir_path or str(preset["backlog_dir_path"])
    args.task_id = args.task_id or str(preset["task_id"])
    args.task_name = args.task_name or str(preset["task_name"])
    args.task_brief = args.task_brief or str(preset["task_brief"])
    args.task_status = args.task_status or str(preset["task_status"])
    args.changed_files = list(args.changed_files or preset["changed_files"])
    args.merge_result_summary = args.merge_result_summary or str(preset["merge_result_summary"])
    return args


def main() -> int:
    args = parse_args()
    python = current_python_executable()
    source_context = {
        "example_project": args.example_project,
        "project_profile_path": str(Path(args.project_profile_path).resolve()),
        "session_handoff_path": str(Path(args.session_handoff_path).resolve()),
        "work_backlog_index_path": str(Path(args.work_backlog_index_path).resolve()),
        "backlog_dir_path": str(Path(args.backlog_dir_path).resolve()),
        "latest_backlog_path": str(Path(args.latest_backlog_path).resolve()) if args.latest_backlog_path else None,
        "task_id": args.task_id,
        "task_name": args.task_name,
        "task_brief": args.task_brief,
        "task_status": args.task_status,
        "changed_files": args.changed_files,
        "merge_result_summary": args.merge_result_summary,
    }

    apply_flag = ["--apply"] if args.apply else []
    scaffold_flag = ["--scaffold"] if args.apply else []

    try:
        latest_backlog_data, latest_backlog_path = run_latest_backlog_step(
            python=python,
            repo_root=REPO_ROOT,
            latest_backlog_script=repo_path("mcp_servers", "latest-backlog", "scripts", "run_latest_backlog.py"),
            work_backlog_index_path=args.work_backlog_index_path,
            backlog_dir_path=args.backlog_dir_path,
            direct_latest_backlog_path=args.latest_backlog_path,
            tool_version=TOOL_VERSION,
        )

        session_start = run_json_command(
            [
                python,
                str(repo_path("skills", "session-start", "scripts", "run_session_start.py")),
                "--session-handoff-path",
                args.session_handoff_path,
                "--work-backlog-index-path",
                args.work_backlog_index_path,
                "--project-profile-path",
                args.project_profile_path,
                *optional_path_flag("--latest-backlog-path", latest_backlog_path),
            ],
            REPO_ROOT,
            step_name="session_start",
        )

        backlog_update = run_json_command(
            [
                python,
                str(repo_path("skills", "backlog-update", "scripts", "run_backlog_update.py")),
                "--project-profile-path",
                args.project_profile_path,
                "--daily-backlog-path",
                latest_backlog_path or "",
                "--mode",
                "update",
                "--task-id",
                args.task_id,
                "--task-name",
                args.task_name,
                "--task-brief",
                args.task_brief,
                "--status",
                args.task_status,
            ],
            REPO_ROOT,
            step_name="backlog_update",
        )

        doc_sync = run_json_command(
            [
                python,
                str(repo_path("skills", "doc-sync", "scripts", "run_doc_sync.py")),
                "--project-profile-path",
                args.project_profile_path,
                "--session-handoff-path",
                args.session_handoff_path,
                "--work-backlog-index-path",
                args.work_backlog_index_path,
                *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                *repeated_flag_args("--changed-file", args.changed_files),
                "--change-summary",
                " / ".join(args.changed_files),
                *apply_flag,
            ],
            REPO_ROOT,
            step_name="doc_sync",
        )

        validation_plan = run_json_command(
            [
                python,
                str(repo_path("skills", "validation-plan", "scripts", "run_validation_plan.py")),
                "--project-profile-path",
                args.project_profile_path,
                "--session-handoff-path",
                args.session_handoff_path,
                *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                *repeated_flag_args("--changed-file", args.changed_files),
                "--change-summary",
                " / ".join(args.changed_files),
                *scaffold_flag,
            ],
            REPO_ROOT,
            step_name="validation_plan",
        )

        code_index_update = run_json_command(
            [
                python,
                str(repo_path("skills", "code-index-update", "scripts", "run_code_index_update.py")),
                "--project-profile-path",
                args.project_profile_path,
                "--work-backlog-index-path",
                args.work_backlog_index_path,
                "--session-handoff-path",
                args.session_handoff_path,
                *repeated_flag_args("--changed-file", args.changed_files),
                "--change-summary",
                " / ".join(args.changed_files),
                *apply_flag,
            ],
            REPO_ROOT,
            step_name="code_index_update",
        )

        suggest_impacted_docs = run_json_command(
            [
                python,
                str(repo_path("mcp_servers", "suggest-impacted-docs", "scripts", "run_suggest_impacted_docs.py")),
                *repeated_flag_args("--changed-file", args.changed_files),
                "--session-handoff-path",
                args.session_handoff_path,
                *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                "--work-backlog-index-path",
                args.work_backlog_index_path,
            ],
            REPO_ROOT,
            step_name="suggest_impacted_docs",
        )

        merge_doc_reconcile = run_json_command(
            [
                python,
                str(repo_path("skills", "merge-doc-reconcile", "scripts", "run_merge_doc_reconcile.py")),
                "--project-profile-path",
                args.project_profile_path,
                "--session-handoff-path",
                args.session_handoff_path,
                "--work-backlog-index-path",
                args.work_backlog_index_path,
                *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                "--merge-result-summary",
                args.merge_result_summary,
                *repeated_flag_args("--changed-file", args.changed_files),
                *apply_flag,
            ],
            REPO_ROOT,
            step_name="merge_doc_reconcile",
        )
    except WorkflowStepError as exc:
        result = build_top_level_step_error_result(
            tool_version=TOOL_VERSION,
            step_error=exc,
            source_context=source_context,
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1
    except Exception as exc:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error=str(exc),
            error_code="demo_workflow_runtime_error",
            warnings=[],
            source_context=source_context,
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1

    warnings = collect_step_warnings(
        latest_backlog_data,
        session_start,
        backlog_update,
        doc_sync,
        validation_plan,
        code_index_update,
        suggest_impacted_docs,
        merge_doc_reconcile,
    )

    orchestration_plan = build_orchestration_plan(
        model_split={
            "orchestrator": "main",
            "doc_worker": "small",
            "code_worker": "small",
            "validation_worker": "small",
        },
        worker_assignments=[
            build_worker_assignment(
                worker="doc-worker",
                model="small",
                responsibilities=[
                    "session_handoff 와 latest backlog 비교",
                    "영향 문서와 허브 문서 후보 요약",
                    "문서 초안 또는 상태 불일치 메모 정리",
                ],
                backing_steps=["session_start", "doc_sync", "merge_doc_reconcile"],
            ),
            build_worker_assignment(
                worker="code-worker",
                model="small",
                responsibilities=[
                    "범위가 명확한 코드/설정 수정",
                    "관련 파일 최소 범위 확인",
                    "수정 리스크와 follow-up 요약",
                ],
                backing_steps=["backlog_update"],
            ),
            build_worker_assignment(
                worker="validation-worker",
                model="small",
                responsibilities=[
                    "검증 명령 실행 또는 실행 후보 점검",
                    "로그와 증빙 수집",
                    "미실행 사유와 기록 위치 정리",
                ],
                backing_steps=["validation_plan", "code_index_update"],
            ),
        ],
        integration_notes=[
            "메인 오케스트레이터는 worker 결과를 통합해 backlog/handoff 반영 방향을 최종 결정한다.",
            "복잡한 설계 변경이나 높은 회귀 위험이 있으면 특정 worker 만 일시적으로 main 모델로 승격할 수 있다.",
        ],
    )

    result = build_runner_success_result(
        tool_version=TOOL_VERSION,
        warnings=warnings,
        orchestration_plan=orchestration_plan,
        source_context=source_context,
        runner_inputs={
            "example_project": args.example_project,
            "task": {
                "task_id": args.task_id,
                "task_name": args.task_name,
                "task_brief": args.task_brief,
                "task_status": args.task_status,
            },
            "change_set": {
                "changed_files": args.changed_files,
                "merge_result_summary": args.merge_result_summary,
            },
        },
        execution_trace=[
            build_execution_trace_step(
                step="latest_backlog",
                status=latest_backlog_data.get("status", "ok"),
                command=None if args.latest_backlog_path else [
                    python,
                    str(repo_path("mcp_servers", "latest-backlog", "scripts", "run_latest_backlog.py")),
                    "--work-backlog-index-path",
                    args.work_backlog_index_path,
                    "--backlog-dir-path",
                    args.backlog_dir_path,
                ],
                used_inputs={
                    "work_backlog_index_path": args.work_backlog_index_path,
                    "backlog_dir_path": args.backlog_dir_path,
                    "direct_latest_backlog_path": args.latest_backlog_path,
                },
                produced_keys=["latest_backlog_path", "candidates", "warnings"],
            ),
            build_execution_trace_step(
                step="session_start",
                status=session_start.get("status", "ok"),
                command=[
                    python,
                    str(repo_path("skills", "session-start", "scripts", "run_session_start.py")),
                    "--session-handoff-path",
                    args.session_handoff_path,
                    "--work-backlog-index-path",
                    args.work_backlog_index_path,
                    "--project-profile-path",
                    args.project_profile_path,
                    *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                ],
                used_inputs={
                    "session_handoff_path": args.session_handoff_path,
                    "work_backlog_index_path": args.work_backlog_index_path,
                    "project_profile_path": args.project_profile_path,
                    "latest_backlog_path": str(latest_backlog_path) if latest_backlog_path else None,
                },
                produced_keys=["summary", "recommended_next_action", "next_documents", "warnings"],
            ),
            build_execution_trace_step(
                step="backlog_update",
                status=backlog_update.get("status", "ok"),
                command=[
                    python,
                    str(repo_path("skills", "backlog-update", "scripts", "run_backlog_update.py")),
                    "--project-profile-path",
                    args.project_profile_path,
                    "--daily-backlog-path",
                    latest_backlog_path or "",
                    "--mode",
                    "update",
                    "--task-id",
                    args.task_id,
                    "--task-name",
                    args.task_name,
                    "--task-brief",
                    args.task_brief,
                    "--status",
                    args.task_status,
                ],
                used_inputs={
                    "project_profile_path": args.project_profile_path,
                    "daily_backlog_path": latest_backlog_path or "",
                    "task_id": args.task_id,
                    "task_name": args.task_name,
                    "task_brief": args.task_brief,
                    "task_status": args.task_status,
                },
                produced_keys=["operation_type", "target_backlog_path", "task_found", "warnings"],
            ),
            build_execution_trace_step(
                step="doc_sync",
                status=doc_sync.get("status", "ok"),
                command=[
                    python,
                    str(repo_path("skills", "doc-sync", "scripts", "run_doc_sync.py")),
                    "--project-profile-path",
                    args.project_profile_path,
                    "--session-handoff-path",
                    args.session_handoff_path,
                    "--work-backlog-index-path",
                    args.work_backlog_index_path,
                    *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                    *repeated_flag_args("--changed-file", args.changed_files),
                    "--change-summary",
                    " / ".join(args.changed_files),
                    *apply_flag,
                ],
                used_inputs={
                    "project_profile_path": args.project_profile_path,
                    "session_handoff_path": args.session_handoff_path,
                    "work_backlog_index_path": args.work_backlog_index_path,
                    "latest_backlog_path": str(latest_backlog_path) if latest_backlog_path else None,
                    "changed_files": args.changed_files,
                },
                produced_keys=["impacted_documents", "hub_update_candidates", "recommended_review_order", "warnings"],
            ),
            build_execution_trace_step(
                step="validation_plan",
                status=validation_plan.get("status", "ok"),
                command=[
                    python,
                    str(repo_path("skills", "validation-plan", "scripts", "run_validation_plan.py")),
                    "--project-profile-path",
                    args.project_profile_path,
                    "--session-handoff-path",
                    args.session_handoff_path,
                    *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                    *repeated_flag_args("--changed-file", args.changed_files),
                    "--change-summary",
                    " / ".join(args.changed_files),
                    *scaffold_flag,
                ],
                used_inputs={
                    "project_profile_path": args.project_profile_path,
                    "session_handoff_path": args.session_handoff_path,
                    "latest_backlog_path": str(latest_backlog_path) if latest_backlog_path else None,
                    "changed_files": args.changed_files,
                },
                produced_keys=["recommended_validation_levels", "recommended_commands", "documentation_checks", "warnings"],
            ),
            build_execution_trace_step(
                step="code_index_update",
                status=code_index_update.get("status", "ok"),
                command=[
                    python,
                    str(repo_path("skills", "code-index-update", "scripts", "run_code_index_update.py")),
                    "--project-profile-path",
                    args.project_profile_path,
                    "--work-backlog-index-path",
                    args.work_backlog_index_path,
                    "--session-handoff-path",
                    args.session_handoff_path,
                    *repeated_flag_args("--changed-file", args.changed_files),
                    "--change-summary",
                    " / ".join(args.changed_files),
                    *apply_flag,
                ],
                used_inputs={
                    "project_profile_path": args.project_profile_path,
                    "work_backlog_index_path": args.work_backlog_index_path,
                    "session_handoff_path": args.session_handoff_path,
                    "changed_files": args.changed_files,
                },
                produced_keys=["index_update_candidates", "priority_index_candidates", "warnings"],
            ),
            build_execution_trace_step(
                step="suggest_impacted_docs",
                status=suggest_impacted_docs.get("status", "ok"),
                command=[
                    python,
                    str(repo_path("mcp_servers", "suggest-impacted-docs", "scripts", "run_suggest_impacted_docs.py")),
                    *repeated_flag_args("--changed-file", args.changed_files),
                    "--session-handoff-path",
                    args.session_handoff_path,
                    *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                    "--work-backlog-index-path",
                    args.work_backlog_index_path,
                ],
                used_inputs={
                    "changed_files": args.changed_files,
                    "session_handoff_path": args.session_handoff_path,
                    "latest_backlog_path": str(latest_backlog_path) if latest_backlog_path else None,
                    "work_backlog_index_path": args.work_backlog_index_path,
                },
                produced_keys=["impacted_documents", "warnings"],
            ),
            build_execution_trace_step(
                step="merge_doc_reconcile",
                status=merge_doc_reconcile.get("status", "ok"),
                command=[
                    python,
                    str(repo_path("skills", "merge-doc-reconcile", "scripts", "run_merge_doc_reconcile.py")),
                    "--project-profile-path",
                    args.project_profile_path,
                    "--session-handoff-path",
                    args.session_handoff_path,
                    "--work-backlog-index-path",
                    args.work_backlog_index_path,
                    *optional_path_flag("--latest-backlog-path", latest_backlog_path),
                    "--merge-result-summary",
                    args.merge_result_summary,
                    *repeated_flag_args("--changed-file", args.changed_files),
                    *apply_flag,
                ],
                used_inputs={
                    "project_profile_path": args.project_profile_path,
                    "session_handoff_path": args.session_handoff_path,
                    "work_backlog_index_path": args.work_backlog_index_path,
                    "latest_backlog_path": str(latest_backlog_path) if latest_backlog_path else None,
                    "merge_result_summary": args.merge_result_summary,
                    "changed_files": args.changed_files,
                },
                produced_keys=["reconcile_targets", "state_conflicts", "reconfirmation_points", "warnings"],
            ),
        ],
        extra_fields={
            "example_project": args.example_project,
            "project_profile_path": str(Path(args.project_profile_path).resolve()),
            "latest_backlog": latest_backlog_data,
            "session_start": session_start,
            "backlog_update": backlog_update,
            "doc_sync": doc_sync,
            "validation_plan": validation_plan,
            "code_index_update": code_index_update,
            "suggest_impacted_docs": suggest_impacted_docs,
            "merge_doc_reconcile": merge_doc_reconcile,
            "workflow_summary": {
                "current_baseline": session_start.get("summary", []),
                "target_backlog_path": backlog_update.get("target_backlog_path"),
                "primary_impacted_documents": doc_sync.get("impacted_documents", []),
                "recommended_validation_levels": validation_plan.get("recommended_validation_levels", []),
                "priority_index_candidates": code_index_update.get("priority_index_candidates", []),
                "reconcile_targets": merge_doc_reconcile.get("reconcile_targets", []),
            },
        },
    )
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
