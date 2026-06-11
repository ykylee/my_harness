#!/usr/bin/env python3
"""Smoke test the existing-project onboarding runner."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

SOURCE_ROOT_FOR_IMPORT = Path(__file__).resolve().parents[2] / "ai-workflow"
if str(SOURCE_ROOT_FOR_IMPORT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT_FOR_IMPORT))

from workflow_kit.common.output_contracts import validate_output_payload


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE_ROOT = REPO_ROOT / "ai-workflow"
BOOTSTRAP_SCRIPT = SOURCE_ROOT / "scripts" / "bootstrap_workflow_kit.py"
ONBOARDING_SCRIPT = SOURCE_ROOT / "scripts" / "run_existing_project_onboarding.py"


def run_json(cmd: list[str], cwd: Path) -> dict[str, object]:
    completed = subprocess.run(
        [sys.executable, *cmd],
        cwd=cwd,
        capture_output=True,
        text=True,
        check=True,
    )
    return json.loads(completed.stdout)


def run_json_allow_failure(cmd: list[str], cwd: Path) -> tuple[int, dict[str, object]]:
    completed = subprocess.run(
        [sys.executable, *cmd],
        cwd=cwd,
        capture_output=True,
        text=True,
        check=False,
    )
    return completed.returncode, json.loads(completed.stdout)


def main() -> int:
    with tempfile.TemporaryDirectory() as tmpdir:
        target_root = Path(tmpdir) / "existing-repo"
        (target_root / "docs").mkdir(parents=True, exist_ok=True)
        (target_root / "src").mkdir(parents=True, exist_ok=True)
        (target_root / "tests").mkdir(parents=True, exist_ok=True)
        (target_root / "README.md").write_text("# Existing Repo\n", encoding="utf-8")
        (target_root / "docs" / "README.md").write_text("# Docs\n", encoding="utf-8")
        (target_root / "package.json").write_text(
            json.dumps(
                {
                    "name": "existing-repo",
                    "scripts": {
                        "dev": "node server.js",
                        "test": "vitest",
                        "test:unit": "vitest run",
                        "test:smoke": "playwright test",
                    },
                }
            ),
            encoding="utf-8",
        )

        bootstrap_payload = run_json(
            [
                str(BOOTSTRAP_SCRIPT),
                "--target-root",
                str(target_root),
                "--project-slug",
                "existing_repo",
                "--project-name",
                "Existing Repo",
                "--adoption-mode",
                "existing",
                "--copy-core-docs",
            ],
            REPO_ROOT,
        )
        generated = bootstrap_payload["generated_files"]
        backlog_dir = Path(generated["daily_backlog"]).parent

        onboarding_payload = run_json(
            [
                str(ONBOARDING_SCRIPT),
                "--project-profile-path",
                str(generated["project_profile"]),
                "--session-handoff-path",
                str(generated["session_handoff"]),
                "--work-backlog-index-path",
                str(generated["work_backlog"]),
                "--backlog-dir-path",
                str(backlog_dir),
                "--repository-assessment-path",
                str(generated["repository_assessment"]),
                "--change-summary",
                "기존 프로젝트 도입 초안과 추정 명령/문서 구조를 실제 저장소 기준으로 정렬한다.",
            ],
            REPO_ROOT,
        )

        output_errors = validate_output_payload(onboarding_payload, family="existing_project_onboarding")
        if output_errors:
            raise AssertionError(f"Existing-project onboarding payload violated output contract: {output_errors}")
        if onboarding_payload["onboarding_mode"] != "existing_project_followup":
            raise AssertionError("Unexpected onboarding mode.")
        if onboarding_payload["repository_assessment"]["summary"]["primary_stack"] != "node":
            raise AssertionError("Expected node primary stack from repository assessment.")
        if onboarding_payload["session_start"]["summary"] is None:
            raise AssertionError("Expected session_start summary field in onboarding output.")
        if not onboarding_payload["validation_plan"]["recommended_validation_levels"]:
            raise AssertionError("Expected validation plan levels in onboarding output.")
        if not onboarding_payload["code_index_update"]["index_update_candidates"]:
            raise AssertionError("Expected code-index-update candidates in onboarding output.")
        if len(onboarding_payload["onboarding_summary"]["recommended_next_steps"]) < 3:
            raise AssertionError("Expected multiple onboarding next steps.")
        if onboarding_payload["orchestration_plan"]["model_split"]["orchestrator"] != "main":
            raise AssertionError("Expected main orchestrator model split in onboarding output.")
        if onboarding_payload["runner_inputs"]["onboarding_mode"] != "existing_project_followup":
            raise AssertionError("Expected runner_inputs to preserve onboarding mode.")
        if len(onboarding_payload["execution_trace"]) != 4:
            raise AssertionError("Expected execution_trace to record all onboarding steps.")
        if onboarding_payload["execution_trace"][1]["step"] != "session_start":
            raise AssertionError("Expected session_start to be the second onboarding trace step.")
        if "index_update_candidates" not in onboarding_payload["execution_trace"][3]["produced_keys"]:
            raise AssertionError("Expected code_index_update trace to expose produced keys.")
        if not onboarding_payload["source_context"]["project_profile_path"]:
            raise AssertionError("Expected onboarding source_context to include project_profile_path.")
        if onboarding_payload["source_context"]["repository_assessment_path"] != str(
            Path(generated["repository_assessment"]).resolve()
        ):
            raise AssertionError("Expected onboarding source_context to retain repository_assessment_path.")
        if onboarding_payload["source_context"]["latest_backlog_path"] != onboarding_payload["latest_backlog"]["latest_backlog_path"]:
            raise AssertionError("Expected onboarding source_context latest_backlog_path to match latest_backlog output.")

        empty_backlog_root = target_root / "empty-backlog-case"
        empty_backlog_root.mkdir(parents=True, exist_ok=True)
        empty_work_backlog = empty_backlog_root / "work_backlog.md"
        empty_work_backlog.write_text(
            "# 작업 백로그 인덱스\n\n- 문서 목적: 테스트\n- 범위: 테스트\n- 대상 독자: 테스트\n- 상태: draft\n- 최종 수정일: 2026-04-22\n- 관련 문서:\n\n## 운영 원칙\n\n- 테스트\n\n## 날짜별 백로그 문서\n",
            encoding="utf-8",
        )
        empty_backlog_dir = empty_backlog_root / "backlog"
        empty_backlog_dir.mkdir(parents=True, exist_ok=True)

        no_backlog_payload = run_json(
            [
                str(ONBOARDING_SCRIPT),
                "--project-profile-path",
                str(generated["project_profile"]),
                "--session-handoff-path",
                str(generated["session_handoff"]),
                "--work-backlog-index-path",
                str(empty_work_backlog),
                "--backlog-dir-path",
                str(empty_backlog_dir),
                "--change-summary",
                "latest backlog 없이도 onboarding 흐름이 계속되는지 확인한다.",
            ],
            REPO_ROOT,
        )
        if no_backlog_payload["status"] != "ok":
            raise AssertionError("Expected onboarding runner to continue without a latest backlog.")
        output_errors = validate_output_payload(no_backlog_payload, family="existing_project_onboarding")
        if output_errors:
            raise AssertionError(f"No-backlog onboarding payload violated output contract: {output_errors}")
        if no_backlog_payload["latest_backlog"]["latest_backlog_path"] is not None:
            raise AssertionError("Expected latest_backlog_path to remain null when no backlog exists.")
        if not no_backlog_payload["latest_backlog"]["warnings"]:
            raise AssertionError("Expected latest_backlog warnings when no backlog exists.")
        if no_backlog_payload["source_context"]["latest_backlog_path"] is not None:
            raise AssertionError("Expected onboarding source_context latest_backlog_path to remain null.")

        failure_code, failure_payload = run_json_allow_failure(
            [
                str(ONBOARDING_SCRIPT),
                "--project-profile-path",
                "/tmp/missing-profile.md",
                "--session-handoff-path",
                str(generated["session_handoff"]),
                "--work-backlog-index-path",
                str(generated["work_backlog"]),
                "--backlog-dir-path",
                str(backlog_dir),
            ],
            REPO_ROOT,
        )
        if failure_code == 0:
            raise AssertionError("Expected onboarding runner failure for missing profile path.")
        if failure_payload["status"] != "error":
            raise AssertionError("Expected structured error payload for onboarding failure.")
        if failure_payload["error_code"] != "missing_required_document":
            raise AssertionError("Expected missing_required_document error code.")
        expected_missing_profile = str(Path("/tmp/missing-profile.md").resolve())
        if failure_payload["source_context"]["project_profile_path"] != expected_missing_profile:
            raise AssertionError("Expected source_context to retain the missing project profile path.")

    print("Existing-project onboarding smoke check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
