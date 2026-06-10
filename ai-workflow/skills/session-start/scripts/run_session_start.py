#!/usr/bin/env python3
"""Prototype runner for the session-start skill."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parents[4]
SOURCE_ROOT = REPO_ROOT / "workflow-source"
if str(SOURCE_ROOT) not in sys.path:
    sys.path.insert(0, str(SOURCE_ROOT))

from workflow_kit import __version__ as TOOL_VERSION
from workflow_kit.common.errors import build_error_result
from workflow_kit.common.normalize import dedupe_normalized_backticked
from workflow_kit.common.paths import resolve_existing_path
from workflow_kit.common.project_docs import (
    find_latest_backlog_path,
    parse_backlog,
    parse_handoff,
    parse_project_profile_session,
)
from workflow_kit.common.reconcile import compare_state_lists
from workflow_kit.common.session_outputs import build_session_summary, make_session_recommended_action

# D-71: LLM Wiki vault 발견 + lint
sys.path.insert(0, str(Path(__file__).resolve().parent))
from wiki_vault_status import (  # noqa: E402
    WikiLintSummary,
    WikiVaultStatus,
    discover_wiki_vault,
    resolve_wiki_vault_path,
)


def main() -> int:
    parser = argparse.ArgumentParser(description="Run the session-start prototype.")
    parser.add_argument("--session-handoff-path", required=True)
    parser.add_argument("--work-backlog-index-path", required=True)
    parser.add_argument("--project-profile-path", required=True)
    parser.add_argument("--latest-backlog-path")
    parser.add_argument(
        "--wiki-vault-path",
        default=None,
        help="LLM Wiki vault 경로 (D-71). 기본: $MYHARNESS_WIKI_VAULT > ~/wiki (존재 시).",
    )
    parser.add_argument(
        "--wiki-lint",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="vault 발견 시 wiki-lint 실행 여부 (기본: true)",
    )
    parser.add_argument(
        "--wiki-lint-timeout",
        type=float,
        default=15.0,
        help="wiki-lint subprocess 타임아웃 (초, 기본: 15)",
    )
    args = parser.parse_args()

    source_context = {
        "session_handoff_path": args.session_handoff_path,
        "work_backlog_index_path": args.work_backlog_index_path,
        "project_profile_path": args.project_profile_path,
        "latest_backlog_path": args.latest_backlog_path,
        "wiki_vault_path": args.wiki_vault_path,
    }

    try:
        session_handoff_path = resolve_existing_path(args.session_handoff_path)
        work_backlog_index_path = resolve_existing_path(args.work_backlog_index_path)
        project_profile_path = resolve_existing_path(args.project_profile_path)
    except FileNotFoundError as exc:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error="필수 입력 문서를 읽을 수 없다.",
            error_code="missing_required_document",
            warnings=["session-start 기준선을 복원할 수 없어 후속 판단을 중단한다."],
            source_context=source_context | {"missing_path_detail": str(exc)},
            recovery_hint="`scripts/bootstrap_workflow_kit.py`를 실행하여 초기 문서를 생성하거나, 인자로 넘어온 경로가 올바른지 확인해야 한다.",
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1

    warnings: list[str] = []
    try:
        handoff = parse_handoff(session_handoff_path)
        warnings.extend(handoff.get("warnings", []))

        profile = parse_project_profile_session(project_profile_path)
        warnings.extend(profile.get("warnings", []))

        latest_backlog_path: Path | None
        if args.latest_backlog_path:
            latest_backlog_path = resolve_existing_path(args.latest_backlog_path)
        else:
            latest_backlog_path = find_latest_backlog_path(work_backlog_index_path)
            if latest_backlog_path is None or not latest_backlog_path.exists():
                latest_backlog_path = None
                warnings.append("최신 backlog 경로를 backlog index 에서 확인하지 못했다.")

        backlog: dict[str, Any] = {
            "tasks": [],
            "in_progress_items": [],
            "blocked_items": [],
            "done_items": [],
            "warnings": [],
        }
        if latest_backlog_path is not None:
            backlog = parse_backlog(latest_backlog_path)
            warnings.extend(backlog.get("warnings", []))

        warnings.extend(
            compare_state_lists(
                handoff.get("in_progress_items", []),
                backlog.get("in_progress_items", []),
                "in_progress",
            )
        )
        warnings.extend(
            compare_state_lists(
                handoff.get("blocked_items", []),
                backlog.get("blocked_items", []),
                "blocked",
            )
        )

        next_documents = dedupe_normalized_backticked(
            [
                str(session_handoff_path),
                str(latest_backlog_path) if latest_backlog_path else "",
                str(project_profile_path),
                *[str(path) for path in handoff.get("next_documents", []) if path.exists()],
            ]
        )

        # === D-71: LLM Wiki vault 발견 + lint ===
        wiki_vault: WikiVaultStatus | None = None
        env_vault = sys.environ.get("MYHARNESS_WIKI_VAULT")
        vault_path = resolve_wiki_vault_path(args.wiki_vault_path, env_value=env_vault)
        if vault_path is not None:
            wiki_vault = discover_wiki_vault(
                vault_path, run_lint=args.wiki_lint, timeout=args.wiki_lint_timeout
            )
            if wiki_vault.exists:
                vault_index = vault_path / "index.md"
                if vault_index.is_file():
                    next_documents = dedupe_normalized_backticked(
                        next_documents + [str(vault_index)]
                    )
                if wiki_vault.warnings:
                    warnings.extend(f"wiki-vault: {w}" for w in wiki_vault.warnings)
                if wiki_vault.lint and wiki_vault.lint.errors > 0:
                    warnings.append(
                        f"wiki-lint: {wiki_vault.lint.errors} error 발견 — "
                        f"보고: {wiki_vault.lint.last_report or '(없음)'}"
                    )

        from workflow_kit.common.schemas import SessionStartOutput

        output_model = SessionStartOutput(
            status="ok",
            tool_version=TOOL_VERSION,
            summary=build_session_summary(handoff, backlog, profile),
            in_progress_items=dedupe_normalized_backticked(
                handoff.get("in_progress_items", []) + backlog.get("in_progress_items", [])
            ),
            blocked_items=dedupe_normalized_backticked(
                handoff.get("blocked_items", []) + backlog.get("blocked_items", [])
            ),
            latest_backlog_path=str(latest_backlog_path) if latest_backlog_path else None,
            next_documents=next_documents,
            recommended_next_action=make_session_recommended_action(warnings, backlog, profile),
            warnings=warnings,
            validation_notes=[],
            environment_constraints=dedupe_normalized_backticked(
                [item for item in [handoff.get("constraints"), profile.get("constraints")] if item]
            ),
            source_documents={
                "session_handoff_path": str(session_handoff_path),
                "work_backlog_index_path": str(work_backlog_index_path),
                "project_profile_path": str(project_profile_path),
            },
            wiki_vault=wiki_vault,
        )
        # WikiVaultStatus 가 plain class 이므로 Pydantic 이 dict 로 받게 변환
        if wiki_vault is not None:
            # Pydantic v2 는 dict 또는 같은 필드를 가진 객체 허용. WikiVaultStatus 가
            # model_dump 가능한 dict-convertible 라서 to_dict() 로 변환.
            output_model = output_model.model_copy(
                update={"wiki_vault": wiki_vault.to_dict()}
            )
        result = output_model.model_dump()
    except FileNotFoundError as exc:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error="참조 문서를 읽는 중 필요한 경로를 확인하지 못했다.",
            error_code="missing_referenced_document",
            warnings=["입력 문서의 링크 또는 명시 경로를 다시 확인해야 한다."],
            source_context=source_context | {"missing_path_detail": str(exc)},
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1
    except Exception as exc:
        result = build_error_result(
            tool_version=TOOL_VERSION,
            error="session-start 실행 중 예기치 않은 오류가 발생했다.",
            error_code="session_start_runtime_error",
            warnings=["파서 또는 입력 문서 형식을 점검한 뒤 다시 실행해야 한다."],
            source_context=source_context | {"exception_type": type(exc).__name__},
        )
        print(json.dumps(result, ensure_ascii=False, indent=2))
        return 1

    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
