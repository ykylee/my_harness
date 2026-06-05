#!/usr/bin/env python3
"""Smoke test the validation-plan prototype."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

SOURCE_ROOT_FOR_IMPORT = Path(__file__).resolve().parents[2] / "workflow-source"
if str(SOURCE_ROOT_FOR_IMPORT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT_FOR_IMPORT))

from workflow_kit.common.output_contracts import validate_output_payload


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
SCRIPT_PATH = SOURCE_ROOT / "skills" / "validation-plan" / "scripts" / "run_validation_plan.py"


def run_validation_with_args(*, expect_success: bool, args: list[str]) -> tuple[int, dict[str, object]]:
    completed = subprocess.run(
        [sys.executable, str(SCRIPT_PATH), *args],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if expect_success and completed.returncode != 0:
        raise AssertionError(f"Expected validation-plan success but got {completed.returncode}: {completed.stderr}")
    if not expect_success and completed.returncode == 0:
        raise AssertionError("Expected validation-plan failure path but command succeeded.")
    return completed.returncode, json.loads(completed.stdout)


def run_validation(example_name: str, changed_files: list[str], change_summary: str) -> dict[str, object]:
    example_root = SOURCE_ROOT / "examples" / example_name
    latest_backlog = sorted((example_root / "backlog").glob("*.md"))[-1]
    _, payload = run_validation_with_args(
        expect_success=True,
        args=[
            "--project-profile-path",
            str(example_root / "PROJECT_PROFILE.md"),
            "--session-handoff-path",
            str(example_root / "session_handoff.md"),
            "--latest-backlog-path",
            str(latest_backlog),
            *sum([["--changed-file", item] for item in changed_files], []),
            "--change-summary",
            change_summary,
        ],
    )
    return payload


def main() -> int:
    acme_payload = run_validation(
        "acme_delivery_platform",
        [
            "app/jobs/delivery_sync.py",
            "docs/operations/runbooks/delivery-sync.md",
        ],
        "delivery sync 재시도 로직과 운영 runbook 동시 수정",
    )
    output_errors = validate_output_payload(acme_payload, family="validation_plan")
    if output_errors:
        raise AssertionError(f"Acme validation-plan payload violated output contract: {output_errors}")
    detected = set(acme_payload["detected_change_types"])
    if not {"code", "docs", "ops"}.issubset(detected):
        raise AssertionError("Acme example should classify code/docs/ops changes.")
    levels = set(acme_payload["recommended_validation_levels"])
    if not {"standard", "release_sensitive"}.issubset(levels):
        raise AssertionError("Acme example should recommend standard and release-sensitive validation.")
    if not acme_payload["recommended_commands"]:
        raise AssertionError("Acme example should recommend at least one validation command.")

    research_payload = run_validation(
        "research_eval_hub",
        [
            "evals/pipelines/report_builder.py",
            "docs/evals/reports/release-report-v2.md",
        ],
        "evaluation report builder 로직과 release report 동시 수정",
    )
    output_errors = validate_output_payload(research_payload, family="validation_plan")
    if output_errors:
        raise AssertionError(f"Research validation-plan payload violated output contract: {output_errors}")
    research_detected = set(research_payload["detected_change_types"])
    if not {"code", "docs", "prompt_or_eval"}.issubset(research_detected):
        raise AssertionError("Research example should classify code/docs/prompt_or_eval changes.")
    if "artifact_sensitive" not in research_payload["recommended_validation_levels"]:
        raise AssertionError("Research example should recommend artifact-sensitive validation.")
    if not research_payload["documentation_checks"]:
        raise AssertionError("Research example should include documentation checks.")

    docs_only_payload = run_validation(
        "acme_delivery_platform",
        [
            "docs/operations/runbooks/delivery-sync.md",
        ],
        "운영 runbook 문서만 수정",
    )
    output_errors = validate_output_payload(docs_only_payload, family="validation_plan")
    if output_errors:
        raise AssertionError(f"Docs-only validation-plan payload violated output contract: {output_errors}")
    if docs_only_payload["detected_change_types"] != ["docs", "ops"]:
        raise AssertionError("Docs-only runbook example should classify docs and ops changes.")
    if not docs_only_payload["recommended_commands"]:
        raise AssertionError("Docs-only example should still recommend quick validation commands when available.")
    if any(
        warning == "프로젝트 프로파일에서 실행 가능한 검증 명령을 충분히 찾지 못했다."
        for warning in docs_only_payload["warnings"]
    ):
        raise AssertionError("Docs-only example should not emit missing-command warning when quick tests exist.")

    workflow_meta_payload = run_validation(
        "acme_delivery_platform",
        [
            "ai-workflow/memory/session_handoff.md",
        ],
        "workflow 상태 문서만 수정",
    )
    if workflow_meta_payload["source_context"]["changed_files"]:
        raise AssertionError("Validation-plan should ignore ai-workflow metadata paths in changed_files.")

    failure_code, failure_payload = run_validation_with_args(
        expect_success=False,
        args=[
            "--project-profile-path",
            str(SOURCE_ROOT / "examples" / "acme_delivery_platform" / "PROJECT_PROFILE.md"),
        ],
    )
    if failure_code == 0:
        raise AssertionError("Expected validation-plan failure for missing change input.")
    output_errors = validate_output_payload(failure_payload, family="validation_plan")
    if output_errors:
        raise AssertionError(f"Validation-plan error payload violated output contract: {output_errors}")
    if failure_payload["error_code"] != "missing_change_input":
        raise AssertionError("Expected missing_change_input error code.")

    print("Validation-plan smoke check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
